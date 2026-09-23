//! Cross-layer production WGSL contracts (ADR-0044), without an AE host.
use super::*;

fn wgsl(fields: &str, annotations: &str, body: &str) -> String {
    format!(r#"{annotations}
struct FxUniforms {{
 u_resolution: vec2<f32>, u_time: f32, u_frame: f32,
 {fields}
}};
@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var smp: sampler;
@group(0) @binding(2) var<uniform> fx: FxUniforms;
@fragment fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {{
 {body}
}}
"#)
}

fn envelope(graph: &str, passes: &[(&str, String)]) -> String {
    let mut s = format!("@dynamicfx 1\n@graph\n{graph}\n@end\n");
    for (name, body) in passes {
        s.push_str(&format!("@pass {name}\n"));
        for line in body.lines() {
            let stripped = line.trim_start();
            if stripped.starts_with('@') {
                s.push_str(&line[..line.len() - stripped.len()]);
                s.push('@');
                s.push_str(stripped);
            } else { s.push_str(line); }
            s.push('\n');
        }
        s.push_str("@endpass\n");
    }
    s
}

fn compile(language: LanguageId, text: &str, previous: Option<&binding::BindingPlan>) -> (u64, Arc<CompiledEffect>) {
    let (code, status, result) = evaluate_committed_source(language, text, previous);
    assert_eq!(code, Diag::Ok, "{status}");
    result.unwrap()
}

#[test]
fn wgsl_snapshot_flattens_and_rebuilds_original_language_and_plan() {
    let text = wgsl("gain: f32,", "// @param gain default:0.25", "return vec4<f32>(fx.gain, uv, 1.0);");
    let (fp, effect) = compile(LanguageId::WGSL, &text, None);
    let local = LocalMutex::new(Local { compiled: Some(Arc::clone(&effect)), token: fp, ..Local::default() });
    let (version, bytes) = <LocalMutex as AdobePluginInstance>::flatten(&local).unwrap();
    let restored = <LocalMutex as AdobePluginInstance>::unflatten(version, &bytes).unwrap();
    let mut restored = restored.lock().unwrap();
    assert_eq!(restored.snapshot.as_ref().unwrap().language, LanguageId::WGSL);
    assert_eq!(restored.snapshot.as_ref().unwrap().source, text);
    resolve_from_snapshot(&mut restored);
    assert_eq!(restored.status_code, Diag::Ok);
    let rebuilt = restored.compiled.as_ref().unwrap();
    assert_eq!(rebuilt.definition.language, LanguageId::WGSL);
    assert!(plan_mappings_equal(&rebuilt.definition.binding, &effect.definition.binding));
    assert_eq!(restored.token, fp);
}

#[test]
fn language_switch_and_alias_preserve_existing_parameter_slots() {
    let glsl = "#version 450\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=2) uniform FxUniforms {vec2 u_resolution;float u_time;float u_frame;float gain;};\nvoid main(){outColor=vec4(gain,0,0,1);}";
    let (_, first) = compile(LanguageId::GLSL, glsl, None);
    let text = wgsl("exposure: f32,", "// @param exposure alias:gain default:0.5", "return vec4<f32>(fx.exposure, 0.0, 0.0, 1.0);");
    let (_, second) = compile(LanguageId::WGSL, &text, Some(&first.definition.binding));
    assert_eq!(first.definition.binding.bindings[0].slots, second.definition.binding.bindings[0].slots);
    assert!(second.definition.binding.bindings[0].inherited);
    assert_ne!(session_token(LanguageId::GLSL, &text), session_token(LanguageId::WGSL, &text));
}

#[test]
fn wgsl_graph_uses_original_modules_and_neutral_grouping() {
    let gen = wgsl("gain: f32,", "// @param gain default:0.25", "return vec4<f32>(uv, fx.gain, 1.0);");
    let inv = wgsl("", "", "return vec4<f32>(vec3<f32>(1.0) - textureSample(image, smp, uv).rgb, 1.0);");
    let text = envelope("pass gen: input -> a\npass invert: a -> b\npass finish: b -> output", &[("gen",gen),("invert",inv.clone()),("finish",inv)]);
    let (_, effect) = compile(LanguageId::WGSL, &text, None);
    assert_eq!(effect.passes.len(), 3);
    assert_eq!(effect.plan.steps.len(), 3);
    assert_eq!(effect.definition.params[0].bank, Some(0));
    assert!(effect.definition.graph.passes[0].source.contains("@fragment"));
    assert!(!effect.definition.graph.passes[0].source.contains("@@fragment"));
    assert_eq!(effect.source, text);
}

#[test]
fn wgsl_external_resources_and_canvas_reach_existing_host_mappings() {
    for hint in ["layer", "gradient", "path", "coverage"] {
        let text = wgsl("reach: f32,", &format!("// @param side hint:{hint}\n// @param reach hint:canvas default:16"), "return textureLoad(side_image, vec2<i32>(0), 0);")
            .replace("@fragment", "@group(0) @binding(3) var side_image: texture_2d<f32>;\n@fragment");
        let text = envelope("pass use: input, side -> output", &[("use",text)]);
        let (_, effect) = compile(LanguageId::WGSL, &text, None);
        assert_eq!(effect.externals.len(), 1, "{hint}");
        assert_eq!(effect.passes[0].extra_input_bindings, vec![3]);
        assert_eq!(effect.definition.canvas_param.as_ref().unwrap().as_str(), "reach");
        assert!(matches!((&effect.externals[0], hint),
            (ExternalSource::Layer { .. }, "layer") |
            (ExternalSource::Gradient { .. }, "gradient") |
            (ExternalSource::Path { .. }, "path") |
            (ExternalSource::Coverage { .. }, "coverage")));
    }
}

#[test]
fn wgsl_temporal_window_and_external_restriction_are_unchanged() {
    let body = wgsl("", "// @window 4", "return textureSample(image, smp, uv) + vec4<f32>(0.1);");
    let text = envelope("pass accumulate: prev -> output", &[("accumulate",body.clone())]);
    let (_, effect) = compile(LanguageId::WGSL, &text, None);
    assert_eq!(effect.window, Some(4));
    let text = envelope("pass accumulate: prev, side -> output", &[("accumulate",format!("// @param side hint:layer\n{body}"))]);
    let (code, _, effect) = evaluate_committed_source(LanguageId::WGSL, &text, None);
    assert_eq!(code, Diag::LayerInTemporalGraph);
    assert!(effect.is_none());
}

#[test]
fn wgsl_failure_reports_language_pass_and_envelope_origin() {
    let text = envelope("pass bad: input -> output", &[("bad", "this is invalid WGSL".into())]);
    let (code, status, result) = evaluate_committed_source(LanguageId::WGSL, &text, None);
    assert_eq!(code, Diag::WgslParse);
    assert!(result.is_none());
    assert!(status.contains("pass `bad` (body starts at source line 6)"), "{status}");
    assert!(status.contains("WGSL error: line 1:"), "{status}");
    assert_eq!(evaluate_committed_source(LanguageId::GLSL, "bad GLSL", None).0, Diag::GlslParse);
}

#[test]
fn unknown_language_snapshot_is_preserved_and_fails_closed() {
    let snapshot = persistence::Snapshot::from_state(LanguageId(999), 17, "future language bytes", &binding::BindingPlan { bindings: vec![] });
    let mut local = Local { snapshot: Some(snapshot.clone()), ..Local::default() };
    resolve_from_snapshot(&mut local);
    assert_eq!(local.status_code, Diag::LanguageUnknown);
    assert!(local.compiled.is_none());
    assert_eq!(local.snapshot, Some(snapshot));
}
