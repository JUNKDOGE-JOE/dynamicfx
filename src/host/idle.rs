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
                            effects.effect_call_generic(
                                &effect_ref,
                                state.plugin_id,
                                layer_time,
                                &ae::Command::CompletelyGeneral,
                                None::<&()>,
                            )?;

                            // CompletelyGeneral published into the process
                            // registry; mirror the token into the primitive
                            // stream so render clones can resolve it.
                            sync_state_token(state, &streams, &raw_streams, &effect_ref, layer_time)?;
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

    Ok(())
}

/// Recompute the session token from the authoritative streams (Language +
/// Source) and mirror it into the StateToken stream when the registry can
/// serve it. Anything unobservable or uncompiled publishes 0, so render
/// clones fail closed instead of reviving stale state.
fn sync_state_token(
    state: &IdleState,
    streams: &ae::aegp::suites::Stream,
    raw_streams: &RawStreamSuite6,
    effect_ref: &ae::aegp::EffectRefHandle,
    layer_time: ae::Time,
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
                            if crate::registry_contains(fp) {
                                TokenState::Active(fp)
                            } else if let Some(code) = crate::failure_code_for(fp) {
                                TokenState::Invalid(code)
                            } else {
                                // Not observed yet this session: leave the
                                // stream alone until CompletelyGeneral has
                                // had its turn.
                                return Ok(());
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

    let desired_f64 = crate::encode_token_state(desired_state);
    // Exact comparison avoids dirtying the project on every scan (the word
    // is ≤ 2^53 and exactly representable).
    if current == desired_f64 {
        // Token already right — but a reopened project restores the token
        // WITHOUT the slot UI (stream renames do not persist; measured in
        // TR-M3-001's first run). Spot-check one slot's name and republish
        // the UI when it disagrees.
        if let crate::TokenState::Active(fp) = desired_state {
            if let Some(compiled) = crate::registry_get(fp) {
                if slot_ui_out_of_date(state, streams, effect_ref, &compiled)? {
                    if let Err(err) =
                        apply_slot_ui(state, streams, raw_streams, effect_ref, &compiled)
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
        if let Some(compiled) = crate::registry_get(desired) {
            if let Err(err) = apply_slot_ui(state, streams, raw_streams, effect_ref, &compiled) {
                crate::diag::log(&format!("idle slot ui failed: {err:?}"));
            }
        }
    }

    raw_streams.set_one_d(state.plugin_id, &token_stream, desired_f64)?;
    crate::diag::log(&format!("idle state token updated: {desired_state:?}"));
    Ok(())
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
    use crate::host::params::{default_slot_name, stream_index_of, ParamKey};

    let probe = SlotRef { kind: PoolKind::Float, index: 0 };
    let Some(stream_index) = stream_index_of(ParamKey::Pool(probe.kind, probe.index)) else {
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
    use crate::binding::{PoolKind, SlotRef, V1_POOLS};
    use crate::host::params::{default_slot_name, stream_index_of, ParamKey};

    let dyn_suite = ae::aegp::suites::DynamicStream::new()?;
    let configs = crate::slot_configs(compiled.definition());
    let mut bound = 0usize;
    let mut defaults_written = 0usize;
    for (kind, capacity) in V1_POOLS {
        for i in 0..*capacity {
            let slot = SlotRef { kind: *kind, index: i };
            let Some(stream_index) = stream_index_of(ParamKey::Pool(*kind, i)) else { continue };
            let stream =
                streams.new_effect_stream_by_index(effect_ref, state.plugin_id, stream_index)?;
            let config = configs.get(&slot);
            let (label, hidden) = match config {
                Some(config) => {
                    bound += 1;
                    (config.label.clone(), false)
                }
                None => (default_slot_name(*kind, i), true),
            };
            dyn_suite.set_stream_name(&stream, &label)?;
            dyn_suite.set_dynamic_stream_flag(
                &stream,
                ae::aegp::DynamicStreamFlags::Hidden,
                false,
                hidden,
            )?;
            // Fresh-binding defaults, scalar (OneD) kinds only for now;
            // color/point defaults need more raw stream-value plumbing and
            // arrive with their value-encoding fixtures.
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
