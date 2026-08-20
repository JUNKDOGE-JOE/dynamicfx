//! DynamicFx — an open shader runtime controlled through ordinary After
//! Effects properties.
//!
//! M1 pipeline (ADRs 0010-0014):
//!
//!   committed expression → strip backtick wrapper → envelope classify
//!   (ADR-0012) → `LanguageFrontend` parse + ABI v1 validation (ADR-0011) →
//!   one-pass `EffectDefinition` + fresh `BindingPlan` (ADR-0013) → SPIR-V
//!   artifact → process registry → wgpu (DX12, ADR-0014) render.
//!
//! UI/render transport: the UI instance publishes a session-local token into
//! the hidden StateToken parameter; render clones resolve it through the
//! process registry. The token layout is an M3 contract — this interim
//! value is meaningless across sessions by design (a reopened project gets a
//! registry miss → pass-through until re-observation), so no persisted byte
//! carries meaning before the M3 ADR (ADR-0009). Nothing is flattened.

mod diag;
mod render;
mod source;

// M3 persistence layers (ADRs 0015-0017).
pub mod diagnostics;
mod gradient;
mod path;
pub mod identity;
pub mod persistence;

// Target-architecture layers (ADRs 0010-0013, 0018-0020). `definition`,
// `frontend`, `binding`, and `plan` are host-agnostic by policy: no AE SDK
// types below `host`.
pub mod binding;
pub mod definition;
pub mod frontend;
pub mod host;
pub mod plan;

use after_effects as ae;
use ae::aegp::suites::{Stream, Utility};
use binding::PoolKind;
use definition::effect::EffectDefinition;
use diagnostics::Diag;
use definition::param::ShaderParamType;
use frontend::envelope::{self, SourceClass, SourceClassError};
use frontend::{LanguageId, UniformBlockLayout};
use host::params::ParamKey;
use std::collections::HashMap as TokenMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, OnceLock,
};

const MATCH_NAME: &str = "DynamicFx";

/// One compiled pass: artifact, upload layout, member→merged-parameter map,
/// and the extra input bindings the module declares.
pub(crate) struct CompiledPass {
    spirv: Vec<u32>,
    layout: UniformBlockLayout,
    param_map: Vec<usize>,
    extra_input_bindings: Vec<u32>,
}

/// Read one gradient's live stop parameters and bake its LUT into the working
/// format (ADR-0031 §5, ADR-0033 §1).
///
/// The LUT rides the render's working format rather than a fixed
/// `Rgba32Float`: at 32-bpc that *is* float — satisfying the decision's reason,
/// which is not quantizing what the rest of the pipeline preserves — while at
/// 8-bpc a float texture would demand `FLOAT32_FILTERABLE`, a feature the
/// adapter is only guaranteed to carry for the deep formats.
///
/// A malformed read is refused, not repaired (ADR-0033 §5): the resource binds
/// transparent black and the reason is logged with its `E54` code.
fn bake_gradient(
    params: &ae::Parameters<host::params::ParamKey>,
    gradient_index: usize,
) -> Option<ExternalPixels> {
    use host::params::{GradientField, ParamKey, STOPS_PER_GRADIENT};

    let float_at = |key: ParamKey| -> Option<f32> {
        params.get(key).ok()?.as_float_slider().ok().map(|f| f.value() as f32)
    };
    let color_at = |key: ParamKey| -> Option<[f32; 3]> {
        let p = params.get(key).ok()?;
        let c = p.as_color().ok()?.value();
        Some([c.red as f32 / 255.0, c.green as f32 / 255.0, c.blue as f32 / 255.0])
    };

    let count = float_at(ParamKey::GradientCount(gradient_index))
        .map(|v| v.round() as i32)
        .unwrap_or(0);
    let mut stops = Vec::with_capacity(STOPS_PER_GRADIENT);
    for stop in 0..STOPS_PER_GRADIENT {
        // Never `?` these away silently: an unreadable stop is a real fault
        // and the project's policy is that nothing degrades to pass-through
        // without a diagnostic.
        let position =
            float_at(ParamKey::GradientStop(gradient_index, stop, GradientField::Position));
        let rgb = color_at(ParamKey::GradientStop(gradient_index, stop, GradientField::Color));
        let alpha =
            float_at(ParamKey::GradientStop(gradient_index, stop, GradientField::Alpha));
        let (Some(position), Some(rgb), Some(alpha)) = (position, rgb, alpha) else {
            diag::log(&diagnostics::status_text(
                Diag::GradientMalformed,
                &format!("gradient {gradient_index}: stop {stop} parameters unreadable"),
            ));
            return None;
        };
        stops.push(gradient::Stop { position, rgba: [rgb[0], rgb[1], rgb[2], alpha] });
    }

    if count < 1 || count as usize > STOPS_PER_GRADIENT {
        diag::log(&diagnostics::status_text(
            Diag::GradientMalformed,
            &format!("gradient {gradient_index}: stop count {count} outside 1..={STOPS_PER_GRADIENT}"),
        ));
        return None;
    }
    let value = gradient::Gradient::from_parameters(count as usize, &stops);
    if let Err(e) = value.validate() {
        diag::log(&diagnostics::status_text(
            Diag::GradientMalformed,
            &format!("gradient {gradient_index} rejected ({e:?}); binding transparent black"),
        ));
        return None;
    }

    Some(ExternalPixels {
        pixels: Vec::new(),
        stride: 0,
        width: gradient::LUT_WIDTH,
        height: 1,
        samples: Some(value.bake_lut()),
        vertices: None,
        ae_pixels: false,
    })
}

/// Check out one AE mask and walk its vertices (ADR-0035).
///
/// Never fails the render: an unassigned selector, a deleted mask, or a path
/// with no segments all yield the empty vertex list, which `path::encode` turns
/// into the documented `1 x 2` zero texture (§5). Each of those is logged,
/// because "the shader sees nothing" and "the checkout broke" look identical
/// from the outside otherwise.
fn read_path(
    in_data: &ae::InData,
    params: &ae::Parameters<host::params::ParamKey>,
    path_index: usize,
) -> Option<ExternalPixels> {
    // One line per *change*, not one per frame: a bound path is read on every
    // SmartRender, and `diag::log` opens and closes the file per call, so
    // unconditional logging would cost a syscall per frame during playback.
    // Silence is not an option either — the first host run could not tell an
    // unassigned selector from a dropped upload, because neither said anything
    // (2026-08-16).
    fn note(path_index: usize, id: u32, count: usize) {
        thread_local! {
            static LAST: std::cell::RefCell<TokenMap<usize, (u32, usize)>> =
                std::cell::RefCell::new(TokenMap::new());
        }
        LAST.with(|last| {
            let mut last = last.borrow_mut();
            if last.get(&path_index) == Some(&(id, count)) {
                return;
            }
            last.insert(path_index, (id, count));
            diag::log(&format!("path {path_index}: id={id} vertices={count}"));
        });
    }

    let empty = || {
        Some(ExternalPixels {
            pixels: Vec::new(),
            stride: 0,
            width: 0,
            height: 0,
            samples: None,
            vertices: Some(Vec::new()),
            ae_pixels: false,
        })
    };

    let key = ParamKey::Pool(binding::PoolKind::Path, path_index);
    let Some(id) = params.get(key).ok().and_then(|p| p.as_path().ok().map(|p| p.path_id()))
    else {
        diag::log(&format!("path {path_index}: selector unreadable"));
        return empty();
    };
    // 0 is PF_PathID_NONE — the user has not picked a mask. Not a fault.
    if id == 0 {
        note(path_index, 0, 0);
        return empty();
    }

    let Ok(suite) = ae::pf::suites::PathQuery::new() else {
        diag::log("path: PathQuerySuite unavailable");
        return empty();
    };
    let outline = suite.checkout_path(
        in_data.effect_ref(),
        id,
        in_data.current_time(),
        in_data.time_step(),
        in_data.time_scale(),
    );
    let outline = match outline {
        // Documented: a non-NONE id can still resolve to nothing, because the
        // mask it named may have been deleted since.
        Ok(None) => return empty(),
        Ok(Some(outline)) => outline,
        Err(e) => {
            diag::log(&format!("path {path_index}: checkout of id {id} failed: {e:?}"));
            return empty();
        }
    };

    // N segments means vertices `[0..=N]`, and a closed path repeats vertex 0
    // at the end — which is what lets a shader walk segments without wrapping
    // the index itself.
    let Ok(segments) = outline.num_segments() else {
        diag::log(&format!("path {path_index}: segment count unreadable"));
        return empty();
    };
    if segments <= 0 {
        return empty();
    }
    let wanted = segments as usize + 1;
    if wanted > path::MAX_VERTICES {
        // No silent caps: say what was dropped, or the render reads as
        // complete when it is not.
        diag::log(&format!(
            "path {path_index}: {wanted} vertices exceeds the {} the texture can carry; \
             delivering the first {}",
            path::MAX_VERTICES,
            path::MAX_VERTICES
        ));
    }
    let mut vertices = Vec::with_capacity(wanted.min(path::MAX_VERTICES));
    for i in 0..wanted.min(path::MAX_VERTICES) {
        match outline.vertex(i as i32) {
            Ok(v) => vertices.push(path::Vertex {
                x: v.x as f32,
                y: v.y as f32,
                tan_in_x: v.tan_in_x as f32,
                tan_in_y: v.tan_in_y as f32,
                tan_out_x: v.tan_out_x as f32,
                tan_out_y: v.tan_out_y as f32,
            }),
            Err(e) => {
                diag::log(&format!("path {path_index}: vertex {i} unreadable: {e:?}"));
                return empty();
            }
        }
    }
    note(path_index, id, vertices.len());
    Some(ExternalPixels {
        pixels: Vec::new(),
        stride: 0,
        width: 0,
        height: 0,
        samples: None,
        vertices: Some(vertices),
        ae_pixels: false,
    })
}

/// Owned pixels for one external resource, staged for the frame being
/// rendered on this thread.
#[derive(Debug, Clone)]
pub(crate) struct ExternalPixels {
    pixels: Vec<u8>,
    stride: usize,
    width: usize,
    height: usize,
    /// ADR-0033: a baked gradient LUT is held as float samples and encoded by
    /// the executor, which is the only place the working format is known.
    /// Deriving depth in the SmartRender arm instead made the whole read fail
    /// silently on the first frame, before any pipeline existed — measured
    /// 2026-08-15 as an all-black ramp with nothing in the log.
    samples: Option<Vec<[f32; 4]>>,
    /// ADR-0035: mask vertices in layer pixels, not yet encoded. Normalizing
    /// needs the frame extent, which this staging point does not have — the
    /// same reason the gradient LUT above is baked depth-independently and
    /// encoded later.
    vertices: Option<Vec<path::Vertex>>,
    /// ADR-0030: `pixels` are AE's own bytes (ARGB, and 8 bytes per pixel at
    /// 16-bpc), not the working RGBA layout. Converted at the encode site.
    ae_pixels: bool,
}

/// What supplies one externally-fed graph resource (ADR-0030, ADR-0032).
/// They differ only in how the pixels are obtained — a layer is checked out,
/// a gradient is baked, a path is walked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExternalSource {
    /// The **AE parameter index** of the Layer selector: the declaration
    /// position plus one, because AE's implicit input layer occupies index 0.
    /// The same convention `host::params::stream_index_of` uses.
    ///
    /// Storing the declaration position here instead is what broke layer
    /// inputs on their first host run (2026-08-16): `PF_CHECKOUT_LAYER` was
    /// handed 110 for the slot AE calls 111 and answered
    /// `BadCallbackParameter`, so every layer read returned nothing and the
    /// effect rendered transparent black — indistinguishable, in the harness
    /// as written, from the documented unassigned-selector behaviour.
    Layer { param_index: usize },
    /// ADR-0033: the gradient's ordinal (its `Pool(Gradient, g)` slot index),
    /// not a declaration index — the value now lives in that gradient's stop
    /// parameters, which are addressed by ordinal.
    Gradient { gradient_index: usize },
    /// ADR-0035: the mask selector's `Pool(Path, i)` slot index. Addressed by
    /// ordinal like a gradient, because the checkout reads the parameter
    /// rather than an AE-assigned checkout id.
    Path { path_index: usize },
}

/// Everything a render clone needs to execute one published definition.
pub(crate) struct CompiledEffect {
    definition: EffectDefinition,
    passes: Vec<CompiledPass>,
    plan: plan::ExecutionPlan,
    /// `Some(W)` when any pass reads `prev` (ADR-0025): each frame
    /// re-simulates `min(F+1, W)` iterations from black, self-contained.
    window: Option<u32>,
    /// The exact committed text (ADR-0016's snapshot source). Storing a
    /// pass body here instead was an M4-latent defect: envelope sources
    /// reopened as raw single-pass effects (caught by the M6 aerender leg).
    source: String,
    /// ADR-0030/0032: what feeds each `TexSlot::External` ordinal. Resolved
    /// once at compile time so neither PreRender nor SmartRender has to
    /// re-derive the graph's resource order per frame.
    externals: Vec<ExternalSource>,
}

/// ADR-0025 §1: iterations for frame F under window W (the F+1 clamp keeps
/// nothing "before the layer start"; negative frames still run once).
fn window_iterations(frame: i64, window: u32) -> u32 {
    (frame + 1).clamp(1, window as i64) as u32
}

#[cfg(test)]
mod temporal_tests {
    use super::window_iterations;

    #[test]
    fn iteration_count_math() {
        assert_eq!(window_iterations(0, 16), 1);
        assert_eq!(window_iterations(4, 16), 5); // ramp-in
        assert_eq!(window_iterations(15, 16), 16);
        assert_eq!(window_iterations(16, 16), 16); // plateau
        assert_eq!(window_iterations(1000, 16), 16);
        assert_eq!(window_iterations(-3, 16), 1); // pre-zero still runs once
        assert_eq!(window_iterations(2, 1), 1);
        assert_eq!(window_iterations(70, 64), 64);
    }
}

/// The ADR-0020 kill switch: aliasing is on unless DYNAMICFX_NO_ALIAS says
/// otherwise (read per evaluation so the harness can flip it per session).
fn alias_enabled() -> bool {
    !std::env::var("DYNAMICFX_NO_ALIAS")
        .is_ok_and(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "on" | "ON"))
}

/// ROI escape hatch (M7): DYNAMICFX_NO_ROI=1 forces full-frame delivery so
/// the equivalence harness can A/B scissored vs full renders byte-exactly.
fn roi_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        !std::env::var("DYNAMICFX_NO_ROI")
            .is_ok_and(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "on" | "ON"))
    })
}

