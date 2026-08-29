//! Main-thread AEGP idle bridge (architecture §5.3).
//!
//! After Effects does not send an effect selector when a script only changes
//! a parameter expression (TR-M0-005). The idle hook locates DynamicFx
//! instances, gives each one a `PF_Cmd_COMPLETELY_GENERAL` observation
//! opportunity, and mirrors the resulting session token into the hidden
//! StateToken stream via AEGP (ParamDef writes are only honored in
//! UserChangedParam contexts).

use crate::frontend::envelope::{self, SourceClass};
use crate::host::params::{LANGUAGE_STREAM_INDEX, SOURCE_STREAM_INDEX, STATE_TOKEN_STREAM_INDEX};
use after_effects as ae;
use ae::aegp::{ItemType, StreamReferenceHandle, StreamValue};
use std::collections::{HashMap, HashSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

const SCAN_INTERVAL: Duration = Duration::from_secs(1);
// AEGP_IdleHook reports its maximum sleep in 1/60-second ticks, not
// milliseconds. Thirty ticks enforces the one-second scan interval without
// asking AE to wake only for this plug-in.
const MAX_SLEEP_TICKS: i32 = 30;

pub struct IdleState {
    plugin_id: ae::aegp::PluginId,
    // Store the host-owned pointer as an integer so the process-lived refcon
    // does not claim pointer ownership or imply cross-thread access.
    pica_basic: usize,
    main_thread: std::thread::ThreadId,
    alive: Arc<AtomicBool>,
    dynamicfx_key: Option<ae::aegp::InstalledEffectKey>,
    last_scan: Option<Instant>,
    group_ui_tokens: HashMap<InstanceKey, u64>,
    group_visibility_failures: HashSet<InstanceKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct InstanceKey {
    project_index: i32,
    item_id: i32,
    layer_id: u32,
    effect_index: i32,
}

impl IdleState {
    pub fn new(
        plugin_id: ae::aegp::PluginId,
        pica_basic: *const ae::sys::SPBasicSuite,
        main_thread: std::thread::ThreadId,
        alive: Arc<AtomicBool>,
    ) -> Self {
        Self {
            plugin_id,
            pica_basic: pica_basic as usize,
            main_thread,
            alive,
            dynamicfx_key: None,
            last_scan: None,
            group_ui_tokens: HashMap::new(),
            group_visibility_failures: HashSet::new(),
        }
    }
}

/// Top-level FFI boundary. The crate's non-AEGP wrapper does not install a
/// PICA context and does not catch panics, so both protections live here.
/// Errors are logged and swallowed: one damaged project item must never take
/// down AE or disable later idle passes.
pub fn idle_callback(state: &mut IdleState, max_sleep: &mut i32) -> Result<(), ae::Error> {
    if *max_sleep <= 0 || *max_sleep > MAX_SLEEP_TICKS {
        *max_sleep = MAX_SLEEP_TICKS;
    }

    if !state.alive.load(Ordering::Acquire) {
        return Ok(());
    }
    if state.last_scan.is_some_and(|last| last.elapsed() < SCAN_INTERVAL) {
        return Ok(());
    }
    state.last_scan = Some(Instant::now());

    let result = catch_unwind(AssertUnwindSafe(|| {
        if std::thread::current().id() != state.main_thread {
            return Err(ae::Error::WrongThread);
        }
        let pica = state.pica_basic as *const ae::sys::SPBasicSuite;
        if pica.is_null() {
            return Err(ae::Error::InvalidCallback);
        }

        // RegisterNonAegp's wrapper does not populate after-effects-rs's
        // thread-local PICA pointer. This guard restores it only for the
        // duration of this main-thread callback.
        let _pica = ae::PicaBasicSuite::from_sp_basic_suite_raw(pica);
        idle_tick(state)
    }));

    match result {
        Ok(Ok(())) => {}
        Ok(Err(err)) => crate::diag::log(&format!("idle scan failed: {err:?}")),
        Err(_) => crate::diag::log("idle scan panicked; callback isolated"),
    }
    Ok(())
}

fn idle_tick(state: &mut IdleState) -> Result<(), ae::Error> {
    let projects = ae::aegp::suites::Project::new()?;
    let items = ae::aegp::suites::Item::new()?;
    let comps = ae::aegp::suites::Comp::new()?;
    let layers = ae::aegp::suites::Layer::new()?;
    let effects = ae::aegp::suites::Effect::new()?;
    let streams = ae::aegp::suites::Stream::new()?;
    // after-effects-rs intentionally omits a safe SetStreamValue wrapper.
    // Keep the one raw suite access behind a balanced, stack-owned guard.
    let raw_streams =
        RawStreamSuite6::acquire(state.pica_basic as *const ae::sys::SPBasicSuite)?;

    if state.dynamicfx_key.is_none() {
        state.dynamicfx_key = find_dynamicfx_key(&effects)?;
    }
    let Some(target_key) = state.dynamicfx_key else {
        return Ok(());
    };

    let mut seen_instances = HashSet::new();
    for project_index in 0..projects.num_projects()? {
        let project = match projects.project_by_index(project_index) {
            Ok(project) => project,
            Err(err) => {
                crate::diag::log(&format!("idle project lookup failed: {err:?}"));
                continue;
            }
        };
        let mut current = match items.first_proj_item(&project) {
            Ok(item) => Some(item),
            Err(err) => {
                crate::diag::log(&format!("idle first project item failed: {err:?}"));
                continue;
            }
        };

        while let Some(item) = current {
            // Fetch the successor before any nested host call.
            current = match items.next_proj_item(&project, &item) {
                Ok(next) => next,
                Err(err) => {
                    crate::diag::log(&format!("idle next project item failed: {err:?}"));
                    break;
                }
            };
            // AEGP_GetCompFromItem errors for folders/footage; classify first
            // so ordinary project contents do not flood the log every second.
            match items.item_type(&item) {
                Ok(ItemType::Comp) => {}
                Ok(_) => continue,
                Err(err) => {
                    crate::diag::log(&format!("idle item type failed: {err:?}"));
                    continue;
                }
            }
            let item_id = match items.item_id(&item) {
                Ok(id) => id,
                Err(err) => {
                    crate::diag::log(&format!("idle item id failed: {err:?}"));
                    continue;
                }
            };
            let comp = match comps.comp_from_item(&item) {
                Ok(Some(comp)) => comp,
                Ok(None) => continue,
                Err(err) => {
                    crate::diag::log(&format!("idle comp lookup failed: {err:?}"));
                    continue;
                }
            };

            let layer_count = match layers.comp_num_layers(&comp) {
                Ok(count) => count,
                Err(err) => {
                    crate::diag::log(&format!("idle layer count failed: {err:?}"));
                    continue;
                }
            };

            for layer_index in 0..layer_count {
                let layer = match layers.comp_layer_by_index(&comp, layer_index) {
                    Ok(layer) => layer,
                    Err(err) => {
                        crate::diag::log(&format!("idle layer lookup failed: {err:?}"));
                        continue;
                    }
                };
                let layer_id = match layers.layer_id(&layer) {
                    Ok(id) => id,
                    Err(err) => {
                        crate::diag::log(&format!("idle layer id failed: {err:?}"));
                        continue;
                    }
                };
                let layer_time =
                    match layers.layer_current_time(&layer, ae::aegp::TimeMode::LayerTime) {
                        Ok(time) => time,
                        Err(err) => {
                            crate::diag::log(&format!("idle layer time failed: {err:?}"));
                            continue;
                        }
                    };

                let effect_count = match effects.layer_num_effects(&layer) {
                    Ok(count) => count,
                    Err(err) => {
                        crate::diag::log(&format!("idle effect count failed: {err:?}"));
                        continue;
                    }
                };

                for effect_index in 0..effect_count {
                    let instance_key = InstanceKey {
                        project_index: project_index as i32,
                        item_id,
                        layer_id,
                        effect_index,
                    };
                    seen_instances.insert(instance_key);
                    let effect_ref = match effects.layer_effect_by_index(
                        &layer,
                        state.plugin_id,
                        effect_index,
                    ) {
                        Ok(effect_ref) => effect_ref,
                        Err(err) => {
                            crate::diag::log(&format!("idle effect lookup failed: {err:?}"));
                            continue;
                        }
                    };

                    // One malformed instance must not stop the project scan.
                    let call_result = catch_unwind(AssertUnwindSafe(|| {
                        if effects.installed_key_from_layer_effect(&effect_ref)? == target_key {
                            let _ = crate::take_general_reply();
                            effects.effect_call_generic(
                                &effect_ref,
                                state.plugin_id,
                                layer_time,
                                &ae::Command::CompletelyGeneral,
                                None::<&()>,
                            )?;
                            let general_reply = crate::take_general_reply();

                            // CompletelyGeneral published into the process
                            // registry; mirror the token into the primitive
                            // stream so render clones can resolve it.
                            sync_state_token(
                                state,
                                &streams,
                                &raw_streams,
                                &effect_ref,
                                layer_time,
                                general_reply,
                                instance_key,
                            )?;
                        }
                        Ok::<(), ae::Error>(())
                    }));

                    // EffectRefHandle is Copy and has no Drop; dispose on
                    // every success and error path.
                    let dispose_result = effects.dispose_effect(&effect_ref);
                    match call_result {
                        Ok(Ok(())) => {}
                        Ok(Err(err)) => {
                            crate::diag::log(&format!("idle effect sync failed: {err:?}"));
                        }
                        Err(_) => crate::diag::log("idle effect sync panicked; instance isolated"),
                    }
                    if let Err(err) = dispose_result {
                        crate::diag::log(&format!("idle effect dispose failed: {err:?}"));
                    }
                }
            }
        }
    }

    state.group_ui_tokens.retain(|key, _| seen_instances.contains(key));
    state.group_visibility_failures.retain(|key| seen_instances.contains(key));

    Ok(())
}

/// Recompute the session token from the authoritative streams (Language +
/// Source) and mirror it into the StateToken stream when the registry can
/// serve it. Anything unobservable or uncompiled publishes 0, so render
/// clones fail closed instead of reviving stale state.
fn sync_state_token(
    state: &mut IdleState,
    streams: &ae::aegp::suites::Stream,
    raw_streams: &RawStreamSuite6,
    effect_ref: &ae::aegp::EffectRefHandle,
    layer_time: ae::Time,
    general_reply: Option<crate::GeneralReply>,
    instance_key: InstanceKey,
) -> Result<(), ae::Error> {
    let language_stream =
        streams.new_effect_stream_by_index(effect_ref, state.plugin_id, LANGUAGE_STREAM_INDEX)?;
    let language_position = match streams.new_stream_value(
        &language_stream,
        state.plugin_id,
        ae::aegp::TimeMode::LayerTime,
        layer_time,
        true,
    )? {
        StreamValue::OneD(value) if value.is_finite() && value >= 1.0 => value as u32,
        _ => 0,
    };
    let language = crate::frontend::language_from_popup_position(language_position);

    use crate::diagnostics::Diag;
    use crate::TokenState;
    // Set when the word below is a *pending mark* rather than a settled
    // outcome. A pending mark may only fill an empty stream, never overwrite
    // one (see the guard after `current` is read).
    let mut pending = false;
    let mut own_compiled = None;
    let desired_state = match language {
        None => TokenState::Invalid(Diag::LanguageUnknown.code()),
        Some(language) => {
            let source_stream = streams.new_effect_stream_by_index(
                effect_ref,
                state.plugin_id,
                SOURCE_STREAM_INDEX,
            )?;
            if !streams.expression_state(&source_stream, state.plugin_id)? {
                TokenState::Uninitialized
            } else {
                let expression = streams.expression_string(&source_stream, state.plugin_id)?;
                match crate::source::extract_source(&expression) {
                    None => TokenState::Invalid(Diag::NotSourceBlock.code()),
                    Some(source) => match envelope::classify(&source) {
                        Err(crate::frontend::envelope::SourceClassError::Oversize { .. }) => {
                            TokenState::Invalid(Diag::SourceOversize.code())
                        }
                        Err(
                            crate::frontend::envelope::SourceClassError::EnvelopeMalformed,
                        ) => TokenState::Invalid(Diag::EnvelopeMalformed.code()),
                        Ok(SourceClass::Envelope { version }) if version != 1 => {
                            TokenState::Invalid(Diag::EnvelopeUnsupported.code())
                        }
                        // v1 envelopes and raw source share the whole-text
                        // fingerprint (ADR-0018 §7): resolve both through
                        // the registry/failure map the observation filled.
                        Ok(SourceClass::Envelope { .. }) | Ok(SourceClass::Raw) => {
                            let fp = crate::session_token(language, &source);
                            let reply_view = general_reply
                                .as_ref()
                                .map(|reply| (reply.token, reply.compiled.is_some(), reply.code));
                            match decide_token(
                                fp,
                                reply_view,
                                crate::registry_contains_source(fp),
                                crate::failure_code_for(fp),
                            ) {
                                TokenDecision::Active => {
                                    own_compiled = general_reply
                                        .as_ref()
                                        .filter(|reply| reply.token == fp)
                                        .and_then(|reply| reply.compiled.clone());
                                    if own_compiled.is_none() {
                                        crate::diag::log(
                                            "idle: publishing token without this instance's artifact; slot ui skipped",
                                        );
                                    }
                                    TokenState::Active(fp)
                                }
                                TokenDecision::Invalid(code) => TokenState::Invalid(code),
                                TokenDecision::Pending => {
                                    // CompletelyGeneral already had its turn
                                    // (it runs immediately before this call)
                                    // and still produced neither a registry
                                    // entry nor a failure code: a refused
                                    // registry insert, a `block_rebind`
                                    // instance, or an expression that changed
                                    // between the two AEGP reads. Publishing
                                    // E53 is what stops a render clone from
                                    // reading this instance as "nothing
                                    // authored yet" — the two states are
                                    // otherwise identical on every signal a
                                    // clone can see.
                                    pending = true;
                                    TokenState::Invalid(Diag::PublicationPending.code())
                                }
                            }
                        }
                    },
                }
            }
        }
    };
    let desired = match desired_state {
        TokenState::Active(fp) => fp,
        _ => 0,
    };

    let token_stream = streams.new_effect_stream_by_index(
        effect_ref,
        state.plugin_id,
        STATE_TOKEN_STREAM_INDEX,
    )?;
    let current = match streams.new_stream_value(
        &token_stream,
        state.plugin_id,
        ae::aegp::TimeMode::LayerTime,
        layer_time,
        true,
    )? {
        StreamValue::OneD(value) => value,
        _ => return Ok(()),
    };

    if state.group_ui_tokens.get(&instance_key).copied() != Some(desired) {
        match apply_group_ui(state, streams, effect_ref, own_compiled.as_deref()) {
            Ok(group_visibility_failed) => {
                if group_visibility_failed
                    && state.group_visibility_failures.insert(instance_key)
                {
                    crate::diag::log(
                        "idle group hidden flags unsupported for this instance; keeping groups visible",
                    );
                }
            }
            Err(err) => crate::diag::log(&format!("idle group ui failed: {err:?}")),
        }
        // Presentation failures deliberately settle on the static/visible
        // fallback instead of probing the same instance every idle tick.
        state.group_ui_tokens.insert(instance_key, desired);
    }

    // A pending mark fills an empty stream only. It must never clobber a
    // reopened project's saved Active word (still the recovery authority
    // when the registry is cold) nor a more specific diagnostic already
    // published for this instance.
    if pending && current != 0.0 {
        return Ok(());
    }

    // The plan word rides beside the token (ADR-0038 §7): the identity of
    // this instance's published plan, 0 when nothing is published. Written
    // only when it differs, like the token, so a scan never dirties the
    // project for nothing.
    let desired_plan = match (&desired_state, own_compiled.as_ref()) {
        (TokenState::Active(_), Some(compiled)) => {
            crate::identity::plan_identity(&compiled.definition().binding)
        }
        _ => 0,
    };
    let plan_stream = streams.new_effect_stream_by_index(
        effect_ref,
        state.plugin_id,
        crate::host::params::plan_token_stream_index(),
    )?;
    let current_plan = match streams.new_stream_value(
        &plan_stream,
        state.plugin_id,
        ae::aegp::TimeMode::LayerTime,
        layer_time,
        true,
    )? {
        StreamValue::OneD(value) => value,
        _ => return Ok(()),
    };
    if !pending && current_plan != desired_plan as f64 {
        raw_streams.set_one_d(state.plugin_id, &plan_stream, desired_plan as f64)?;
        crate::diag::log(&format!("idle plan token updated: {desired_plan:#x}"));
    }

    let desired_f64 = crate::encode_token_state(desired_state);
    // Exact comparison avoids dirtying the project on every scan (the word
    // is ≤ 2^53 and exactly representable).
    if current == desired_f64 {
        // Token already right — but a reopened project restores the token
        // WITHOUT the slot UI (stream renames do not persist; measured in
        // TR-M3-001's first run). Spot-check one slot's name and republish
        // the UI when it disagrees.
        if matches!(desired_state, crate::TokenState::Active(_)) {
            if let Some(compiled) = own_compiled.as_ref() {
                if slot_ui_out_of_date(state, streams, effect_ref, compiled)? {
                    if let Err(err) =
                        apply_slot_ui(state, streams, raw_streams, effect_ref, compiled)
                    {
                        crate::diag::log(&format!("idle slot ui refresh failed: {err:?}"));
                    }
                }
            }
        }
        return Ok(());
    }

    // Scripted writes get no UI callback, so the idle observer completes the
    // publication itself: slot names, visibility, and fresh-binding defaults
    // via AEGP, then the token. Failures log and skip — the token still
    // publishes so rendering works.
    if desired != 0 {
        if let Some(compiled) = own_compiled.as_ref() {
            if let Err(err) = apply_slot_ui(state, streams, raw_streams, effect_ref, compiled) {
                crate::diag::log(&format!("idle slot ui failed: {err:?}"));
            }
        }
    }

    raw_streams.set_one_d(state.plugin_id, &token_stream, desired_f64)?;
    crate::diag::log(&format!("idle state token updated: {desired_state:?}"));
    Ok(())
}

fn apply_group_ui(
    state: &IdleState,
    streams: &ae::aegp::suites::Stream,
    effect_ref: &ae::aegp::EffectRefHandle,
    compiled: Option<&crate::CompiledEffect>,
) -> Result<bool, ae::Error> {
    use crate::host::params::{group_hidden, pass_group_name, stream_index_of, ParamKey};

    let dyn_suite = ae::aegp::suites::DynamicStream::new()?;
    let definition = compiled.map(crate::CompiledEffect::definition);
    let plan = definition.map(|definition| &definition.binding);

    for group in 0..crate::binding::BANK_GROUPS {
        let live_name = definition
            .and_then(|definition| definition.graph.passes.get(group))
            .map(|pass| pass.name.as_str());
        let label = pass_group_name(group, live_name);
        let Some(index) = stream_index_of(ParamKey::PassGroupStart(group)) else { continue };
        let stream = streams.new_effect_stream_by_index(effect_ref, state.plugin_id, index)?;
        if let Err(err) = dyn_suite.set_stream_name(&stream, &label) {
            crate::diag::log(&format!("idle pass group name failed ({group}): {err:?}"));
        }
    }

    let keys = (0..crate::binding::BANK_GROUPS)
        .flat_map(|group| {
            [ParamKey::PassGroupStart(group), ParamKey::PassGroupEnd(group)]
        })
        .chain((0..crate::host::params::GRADIENTS).flat_map(|gradient| {
            [
                ParamKey::GradientGroupStart(gradient),
                ParamKey::GradientGroupEnd(gradient),
            ]
        }));
    let keys: Vec<_> = keys.collect();
    #[cfg(feature = "editor")]
    let keys = {
        let mut keys = keys;
        keys.extend((0..crate::host::params::GRADIENTS).map(ParamKey::GradientCanvas));
        keys
    };
    for key in &keys {
        let hidden = group_hidden(plan, *key).expect("only presentation group rows are walked");
        let Some(index) = stream_index_of(*key) else { continue };
        let stream = streams.new_effect_stream_by_index(effect_ref, state.plugin_id, index)?;
        if dyn_suite
            .set_dynamic_stream_flag(
                &stream,
                ae::aegp::DynamicStreamFlags::Hidden,
                false,
                hidden,
            )
            .is_err()
        {
            for restore_key in &keys {
                let Some(restore_index) = stream_index_of(*restore_key) else { continue };
                if let Ok(restore_stream) = streams.new_effect_stream_by_index(
                    effect_ref,
                    state.plugin_id,
                    restore_index,
                ) {
                    let _ = dyn_suite.set_dynamic_stream_flag(
                        &restore_stream,
                        ae::aegp::DynamicStreamFlags::Hidden,
                        false,
                        false,
                    );
                }
            }
            return Ok(true);
        }
    }
    Ok(false)
}

/// What the StateToken stream should say for one instance (ADR-0038 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenDecision {
    Active,
    Invalid(u16),
    /// Neither the instance nor the registry can account for the source yet;
    /// the caller publishes the E53 pending mark.
    Pending,
}

/// The instance's own reply decides first — its artifact, or its own
/// diagnostic — so another instance's success on the same text can neither
/// mark a failed instance `Active` nor hide its real code. Only a missing
/// or mismatched reply (the call failed, or the expression changed between
/// the two stream reads) falls back to the per-source view the observation
/// filled. `reply` is `(token, has_artifact, code)`.
fn decide_token(
    fp: u64,
    reply: Option<(u64, bool, crate::diagnostics::Diag)>,
    source_registered: bool,
    failure_code: Option<u16>,
) -> TokenDecision {
    use crate::diagnostics::Diag;
    match reply {
        Some((token, true, _)) if token == fp => TokenDecision::Active,
        Some((_, _, code)) if code != Diag::Ok => TokenDecision::Invalid(code.code()),
        _ if source_registered => TokenDecision::Active,
        _ => match failure_code {
            Some(code) => TokenDecision::Invalid(code),
            None => TokenDecision::Pending,
        },
    }
}

#[cfg(test)]
mod token_decision_tests {
    use super::{decide_token, TokenDecision};
    use crate::diagnostics::Diag;

    const FP: u64 = 0x1234;

    #[test]
    fn own_artifact_wins() {
        assert_eq!(decide_token(FP, Some((FP, true, Diag::Ok)), false, None), TokenDecision::Active);
        // Even when the per-source view disagrees.
        assert_eq!(
            decide_token(FP, Some((FP, true, Diag::Ok)), false, Some(Diag::PoolOverflow.code())),
            TokenDecision::Active
        );
    }

    #[test]
    fn own_failure_is_not_masked_by_another_instance() {
        assert_eq!(
            decide_token(FP, Some((0, false, Diag::PoolOverflow)), true, None),
            TokenDecision::Invalid(Diag::PoolOverflow.code())
        );
        assert_eq!(
            decide_token(FP, Some((0, false, Diag::SnapshotSchemaUnknown)), true, None),
            TokenDecision::Invalid(Diag::SnapshotSchemaUnknown.code())
        );
    }

    #[test]
    fn no_usable_reply_falls_back_to_the_source_view() {
        assert_eq!(decide_token(FP, None, true, None), TokenDecision::Active);
        assert_eq!(
            decide_token(FP, None, false, Some(Diag::PoolOverflow.code())),
            TokenDecision::Invalid(Diag::PoolOverflow.code())
        );
        assert_eq!(decide_token(FP, None, false, None), TokenDecision::Pending);
        // A reply for a different text (the expression changed between the
        // two reads) is not this instance's verdict on `fp`.
        assert_eq!(decide_token(FP, Some((FP + 1, true, Diag::Ok)), true, None), TokenDecision::Active);
        assert_eq!(decide_token(FP, Some((FP + 1, true, Diag::Ok)), false, None), TokenDecision::Pending);
    }
}

/// One-read staleness probe: compare the first pool slot's current stream
/// name against what the plan wants (bound label or default name). Reopened
/// projects restore parameter values but not stream renames, so a mismatch
/// here means the whole slot UI needs republishing.
fn slot_ui_out_of_date(
    state: &IdleState,
    streams: &ae::aegp::suites::Stream,
    effect_ref: &ae::aegp::EffectRefHandle,
    compiled: &crate::CompiledEffect,
) -> Result<bool, ae::Error> {
    use crate::binding::{PoolKind, SlotRef};
    use crate::host::params::{default_slot_name, stream_index_of};

    let probe = SlotRef { kind: PoolKind::Float, index: 0 };
    let Some(stream_index) = stream_index_of(crate::host::params::key_for_slot(
        probe.kind,
        probe.index,
    )) else {
        return Ok(false);
    };
    let configs = crate::slot_configs(compiled.definition());
    let expected = configs
        .get(&probe)
        .map(|c| c.label.clone())
        .unwrap_or_else(|| default_slot_name(probe.kind, probe.index));
    let stream = streams.new_effect_stream_by_index(effect_ref, state.plugin_id, stream_index)?;
    let actual = streams.stream_name(&stream, state.plugin_id, false)?;
    Ok(actual != expected)
}

/// Apply slot labels, Hidden flags, and fresh-binding scalar defaults for
/// every pool slot via AEGP (legal on the main-thread idle path; ParamDef
/// writes are not honored here). Defaults never touch inherited bindings,
/// so user values and keyframes survive re-binds.
fn apply_slot_ui(
    state: &IdleState,
    streams: &ae::aegp::suites::Stream,
    raw_streams: &RawStreamSuite6,
    effect_ref: &ae::aegp::EffectRefHandle,
    compiled: &crate::CompiledEffect,
) -> Result<(), ae::Error> {
    use crate::binding::{all_pools, PoolKind, SlotRef};
    use crate::host::params::{default_slot_name, key_for_slot, stream_index_of};

    let dyn_suite = ae::aegp::suites::DynamicStream::new()?;
    let configs = crate::slot_configs(compiled.definition());
    let mut bound = 0usize;
    let mut defaults_written = 0usize;
    for (kind, capacity) in all_pools() {
        for i in 0..capacity {
            let slot = SlotRef { kind, index: i };
            let Some(stream_index) = stream_index_of(key_for_slot(kind, i)) else { continue };
            let stream =
                streams.new_effect_stream_by_index(effect_ref, state.plugin_id, stream_index)?;
            let config = configs.get(&slot);
            let (label, hidden) = match config {
                Some(config) => {
                    bound += 1;
                    (config.label.clone(), false)
                }
                None => (default_slot_name(kind, i), true),
            };
            dyn_suite.set_stream_name(&stream, &label)?;
            dyn_suite.set_dynamic_stream_flag(
                &stream,
                ae::aegp::DynamicStreamFlags::Hidden,
                false,
                hidden,
            )?;
            // Fresh-binding defaults: scalar (OneD) kinds, and Color
            // streams via the four-component color value (ADR-0026).
            if let Some(config) = config {
                let scalar = matches!(
                    kind,
                    PoolKind::Float | PoolKind::Integer | PoolKind::Bool | PoolKind::Angle
                );
                if config.fresh && scalar {
                    if let Some(default) = config.default {
                        raw_streams.set_one_d(state.plugin_id, &stream, default as f64)?;
                        defaults_written += 1;
                    }
                }
                if config.fresh && kind == PoolKind::Color {
                    if let Some([r, g, b, _a]) = config.color_default {
                        // Alpha rides the companion Float slot; the color
                        // stream itself is written opaque.
                        raw_streams.set_color(
                            state.plugin_id,
                            &stream,
                            r as f64,
                            g as f64,
                            b as f64,
                        )?;
                        defaults_written += 1;
                    }
                }
            }
        }
    }
    crate::diag::log(&format!(
        "idle slot ui applied: {bound} bound, {defaults_written} defaults written"
    ));
    Ok(())
}

/// Minimal raw wrapper for the one StreamSuite6 operation that
/// after-effects-rs 0.4 does not expose. Acquisition and release are paired
/// by Drop, including early returns and unwinding through the panic guard.
struct RawStreamSuite6 {
    pica_basic: *const ae::sys::SPBasicSuite,
    suite: *const ae::sys::AEGP_StreamSuite6,
}

impl RawStreamSuite6 {
    fn acquire(pica_basic: *const ae::sys::SPBasicSuite) -> Result<Self, ae::Error> {
        if pica_basic.is_null() {
            return Err(ae::Error::InvalidCallback);
        }

        let acquire = unsafe { (*pica_basic).AcquireSuite }.ok_or(ae::Error::MissingSuite)?;
        let mut suite_ptr: *const std::ffi::c_void = ptr::null();
        let err = unsafe {
            acquire(
                ae::sys::kAEGPStreamSuite.as_ptr().cast(),
                ae::sys::kAEGPStreamSuiteVersion6 as i32,
                &mut suite_ptr,
            )
        };
        if err != ae::sys::kSPNoError as i32 {
            crate::diag::log(&format!("raw StreamSuite6 acquire failed: {err}"));
            return Err(ae::Error::MissingSuite);
        }

        let guard = Self { pica_basic, suite: suite_ptr.cast() };
        if guard.suite.is_null() {
            // Acquire succeeded, so let Drop balance the host reference even
            // though the returned pointer is unusable.
            drop(guard);
            return Err(ae::Error::MissingSuite);
        }
        Ok(guard)
    }

    fn set_one_d(
        &self,
        plugin_id: ae::aegp::PluginId,
        stream: &StreamReferenceHandle,
        value: f64,
    ) -> Result<(), ae::Error> {
        let set_stream_value =
            unsafe { (*self.suite).AEGP_SetStreamValue }.ok_or(ae::Error::MissingSuite)?;
        // Match AEFX_CLR_STRUCT from the SDK examples before selecting the
        // active union member; the struct is a handle plus a numeric union,
        // so all-zero is a valid empty initialization.
        let mut stream_value: ae::sys::AEGP_StreamValue2 = unsafe { std::mem::zeroed() };
        stream_value.streamH = stream.as_ptr();
        stream_value.val.one_d = value;
        let err = unsafe { set_stream_value(plugin_id, stream.as_ptr(), &mut stream_value) };
        if err == ae::sys::A_Err_NONE {
            Ok(())
        } else {
            crate::diag::log(&format!("AEGP_SetStreamValue failed: {err}"));
            Err(ae::Error::Generic)
        }
    }

    /// ADR-0026: write a Color stream's RGBA value (alpha fixed opaque —
    /// the shader-facing alpha rides the companion Float slot).
    fn set_color(
        &self,
        plugin_id: ae::aegp::PluginId,
        stream: &StreamReferenceHandle,
        r: f64,
        g: f64,
        b: f64,
    ) -> Result<(), ae::Error> {
        let set_stream_value =
            unsafe { (*self.suite).AEGP_SetStreamValue }.ok_or(ae::Error::MissingSuite)?;
        let mut stream_value: ae::sys::AEGP_StreamValue2 = unsafe { std::mem::zeroed() };
        stream_value.streamH = stream.as_ptr();
        stream_value.val.color = ae::sys::AEGP_ColorVal {
            alphaF: 1.0,
            redF: r,
            greenF: g,
            blueF: b,
        };
        let err = unsafe { set_stream_value(plugin_id, stream.as_ptr(), &mut stream_value) };
        if err == ae::sys::A_Err_NONE {
            Ok(())
        } else {
            crate::diag::log(&format!("AEGP_SetStreamValue(color) failed: {err}"));
            Err(ae::Error::Generic)
        }
    }
}

impl Drop for RawStreamSuite6 {
    fn drop(&mut self) {
        if self.pica_basic.is_null() {
            return;
        }
        let Some(release) = (unsafe { (*self.pica_basic).ReleaseSuite }) else {
            crate::diag::log("raw StreamSuite6 release function missing");
            return;
        };
        let err = unsafe {
            release(
                ae::sys::kAEGPStreamSuite.as_ptr().cast(),
                ae::sys::kAEGPStreamSuiteVersion6 as i32,
            )
        };
        if err != ae::sys::kSPNoError as i32 {
            crate::diag::log(&format!("raw StreamSuite6 release failed: {err}"));
        }
    }
}

fn find_dynamicfx_key(
    effects: &ae::aegp::suites::Effect,
) -> Result<Option<ae::aegp::InstalledEffectKey>, ae::Error> {
    let mut key = ae::aegp::InstalledEffectKey::None;
    loop {
        key = effects.next_installed_effect(key)?;
        if key == ae::aegp::InstalledEffectKey::None {
            return Ok(None);
        }
        match effects.effect_match_name(key) {
            Ok(name) if name == crate::MATCH_NAME => return Ok(Some(key)),
            Ok(_) => {}
            Err(err) => crate::diag::log(&format!("idle installed effect name failed: {err:?}")),
        }
    }
}