/// M7 measurement gate: when DYNAMICFX_PERF=1 (cold start), every render
/// logs one machine-parsable `perf:` line. Span collection is always on
/// (a few Instant reads); only the logging is gated.
fn perf_log_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("DYNAMICFX_PERF")
            .is_ok_and(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "on" | "ON"))
    })
}

/// Session-local process registry: token → compiled effect. UI instances
/// insert; render clones resolve. Cleared only by process exit; a stale
/// persisted token from an earlier session simply misses here.
fn registry() -> &'static Mutex<TokenMap<u64, Arc<CompiledEffect>>> {
    static REGISTRY: OnceLock<Mutex<TokenMap<u64, Arc<CompiledEffect>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(TokenMap::new()))
}

pub(crate) fn registry_get(token: u64) -> Option<Arc<CompiledEffect>> {
    registry().lock().ok()?.get(&token).cloned()
}

impl CompiledEffect {
    pub(crate) fn definition(&self) -> &EffectDefinition {
        &self.definition
    }
}

/// Per-slot configuration derived from one definition: display label
/// (annotation label or the ParamId; vec4 alpha companions suffixed " A"),
/// optional range/default metadata, and whether the binding is fresh
/// (defaults are only ever written to fresh bindings). Shared by the
/// UI-callback configuration and the idle observer's AEGP publication.
pub(crate) struct SlotConfig {
    pub label: String,
    pub min: Option<f32>,
    pub max: Option<f32>,
    /// Scalar default for this slot (component 0, or component 3 for the
    /// vec4 alpha companion).
    pub default: Option<f32>,
    /// Full RGBA default for Color slots (ADR-0026); alpha rides the
    /// companion Float slot, so the color stream itself gets alpha 1.0.
    pub color_default: Option<[f32; 4]>,
    pub fresh: bool,
}

pub(crate) fn slot_configs(
    defn: &EffectDefinition,
) -> std::collections::HashMap<binding::SlotRef, SlotConfig> {
    let mut configs = std::collections::HashMap::new();
    for (decl, bound) in defn.params.iter().zip(defn.binding.bindings.iter()) {
        let base = decl
            .ui
            .label
            .clone()
            .unwrap_or_else(|| decl.id.as_str().to_string());
        for (j, slot) in bound.slots.iter().enumerate() {
            let (label, default) = if j == 0 {
                (base.clone(), decl.ui.default.as_ref().map(|d| d[0]))
            } else {
                // The vec4 alpha companion rides component 3.
                (format!("{base} A"), decl.ui.default.as_ref().and_then(|d| d.get(3).copied()))
            };
            let color_default = (j == 0 && slot.kind == binding::PoolKind::Color)
                .then(|| {
                    decl.ui.default.as_ref().and_then(|d| {
                        (d.len() >= 3).then(|| [d[0], d[1], d[2], d.get(3).copied().unwrap_or(1.0)])
                    })
                })
                .flatten();
            configs.insert(
                *slot,
                SlotConfig {
                    label,
                    min: (j == 0).then_some(decl.ui.min).flatten(),
                    max: (j == 0).then_some(decl.ui.max).flatten(),
                    default,
                    color_default,
                    fresh: !bound.inherited,
                },
            );
        }
    }
    configs
}

pub(crate) fn registry_contains(token: u64) -> bool {
    registry().lock().is_ok_and(|map| map.contains_key(&token))
}

/// Insert under `token`; returns false when a different source already owns
/// the token (51-bit truncation collision — fail closed, never render the
/// wrong shader).
fn registry_insert(token: u64, compiled: Arc<CompiledEffect>) -> bool {
    let Ok(mut map) = registry().lock() else { return false };
    match map.get(&token) {
        Some(existing)
            if existing.definition.graph.passes[0].source
                != compiled.definition.graph.passes[0].source =>
        {
            diag::log("session token collision; publication refused");
            false
        }
        _ => {
            map.insert(token, compiled);
            true
        }
    }
}

/// Session token for one (language, committed source) pair: the ADR-0017
/// `dfx:token:v1` fingerprint (51-bit truncated BLAKE3, nonzero).
pub(crate) fn session_token(language: LanguageId, source: &str) -> u64 {
    identity::token_fingerprint(language, source)
}

/// Decoded StateToken states (ADR-0015 §1): `(payload << 2) | state` in one
/// exact f64 integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenState {
    Uninitialized,
    Active(u64),
    Invalid(u16),
    Corrupt,
}

pub(crate) fn encode_token_state(state: TokenState) -> f64 {
    (match state {
        TokenState::Uninitialized => 0,
        TokenState::Active(fp) => {
            debug_assert!(fp != 0 && fp < (1u64 << 51));
            (fp << 2) | 0b01
        }
        TokenState::Invalid(code) => ((code as u64) << 2) | 0b10,
        TokenState::Corrupt => unreachable!("corrupt is a decode result, never encoded"),
    }) as f64
}

pub(crate) fn decode_token_state(value: f64) -> TokenState {
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 || value > (1u64 << 53) as f64 {
        return TokenState::Corrupt;
    }
    let word = value as u64;
    let payload = word >> 2;
    match word & 0b11 {
        0b00 if payload == 0 => TokenState::Uninitialized,
        0b01 if payload != 0 => TokenState::Active(payload),
        0b10 if payload <= u16::MAX as u64 => TokenState::Invalid(payload as u16),
        _ => TokenState::Corrupt,
    }
}

/// The desired token word for an instance's current state (shared by the
/// UCP publication and the idle sync).
pub(crate) fn desired_token_state(token: u64, status_code: Diag) -> TokenState {
    if token != 0 {
        TokenState::Active(token)
    } else if matches!(status_code, Diag::Ok | Diag::NoExpression) {
        TokenState::Uninitialized
    } else {
        TokenState::Invalid(status_code.code())
    }
}

#[cfg(test)]
mod token_tests {
    use super::*;

    #[test]
    fn every_state_round_trips() {
        for state in [
            TokenState::Uninitialized,
            TokenState::Active(1),
            TokenState::Active((1u64 << 51) - 1),
            TokenState::Invalid(diagnostics::Diag::GlslParse.code()),
            TokenState::Invalid(u16::MAX),
        ] {
            assert_eq!(decode_token_state(encode_token_state(state)), state);
        }
    }

    #[test]
    fn corrupt_values_fail_closed() {
        for bad in [
            f64::NAN,
            f64::INFINITY,
            -4.0,
            1.5,
            (1u64 << 53) as f64 + 8.0,
            // Reserved state 0b11.
            0b11 as f64,
            // Uninitialized state with nonzero payload.
            (7u64 << 2) as f64,
        ] {
            assert_eq!(decode_token_state(bad), TokenState::Corrupt, "{bad}");
        }
        // Active with zero payload is also corrupt (fingerprints are nonzero).
        assert_eq!(decode_token_state(0b01 as f64), TokenState::Corrupt);
    }

    #[test]
    fn desired_state_mapping() {
        assert_eq!(desired_token_state(42, Diag::Ok), TokenState::Active(42));
        assert_eq!(desired_token_state(0, Diag::Ok), TokenState::Uninitialized);
        assert_eq!(desired_token_state(0, Diag::NoExpression), TokenState::Uninitialized);
        assert_eq!(
            desired_token_state(0, Diag::PoolOverflow),
            TokenState::Invalid(Diag::PoolOverflow.code())
        );
    }

    /// The not-ready contract: an instance whose source is committed but
    /// whose definition is unpublished must not encode to the same word as an
    /// instance that was never authored. Before E53 both produced 0, and a
    /// render clone has no other signal to separate them — no snapshot
    /// either way, and the `…`;0 Source expression evaluates to the slider's
    /// own 0.0 default. This assertion is what makes a scripted readiness
    /// poll (property 5) able to tell "still pending" from "nothing here".
    #[test]
    fn pending_publication_is_distinguishable_from_never_authored() {
        let never_authored = encode_token_state(desired_token_state(0, Diag::Ok));
        let pending = encode_token_state(desired_token_state(0, Diag::PublicationPending));

        assert_eq!(never_authored, 0.0);
        assert_ne!(pending, never_authored);
        assert_eq!(
            decode_token_state(pending),
            TokenState::Invalid(Diag::PublicationPending.code())
        );
        // And it stays separable from a resolved definition.
        assert_ne!(pending, encode_token_state(desired_token_state(42, Diag::Ok)));
    }
}

/// ADR-0030 layer inputs, exercised end-to-end through the real compile path.
#[cfg(test)]
mod layer_param_tests {
    use super::*;

    /// `graph` is spliced in so each case varies only the manifest line.
    fn source(graph: &str) -> String {
        format!(
            "@dynamicfx 1\n@graph\n{graph}\n@end\n@pass main\n\
             #version 450\n\
             // @param depth_map label:\"Depth Map\" hint:layer\n\
             layout(location = 0) in vec2 v_uv;\n\
             layout(location = 0) out vec4 outColor;\n\
             layout(set = 0, binding = 0) uniform texture2D u_in;\n\
             layout(set = 0, binding = 1) uniform sampler u_s;\n\
             layout(set = 0, binding = 2) uniform FxUniforms {{\n\
             \x20   vec2 u_resolution;\n\
             \x20   float u_time;\n\
             \x20   float u_frame;\n\
             }};\n\
             layout(set = 0, binding = 3) uniform texture2D u_depth;\n\
             void main() {{\n\
             \x20   float d = texture(sampler2D(u_depth, u_s), v_uv).r;\n\
             \x20   outColor = texture(sampler2D(u_in, u_s), v_uv + vec2(d * 0.1, 0.0));\n\
             }}\n\
             @endpass\n"
        )
    }

    fn compile(graph: &str) -> (Diag, String, Option<(u64, Arc<CompiledEffect>)>) {
        evaluate_committed_source(frontend::LanguageId::GLSL, &source(graph), None)
    }

    /// The core of ADR-0030 §1/§3: a layer name is a legal pass input that no
    /// pass writes. Before this, the "every input has a writer" rule made it
    /// an E6.
    #[test]
    fn layer_name_is_a_legal_input_without_a_writer() {
        let (code, status, compiled) = compile("pass main: input, depth_map -> output");
        assert_eq!(code, Diag::Ok, "{status}");
        let (_, effect) = compiled.expect("layer graph should compile");
        let layer_params: Vec<_> = effect
            .definition
            .params
            .iter()
            .filter(|p| p.ty == definition::param::ShaderParamType::Layer)
            .collect();
        assert_eq!(layer_params.len(), 1);
        assert_eq!(layer_params[0].id.as_str(), "depth_map");
    }

    /// It must land in the Layer pool, not borrow a Float/Color slot.
    #[test]
    fn layer_param_binds_to_the_layer_pool() {
        let (_, _, compiled) = compile("pass main: input, depth_map -> output");
        let (_, effect) = compiled.expect("layer graph should compile");
        let index = effect
            .definition
            .params
            .iter()
            .position(|p| p.id.as_str() == "depth_map")
            .expect("declared");
        let slots = &effect.definition.binding.bindings[index].slots;
        assert_eq!(slots.len(), 1, "a layer consumes exactly one slot");
        assert_eq!(slots[0].kind, binding::PoolKind::Layer);
    }

    #[test]
    fn layer_name_cannot_be_written_or_name_a_pass() {
        let (code, status, _) = compile("pass main: input -> depth_map\npass o: depth_map -> output");
        assert_eq!(code, Diag::EnvelopeSyntax, "{status}");

        let (code, status, _) = compile("pass depth_map: input -> output");
        assert_eq!(code, Diag::EnvelopeSyntax, "{status}");
    }

    /// ADR-0030 §6: fail closed rather than silently reuse the requested
    /// frame's layer pixels for every re-simulated iteration.
    #[test]
    fn layer_input_in_a_temporal_graph_is_refused() {
        let (code, status, compiled) = compile("pass main: input, depth_map, prev -> output");
        // Rejected for the temporal combination specifically — the graph is
        // otherwise valid, so an E6 here would mean the fixture drifted.
        assert_eq!(code, Diag::LayerInTemporalGraph, "{status}");
        assert!(compiled.is_none());
        assert!(status.contains("depth_map"), "names the offending input: {status}");
    }

    /// A name cannot be both a layer input and an FxUniforms member — the two
    /// would compete for one ParamId.
    #[test]
    fn hint_layer_on_a_uniform_member_is_rejected() {
        let source = source("pass main: input, depth_map -> output")
            .replace("    float u_frame;\n", "    float u_frame;\n    float depth_map;\n");
        let (code, status, compiled) =
            evaluate_committed_source(frontend::LanguageId::GLSL, &source, None);
        assert_eq!(code, Diag::ParamRejected, "{status}");
        assert!(compiled.is_none());
    }
}

/// ADR-0035 paths, through the same compile path as layer inputs and
/// gradients. That they need no new grammar, no new binding rule and no new
/// declaration mechanism is the decision being tested here, as much as the
/// behaviour is.
#[cfg(test)]
mod path_param_tests {
    use super::*;

    fn source(graph: &str) -> String {
        format!(
            "@dynamicfx 1
@graph
{graph}
@end
@pass main
             #version 450
             // @param outline label:\"Outline\" hint:path
             layout(location = 0) in vec2 v_uv;
             layout(location = 0) out vec4 outColor;
             layout(set = 0, binding = 0) uniform texture2D u_in;
             layout(set = 0, binding = 1) uniform sampler u_s;
             layout(set = 0, binding = 2) uniform FxUniforms {{
                 vec2 u_resolution;
                 float u_time;
                 float u_frame;
             }};
             layout(set = 0, binding = 3) uniform texture2D u_path;
             void main() {{
                 vec2 v0 = texelFetch(sampler2D(u_path, u_s), ivec2(0, 0), 0).xy;
                 float n = float(textureSize(sampler2D(u_path, u_s), 0).x);
                 float d = distance(v_uv, v0) * n;
                 outColor = texture(sampler2D(u_in, u_s), v_uv) * d;
             }}
             @endpass
"
        )
    }

    fn compile(graph: &str) -> (Diag, String, Option<(u64, Arc<CompiledEffect>)>) {
        evaluate_committed_source(frontend::LanguageId::GLSL, &source(graph), None)
    }

    /// ADR-0035 §2: a path name is a legal pass input that no pass writes, and
    /// it lands in the Path pool rather than borrowing another kind's slot.
    #[test]
    fn path_named_in_the_graph_binds_to_the_path_pool() {
        let (code, status, compiled) = compile("pass main: input, outline -> output");
        assert_eq!(code, Diag::Ok, "{status}");
        let (_, effect) = compiled.expect("path graph should compile");
        let index = effect
            .definition
            .params
            .iter()
            .position(|p| p.id.as_str() == "outline")
            .expect("declared");
        assert_eq!(
            effect.definition.params[index].ty,
            definition::param::ShaderParamType::Path
        );
        let slots = &effect.definition.binding.bindings[index].slots;
        assert_eq!(slots.len(), 1, "a path consumes exactly one slot");
        assert_eq!(slots[0].kind, binding::PoolKind::Path);
        // It must reach the render side as a path source, not as some other
        // external that happens to occupy the same ordinal.
        assert_eq!(effect.externals, vec![ExternalSource::Path { path_index: 0 }]);
    }

    /// §2 again, from the other side: read-only means read-only.
    #[test]
    fn path_name_cannot_be_written_or_name_a_pass() {
        let (code, status, _) = compile("pass main: input -> outline
pass o: outline -> output");
        assert_eq!(code, Diag::EnvelopeSyntax, "{status}");

        let (code, status, _) = compile("pass outline: input -> output");
        assert_eq!(code, Diag::EnvelopeSyntax, "{status}");
    }

    /// §7: refused for ADR-0030 §6's reason — re-simulation would need the
    /// path checked out at every iterated frame, a cost never measured.
    #[test]
    fn path_input_in_a_temporal_graph_is_refused() {
        let (code, status, compiled) = compile("pass main: input, outline, prev -> output");
        assert_eq!(code, Diag::LayerInTemporalGraph, "{status}");
        assert!(compiled.is_none());
        assert!(status.contains("outline"), "names the offending input: {status}");
        assert!(status.contains("path input"), "says which kind it was: {status}");
    }

    /// A name cannot be both a path input and an FxUniforms member.
    #[test]
    fn hint_path_on_a_uniform_member_is_rejected() {
        let source = source("pass main: input, outline -> output")
            .replace("    float u_frame;
", "    float u_frame;
    float outline;
");
        let (code, status, compiled) =
            evaluate_committed_source(frontend::LanguageId::GLSL, &source, None);
        assert_eq!(code, Diag::ParamRejected, "{status}");
        assert!(compiled.is_none());
    }
}

/// ADR-0031/0032 gradients, through the same compile path as layer inputs —
/// which is the point of ADR-0032: one rule, one code path, two kinds.
#[cfg(test)]
mod gradient_param_tests {
    use super::*;

    fn source(graph: &str, annotations: &str, extra_binding: &str) -> String {
        format!(
            "@dynamicfx 1\n@graph\n{graph}\n@end\n@pass main\n\
             #version 450\n{annotations}\
             layout(location = 0) in vec2 v_uv;\n\
             layout(location = 0) out vec4 outColor;\n\
             layout(set = 0, binding = 0) uniform texture2D u_in;\n\
             layout(set = 0, binding = 1) uniform sampler u_s;\n\
             layout(set = 0, binding = 2) uniform FxUniforms {{\n\
             \x20   vec2 u_resolution;\n\
             \x20   float u_time;\n\
             \x20   float u_frame;\n\
             }};\n\
             layout(set = 0, binding = 3) uniform texture2D u_ramp;\n{extra_binding}\
             void main() {{\n\
             \x20   float t = texture(sampler2D(u_in, u_s), v_uv).r;\n\
             \x20   outColor = texture(sampler2D(u_ramp, u_s), vec2(t, 0.5));\n\
             }}\n\
             @endpass\n"
        )
    }

    const RAMP: &str = "// @param heat_ramp label:\"Heat Ramp\" hint:gradient\n";

    #[test]
    fn gradient_named_in_the_graph_binds_to_the_gradient_pool() {
        let (code, status, compiled) = evaluate_committed_source(
            frontend::LanguageId::GLSL,
            &source("pass main: input, heat_ramp -> output", RAMP, ""),
            None,
        );
        assert_eq!(code, Diag::Ok, "{status}");
        let (_, effect) = compiled.expect("gradient graph should compile");
        let index = effect
            .definition
            .params
            .iter()
            .position(|p| p.id.as_str() == "heat_ramp")
            .expect("declared");
        assert_eq!(
            effect.definition.params[index].ty,
            definition::param::ShaderParamType::Gradient
        );
        let slots = &effect.definition.binding.bindings[index].slots;
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].kind, binding::PoolKind::Gradient);
    }

    /// ADR-0032's whole reason for existing: layers and gradients take their
    /// bindings from one rule — graph order — so a pass reading both gets
    /// bindings 3 and 4 in the order the manifest names them.
    #[test]
    fn a_layer_and_a_gradient_share_one_binding_rule() {
        let annotations = format!("{RAMP}// @param depth_map hint:layer\n");
        let (code, status, compiled) = evaluate_committed_source(
            frontend::LanguageId::GLSL,
            &source(
                "pass main: input, heat_ramp, depth_map -> output",
                &annotations,
                "layout(set = 0, binding = 4) uniform texture2D u_depth;\n",
            ),
            None,
        );
        assert_eq!(code, Diag::Ok, "{status}");
        let (_, effect) = compiled.expect("mixed graph should compile");
        // Graph order decides the ordinals, so the gradient (named first) is
        // external 0 and the layer is external 1.
        assert!(matches!(
            effect.externals.as_slice(),
            [ExternalSource::Gradient { .. }, ExternalSource::Layer { .. }]
        ));
    }

    #[test]
    fn gradient_cannot_be_written_or_name_a_pass() {
        let (code, _, _) = evaluate_committed_source(
            frontend::LanguageId::GLSL,
            &source("pass heat_ramp: input -> output", RAMP, ""),
            None,
        );
        assert_eq!(code, Diag::EnvelopeSyntax);
    }
}

/// The shipped `examples/` sources are compiled by the real pipeline here, so
/// a grammar, ABI, or annotation change that would break a user's copy-paste
/// breaks the build first. These are the exact bytes in the public repo —
/// `include_str!` means the test cannot drift from the file it documents.
#[cfg(test)]
mod example_tests {
    use super::*;

    fn compiles(name: &str, source: &str) {
        let (code, status, compiled) =
            evaluate_committed_source(frontend::LanguageId::GLSL, source, None);
        assert!(
            compiled.is_some(),
            "examples/{name} failed to compile: E{} {status}",
            code.code()
        );
        assert_eq!(code, Diag::Ok, "examples/{name}: {status}");
    }

    #[test]
    fn thermal_example_compiles() {
        compiles("thermal.glsl", include_str!("../examples/thermal.glsl"));
    }

    #[test]
    fn orb_example_compiles() {
        compiles("orb.glsl", include_str!("../examples/orb.glsl"));
    }

    #[test]
    fn apple_thermal_example_compiles() {
        compiles("apple-thermal.glsl", include_str!("../examples/apple-thermal.glsl"));
    }

    #[test]
    fn ink_bleed_example_compiles() {
        compiles("ink-bleed.glsl", include_str!("../examples/ink-bleed.glsl"));
    }
}

/// Failed observations keyed by attempt fingerprint, so the idle sync can
/// publish the real diagnostic code for a source it cannot recompile itself.
/// Session-local; successful compiles remove their entry.
fn failure_codes() -> &'static Mutex<TokenMap<u64, u16>> {
    static FAILURES: OnceLock<Mutex<TokenMap<u64, u16>>> = OnceLock::new();
    FAILURES.get_or_init(|| Mutex::new(TokenMap::new()))
}

pub(crate) fn failure_code_for(fp: u64) -> Option<u16> {
    failure_codes().lock().ok()?.get(&fp).copied()
}

struct Global {
    plugin_id: OnceLock<ae::aegp::PluginId>,
    /// Thread the plugin was set up on (= AE main thread). AEGP suite calls
    /// are only legal here.
    main_thread: std::thread::ThreadId,
    /// The non-AEGP idle registration has no unregister token and its refcon
    /// is process-lived; this flag turns late callbacks into no-ops.
    idle_alive: Arc<AtomicBool>,
    idle_registered: bool,
}

impl Default for Global {
    fn default() -> Self {
        // The macro constructs the global during PF_Cmd_GLOBAL_SETUP on the
        // main thread with PICA already initialized — the documented moment
        // for AEGP_RegisterWithAEGP.
        let plugin_id = OnceLock::new();
        if let Ok(u) = Utility::new() {
            if let Ok(id) = u.register_with_aegp(MATCH_NAME) {
                let _ = plugin_id.set(id);
                diag::log("registered with AEGP at global setup");
            }
        }
        Self {
            plugin_id,
            main_thread: std::thread::current().id(),
            idle_alive: Arc::new(AtomicBool::new(true)),
            idle_registered: false,
        }
    }
}

impl Drop for Global {
    fn drop(&mut self) {
        self.idle_alive.store(false, Ordering::Release);
    }
}

impl Global {
    fn plugin_id(&self) -> Result<ae::aegp::PluginId, Error> {
        if let Some(id) = self.plugin_id.get() {
            return Ok(*id);
        }
        let id = Utility::new()?.register_with_aegp(MATCH_NAME)?;
        Ok(*self.plugin_id.get_or_init(|| id))
    }
}

ae::define_effect!(Global, LocalMutex, ParamKey);

type LocalMutex = Mutex<Local>;

struct Local {
    /// Status text last written into the Status parameter name.
    status: String,
    /// Desired status text from the most recent observation. Observation can
    /// happen in contexts that must not touch parameters (idle bridge,
    /// render), so the text lands on the next UI callback.
    status_text: String,
    /// Diagnostic class behind `status_text` (ADR-0015 registry).
    status_code: Diag,
    /// Fingerprint of the last observation attempt, so repeated callbacks
    /// skip redundant frontend runs. The Compile button forces a re-run.
    last_attempt: Option<u64>,
    /// Session fingerprint of the currently resolved definition (0 = none).
    token: u64,
    compiled: Option<Arc<CompiledEffect>>,
    pipelines: Option<render::PipelineSet>,
    /// Restored ADR-0016 snapshot: the render clone's authority and the UI
    /// side's slot-inheritance seed. Never overrides a fresh observation.
    snapshot: Option<persistence::Snapshot>,
    /// SnapshotSchemaUnknown refuses implicit re-binding (ADR-0016 §1);
    /// only an explicit Compile clears this.
    block_rebind: bool,
    /// Token whose binding the slot names were last configured for. Kept
    /// separate from `visibility_token` so a transient failure of either
    /// path retries without redoing the other (prototype lesson).
    configured_token: Option<u64>,
    visibility_token: Option<u64>,
    /// Cached GPU frame resources (M7 item 1); rebuilt automatically when
    /// token/depth/size/plan shape change. Guarded by the instance lock the
    /// whole render holds, so MFR clones never share one.
    frame_cache: Option<render::FrameCache>,
    /// Reusable conversion scratch (AE→working and working→AE) — the
    /// per-render megabyte allocations showed up in the baseline totals.
    scratch_in: Vec<u8>,
    scratch_out: Vec<u8>,
}

impl Default for Local {
    fn default() -> Self {
        Self {
            // Matches the name declared at PARAMS_SETUP so the first UI
            // callback does not rename for nothing.
            status: "idle".to_string(),
            status_text: "idle".to_string(),
            status_code: Diag::Ok,
            last_attempt: None,
            token: 0,
            compiled: None,
            pipelines: None,
            snapshot: None,
            block_rebind: false,
            configured_token: None,
            visibility_token: None,
            frame_cache: None,
            scratch_in: Vec::new(),
            scratch_out: Vec::new(),
        }
    }
}

thread_local! {
    /// Smart-render ROI hand-off (window origin in layer space): set by the
    /// SmartRender arm, taken at render entry ON THE SAME THREAD. A field on
    /// `Local` would race concurrent frames of one instance under MFR
    /// (`SUPPORTS_THREADED_RENDERING`, ADR-0023 §4).
    static SMART_WINDOW: std::cell::Cell<Option<(i32, i32)>> =
        const { std::cell::Cell::new(None) };

    /// ADR-0030 layer pixels for the frame being rendered on THIS thread,
    /// parallel to `TexSlot::Layer` ordinals. Same reasoning as SMART_WINDOW:
    /// an instance field would race concurrent MFR frames. Owned copies —
    /// the checked-out `Layer` borrows the callbacks and cannot outlive the
    /// SmartRender arm, and the copy cost is visible in the upload span.
    /// Layer checkout ids `PF_CHECKOUT_LAYER` actually accepted this frame.
    /// PreRender fills it, SmartRender reads and clears it — the two run on the
    /// same thread per frame, like `SMART_LAYERS` beside it.
    static SMART_CHECKOUTS: std::cell::RefCell<Vec<u32>> =
        const { std::cell::RefCell::new(Vec::new()) };

    static SMART_LAYERS: std::cell::RefCell<Vec<Option<ExternalPixels>>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// FNV-1a over an observation kind tag, the selected language, and the
/// observed text; change detection only, never persisted.
fn observation_fingerprint(kind: u8, language: LanguageId, text: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in [kind]
        .into_iter()
        .chain(language.0.to_le_bytes())
        .chain(text.bytes())
    {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

impl AdobePluginGlobal for Global {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<ParamKey>,
        _: ae::InData,
        _: ae::OutData,
    ) -> Result<(), Error> {
        diag::log("params setup: ADR-0013 topology");
        host::params::setup(params)?;
        diag::log("params setup: complete");
        Ok(())
    }

    fn handle_command(
        &mut self,
        command: ae::Command,
        in_data: ae::InData,
        _: ae::OutData,
        // The global handler took `params` only for the ADR-0031 §7 custom-UI
        // editor, which is gone. Everything else that touches parameters runs
        // on the instance side, where the sequence handle is available.
        _params: &mut ae::Parameters<ParamKey>,
    ) -> Result<(), ae::Error> {
        // The §5.3 idle observer: scripted expression writes never arrive as
        // UserChangedParam (TR-M0-005), so a main-thread idle scan gives each
        // instance its observation opportunity and mirrors the session token.
        if matches!(command, ae::Command::GlobalSetup) && !self.idle_registered {
            let plugin_id = self.plugin_id()?;
            let state = host::idle::IdleState::new(
                plugin_id,
                in_data.pica_basic_suite_ptr(),
                self.main_thread,
                Arc::clone(&self.idle_alive),
            );
            ae::aegp::suites::RegisterNonAegp::new()?.register_idle_hook(
                plugin_id,
                Box::new(host::idle::idle_callback),
                state,
            )?;
            self.idle_registered = true;
            diag::log("AEGP idle hook registered");
        }
        // Visibility for the smart path's sequence-data availability: the
        // define_effect dispatch silently skips instance commands (including
        // SmartRender) when both the raw pointer and the const-suite path
        // yield null, which leaves the output world untouched.
        if matches!(
            command,
            ae::Command::SmartPreRender { .. } | ae::Command::SmartRender { .. }
        ) {
            let raw_null = unsafe { (*in_data.as_ptr()).sequence_data.is_null() };
            let const_ok = in_data
                .effect()
                .const_sequence_data()
                .map(|p| !p.is_null())
                .unwrap_or(false);
            let smart = matches!(command, ae::Command::SmartRender { .. });
            // Per-render log policy (M7): serialized file appends on every
            // render cost real time under MFR — opt-in only.
            diag::verbose(&format!(
                "smart cmd (render={smart}): seq_raw_null={raw_null} const_seq_ok={const_ok}"
            ));
        }
        Ok(())
    }
}

/// What one observation of the Source expression yielded.
enum Observation {
    NoExpression,
    NotSourceBlock,
    Committed(String),
}

/// AEGP effect references are caller-owned and the Rust wrapper is Copy with
/// no Drop implementation. Keep acquisition and disposal paired even when
/// expression access fails part-way through.
fn with_current_effect_ref<T>(
    plugin: &mut PluginState,
    f: impl FnOnce(&ae::aegp::EffectRefHandle) -> Result<T, Error>,
) -> Result<T, Error> {
    let plugin_id = plugin
        .global
        .plugin_id()
        .inspect_err(|e| diag::log(&format!("plugin_id failed: {e:?}")))?;
    let pf_iface = ae::aegp::suites::PFInterface::new()
        .inspect_err(|e| diag::log(&format!("PFInterface suite failed: {e:?}")))?;
    let effect_suite = ae::aegp::suites::Effect::new()
        .inspect_err(|e| diag::log(&format!("Effect suite failed: {e:?}")))?;
    let effect_ref = pf_iface
        .new_effect_for_effect(plugin.in_data.effect_ref(), plugin_id)
        .inspect_err(|e| diag::log(&format!("new_effect_for_effect failed: {e:?}")))?;
    let result = f(&effect_ref);
    let dispose_result = effect_suite.dispose_effect(&effect_ref);
    match (result, dispose_result) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(err), _) => Err(err),
        (Ok(_), Err(err)) => Err(err),
    }
}

/// Read the committed expression from the Source stream (main thread only)
/// and extract the backtick-wrapped committed source.
fn observe_source(plugin: &mut PluginState) -> Result<Observation, Error> {
    let plugin_id = plugin.global.plugin_id()?;
    let index = plugin
        .params
        .index(ParamKey::Source)
        .ok_or(Error::InvalidIndex)? as i32;

    with_current_effect_ref(plugin, |effect_ref| {
        let stream_suite = Stream::new()
            .inspect_err(|e| diag::log(&format!("Stream suite failed: {e:?}")))?;
        let stream = stream_suite
            .new_effect_stream_by_index(effect_ref, plugin_id, index)
            .inspect_err(|e| diag::log(&format!("new_effect_stream_by_index failed: {e:?}")))?;

        let expr_state = stream_suite
            .expression_state(&stream, plugin_id)
            .inspect_err(|e| diag::log(&format!("expression_state failed: {e:?}")))?;
        if !expr_state {
            return Ok(Observation::NoExpression);
        }

        let expression = stream_suite
            .expression_string(&stream, plugin_id)
            .inspect_err(|e| diag::log(&format!("expression_string failed: {e:?}")))?;
        Ok(match source::extract_source(&expression) {
            Some(source) => Observation::Committed(source),
            None => Observation::NotSourceBlock,
        })
    })
}

/// The selected language from the popup, mapped through the stable-ID
/// registry (ADR-0010).
fn selected_language(plugin: &mut PluginState) -> Option<LanguageId> {
    let position = plugin
        .params
        .get(ParamKey::Language)
        .ok()?
        .as_popup()
        .ok()?
        .value();
    frontend::language_from_popup_position(u32::try_from(position).ok()?)
}

/// Observation core: read language + expression, classify, compile, publish
/// into the process registry, and update `local` (including the desired
/// status text). Touches NO parameters, so it is legal from UI callbacks,
/// CompletelyGeneral, and main-thread render. Returns whether the attempt
/// changed anything.
fn observe_core(plugin: &mut PluginState, local: &mut Local, force: bool) -> Result<bool, Error> {
    // SnapshotSchemaUnknown refuses implicit re-binding: fresh allocation
    // could silently misalign keyframes (ADR-0016 §1). Explicit Compile
    // (force) is the user's consent to re-bind.
    if local.block_rebind && !force {
        return Ok(false);
    }
    if force {
        local.block_rebind = false;
    }

    let Some(language) = selected_language(plugin) else {
        local.status_code = Diag::LanguageUnknown;
        local.status_text = "language selection unknown".to_string();
        return Ok(true);
    };

    let observation = observe_source(plugin)?;
    let (attempt, code, status, compiled) = match observation {
        Observation::NoExpression => (
            observation_fingerprint(1, language, ""),
            Diag::NoExpression,
            "no expression on Source".to_string(),
            None,
        ),
        Observation::NotSourceBlock => (
            observation_fingerprint(2, language, ""),
            Diag::NotSourceBlock,
            "expression is not a `...`;0 source block".to_string(),
            None,
        ),
        Observation::Committed(committed) => {
            let attempt = observation_fingerprint(3, language, &committed);
            if !force && local.last_attempt == Some(attempt) {
                return Ok(false);
            }
            // Slots follow stable IDs across definition changes (ADR-0013
            // §2): the previous plan — live binding or restored snapshot —
            // seeds reuse so keyframes survive edits and reopen alike.
            let previous = local
                .compiled
                .as_ref()
                .map(|c| c.definition.binding.clone())
                .or_else(|| local.snapshot.as_ref().map(|s| s.to_previous_plan()));
            let (code, status, compiled) =
                evaluate_committed_source(language, &committed, previous.as_ref());
            // Record the outcome for the idle sync's Invalid publication.
            let fp = session_token(language, &committed);
            if let Ok(mut failures) = failure_codes().lock() {
                if compiled.is_some() {
                    failures.remove(&fp);
                } else {
                    failures.insert(fp, code.code());
                }
            }
            (attempt, code, status, compiled)
        }
    };

    if !force && local.last_attempt == Some(attempt) {
        return Ok(false);
    }
    local.last_attempt = Some(attempt);
    local.status_code = code;
    local.status_text = status;
    // Fail closed: any non-compiling observation clears the resolved state;
    // render passes through instead of reviving a stale definition.
    match compiled {
        Some((token, effect)) => {
            let published = registry_insert(token, Arc::clone(&effect));
            if !published {
                // The registry refused this fingerprint (ADR-0015 §"Costs":
                // the insert guards against cross-content collisions). The
                // effect compiled, but no render clone can resolve it, so the
                // frame passes through. Say that instead of leaving a
                // success status over a pass-through render.
                local.status_code = Diag::PublicationPending;
                local.status_text =
                    "compiled, but publication was refused; press Compile".to_string();
            }
            local.token = if published { token } else { 0 };
            local.compiled = Some(effect);
            local.pipelines = None;
        }
        None => {
            local.token = 0;
            local.compiled = None;
            local.pipelines = None;
        }
    }
    Ok(true)
}

/// Classify committed source (ADR-0012), parse the envelope grammar when
/// present (ADR-0018), run the selected frontend per pass, and lower into a
/// unified graph — raw single-pass input is an implicit one-pass manifest
/// (ADR-0003). On success returns the token + compiled effect.
fn evaluate_committed_source(
    language: LanguageId,
    committed: &str,
    previous: Option<&binding::BindingPlan>,
) -> (Diag, String, Option<(u64, Arc<CompiledEffect>)>) {
    let (manifest, bodies) = match envelope::classify(committed) {
        Err(SourceClassError::Oversize { bytes }) => {
            return (
                Diag::SourceOversize,
                format!("source exceeds the 4 MiB cap ({bytes} bytes); nothing was compiled"),
                None,
            )
        }
        Err(SourceClassError::EnvelopeMalformed) => {
            return (
                Diag::EnvelopeMalformed,
                "envelope marker line is malformed (expected `@dynamicfx <version>`)".to_string(),
                None,
            )
        }
        Ok(SourceClass::Envelope { version: 1 }) => {
            match frontend::grammar::parse_envelope(committed) {
                Ok(env) => (env.passes, env.bodies),
                Err(e) => {
                    return (
                        Diag::EnvelopeSyntax,
                        format!("envelope line {}: {}", e.line, e.message),
                        None,
                    )
                }
            }
        }
        Ok(SourceClass::Envelope { version }) => {
            return (
                Diag::EnvelopeUnsupported,
                format!("envelope v{version} is not supported (this build implements v1)"),
                None,
            )
        }
        Ok(SourceClass::Raw) => {
            (definition::effect::single_pass_manifest(), vec![committed.to_string()])
        }
    };

    let layer_names = frontend::annotation::layer_param_names(committed);
    let gradient_names = frontend::annotation::gradient_param_names(committed);
    let path_names = frontend::annotation::path_param_names(committed);
    let uses_prev = manifest
        .iter()
        .any(|p| p.inputs.iter().any(|i| i == frontend::grammar::RES_PREV));
    // ADR-0030 §6 and ADR-0035 §7 refuse for the same reason and therefore
    // share the diagnostic: windowed re-simulation would need the resource at
    // every iterated frame, a cost never measured. Gradients are exempt — a
    // bake is arithmetic, with no host round trip to repeat.
    let used_externals: Vec<(&str, &String)> = manifest
        .iter()
        .flat_map(|p| p.inputs.iter())
        .filter_map(|i| {
            if layer_names.iter().any(|l| l == i) {
                Some(("layer input", i))
            } else if path_names.iter().any(|l| l == i) {
                Some(("path input", i))
            } else {
                None
            }
        })
        .collect();
    if uses_prev && !used_externals.is_empty() {
        let (what, name) = used_externals[0];
        return (
            Diag::LayerInTemporalGraph,
            format!(
                "{what} `{name}` cannot be used in a graph that reads `prev`; \
                 windowed re-simulation would need it at every iterated frame"
            ),
            None,
        );
    }

    let Some(frontend_impl) = frontend::frontend_for(language) else {
        return (
            Diag::LanguageUnknown,
            format!("language id {} is not implemented", language.0),
            None,
        );
    };
    // Annotations parse once over the whole committed text (ADR-0018 §6).
    let annotations = match frontend::annotation::parse_annotations(committed) {
        Ok(annotations) => annotations,
        Err(e) => {
            return (
                Diag::ParamRejected,
                format!("@param line {}: {}", e.line, e.message),
                None,
            )
        }
    };

    let mut pass_modules = Vec::with_capacity(manifest.len());
    for (pass, body) in manifest.iter().zip(&bodies) {
        match frontend_impl.parse_module(body, &annotations, pass.inputs.len()) {
            Ok(module) => pass_modules.push(module),
            Err(err) => {
                let (code, text) = frontend_error_status(err);
                return (code, format!("pass `{}`: {text}", pass.name), None);
            }
        }
    }

    let mut spirvs = Vec::with_capacity(pass_modules.len());
    for (pass, module) in manifest.iter().zip(&pass_modules) {
        match render::compile_spirv(&module.module) {
            Ok(spirv) => spirvs.push(spirv),
            Err(e) => {
                return (
                    Diag::SpirvEmit,
                    format!("pass `{}`: SPIR-V emission failed: {e}", pass.name),
                    None,
                )
            }
        }
    }

    // ADR-0030/0032: layer inputs and gradients are declared by annotation and
    // never reflected,
    // so they are appended here — after the module's own members, in each
    // pass's graph-input order — to reach pool allocation and the AE control
    // list. `lower_graph`'s effect-wide merge dedupes a layer read by several
    // passes into one parameter, exactly as it does for uniform members.
    let mut per_pass_params: Vec<Vec<definition::param::ParamDeclaration>> =
        pass_modules.iter().map(|m| m.params.clone()).collect();
    for (params, pass) in per_pass_params.iter_mut().zip(manifest.iter()) {
        for input in &pass.inputs {
            let ty = if layer_names.iter().any(|l| l == input) {
                definition::param::ShaderParamType::Layer
            } else if gradient_names.iter().any(|g| g == input) {
                definition::param::ShaderParamType::Gradient
            } else if path_names.iter().any(|g| g == input) {
                definition::param::ShaderParamType::Path
            } else {
                continue;
            };
            let Ok(id) = definition::param::ParamId::new(input) else { continue };
            let ui = annotations
                .get(input)
                .map(|a| definition::param::ParamUiMeta {
                    label: a.label.clone(),
                    ..Default::default()
                })
                .unwrap_or_default();
            let aliases = annotations.get(input).map(|a| a.aliases.clone()).unwrap_or_default();
            params.push(definition::param::ParamDeclaration { id, ty, aliases, ui });
        }
    }
    let (def, maps) = match definition::effect::lower_graph(
        language,
        &manifest,
        &bodies,
        &per_pass_params,
        previous,
    ) {
        Ok(result) => result,
        Err(definition::effect::LowerError::ParamTypeConflict { name }) => {
            return (
                Diag::ParamRejected,
                format!("parameter `{name}` has conflicting types across passes"),
                None,
            )
        }
        Err(definition::effect::LowerError::Binding(err)) => {
            let code = match &err {
                binding::BindingError::PoolOverflow { .. } => Diag::PoolOverflow,
                binding::BindingError::Declarations(_) => Diag::AliasConflict,
            };
            return (code, format!("definition rejected: {err:?}"), None);
        }
    };

    let exec_plan = plan::build_plan(&manifest, alias_enabled());
    let passes = pass_modules
        .into_iter()
        .zip(spirvs)
        .zip(maps)
        .map(|((module, spirv), param_map)| CompiledPass {
            spirv,
            layout: module.layout,
            param_map,
            extra_input_bindings: module.extra_input_bindings,
        })
        .collect();

    let status = format!(
        "compiled: {} pass{}, {} params",
        def.graph.passes.len(),
        if def.graph.passes.len() == 1 { "" } else { "es" },
        def.params.len()
    );
    let uses_prev = exec_plan
        .steps
        .iter()
        .any(|s| s.inputs.iter().any(|i| *i == plan::TexSlot::History));
    let window = if uses_prev {
        match frontend::annotation::parse_window(committed) {
            Ok(declared) => Some(declared.unwrap_or(frontend::annotation::WINDOW_DEFAULT)),
            Err(e) => {
                return (
                    Diag::ParamRejected,
                    format!("@window line {}: {}", e.line, e.message),
                    None,
                )
            }
        }
    } else {
        None
    };
    let token = session_token(language, committed);
    // ADR-0030: resolve each layer ordinal to the AE parameter index that
    // feeds it, once, here — the render path must not re-derive graph order
    // per frame, and PreRender needs the same indexes SmartRender will use.
    let declaration = host::params::declaration_order();
    let externals: Vec<ExternalSource> = plan::external_order(&manifest)
        .iter()
        .filter_map(|name| {
            let index = def.params.iter().position(|p| p.id.as_str() == name)?;
            let slot = def.binding.bindings.get(index)?.slots.first()?;
            let param_index = declaration
                .iter()
                .position(|k| *k == host::params::ParamKey::Pool(slot.kind, slot.index))?;
            match slot.kind {
                // +1: `param_index` is a declaration position, and every AE
                // parameter index is one higher (input layer at 0).
                binding::PoolKind::Layer => {
                    Some(ExternalSource::Layer { param_index: param_index + 1 })
                }
                binding::PoolKind::Gradient => {
                    Some(ExternalSource::Gradient { gradient_index: slot.index })
                }
                binding::PoolKind::Path => {
                    Some(ExternalSource::Path { path_index: slot.index })
                }
                // Unreachable: only these three kinds reach the graph as
                // resources. Dropping anything else keeps the ordinals aligned
                // with what the render side can actually supply.
                _ => None,
            }
        })
        .collect();
    let compiled = Arc::new(CompiledEffect {
        definition: def,
        passes,
        plan: exec_plan,
        window,
        source: committed.to_string(),
        externals,
    });
    (Diag::Ok, status, Some((token, compiled)))
}

fn frontend_error_status(err: frontend::FrontendError) -> (Diag, String) {
    match err {
        frontend::FrontendError::Parse(msg) => (Diag::GlslParse, format!("GLSL error: {msg}")),
        frontend::FrontendError::Abi(msg) => {
            (Diag::AbiViolation, format!("ABI v1 violation: {msg}"))
        }
        frontend::FrontendError::Param(msg) => {
            (Diag::ParamRejected, format!("parameter rejected: {msg}"))
        }
    }
}

/// Hide or show one slot stream via the AEGP DynamicStream suite — the same
/// mechanism built-in effects use for dynamic parameter visibility.
fn set_slot_hidden(
    plugin: &mut PluginState,
    effect_ref: &ae::aegp::EffectRefHandle,
    stream_index: i32,
    hidden: bool,
) -> Result<(), Error> {
    let plugin_id = plugin.global.plugin_id()?;
    let stream_suite = Stream::new()?;
    let dyn_suite = ae::aegp::suites::DynamicStream::new()?;
    let stream = stream_suite.new_effect_stream_by_index(effect_ref, plugin_id, stream_index)?;
    if std::env::var("DYNAMICFX_VIS_PROBE").is_ok() {
        let name = stream_suite
            .stream_name(&stream, plugin_id, false)
            .unwrap_or_else(|e| format!("<err {e:?}>"));
        diag::log(&format!(
            "vis probe: stream_index={stream_index} aegp_name=[{name}] -> hidden={hidden}"
        ));
    }
    dyn_suite.set_dynamic_stream_flag(&stream, ae::aegp::DynamicStreamFlags::Hidden, false, hidden)
}

/// **Known limitation, recorded rather than fixed (2026-08-16).** Pool slots
/// added after 0.0.2 — Layer, Gradient, Point 3D, Path — display their pool
/// name ("Mask 01") in Effect Controls instead of the shader's own label,
/// while every V1 kind displays the label correctly.
///
/// Measured on AE 2025 **and** 2026: from a single shader declaring both, the
/// Colour slot read back as `Tint` and the Point 3D slot beside it as
/// `Point 3D 01`, and a re-read a full idle window later was unchanged — so it
/// is not refresh timing. `PF_UpdateParamUI` returns success for these kinds
/// and AE ignores the name.
///
/// The obvious second route, `AEGP_SetStreamName` — the same AEGP-braces trick
/// that fixes the Hidden flag for `PF_Param_ANGLE` rows — **was tried and
/// hangs After Effects**: it is documented as Undoable, and calling it from the
/// slot-configure path froze the host before a single leg completed (artifact
/// `4FD125F9…`, harness timed out at 600 s with AE unresponsive). Shipping a
/// cosmetic label fix that can freeze a user's session is a bad trade, so the
/// route stays closed until the undo-group requirement is understood.
///
/// Consequence for users: a `hint:layer` / `hint:gradient` / `hint:point3d` /
/// `hint:path` control shows a generic name. Its value, keyframes, identity and
/// render behaviour are all unaffected.

/// Configure pool-slot UI from the resolved definition: bound slots take
/// their ParamId as the label (vec4 alpha companions get an " A" suffix) and
/// become visible; unbound slots return to their default names and hide.
/// UI contexts only, and never before a definition exists — changing
/// DynamicStream visibility while AE is still constructing the property tree
/// during addProperty() breaks the scripting API's child lookup (prototype
/// lesson, kept as a hard rule).
fn configure_slots(plugin: &mut PluginState, local: &mut Local) {
    // No definition (fresh instance, cleared source): hide the ENTIRE pool
    // once — an uncompiled effect exposing 104 unbound controls floods the
    // Effect Controls panel and measurably drags it (user report).
    let Some(compiled) = local.compiled.clone() else {
        if local.visibility_token != Some(0) {
            let empty = std::collections::HashMap::new();
            if apply_visibility(plugin, &empty).is_ok() {
                local.visibility_token = Some(0);
                local.configured_token = None;
                diag::log("slots hidden (no definition)");
            }
        }
        return;
    };
    let token = local.token;
    let configure_names = local.configured_token != Some(token);
    let configure_visibility = local.visibility_token != Some(token);
    if !configure_names && !configure_visibility {
        return;
    }

    let configs = slot_configs(&compiled.definition);

    let mut names_ok = true;
    if configure_names {
        // Every pool, not just the V1 ones: a bound `hint:layer` slot must
        // carry the shader's own name exactly as a bound float does. The
        // growth pools were left out of this loop when they were appended, so
        // layer inputs read as "Layer 01" whatever the shader called them.
        for (kind, capacity) in binding::all_pools() {
            // The Gradient slot is inert and permanently invisible
            // (ADR-0033 §6), so its label would never be seen. The gradient's
            // shader name goes on the stop rows below instead — the rows the
            // user actually edits.
            if *kind == binding::PoolKind::Gradient {
                continue;
            }
            for i in 0..*capacity {
                let slot = binding::SlotRef { kind: *kind, index: i };
                let config = configs.get(&slot);
                let label = config
                    .map(|c| c.label.clone())
                    .unwrap_or_else(|| host::params::default_slot_name(*kind, i));
                match plugin.params.get_mut(ParamKey::Pool(*kind, i)) {
                    Ok(mut p) => {
                        names_ok &= p.set_name(&label).is_ok();
                        // Range/default metadata for bound scalar slots
                        // (annotation-driven; display metadata only — the
                        // default VALUE is written by the idle observer to
                        // fresh bindings via AEGP).
                        //
                        // ADR-0037 §2: `@param min:/max:` is the SLIDER range.
                        // The SDK header on `PF_UpdateParamUI` lists
                        // slider_min/slider_max/precision/display_flags as the
                        // only slider fields it changes; the valid range is
                        // fixed at PARAMS_SETUP (wide, `params.rs`) and is what
                        // After Effects clamps a rendered value to AND what it
                        // validates a scripted setValue against. The valid_*
                        // writes below are a measured no-op on both paths:
                        // TR-0037-001 set `min:2 max:200` then setValue(0.3)
                        // and AE *accepted* it on 2025 and 2026. They stay
                        // only so the stored def is internally consistent;
                        // nothing reads them back. A binding without a
                        // declared range gets the display default slider range
                        // and the wide valid range, so a slot that once
                        // carried `2..200` does not hand its old range to the
                        // next parameter that lands on it.
                        if let Some(config) = config {
                            if let Ok(mut param) = p.as_param_mut() {
                                match &mut param {
                                    ae::Param::FloatSlider(f) => {
                                        let (valid, slider) = (
                                            host::params::POOL_FLOAT_VALID_RANGE,
                                            host::params::POOL_FLOAT_SLIDER_RANGE,
                                        );
                                        let (smin, smax, vmin, vmax) = match (config.min, config.max) {
                                            (Some(min), Some(max)) => (min, max, min, max),
                                            _ => (slider.0, slider.1, valid.0, valid.1),
                                        };
                                        f.set_slider_min(smin);
                                        f.set_slider_max(smax);
                                        f.set_valid_min(vmin);
                                        f.set_valid_max(vmax);
                                        if let Some(default) = config.default {
                                            f.set_default(default as f64);
                                        }
                                        // ADR-0028 belt-and-braces: keep the
                                        // display precision explicit on every
                                        // def write so no host path can zero
                                        // it back to integer stepping.
                                        f.set_precision(ae::Precision::Hundredths);
                                    }
                                    ae::Param::Slider(s) => {
                                        let (valid, slider) = (
                                            host::params::POOL_INT_VALID_RANGE,
                                            host::params::POOL_INT_SLIDER_RANGE,
                                        );
                                        let (smin, smax, vmin, vmax) = match (config.min, config.max) {
                                            (Some(min), Some(max)) => {
                                                (min as i32, max as i32, min as i32, max as i32)
                                            }
                                            _ => (slider.0, slider.1, valid.0, valid.1),
                                        };
                                        s.set_slider_min(smin);
                                        s.set_slider_max(smax);
                                        s.set_valid_min(vmin);
                                        s.set_valid_max(vmax);
                                        if let Some(default) = config.default {
                                            s.set_default(default as i32);
                                        }
                                    }
                                    ae::Param::Angle(a) => {
                                        if let Some(default) = config.default {
                                            a.set_default(default);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        names_ok &= p.update_param_ui().is_ok();
                    }
                    Err(e) => {
                        names_ok = false;
                        diag::log(&format!("slot name lookup failed: {e:?}"));
                    }
                }
            }
        }
        // ADR-0033 presentation: the gradient's shader name lands on the count
        // and stop rows, because the slot that would normally carry it is
        // invisible. PF caps a parameter name at 31 characters, so the label is
        // clipped up front rather than cut mid-suffix by the host.
        for g in 0..host::params::GRADIENTS {
            let slot = binding::SlotRef { kind: binding::PoolKind::Gradient, index: g };
            let label = configs
                .get(&slot)
                .map(|c| clip(&c.label, 20))
                .unwrap_or_else(|| format!("G{:02}", g + 1));
            if let Ok(mut p) = plugin.params.get_mut(ParamKey::GradientCount(g)) {
                names_ok &= p.set_name(&format!("{label} Stops")).is_ok();
                names_ok &= p.update_param_ui().is_ok();
            }
            for stop in 0..host::params::STOPS_PER_GRADIENT {
                for (field, suffix) in [
                    (host::params::GradientField::Position, "Pos"),
                    (host::params::GradientField::Color, "Color"),
                    (host::params::GradientField::Alpha, "Alpha"),
                ] {
                    let key = ParamKey::GradientStop(g, stop, field);
                    if let Ok(mut p) = plugin.params.get_mut(key) {
                        let name = format!("{label} {:02} {suffix}", stop + 1);
                        names_ok &= p.set_name(&name).is_ok();
                        names_ok &= p.update_param_ui().is_ok();
                    }
                }
            }
        }
    }

    let mut visibility_ok = true;
    if configure_visibility {
        visibility_ok = apply_visibility(plugin, &configs).is_ok();
    }

    if configure_names && names_ok {
        local.configured_token = Some(token);
    }
    if configure_visibility && visibility_ok {
        local.visibility_token = Some(token);
    }
    diag::log(&format!(
        "slots configured: {} bound (names_ok={names_ok}, visibility_ok={visibility_ok})",
        configs.len()
    ));
}

/// Clip a label to `n` characters on a char boundary. PF parameter names are
/// capped at 31 bytes, and a byte-wise cut could split a multi-byte character.
fn clip(label: &str, n: usize) -> String {
    label.chars().take(n).collect()
}

/// Apply hidden flags for every pool slot (bound = visible).
fn apply_visibility(
    plugin: &mut PluginState,
    configs: &std::collections::HashMap<binding::SlotRef, SlotConfig>,
) -> Result<(), Error> {
    let plugin_id = plugin.global.plugin_id()?;
    let pf_iface = ae::aegp::suites::PFInterface::new()?;
    let effect_suite = ae::aegp::suites::Effect::new()?;
    let effect_ref = pf_iface.new_effect_for_effect(plugin.in_data.effect_ref(), plugin_id)?;
    let mut result = Ok(());

    // ADR-0033 presentation. The reference gradient effect keeps every stop
    // parameter in the topology but shows only ONE stop group at a time — the
    // ramp bar selects, the group below edits. Listing all eight stops as 24
    // flat rows is the storage model leaking into the UI (user report,
    // 2026-08-15). Same machinery as the pool slots: a bound gradient shows
    // its count, its preview, and the selected stop's three rows; everything
    // else hides.
    for g in 0..host::params::GRADIENTS {
        let bound = configs
            .keys()
            .any(|s| s.kind == binding::PoolKind::Gradient && s.index == g);
        let live = plugin
            .params
            .get(ParamKey::GradientCount(g))
            .ok()
            .and_then(|p| p.as_float_slider().ok().map(|f| f.value().round() as usize))
            .unwrap_or(host::params::DEFAULT_LIVE_STOPS);
        for key in [ParamKey::GradientCount(g)] {
            if let Some(index) = host::params::stream_index_of(key) {
                if let Err(e) = set_slot_hidden(plugin, &effect_ref, index, !bound) {
                    diag::log(&format!("gradient {g} count hidden flag failed: {e:?}"));
                    result = Err(e);
                }
            }
            if let Ok(mut p) = plugin.params.get_mut(key) {
                let mut flags = p.ui_flags();
                if flags.contains(ae::ParamUIFlags::INVISIBLE) == bound {
                    flags.set(ae::ParamUIFlags::INVISIBLE, !bound);
                    p.set_ui_flags(flags);
                }
            }
        }
        for stop in 0..host::params::STOPS_PER_GRADIENT {
            // The count owns how many stop groups are on screen. With the
            // editor gone there is nothing to select a stop with, so showing
            // every live stop is the only presentation that leaves them all
            // reachable — and the count is already the value's own truth.
            let shown = bound && stop < live;
            for field in [
                host::params::GradientField::Position,
                host::params::GradientField::Color,
                host::params::GradientField::Alpha,
            ] {
                let key = ParamKey::GradientStop(g, stop, field);
                if let Some(index) = host::params::stream_index_of(key) {
                    if let Err(e) = set_slot_hidden(plugin, &effect_ref, index, !shown) {
                        diag::log(&format!("gradient {g} stop {stop} hidden flag failed: {e:?}"));
                        result = Err(e);
                    }
                }
                if let Ok(mut p) = plugin.params.get_mut(key) {
                    let mut flags = p.ui_flags();
                    if flags.contains(ae::ParamUIFlags::INVISIBLE) == shown {
                        flags.set(ae::ParamUIFlags::INVISIBLE, !shown);
                        p.set_ui_flags(flags);
                    }
                }
            }
        }
    }

    for (kind, capacity) in binding::all_pools() {
        for i in 0..*capacity {
            let slot = binding::SlotRef { kind: *kind, index: i };
            let Some(stream_index) = host::params::stream_index_of(ParamKey::Pool(*kind, i))
            else {
                continue;
            };
            // The Gradient slot is inert (ADR-0033 §6) — hidden whether or not
            // a shader binds it. Everything else shows exactly when bound,
            // which the growth pools never did: four "Layer" rows sat in the
            // panel of every instance, bound or not.
            let hidden = !configs.contains_key(&slot) || *kind == binding::PoolKind::Gradient;
            if let Err(e) = set_slot_hidden(plugin, &effect_ref, stream_index, hidden) {
                diag::log(&format!("slot hidden flag failed ({kind:?} {i}): {e:?}"));
                result = Err(e);
            }
            // Belt to the AEGP braces: the Effect Controls panel ignores the
            // dynamic-stream Hidden flag for PF_Param_ANGLE rows (measured:
            // flags land, AEGP names read back correctly, the dials render
            // anyway — and nameless). PF_PUI_INVISIBLE through
            // PF_UpdateParamUI is honored for every param type.
            if let Ok(mut p) = plugin.params.get_mut(ParamKey::Pool(*kind, i)) {
                let mut flags = p.ui_flags();
                if flags.contains(ae::ParamUIFlags::INVISIBLE) != hidden {
                    flags.set(ae::ParamUIFlags::INVISIBLE, hidden);
                    p.set_ui_flags(flags);
                    let _ = p.update_param_ui();
                }
            }
        }
    }
    let dispose = effect_suite.dispose_effect(&effect_ref);
    result.and(dispose)
}

/// Mirror the status text into the Status parameter's name. UI contexts only.
fn set_status(plugin: &mut PluginState, local: &mut Local, status: String) {
    // Failures carry their stable code up front; the 31-char PF name limit
    // truncates text, never the code (ADR-0015 §4).
    let status = diagnostics::status_text(local.status_code, &status);
    if local.status == status {
        return;
    }
    diag::log(&format!("status: {status}"));
    let updated = match plugin.params.get_mut(ParamKey::Status) {
        Ok(mut param) => {
            let label = format!("Status: {status}");
            let name_ok = param.set_name(&label).is_ok();
            let ui_ok = param.update_param_ui().is_ok();
            name_ok && ui_ok
        }
        Err(e) => {
            diag::log(&format!("get_mut(Status) failed: {e:?}"));
            false
        }
    };
    if updated {
        local.status = status;
    }
}

/// Publish the ADR-0015 token word into the StateToken parameter stream.
/// Legal only in a UserChangedParam context (ParamDef writes elsewhere are
/// ignored by AE); the idle observer mirrors it via AEGP for scripted paths.
fn publish_token_param(plugin: &mut PluginState, local: &Local) {
    let desired = encode_token_state(desired_token_state(local.token, local.status_code));
    if let Ok(mut p) = plugin.params.get_mut(ParamKey::StateToken) {
        if let Ok(mut param) = p.as_param_mut() {
            if let ae::Param::FloatSlider(def) = &mut param {
                if def.value() != desired {
                    def.set_value(desired);
                    p.set_value_changed();
                }
            }
        }
    }
}

/// Current values for every bound slot, in declaration order, encoded per
/// ABI v1 (value-encoding semantics are fixture-pinned at M2; v1 passes
/// sliders raw, colors as 0..1 RGB, points normalized to the frame,
/// angles in degrees).
fn read_bound_values(
    plugin: &mut PluginState,
    compiled: &CompiledEffect,
    width: usize,
    height: usize,
) -> Vec<[f32; 4]> {
    let defn = &compiled.definition;
    defn.params
        .iter()
        .zip(defn.binding.bindings.iter())
        .map(|(decl, bound)| {
            let mut out = [0f32; 4];
            let slot_key = |j: usize| {
                bound
                    .slots
                    .get(j)
                    .map(|s| ParamKey::Pool(s.kind, s.index))
            };
            match decl.ty {
                ShaderParamType::Float => {
                    if let Some(key) = slot_key(0) {
                        if let Ok(p) = plugin.params.get(key) {
                            if let Ok(f) = p.as_float_slider() {
                                out[0] = f.value() as f32;
                            }
                        }
                    }
                }
                ShaderParamType::AngleFloat => {
                    if let Some(key) = slot_key(0) {
                        if let Ok(p) = plugin.params.get(key) {
                            if let Ok(a) = p.as_angle() {
                                out[0] = a.value();
                            }
                        }
                    }
                }
                ShaderParamType::Int => {
                    if let Some(key) = slot_key(0) {
                        if let Ok(p) = plugin.params.get(key) {
                            if let Ok(s) = p.as_slider() {
                                out[0] = s.value() as f32;
                            }
                        }
                    }
                }
                ShaderParamType::Bool => {
                    if let Some(key) = slot_key(0) {
                        if let Ok(p) = plugin.params.get(key) {
                            if let Ok(c) = p.as_checkbox() {
                                out[0] = if c.value() { 1.0 } else { 0.0 };
                            }
                        }
                    }
                }
                ShaderParamType::Vec2 => {
                    if let Some(key) = slot_key(0) {
                        if let Ok(p) = plugin.params.get(key) {
                            if let Ok(pt) = p.as_point() {
                                let (px, py) = pt.value();
                                out[0] = px as f32 / width.max(1) as f32;
                                out[1] = py as f32 / height.max(1) as f32;
                            }
                        }
                    }
                }
                // ADR-0034 §3. `x` and `y` are normalized to the frame
                // exactly as Vec2 is, so a point and a point-3D mean the same
                // thing in the same shader; `z` is passed in pixels because
                // there is no third frame dimension to divide by, and
                // inventing one (height? the diagonal?) would be a convention
                // the shader author cannot predict. The asymmetry is
                // documented, not hidden.
                ShaderParamType::Point3D => {
                    if let Some(key) = slot_key(0) {
                        if let Ok(p) = plugin.params.get(key) {
                            if let Ok(pt) = p.as_point3d() {
                                let (px, py, pz) = pt.value();
                                out[0] = px as f32 / width.max(1) as f32;
                                out[1] = py as f32 / height.max(1) as f32;
                                out[2] = pz as f32;
                            }
                        }
                    }
                }
                // ADR-0030/0035: these are texture bindings, never uniform
                // members, so they contribute no words. The slot stays zeroed
                // and is never read — no pass layout points at it.
                ShaderParamType::Layer
                | ShaderParamType::Gradient
                | ShaderParamType::Path => {}
                ShaderParamType::Vec3Color | ShaderParamType::Vec4Color => {
                    if let Some(key) = slot_key(0) {
                        if let Ok(p) = plugin.params.get(key) {
                            if let Ok(c) = p.as_color() {
                                let px = c.value();
                                out[0] = px.red as f32 / 255.0;
                                out[1] = px.green as f32 / 255.0;
                                out[2] = px.blue as f32 / 255.0;
                            }
                        }
                    }
                    if decl.ty == ShaderParamType::Vec4Color {
                        out[3] = 1.0;
                        if let Some(key) = slot_key(1) {
                            if let Ok(p) = plugin.params.get(key) {
                                if let Ok(f) = p.as_float_slider() {
                                    out[3] = f.value() as f32;
                                }
                            }
                        }
                    }
                }
            }
            let _ = PoolKind::Float; // kinds are carried by the slots themselves
            out
        })
        .collect()
}

/// Resolve the transported definition from the StateToken: process registry
/// first, then the persisted snapshot (ADR-0015 §2, ADR-0016 §4).
///
/// Extracted from `render` so SmartRender can run it **before** staging
/// external resources. On a render clone's very first frame `local.compiled`
/// is still `None` when staging runs — resolution used to happen inside
/// `render`, which is after — so every external was staged as absent and the
/// frame rendered against the zero texture. Measured on the host 2026-08-16:
/// a path input read as unassigned on the first render and correctly on every
/// render after it, with nothing in the log to say why. Layers and gradients
/// had the same hole.
fn resolve_transported_definition(plugin: &PluginState, local: &mut Local) {
    let word = match plugin.params.get(ParamKey::StateToken) {
        Ok(p) => match p.as_float_slider() {
            Ok(slider) => slider.value(),
            Err(_) => f64::NAN,
        },
        Err(_) => f64::NAN,
    };
    match decode_token_state(word) {
        TokenState::Active(fp) if fp == local.token => {}
        TokenState::Active(fp) => {
            if let Some(compiled) = registry_get(fp) {
                local.token = fp;
                local.compiled = Some(compiled);
                local.pipelines = None;
                diag::log("definition resolved from process registry");
            } else if local.snapshot.is_some() {
                // Fresh process (reopen/aerender): the snapshot is
                // the authority. A fingerprint mismatch means a torn
                // token/snapshot pair — the checksummed snapshot
                // wins (ADR-0015 §2).
                if local.snapshot.as_ref().is_some_and(|s| s.fingerprint != fp) {
                    diag::log("token/snapshot fingerprint mismatch; snapshot wins");
                }
                resolve_from_snapshot(local);
            } else {
                diag::verbose("token missed registry with no snapshot; passing through");
                local.token = 0;
                local.compiled = None;
                local.pipelines = None;
            }
        }
        TokenState::Uninitialized => {
            if local.compiled.is_some() {
                local.token = 0;
                local.compiled = None;
                local.pipelines = None;
            } else if local.snapshot.is_some() {
                // Token stream says "nothing" but a snapshot exists:
                // torn pair, snapshot wins.
                resolve_from_snapshot(local);
            }
        }
        TokenState::Invalid(code) => {
            if local.compiled.is_some() {
                local.token = 0;
                local.compiled = None;
                local.pipelines = None;
            }
            diag::verbose(&format!("token carries diagnostic E{code}; passing through"));
        }
        TokenState::Corrupt => {
            diag::log("state token corrupt; passing through (E52)");
            local.token = 0;
            local.compiled = None;
            local.pipelines = None;
        }
    }
}

/// Rebuild the compiled effect from the restored snapshot (the render
/// clone's authority when the process registry cannot serve — ADR-0016 §4).
/// Pure computation plus naga; no AEGP, so any thread may call it.
fn resolve_from_snapshot(local: &mut Local) {
    let Some(snapshot) = local.snapshot.clone() else { return };
    let previous = snapshot.to_previous_plan();
    let (code, status, compiled) =
        evaluate_committed_source(snapshot.language, &snapshot.source, Some(&previous));
    match compiled {
        Some((fp, effect)) => {
            registry_insert(fp, Arc::clone(&effect));
            local.token = fp;
            local.compiled = Some(effect);
            local.pipelines = None;
            local.status_code = Diag::Ok;
            local.status_text = status;
            diag::log("definition rebuilt from snapshot");
        }
        None => {
            // A snapshot that no longer compiles (e.g. compiler drift):
            // fail closed with the real diagnostic.
            local.token = 0;
            local.compiled = None;
            local.pipelines = None;
            local.status_code = code;
            local.status_text = status;
            diag::log(&format!("snapshot rebuild failed: E{}", code.code()));
        }
    }
}

/// Copy the input frame to the output unchanged (the fail-closed path).
/// Copy the input to the output unchanged (the fail-closed path), honoring
/// world geometry: the input world sits at its ORIGIN in layer space (tight
/// text/shape buffers carry non-zero origins) and the output world starts
/// at the request corner (`window`). Coordinate-paired iteration ignored
/// both and pinned tight buffers to the corner — measured.
fn passthrough(in_layer: &Layer, out_layer: &mut Layer, window: (i32, i32)) -> Result<(), Error> {
    let px = match (in_layer.world_type(), out_layer.world_type()) {
        (ae::aegp::WorldType::U8, ae::aegp::WorldType::U8) => 4usize,
        (ae::aegp::WorldType::U15, ae::aegp::WorldType::U15) => 8,
        (ae::aegp::WorldType::F32, ae::aegp::WorldType::F32) => 16,
        _ => {
            // Mismatched worlds cannot be copied; deliver defined black.
            out_layer.buffer_mut().fill(0);
            return Ok(());
        }
    };
    let (rw, rh) = (in_layer.width(), in_layer.height());
    let (out_w, out_h) = (out_layer.width(), out_layer.height());
    let origin = in_layer.origin();
    let dest_x = origin.h - window.0;
    let dest_y = origin.v - window.1;
    let dx = dest_x.max(0) as usize;
    let dy = dest_y.max(0) as usize;
    let ox = (-dest_x).max(0) as usize;
    let oy = (-dest_y).max(0) as usize;
    let cw = out_w.saturating_sub(dx).min(rw.saturating_sub(ox));
    let ch = out_h.saturating_sub(dy).min(rh.saturating_sub(oy));
    let in_stride = in_layer.buffer_stride();
    let in_buf = in_layer.buffer().to_vec();
    let out_stride = out_layer.buffer_stride();
    let out_buf = out_layer.buffer_mut();
    out_buf.fill(0);
    for y in 0..ch {
        let src = (oy + y) * in_stride + ox * px;
        let dst = (dy + y) * out_stride + dx * px;
        out_buf[dst..dst + cw * px].copy_from_slice(&in_buf[src..src + cw * px]);
    }
    Ok(())
}

impl AdobePluginInstance for LocalMutex {
    fn flatten(&self) -> Result<(u16, Vec<u8>), Error> {
        // ADR-0016 snapshot: language + fingerprint + exact source + slot
        // map. No definition → empty payload (a fresh instance).
        let local = self.lock().map_err(|_| Error::Generic)?;
        let Some(compiled) = &local.compiled else {
            return Ok((1, Vec::new()));
        };
        let defn = &compiled.definition;
        let snapshot = persistence::Snapshot::from_state(
            defn.language,
            local.token,
            &compiled.source,
            &defn.binding,
        );
        match persistence::encode(&snapshot) {
            Ok(bytes) => Ok((1, bytes)),
            Err(e) => {
                // Construction-bug guard (ADR-0016 §3): refuse to persist a
                // bad snapshot; the expression stream remains the recovery
                // authority.
                diag::log(&format!("snapshot encode refused: {e:?}"));
                Ok((1, Vec::new()))
            }
        }
    }

    fn unflatten(_version: u16, serialized: &[u8]) -> Result<Self, Error> {
        // Empty payload = fresh instance. Anything else must pass the full
        // ADR-0016 validation; prototype bytes fail its magic check and are
        // discarded (ADR-0004: no compatibility promise).
        if serialized.is_empty() {
            return Ok(Mutex::new(Local::default()));
        }
        let local = match persistence::decode(serialized) {
            Ok(snapshot) => Local { snapshot: Some(snapshot), ..Local::default() },
            Err(persistence::DecodeError::Corrupt(what)) => {
                diag::log(&format!("snapshot corrupt ({what}); expression path recovers"));
                Local {
                    status_code: Diag::SnapshotCorrupt,
                    status_text: format!("snapshot corrupt ({what})"),
                    ..Local::default()
                }
            }
            Err(persistence::DecodeError::SchemaUnknown { schema, flags }) => {
                diag::log(&format!(
                    "snapshot schema unknown (schema={schema} flags={flags}); Compile to re-bind"
                ));
                Local {
                    status_code: Diag::SnapshotSchemaUnknown,
                    status_text: format!("snapshot schema {schema} is newer; Compile to re-bind"),
                    block_rebind: true,
                    ..Local::default()
                }
            }
        };
        Ok(Mutex::new(local))
    }

    fn render(
        &self,
        plugin: &mut PluginState,
        in_layer: &Layer,
        out_layer: &mut Layer,
    ) -> Result<(), ae::Error> {
        // Comp depth → working format (ADR-0021 §1); shaders never see the
        // depth. Both worlds must agree or we fail closed to pass-through.
        let depth = match (in_layer.world_type(), out_layer.world_type()) {
            (ae::aegp::WorldType::U8, ae::aegp::WorldType::U8) => Some(render::Depth::U8),
            (ae::aegp::WorldType::U15, ae::aegp::WorldType::U15) => Some(render::Depth::U15),
            (ae::aegp::WorldType::F32, ae::aegp::WorldType::F32) => Some(render::Depth::F32),
            (i, o) => {
                diag::log(&format!("render: unusable world pair {i:?}/{o:?}; passing through"));
                None
            }
        };
        let on_main_thread = std::thread::current().id() == plugin.global.main_thread;
        let mut local = self.lock().map_err(|_| Error::Generic)?;
        // ROI window stashed by the SmartRender arm on this thread; legacy
        // renders get the whole frame at origin zero.
        let window = SMART_WINDOW.with(|w| w.take()).unwrap_or((0, 0));

        // Opportunistic main-thread observation (also aerender's only path
        // until M3 persistence): a fresh observation is authoritative over
        // the transported token (architecture §5.1). Status text lands on
        // the next UI callback; render must not touch parameters.
        let mut observed_now = false;
        if on_main_thread {
            match observe_core(plugin, &mut local, false) {
                Ok(_) => observed_now = true,
                Err(e) => diag::verbose(&format!("main-thread observe failed: {e:?}")),
            }
        }

        if !observed_now {
            resolve_transported_definition(plugin, &mut local);
        }

        let mut rendered = false;
        diag::verbose(&format!(
            "render enter: {:?} in={}x{} out={}x{} win=({},{}) token={} compiled={} snap={} t={} step={} lstep={} scale={}",
            depth,
            in_layer.width(),
            in_layer.height(),
            out_layer.width(),
            out_layer.height(),
            window.0,
            window.1,
            local.token,
            local.compiled.is_some(),
            local.snapshot.is_some(),
            plugin.in_data.current_time(),
            plugin.in_data.time_step(),
            plugin.in_data.local_time_step(),
            plugin.in_data.time_scale()
        ));
        if let (Some(depth), true) = (depth, local.compiled.is_some()) {
            if let Some(gpu) = render::gpu() {
                if !depth.supported_by(gpu) {
                    diag::log(&format!(
                        "render: {depth:?} working format unsupported by adapter; passing through"
                    ));
                    drop(local);
                    passthrough(in_layer, out_layer, window)?;
                    return Ok(());
                }
                let stale = local
                    .pipelines
                    .as_ref()
                    .is_none_or(|set| set.token != local.token || set.depth != depth);
                if stale {
                    let compiled = local.compiled.as_ref().expect("checked above");
                    let mut passes = Vec::with_capacity(compiled.passes.len());
                    let mut ok = true;
                    for pass in &compiled.passes {
                        match render::build_pipeline(
                            gpu,
                            &pass.spirv,
                            &pass.layout,
                            &pass.extra_input_bindings,
                            depth,
                        ) {
                            Ok(p) => passes.push(p),
                            Err(e) => {
                                diag::log(&format!("build_pipeline failed: {e}"));
                                ok = false;
                                break;
                            }
                        }
                    }
                    if ok {
                        local.pipelines =
                            Some(render::PipelineSet { token: local.token, depth, passes });
                        // Adapter identity + working format + plan shape/peak
                        // transient memory ride along for host evidence
                        // (ADR-0014 §6, ADR-0020 §6, ADR-0021 §1).
                        let compiled = local.compiled.as_ref().expect("checked above");
                        let w = out_layer.width().min(in_layer.width());
                        let h = out_layer.height().min(in_layer.height());
                        diag::log(&format!(
                            "pipelines built: {} pass(es), plan {} step(s), {} physical intermediate(s) (~{} KiB transient at {}x{}, {:?} working format) ({})",
                            compiled.passes.len(),
                            compiled.plan.steps.len(),
                            compiled.plan.physical_count,
                            compiled.plan.physical_count * w * h * depth.bpp() / 1024,
                            w,
                            h,
                            depth.wgpu_format(),
                            gpu.adapter_summary
                        ));
                    }
                }

                if let (Some(_), Some(compiled)) = (&local.pipelines, &local.compiled) {
                    // ABI v1 uv/u_resolution span the full input frame
                    // (ADR-0011 §6); AE's ROI requests only narrow what is
                    // written back, never what is rendered. Placement: the
                    // input world sits at its ORIGIN in layer space (tight
                    // buffers — text/shape layers — carry non-zero origins;
                    // ignoring them pinned content to the corner, measured),
                    // and the output world starts at the request's corner,
                    // so input (0,0) lands at origin − request.
                    let rw = in_layer.width();
                    let rh = in_layer.height();
                    let origin = in_layer.origin();
                    let dest_x = origin.h - window.0;
                    let dest_y = origin.v - window.1;
                    let dx = dest_x.max(0) as usize;
                    let dy = dest_y.max(0) as usize;
                    let ox = (-dest_x).max(0) as usize;
                    let oy = (-dest_y).max(0) as usize;
                    let ow = out_layer
                        .width()
                        .saturating_sub(dx)
                        .min(rw.saturating_sub(ox));
                    let oh = out_layer
                        .height()
                        .saturating_sub(dy)
                        .min(rh.saturating_sub(oy));
                    if rw > 0 && rh > 0 && ow > 0 && oh > 0 {
                        let time = plugin.in_data.current_timestamp() as f32;
                        let frame = plugin.in_data.current_frame() as f32;
                        let t_render = std::time::Instant::now();

                        let compiled = Arc::clone(compiled);
                        // Split-borrow the instance state: pipelines stay
                        // shared while the frame cache and scratch buffers
                        // are taken mutably for this render.
                        let Local {
                            pipelines,
                            frame_cache,
                            scratch_in,
                            scratch_out,
                            ..
                        } = &mut *local;

                        // Boundary conversions are exact and reversible per
                        // depth (ADR-0021 §2 as amended by ADR-0022):
                        // reorder for U8/F32, the lossless U15↔f32 mapping
                        // for 16-bpc.
                        let bpp = depth.bpp();
                        scratch_in.resize(rw * rh * bpp, 0);
                        let input_px: &mut [u8] = scratch_in;
                        let (in_buf, in_stride) = (in_layer.buffer(), in_layer.buffer_stride());
                        match depth {
                            render::Depth::U8 => {
                                render::argb8_to_rgba8(in_buf, in_stride, rw, rh, input_px)
                            }
                            render::Depth::U15 => render::argb_u15_to_rgba_f32(
                                in_buf, in_stride, rw, rh, input_px,
                            ),
                            render::Depth::F32 => render::argb_f32_to_rgba_f32(
                                in_buf, in_stride, rw, rh, input_px,
                            ),
                        }
                        let conv_in_ms = t_render.elapsed().as_secs_f32() * 1000.0;

                        let global_values = read_bound_values(plugin, &compiled, rw, rh);
                        // Per-pass uniform values via each pass's member map.
                        let per_pass_values: Vec<Vec<[f32; 4]>> = compiled
                            .passes
                            .iter()
                            .map(|p| p.param_map.iter().map(|&g| global_values[g]).collect())
                            .collect();
                        let set = pipelines.as_ref().expect("checked above");

                        // ROI: deliver only the requested window (already
                        // computed as ox/oy/ow/oh in input-world coords);
                        // the escape hatch forces full-frame for A/B
                        // equivalence runs.
                        let rect = if roi_enabled() {
                            (ox, oy, ow, oh)
                        } else {
                            (0, 0, rw, rh)
                        };
                        let (rect_x, rect_y, rect_w, rect_h) = rect;
                        scratch_out.resize(rect_w * rect_h * bpp, 0);
                        let mut iters: u32 = 1;
                        // ADR-0025: temporal frames re-simulate min(F+1, W)
                        // iterations from black within this render. The whole
                        // window now runs inside ONE execute over cached
                        // resources: single input upload, ping/pong reuse,
                        // zero-texture basis for iteration 0, readback only
                        // on the last iteration. Every frame stays
                        // self-contained: no cross-frame state exists.
                        let window = compiled.window.map(|w| {
                            let frame_index = frame.round() as i64;
                            let n = window_iterations(frame_index, w);
                            iters = n;
                            let scale = plugin.in_data.time_scale();
                            let dt = if scale > 0 {
                                plugin.in_data.time_step() as f32 / scale as f32
                            } else {
                                0.0
                            };
                            diag::verbose(&format!(
                                "temporal window: {n} iteration(s) (W={w}) at frame {frame_index} (raw {frame})"
                            ));
                            (n, dt)
                        });
                        // Budget enforcement (M7): shapes over the cache cap
                        // render with transient resources (built and dropped
                        // this render) instead of pinning VRAM.
                        let mut transient: Option<render::FrameCache> = None;
                        let cache_slot: &mut Option<render::FrameCache> = if render::cache_within_budget(
                            depth,
                            rw,
                            rh,
                            compiled.plan.physical_count,
                            window.is_some(),
                        ) {
                            frame_cache
                        } else {
                            if frame_cache.is_some() {
                                *frame_cache = None;
                            }
                            diag::verbose("frame cache over budget; transient resources this render");
                            &mut transient
                        };
                        render::ensure_frame_cache(
                            gpu,
                            cache_slot,
                            set.token,
                            depth,
                            rw,
                            rh,
                            compiled.plan.physical_count,
                            window.is_some(),
                        );
                        let cache = cache_slot.as_mut().expect("just ensured");
                        // ADR-0029: shaders see the logical full-resolution
                        // frame size regardless of preview downsampling.
                        let ds_x = plugin.in_data.downsample_x();
                        let ds_y = plugin.in_data.downsample_y();
                        let logical_res = (
                            render::logical_size(rw, ds_x.num, ds_x.den),
                            render::logical_size(rh, ds_y.num, ds_y.den),
                        );
                        // ADR-0030: pixels checked out by the SmartRender
                        // arm on this thread, in TexSlot::Layer ordinal order.
                        // Absent entries bind transparent black (§5).
                        let borrowed = SMART_LAYERS.with(|l| l.borrow().clone());
                        // Encode any baked gradient here: this is the first
                        // point where the working format is known, which is
                        // exactly why the bake itself is depth-independent.
                        // `(bytes, width, height, float32)` per resource. Both
                        // encodings land here rather than at staging time
                        // because both need something only the render knows:
                        // the gradient needs the working depth, and the path
                        // needs the frame extent to normalize against
                        // (ADR-0035 §3).
                        type Encoded = (Vec<u8>, usize, usize, bool);
                        let encoded: Vec<Option<Encoded>> = borrowed
                            .iter()
                            .map(|entry| {
                                let e = entry.as_ref()?;
                                if let Some(vertices) = e.vertices.as_ref() {
                                    let (width, samples) =
                                        path::encode(vertices, rw as f32, rh as f32);
                                    return Some((
                                        render::encode_samples(&samples, render::Depth::F32),
                                        width,
                                        path::ROWS,
                                        true,
                                    ));
                                }
                                if e.ae_pixels {
                                    // ADR-0030 layer pixels: AE's ARGB (and
                                    // U15 at 16-bpc) into the working RGBA
                                    // layout, through the same converters the
                                    // effect's own input uses.
                                    let mut out = vec![0u8; e.width * e.height * depth.bpp()];
                                    match depth {
                                        render::Depth::U8 => render::argb8_to_rgba8(
                                            &e.pixels, e.stride, e.width, e.height, &mut out,
                                        ),
                                        render::Depth::U15 => render::argb_u15_to_rgba_f32(
                                            &e.pixels, e.stride, e.width, e.height, &mut out,
                                        ),
                                        render::Depth::F32 => render::argb_f32_to_rgba_f32(
                                            &e.pixels, e.stride, e.width, e.height, &mut out,
                                        ),
                                    }
                                    return Some((out, e.width, e.height, false));
                                }
                                let samples = e.samples.as_ref()?;
                                Some((
                                    render::encode_samples(samples, depth),
                                    e.width,
                                    e.height,
                                    false,
                                ))
                            })
                            .collect();
                        let externals: Vec<Option<render::ExternalTexture>> = borrowed
                            .iter()
                            .zip(encoded.iter())
                            .map(|(entry, enc)| {
                                let e = entry.as_ref()?;
                                match enc {
                                    Some((bytes, width, height, float32)) => {
                                        let bpp = if *float32 { 16 } else { depth.bpp() };
                                        Some(render::ExternalTexture {
                                            pixels: bytes.as_slice(),
                                            stride: width * bpp,
                                            width: *width,
                                            height: *height,
                                            float32: *float32,
                                        })
                                    }
                                    None => Some(render::ExternalTexture {
                                        pixels: e.pixels.as_slice(),
                                        stride: e.stride,
                                        width: e.width,
                                        height: e.height,
                                        float32: false,
                                    }),
                                }
                            })
                            .collect();
                        let result = render::execute_plan(
                            gpu,
                            set,
                            &compiled.plan,
                            &per_pass_values,
                            input_px,
                            rw * bpp,
                            rw,
                            rh,
                            time,
                            frame,
                            logical_res,
                            scratch_out,
                            rect_w * bpp,
                            window,
                            rect,
                            &externals,
                            cache,
                        );
                        match result {
                            Ok(exec) => {
                                // Write the requested window out of the
                                // full-frame result, at the right offset
                                // inside the (possibly larger) output world.
                                let t_out = std::time::Instant::now();
                                let out_stride = out_layer.buffer_stride();
                                let full_out_w = out_layer.width();
                                let full_out_h = out_layer.height();
                                let out_buf = out_layer.buffer_mut();
                                if dx > 0 || dy > 0 || ow < full_out_w || oh < full_out_h {
                                    // Areas outside the layer are transparent
                                    // black in every working encoding.
                                    out_buf.fill(0);
                                }
                                // AE-side pixel size differs from the working
                                // texel size at 16-bpc (U15 is 8 bytes).
                                let ae_px = match depth {
                                    render::Depth::U8 => 4,
                                    render::Depth::U15 => 8,
                                    render::Depth::F32 => 16,
                                };
                                let dst = &mut out_buf[dy * out_stride + dx * ae_px..];
                                // scratch_out holds exactly the delivered
                                // rect; the window is rect-local (equal to
                                // (0,0) under ROI, (ox,oy) when full).
                                let win = &scratch_out
                                    [((oy - rect_y) * rect_w + (ox - rect_x)) * bpp..];
                                match depth {
                                    render::Depth::U8 => render::rgba8_to_argb8(
                                        win, rect_w * bpp, ow, oh, dst, out_stride,
                                    ),
                                    render::Depth::U15 => render::rgba_f32_to_argb_u15(
                                        win, rect_w * bpp, ow, oh, dst, out_stride,
                                    ),
                                    render::Depth::F32 => render::rgba_f32_to_argb_f32(
                                        win, rect_w * bpp, ow, oh, dst, out_stride,
                                    ),
                                }
                                rendered = true;
                                if perf_log_enabled() {
                                    // One machine-parsable line per render
                                    // (audit 07 measurement plan). Spans in
                                    // ms; total covers conversion in → AE
                                    // write-back inclusive.
                                    let conv_out_ms =
                                        t_out.elapsed().as_secs_f32() * 1000.0;
                                    let total_ms =
                                        t_render.elapsed().as_secs_f32() * 1000.0;
                                    // t0 in unix ms: interval overlap across
                                    // renders is the MFR-concurrency measure
                                    // (audit 07 eligibility review).
                                    let t0_ms = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .map(|d| d.as_millis() as u64)
                                        .unwrap_or(0)
                                        .saturating_sub(total_ms as u64);
                                    diag::log(&format!(
                                        "perf: t0={} depth={:?} dims={}x{} rect={}x{} passes={} iters={} frame={} conv_in={:.3} upload={:.3} gpu={:.3} readback={:.3} conv_out={:.3} total={:.3}",
                                        t0_ms,
                                        depth,
                                        rw,
                                        rh,
                                        rect_w,
                                        rect_h,
                                        compiled.passes.len(),
                                        iters,
                                        frame.round() as i64,
                                        conv_in_ms,
                                        exec.upload_ms,
                                        exec.gpu_ms,
                                        exec.readback_ms,
                                        conv_out_ms,
                                        total_ms,
                                    ));
                                } else {
                                    diag::verbose("execute_plan ok");
                                }
                            }
                            Err(e) => diag::log(&format!("execute_plan failed: {e}")),
                        }
                    }
                }
            } else {
                diag::verbose("render: gpu unavailable (DX12 adapter missing)");
            }
        }

        drop(local);
        if !rendered {
            passthrough(in_layer, out_layer, window)?;
        }
        Ok(())
    }

    fn handle_command(&mut self, plugin: &mut PluginState, command: Command) -> Result<(), Error> {
        match command {
            Command::UserChangedParam { param_index } => {
                let force = plugin.params.index(ParamKey::Compile) == Some(param_index);
                // ADR-0028: the Details button pops the full status text —
                // the Status row's name is capped at 31 chars by PF.
                if plugin.params.index(ParamKey::Details) == Some(param_index) {
                    let text = {
                        let local = self.lock().map_err(|_| Error::Generic)?;
                        format!(
                            "DynamicFX status\n\n{}\n\n(diagnostic code E{})",
                            local.status_text,
                            local.status_code.code()
                        )
                    };
                    host::show_info_dialog("DynamicFX", &text);
                    return Ok(());
                }
                let mut local = self.lock().map_err(|_| Error::Generic)?;
                observe_core(plugin, &mut local, force)?;
                // UI callbacks always try to land the desired status text —
                // observation may have happened earlier in a context that
                // could not touch parameters (idle bridge, render).
                let text = local.status_text.clone();
                set_status(plugin, &mut local, text);
                configure_slots(plugin, &mut local);
                // Commit context: mirror the token word into the stream so
                // render clones can resolve it.
                publish_token_param(plugin, &local);
                drop(local);
                plugin.out_data.set_force_rerender();
            }
            Command::UpdateParamsUi => {
                let mut local = self.lock().map_err(|_| Error::Generic)?;
                let changed = observe_core(plugin, &mut local, false)?;
                let text = local.status_text.clone();
                set_status(plugin, &mut local, text);
                configure_slots(plugin, &mut local);
                drop(local);
                if changed {
                    plugin.out_data.set_force_rerender();
                }
            }
            // The idle observer's per-instance bridge: observation and
            // registry publication only — no parameter writes here (the idle
            // hook mirrors the token via AEGP afterwards; the status text
            // lands on the next UI callback).
            Command::CompletelyGeneral => {
                let mut local = self.lock().map_err(|_| Error::Generic)?;
                if observe_core(plugin, &mut local, false)? {
                    diag::log(&format!("idle observation: {}", local.status_text));
                    drop(local);
                    plugin.out_data.set_force_rerender();
                }
            }
            // AE can send sequence (re)setup while addProperty() is still
            // constructing the scripting property tree; AEGP conversion in
            // that window fails. Observation waits for a UI callback or idle.
            Command::SequenceSetup | Command::SequenceResetup => {
                diag::verbose("sequence (re)setup: observation deferred");
            }
            // ADR-0031 §7: the gradient editor. Custom-UI events arrive on
            // the main thread for the row that owns the control area.
            // SmartFX entry (M5): AE only hands float worlds to smart
            // effects (FLOAT_COLOR_AWARE rides SUPPORTS_SMART_RENDER), so
            // the smart path exists for image correctness; performance-side
            // SmartRender work (caching, checkout narrowing, MFR) is M7.
            Command::SmartPreRender { mut extra } => {
                let mut req = extra.output_request();
                let requested = req.rect;
                let cb = extra.callbacks();
                let in_data = plugin.in_data;
                // ABI v1 fixes uv/u_resolution to the full layer frame
                // (ADR-0011 §6). AE's ROI requests (a sampleImage of a few
                // pixels arrives as a ~12x12 rect — measured live) must not
                // shrink the render, so the INPUT checkout covers the whole
                // layer. The declared result must echo the request (AE
                // errors 25::237 on anything larger — measured live); the
                // GPU renders full-frame and only the requested window is
                // written back, guided by the origin stashed here.
                req.rect.left = 0;
                req.rect.top = 0;
                req.rect.right = in_data.width();
                req.rect.bottom = in_data.height();
                match cb.checkout_layer(
                    0,
                    0,
                    &req,
                    in_data.current_time(),
                    in_data.time_step(),
                    in_data.time_scale(),
                ) {
                    Ok(_) => {
                        let r = ae::Rect {
                            left: requested.left,
                            top: requested.top,
                            right: requested.right,
                            bottom: requested.bottom,
                        };
                        extra.set_result_rect(r);
                        // max_result_rect declares where the shader COULD
                        // produce content: the whole frame. Echoing the tiny
                        // request here made AE cache "empty everywhere else",
                        // so later samples outside the first ROI returned
                        // permanent black without rendering (measured live).
                        // It must also CONTAIN the result rect — requests can
                        // reach outside the layer bounds and a bare full
                        // frame then fails 25::237 (measured live) — so take
                        // the union.
                        extra.set_max_result_rect(ae::Rect {
                            left: requested.left.min(0),
                            top: requested.top.min(0),
                            right: requested.right.max(in_data.width()),
                            bottom: requested.bottom.max(in_data.height()),
                        });
                        extra.set_pre_render_data::<(i32, i32)>((requested.left, requested.top));

                        // ADR-0030: one checkout per bound layer parameter,
                        // with the SAME full-frame rect and time as the input
                        // so `uv` addresses the same point in every texture
                        // (§4 comp space). Checkout ids start at 1 — 0 is the
                        // effect's own input. A failed checkout is logged and
                        // left unbound: the shader then reads zeros (§5)
                        // rather than the frame failing.
                        let externals = {
                            let mut local = self.lock().map_err(|_| Error::Generic)?;
                            // Resolve here too, not only in SmartRender. AE
                            // counts checkouts: if PreRender skips a layer this
                            // frame and SmartRender then asks for its pixels,
                            // the host aborts the render with "Node received
                            // more checkout requests than expected" (reported
                            // from interactive use, 2026-08-16, after the
                            // SmartRender side was fixed alone). The two sides
                            // must see the same definition or neither may act.
                            if local.compiled.is_none() {
                                resolve_transported_definition(plugin, &mut local);
                            }
                            local
                                .compiled
                                .as_ref()
                                .map(|c| c.externals.clone())
                                .unwrap_or_default()
                        };
                        // Exactly the ids AE accepted. SmartRender asks for
                        // these and checks in these — never a superset, which
                        // is the same accounting mistake from the other end.
                        let mut checked_out: Vec<u32> = Vec::new();
                        for (ordinal, source) in externals.iter().enumerate() {
                            // Gradients and paths are not checked out as layers.
                            let ExternalSource::Layer { param_index } = source else { continue };
                            let id = ordinal as u32 + 1;
                            match cb.checkout_layer(
                                *param_index as i32,
                                id as i32,
                                &req,
                                in_data.current_time(),
                                in_data.time_step(),
                                in_data.time_scale(),
                            ) {
                                Ok(_) => checked_out.push(id),
                                Err(e) => diag::log(&format!(
                                    "layer checkout failed (param {param_index}): {e:?}"
                                )),
                            }
                        }
                        SMART_CHECKOUTS.with(|c| *c.borrow_mut() = checked_out);
                    }
                    Err(e) => diag::log(&format!("smart pre-render checkout failed: {e:?}")),
                }
            }
            Command::SmartRender { extra } => {
                let window = extra
                    .pre_render_data::<(i32, i32)>()
                    .copied()
                    .unwrap_or((0, 0));
                let cb = extra.callbacks();
                let checked_out = match cb.checkout_layer_pixels(0) {
                    Ok(v) => v,
                    Err(e) => {
                        diag::log(&format!("smart render input checkout failed: {e:?}"));
                        None
                    }
                };
                // ADR-0030: copy each bound layer's pixels for this frame.
                // The `Layer` borrows the callbacks and cannot outlive this
                // arm, so the bytes are owned; the cost rides the upload span.
                let externals = {
                    let mut local = self.lock().map_err(|_| Error::Generic)?;
                    // Resolve BEFORE reading `externals`: see
                    // `resolve_transported_definition`. Without this the first
                    // frame of every render clone stages nothing.
                    if local.compiled.is_none() {
                        resolve_transported_definition(plugin, &mut local);
                    }
                    local
                        .compiled
                        .as_ref()
                        .map(|c| c.externals.clone())
                        .unwrap_or_default()
                };
                let mut staged: Vec<Option<ExternalPixels>> = Vec::new();
                for (ordinal, source) in externals.iter().enumerate() {
                    match source {
                        ExternalSource::Layer { .. } => {
                            let id = ordinal as u32 + 1;
                            // PreRender is the authority on which ids exist
                            // this frame; asking for one it did not request is
                            // what AE reports as an internal verification
                            // failure.
                            if !SMART_CHECKOUTS.with(|c| c.borrow().contains(&id)) {
                                staged.push(None);
                                continue;
                            }
                            match cb.checkout_layer_pixels(id) {
                                // `None` is legitimate — an unassigned
                                // selector, or an adjustment layer with
                                // nothing under it (ADR-0030 §5).
                                Ok(Some(layer)) => {
                                    let stride = layer.buffer_stride();
                                    // Raw AE bytes: ARGB, and at 16-bpc a
                                    // 4x16-bit U15 pixel. The working format is
                                    // RGBA, so these are converted at the
                                    // encode site in `render`, where the depth
                                    // is known — exactly like the gradient LUT.
                                    // Uploading them unconverted put alpha in
                                    // the red channel and shifted every other
                                    // channel one place: a cyan solid read back
                                    // as magenta, arithmetically exactly
                                    // (a,r,g,b) (measured 2026-08-16, the first
                                    // run in which the checkout itself worked).
                                    diag::log(&format!(
                                        "layer {id}: {}x{} stride {stride}",
                                        layer.width(),
                                        layer.height()
                                    ));
                                    staged.push(Some(ExternalPixels {
                                        pixels: layer.buffer().to_vec(),
                                        stride,
                                        width: layer.width(),
                                        height: layer.height(),
                                        samples: None,
                                        vertices: None,
                                        ae_pixels: true,
                                    }));
                                }
                                Ok(None) => {
                                    // Legitimate, but indistinguishable in the
                                    // render from a failed read — so say which
                                    // it was (2026-08-16: a broken checkout and
                                    // an unassigned selector produced the same
                                    // silent transparent black).
                                    diag::log(&format!("layer {id}: no pixels (selector unset?)"));
                                    staged.push(None);
                                }
                                Err(e) => {
                                    diag::log(&format!(
                                        "layer pixels checkout {id} failed: {e:?}"
                                    ));
                                    staged.push(None);
                                }
                            }
                        }
                        ExternalSource::Gradient { gradient_index } => {
                            staged.push(bake_gradient(plugin.params, *gradient_index));
                        }
                        ExternalSource::Path { path_index } => {
                            staged.push(read_path(
                                &plugin.in_data,
                                plugin.params,
                                *path_index,
                            ));
                        }
                    }
                }
                SMART_LAYERS.with(|l| *l.borrow_mut() = staged);

                if let Ok(Some(mut out_layer)) = cb.checkout_output() {
                    if let Some(in_layer) = &checked_out {
                        SMART_WINDOW.with(|w| w.set(Some(window)));
                        AdobePluginInstance::render(self, plugin, in_layer, &mut out_layer)?;
                    } else {
                        // No input frame (adjustment layer over nothing):
                        // deliver transparent black rather than garbage.
                        // All-zero bytes are transparent black in every
                        // working encoding (U8/U15/F32).
                        out_layer.buffer_mut().fill(0);
                    }
                }
                SMART_LAYERS.with(|l| l.borrow_mut().clear());
                for id in SMART_CHECKOUTS.with(|c| c.borrow().clone()) {
                    let _ = cb.checkin_layer_pixels(id);
                }
                SMART_CHECKOUTS.with(|c| c.borrow_mut().clear());
                if checked_out.is_some() {
                    let _ = cb.checkin_layer_pixels(0);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
