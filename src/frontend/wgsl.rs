//! Production WGSL frontend (ADR-0044). Parse and validate the original Naga
//! module, then share its reflected uniform/annotation model with GLSL.

use super::annotation::Annotation;
use super::shared::{reflect_user_params, truncate};
use super::{FrontendError, LanguageFrontend, LanguageId, PassModule};
use naga::{AddressSpace, Binding, ScalarKind, ShaderStage, TypeInner, VectorSize};
use std::collections::{HashMap, HashSet};

pub struct WgslFrontend;

/// The renderer requests wgpu's default limits, including a 64 KiB uniform
/// binding. Explicit WGSL padding must not bypass that resource budget.
const MAX_UNIFORM_BYTES: u32 = 65_536;

impl LanguageFrontend for WgslFrontend {
    fn language(&self) -> LanguageId {
        LanguageId::WGSL
    }

    fn parse_module(
        &self,
        source: &str,
        annotations: &HashMap<String, Annotation>,
        allowed_inputs: usize,
    ) -> Result<PassModule, FrontendError> {
        let module = naga::front::wgsl::parse_str(source).map_err(|error| {
            FrontendError::Parse(diagnostic(
                error.location(source),
                &error.emit_to_string(source),
            ))
        })?;

        // Surface checks precede semantic validation so unused declarations
        // cannot smuggle resources/stages outside the runtime's interface.
        check_entry_point(&module)?;
        if !module.overrides.is_empty() {
            return Err(FrontendError::Abi(
                "pipeline overrides are outside the WGSL interface".into(),
            ));
        }
        let extra_input_bindings = check_bindings(&module, source, allowed_inputs)?;

        // No optional language capabilities are enabled by the renderer.
        // In particular, accepting f16/subgroups here would only postpone
        // their failure until pipeline creation in an AE render callback.
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .map_err(|error| {
            FrontendError::Parse(diagnostic(
                error.location(source),
                &error.emit_to_string(source),
            ))
        })?;

        let (params, layout) = reflect_user_params(&module, annotations).map_err(|error| {
            // The declaration is an accurate source anchor even when a
            // member-specific span is unavailable. Do not invent its line.
            let location = module.global_variables.iter().find_map(|(handle, var)| {
                var.binding
                    .filter(|b| b.group == 0 && b.binding == 2)
                    .and_then(|_| span_location(module.global_variables.get_span(handle), source))
            });
            match error {
                FrontendError::Abi(message) => FrontendError::Abi(diagnostic(location, &message)),
                FrontendError::Param(message) => {
                    FrontendError::Param(diagnostic(location, &message))
                }
                // This variant carries a ParamId, not diagnostic text.
                other => other,
            }
        })?;
        Ok(PassModule {
            module,
            params,
            layout,
            extra_input_bindings,
        })
    }
}

/// Keep the pass-local 1-based location first so short host status strings
/// remain actionable. Columns are UTF-8 byte positions, following Naga.
/// Envelope callers also report the body's source start.
fn diagnostic(location: Option<naga::SourceLocation>, message: &str) -> String {
    let text = if let Some(location) = location {
        format!(
            "line {}:{}: {}",
            location.line_number,
            location.line_position,
            message.trim()
        )
    } else {
        message.trim().to_owned()
    };
    truncate(&text, 1200)
}

fn span_location(span: naga::Span, source: &str) -> Option<naga::SourceLocation> {
    span.is_defined().then(|| span.location(source))
}

fn float_vector(inner: &TypeInner, size: VectorSize) -> bool {
    matches!(inner, TypeInner::Vector {
        size: actual,
        scalar: naga::Scalar { kind: ScalarKind::Float, width: 4 },
    } if *actual == size)
}

/// Accept both direct attributes and the idiomatic WGSL IO struct syntax.
/// Nested IO structs have no neutral ABI representation and are rejected.
fn io_fields<'a>(
    module: &'a naga::Module,
    ty: naga::Handle<naga::Type>,
    binding: &'a Option<Binding>,
) -> Result<Vec<(&'a TypeInner, &'a Binding)>, FrontendError> {
    if let Some(binding) = binding {
        return Ok(vec![(&module.types[ty].inner, binding)]);
    }
    let TypeInner::Struct { members, .. } = &module.types[ty].inner else {
        return Err(FrontendError::Abi(
            "entry IO requires explicit location/builtin attributes".into(),
        ));
    };
    members
        .iter()
        .map(|member| {
            let binding = member.binding.as_ref().ok_or_else(|| {
                FrontendError::Abi(
                    "entry IO struct members require explicit location/builtin attributes".into(),
                )
            })?;
            Ok((&module.types[member.ty].inner, binding))
        })
        .collect()
}

fn check_entry_point(module: &naga::Module) -> Result<(), FrontendError> {
    if module.entry_points.len() != 1 {
        return Err(FrontendError::Abi(
            "require exactly one @fragment entry point named `main`".into(),
        ));
    }
    let entry = &module.entry_points[0];
    if entry.name != "main" || entry.stage != ShaderStage::Fragment {
        return Err(FrontendError::Abi("require @fragment fn main".into()));
    }
    if entry.early_depth_test.is_some() {
        return Err(FrontendError::Abi(
            "early depth tests are outside the WGSL interface".into(),
        ));
    }
    let mut uv = false;
    let mut position = false;
    for argument in &entry.function.arguments {
        for (inner, binding) in io_fields(module, argument.ty, &argument.binding)? {
            match binding {
                Binding::Location {
                    location: 0,
                    interpolation: Some(naga::Interpolation::Perspective),
                    sampling: Some(naga::Sampling::Center),
                    blend_src: None,
                    per_primitive: false,
                } if float_vector(inner, VectorSize::Bi) && !uv => uv = true,
                Binding::BuiltIn(naga::BuiltIn::Position { .. })
                    if float_vector(inner, VectorSize::Quad) && !position => position = true,
                _ => return Err(FrontendError::Abi(
                    "inputs may only be one location-0 vec2<f32> UV with perspective/center interpolation and one builtin(position) vec4<f32>".into(),
                )),
            }
        }
    }
    let result =
        entry.function.result.as_ref().ok_or_else(|| {
            FrontendError::Abi("missing location-0 vec4<f32> fragment output".into())
        })?;
    let fields = io_fields(module, result.ty, &result.binding)?;
    if fields.len() != 1
        || !matches!(fields[0], (inner, Binding::Location {
        location: 0, blend_src: None, per_primitive: false, ..
    }) if float_vector(inner, VectorSize::Quad))
    {
        return Err(FrontendError::Abi(
            "require exactly one location-0 vec4<f32> color output".into(),
        ));
    }
    Ok(())
}

fn check_bindings(
    module: &naga::Module,
    source: &str,
    allowed_inputs: usize,
) -> Result<Vec<u32>, FrontendError> {
    if !(1..=super::grammar::MAX_INPUTS_PER_PASS).contains(&allowed_inputs) {
        return Err(FrontendError::Abi(
            "invalid manifest input count for WGSL module".into(),
        ));
    }
    let max_extra = 2 + (allowed_inputs - 1) as u32;
    let mut seen = HashSet::new();
    let mut extra = Vec::new();
    for (handle, var) in module.global_variables.iter() {
        let fail = |message: String| {
            FrontendError::Abi(diagnostic(
                span_location(module.global_variables.get_span(handle), source),
                &message,
            ))
        };
        let Some(binding) = var.binding else {
            if var.space != AddressSpace::Private {
                return Err(fail(
                    "unbound resource/address space is outside the WGSL interface".into(),
                ));
            }
            continue;
        };
        if binding.group != 0 {
            return Err(fail(format!(
                "descriptor group {} is reserved (v1 uses group 0 only)",
                binding.group
            )));
        }
        if !seen.insert(binding.binding) {
            return Err(fail(format!("duplicate binding {}", binding.binding)));
        }
        let inner = &module.types[var.ty].inner;
        let texture = var.space == AddressSpace::Handle
            && matches!(
                inner,
                TypeInner::Image {
                    dim: naga::ImageDimension::D2,
                    arrayed: false,
                    class: naga::ImageClass::Sampled {
                        kind: ScalarKind::Float,
                        multi: false
                    },
                }
            );
        match binding.binding {
            0 if !texture => return Err(fail("binding 0 must be texture_2d<f32>".into())),
            0 => {},
            1 if var.space == AddressSpace::Handle && matches!(inner, TypeInner::Sampler { comparison: false }) => {},
            1 => return Err(fail("binding 1 must be a non-comparison sampler".into())),
            2 => {
                let TypeInner::Struct { span, .. } = inner else {
                    return Err(fail("binding 2 must be a uniform struct".into()));
                };
                if var.space != AddressSpace::Uniform {
                    return Err(fail("binding 2 must use var<uniform>".into()));
                }
                if *span > MAX_UNIFORM_BYTES {
                    return Err(fail(format!("uniform block spans {span} bytes; the WGSL limit is {MAX_UNIFORM_BYTES}")));
                }
            },
            b @ 3..=15 if b > max_extra => return Err(fail(format!(
                "binding {b} has no manifest input feeding it (this pass declares {allowed_inputs} input(s))",
            ))),
            b @ 3..=15 if !texture => return Err(fail(format!("binding {b} must be texture_2d<f32>"))),
            b @ 3..=15 => extra.push(b),
            b => return Err(fail(format!("binding {b} is outside the v1 interface"))),
        }
    }
    if !seen.contains(&2) {
        return Err(FrontendError::Abi(
            "missing uniform struct at (group = 0, binding = 2)".into(),
        ));
    }
    extra.sort_unstable();
    Ok(extra)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::param::ShaderParamType;

    fn shader(members: &str) -> String {
        format!(
            r#"struct FxUniforms {{
    u_resolution: vec2<f32>,
    u_time: f32,
    u_frame: f32,
    {members}
}}
@group(0) @binding(2) var<uniform> fx: FxUniforms;
@fragment fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {{
    return vec4<f32>(uv, fx.u_time, 1.0);
}}
"#
        )
    }

    fn parse(source: &str) -> Result<PassModule, FrontendError> {
        parse_inputs(source, 1)
    }

    fn parse_inputs(source: &str, count: usize) -> Result<PassModule, FrontendError> {
        let annotations = super::super::annotation::parse_annotations(source).map_err(|error| {
            FrontendError::Param(format!("@param line {}: {}", error.line, error.message))
        })?;
        WgslFrontend.parse_module(source, &annotations, count)
    }

    fn assert_abi(source: &str, contains: &str) {
        let error = parse(source).expect_err("bad WGSL ABI must fail closed");
        assert!(
            matches!(error, FrontendError::Abi(ref message) if message.contains(contains)),
            "{error:?}"
        );
    }

    #[test]
    fn original_ir_reflects_all_parameter_kinds_and_matches_glsl() {
        let annotations = r#"// @param flag hint:bool default:1
// @param tint hint:color default:0.1,0.2,0.3
// @param overlay hint:color default:#4080FF80
// @param angle hint:angle default:90 alias:old_angle
// @param point3 hint:point3d
// @param gain label:"控制 Gain" min:0 max:2 default:0.75
// @param reach hint:canvas min:0 max:256 default:32
"#;
        let source = format!("{annotations}{}", shader("gain: f32, count: i32, flag: i32, point: vec2f, tint: vec3f, overlay: vec4f, angle: f32, point3: vec3f, reach: f32,"));
        let parsed = parse(&source).unwrap();
        assert_eq!(
            parsed.params.iter().map(|p| p.ty).collect::<Vec<_>>(),
            vec![
                ShaderParamType::Float,
                ShaderParamType::Int,
                ShaderParamType::Bool,
                ShaderParamType::Vec2,
                ShaderParamType::Vec3Color,
                ShaderParamType::Vec4Color,
                ShaderParamType::AngleFloat,
                ShaderParamType::Point3D,
                ShaderParamType::Float,
            ]
        );
        assert_eq!(
            parsed
                .layout
                .entries
                .iter()
                .map(|e| e.offset)
                .collect::<Vec<_>>(),
            vec![16, 20, 24, 32, 48, 64, 80, 96, 108]
        );
        assert_eq!(parsed.layout.block_size, 112);
        assert!(parsed.params[8].canvas);
        assert_eq!(parsed.params[6].aliases[0].as_str(), "old_angle");
        assert_eq!(parsed.params[0].ui.label.as_deref(), Some("控制 Gain"));
        let glsl = format!(
            r#"#version 450
{annotations}
layout(location=0) in vec2 uv;
layout(location=0) out vec4 color;
layout(set=0,binding=2,std140) uniform FxUniforms {{
    vec2 u_resolution; float u_time; float u_frame;
    float gain; int count; int flag; vec2 point; vec3 tint;
    vec4 overlay; float angle; vec3 point3; float reach;
}};
void main() {{ color=vec4(uv,u_time,1.0); }}
"#
        );
        let annotations = super::super::annotation::parse_annotations(&glsl).unwrap();
        let other = super::super::glsl::GlslFrontend
            .parse_module(&glsl, &annotations, 1)
            .unwrap();
        assert_eq!(parsed.params.len(), other.params.len());
        for (wgsl, glsl) in parsed.params.iter().zip(&other.params) {
            assert_eq!(
                (
                    &wgsl.id,
                    wgsl.ty,
                    &wgsl.aliases,
                    &wgsl.ui,
                    wgsl.canvas,
                    wgsl.bank
                ),
                (
                    &glsl.id,
                    glsl.ty,
                    &glsl.aliases,
                    &glsl.ui,
                    glsl.canvas,
                    glsl.bank
                )
            );
        }
        assert_eq!(parsed.layout, other.layout);
        crate::render::compile_spirv(&parsed.module).expect("production WGSL SPIR-V");
    }

    #[test]
    fn native_padding_and_tail_span_are_uploaded_without_a_glsl_adapter() {
        assert_eq!(parse(&shader("")).unwrap().layout.block_size, 16);
        assert_eq!(parse(&shader("gain: f32,")).unwrap().layout.block_size, 24);
        let parsed = parse(&shader(
            "@align(32) gain: f32, @size(32) tint: vec3f, tail: f32,",
        ))
        .unwrap();
        assert_eq!(
            parsed
                .layout
                .entries
                .iter()
                .map(|e| e.offset)
                .collect::<Vec<_>>(),
            vec![32, 48, 80]
        );
        assert_eq!(parsed.layout.block_size, 96);
        crate::render::compile_spirv(&parsed.module).expect("padded original IR emits SPIR-V");
    }

    #[test]
    fn explicit_padding_cannot_exceed_the_renderer_uniform_limit() {
        assert_eq!(
            wgpu::Limits::default().max_uniform_buffer_binding_size,
            u64::from(MAX_UNIFORM_BYTES)
        );
        assert_eq!(
            parse(&shader("@size(65520) gain: f32,"))
                .unwrap()
                .layout
                .block_size,
            65536
        );
        assert_abi(&shader("@size(65524) gain: f32,"), "WGSL limit is 65536");
        assert_abi(
            &shader("@size(536870912) gain: f32,"),
            "WGSL limit is 65536",
        );
        // Beyond Naga's own type-size limit, parsing rejects first.
        assert!(parse(&shader("@size(1073741824) gain: f32,")).is_err());
    }

    #[test]
    fn io_structs_and_optional_generator_inputs_use_the_same_interface() {
        let base = shader("");
        let generator = base
            .replace("@location(0) uv: vec2<f32>", "")
            .replace("vec4<f32>(uv, fx.u_time, 1.0)", "vec4<f32>(1.0)");
        parse(&generator).unwrap();
        let position = generator.replace("fn main()", "fn main(@builtin(position) pixel: vec4f)");
        parse(&position).unwrap();
        let structured = format!("struct Input {{ @location(0) uv: vec2f, @builtin(position) pixel: vec4f }}\nstruct Output {{ @location(0) color: vec4f }}\n{}", base
            .replace("@location(0) uv: vec2<f32>", "input: Input")
            .replace("-> @location(0) vec4<f32>", "-> Output")
            .replace("return vec4<f32>(uv, fx.u_time, 1.0);", "return Output(vec4f(input.uv, fx.u_time, 1.0));"));
        let parsed = parse(&structured).unwrap();
        crate::render::compile_spirv(&parsed.module).unwrap();
    }

    #[test]
    fn entry_stages_counts_results_and_interpolation_fail_closed() {
        let source = shader("");
        assert_abi(&source.replace("fn main", "fn other"), "@fragment fn main");
        assert_abi(&source.replace("@fragment", "@vertex"), "@fragment fn main");
        assert_abi(
            &format!("{source}\n@compute @workgroup_size(1) fn other() {{}}"),
            "exactly one",
        );
        assert_abi(&source[..source.find("@fragment").unwrap()], "exactly one");
        assert_abi(
            &source.replace("-> @location(0)", "-> @location(1)"),
            "one location-0",
        );
        for input in [
            "@location(1) uv: vec2f",
            "@location(0) uv: vec3f",
            "@location(0) @interpolate(flat) uv: vec2f",
            "@location(0) @interpolate(linear) uv: vec2f",
            "@location(0) @interpolate(perspective, centroid) uv: vec2f",
            "@builtin(front_facing) uv: bool",
            "@builtin(sample_index) uv: u32",
        ] {
            assert_abi(
                &source
                    .replace("@location(0) uv: vec2<f32>", input)
                    .replace("vec4<f32>(uv, fx.u_time, 1.0)", "vec4<f32>(1.0)"),
                "inputs may only",
            );
        }
        for extra in [
            "@location(1) second: vec4f",
            "@builtin(frag_depth) depth: f32",
            "@builtin(sample_mask) mask: u32",
        ] {
            let bad = format!(
                "struct Output {{ @location(0) color: vec4f, {extra} }}\n{}",
                source.replace("-> @location(0) vec4<f32>", "-> Output")
            );
            assert_abi(&bad, "one location-0");
        }
        let missing = source
            .replace("-> @location(0) vec4<f32>", "")
            .replace("return vec4<f32>(uv, fx.u_time, 1.0);", "return;");
        assert_abi(&missing, "missing location-0");
    }

    #[test]
    fn required_uniform_head_has_exact_types_names_order_and_offsets() {
        let source = shader("");
        for bad in [
            source.replace("u_time: f32", "u_time: i32"),
            source.replace("u_frame: f32", "other: f32"),
            source.replace("u_resolution: vec2<f32>", "u_resolution: vec3<f32>"),
            source.replace("u_time: f32", "@align(16) u_time: f32"),
        ] {
            // Make the body independent of head types so semantic errors
            // cannot hide the interface rejection under test.
            assert_abi(
                &bad.replace("vec4<f32>(uv, fx.u_time, 1.0)", "vec4<f32>(1.0)"),
                "head mismatch",
            );
        }
        assert_abi(
            &source.replace("    u_frame: f32,\n", ""),
            "head must start",
        );
        assert_abi(
            &source.replace(
                "@group(0) @binding(2) var<uniform> fx: FxUniforms;",
                "var<private> fx: FxUniforms;",
            ),
            "missing uniform",
        );
        assert_abi(
            &source.replace("var<uniform>", "var<storage, read>"),
            "var<uniform>",
        );
        assert_abi(
            &source
                .replace("var<uniform> fx: FxUniforms", "var<uniform> fx: vec4f")
                .replace("vec4<f32>(uv, fx.u_time, 1.0)", "vec4<f32>(1.0)"),
            "uniform struct",
        );
        parse(&source.replace("FxUniforms", "MyParameters")).unwrap();
    }

    #[test]
    fn wrong_output_types_and_duplicate_uv_inputs_are_rejected() {
        let source = shader("");
        for ty in ["vec3<f32>", "vec4<i32>", "f32"] {
            let bad = source
                .replace(
                    "-> @location(0) vec4<f32>",
                    &format!("-> @location(0) {ty}"),
                )
                .replace("vec4<f32>(uv, fx.u_time, 1.0)", &format!("{ty}(0)"));
            assert_abi(&bad, "one location-0");
        }
        assert_abi(
            &source.replace("@location(0) uv: vec2<f32>", "uv: vec2<f32>"),
            "explicit location/builtin",
        );
        assert_abi(
            &source.replace(
                "@location(0) uv: vec2<f32>",
                "@location(0) uv: vec2<f32>, @location(0) other: vec2f",
            ),
            "inputs may only",
        );
    }

    #[test]
    fn resource_types_are_checked_even_when_unused() {
        let source = shader("");
        for ty in [
            "texture_cube<f32>",
            "texture_2d_array<f32>",
            "texture_multisampled_2d<f32>",
            "texture_depth_2d",
            "texture_2d<i32>",
            "texture_2d<u32>",
            "sampler",
            "texture_storage_2d<rgba8unorm, write>",
        ] {
            assert_abi(
                &format!("{source}\n@group(0) @binding(0) var image: {ty};"),
                "binding 0",
            );
        }
        assert_abi(
            &format!("{source}\n@group(0) @binding(1) var s: sampler_comparison;"),
            "non-comparison sampler",
        );
        assert_abi(
            &format!("{source}\n@group(1) @binding(0) var image: texture_2d<f32>;"),
            "group 1 is reserved",
        );
        assert_abi(
            &format!("{source}\n@group(0) @binding(16) var image: texture_2d<f32>;"),
            "outside the v1",
        );
        assert_abi(
            &format!("{source}\nvar<workgroup> shared_value: u32;"),
            "unbound resource",
        );
        assert_abi(&format!("{source}\n@group(0) @binding(0) var a: texture_2d<f32>;\n@group(0) @binding(0) var b: texture_2d<f32>;"), "duplicate binding");
    }

    #[test]
    fn sampling_and_manifest_extra_resources_compile_to_spirv() {
        let mut source = shader("").replace("return vec4<f32>(uv, fx.u_time, 1.0);", "return textureSample(a, s, uv) + textureSample(b, s, uv) + textureSample(c, s, uv) + textureSample(d, s, uv);");
        source.push_str("\n@group(0) @binding(0) var a: texture_2d<f32>;\n@group(0) @binding(1) var s: sampler;\n@group(0) @binding(5) var d: texture_2d<f32>;\n@group(0) @binding(3) var b: texture_2d<f32>;\n@group(0) @binding(4) var c: texture_2d<f32>;\n");
        let parsed = parse_inputs(&source, 4).unwrap();
        assert_eq!(parsed.extra_input_bindings, vec![3, 4, 5]);
        crate::render::compile_spirv(&parsed.module).unwrap();
        assert!(
            matches!(parse_inputs(&source, 3), Err(FrontendError::Abi(ref message)) if message.contains("no manifest input"))
        );
        assert!(matches!(
            parse_inputs(&source, 0),
            Err(FrontendError::Abi(_))
        ));
        assert!(matches!(
            parse_inputs(&source, usize::MAX),
            Err(FrontendError::Abi(_))
        ));
        let wrong = shader("") + "\n@group(0) @binding(3) var image: texture_depth_2d;";
        assert!(
            matches!(parse_inputs(&wrong, 2), Err(FrontendError::Abi(ref message)) if message.contains("texture_2d<f32>"))
        );
    }

    #[test]
    fn unsupported_uniform_members_and_optional_features_are_rejected() {
        for ty in ["u32", "vec2<i32>", "mat4x4f", "array<f32, 4>"] {
            assert!(
                matches!(
                    parse(&shader(&format!("unsupported: {ty},"))),
                    Err(FrontendError::Param(_)) | Err(FrontendError::Parse(_))
                ),
                "{ty}"
            );
        }
        assert!(
            parse(&shader("flag: bool,")).is_err(),
            "WGSL bool is not host-shareable"
        );
        assert_abi(
            &format!("override gain: f32 = 1.0;\n{}", shader("")),
            "pipeline overrides",
        );
        let f16 = format!(
            "enable f16;\n{}",
            shader("").replace(
                "return vec4<f32>(uv, fx.u_time, 1.0);",
                "let h: f16 = 1.0h; return vec4<f32>(f32(h));"
            )
        );
        assert!(
            matches!(parse(&f16), Err(FrontendError::Parse(_))),
            "f16 is not requested on the runtime device"
        );
        let subgroup = format!(
            "enable subgroups;\n{}",
            shader("").replace(
                "return vec4<f32>(uv, fx.u_time, 1.0);",
                "return subgroupAdd(vec4f(1.0));"
            )
        );
        assert!(
            parse(&subgroup).is_err(),
            "subgroups are not requested on the runtime device"
        );
    }

    #[test]
    fn annotation_semantics_are_identical_and_errors_do_not_half_apply() {
        for (member, annotation, class) in [
            ("x: i32,", "hint:angle", "param"),
            ("x: f32,", "hint:bool", "param"),
            ("x: f32,", "hint:point3d", "param"),
            ("x: vec2f,", "default:0,0", "param"),
            ("x: vec3f,", "hint:point3d default:0,0,1", "param"),
            ("x: f32,", "hint:layer", "param"),
            ("x: f32,", "hint:gradient", "param"),
            ("x: f32,", "hint:path", "param"),
            ("x: i32,", "hint:canvas", "canvas"),
            ("x: f32,", "min:2 max:1", "param"),
            // Existing GLSL behavior: HEX expands to RGBA, which vec3
            // rejects before upload. This change does not redefine it.
            ("x: vec3f,", "hint:color default:#112233", "param"),
        ] {
            let error =
                parse(&format!("// @param x {annotation}\n{}", shader(member))).unwrap_err();
            assert!(
                match class {
                    "canvas" => matches!(error, FrontendError::CanvasWrongKind(_)),
                    _ => matches!(error, FrontendError::Param(_)),
                },
                "{member} {annotation}: {error:?}"
            );
        }
        assert!(matches!(
            parse(&shader("dfx_reserved: f32,")),
            Err(FrontendError::Param(_))
        ));
        assert_eq!(
            parse(&format!("// @param stale default:1\n{}", shader("")))
                .unwrap()
                .params
                .len(),
            0
        );
    }

    #[test]
    fn parse_and_validation_diagnostics_preserve_unicode_line_and_column() {
        let source = format!("// 中文说明\r\n{}", shader("")).replace(
            "return vec4<f32>(uv, fx.u_time, 1.0);",
            "return no_such_value;",
        );
        let error = parse(&source).unwrap_err();
        assert!(
            matches!(error, FrontendError::Parse(ref message) if message.starts_with("line 10:") && message.contains("no_such_value")),
            "{error:?}"
        );
        let invalid = shader("").replace(
            "return vec4<f32>(uv, fx.u_time, 1.0);",
            "return vec3f(1.0);",
        );
        let error = parse(&invalid).unwrap_err();
        assert!(
            matches!(error, FrontendError::Parse(ref message) if message.starts_with("line ") && message.contains("invalid")),
            "{error:?}"
        );
    }

    #[test]
    fn wgsl_attributes_escape_in_envelopes_and_source_lines_follow_manifest_order() {
        let body = shader("gain: f32,");
        let escaped = body
            .lines()
            .map(|line| {
                if line.starts_with('@') {
                    format!("@{line}\n")
                } else {
                    format!("{line}\n")
                }
            })
            .collect::<String>();
        let source = format!("@dynamicfx 1\n@graph\npass first: input -> tmp\npass second: tmp, input -> output\n@end\n@pass second\n{escaped}@endpass\n@pass first\n{escaped}@endpass\n");
        let envelope = super::super::grammar::parse_envelope(&source).unwrap();
        assert_eq!(envelope.bodies, vec![body.clone(), body]);
        assert_eq!(envelope.body_start_lines, vec![19, 7]);
        let crlf = super::super::grammar::parse_envelope(&source.replace('\n', "\r\n")).unwrap();
        assert_eq!(crlf.body_start_lines, envelope.body_start_lines);
        assert_eq!(crlf.bodies, envelope.bodies);
        let modules = envelope
            .bodies
            .iter()
            .zip(&envelope.passes)
            .map(|(body, pass)| parse_inputs(body, pass.inputs.len()).unwrap())
            .collect::<Vec<_>>();
        let params = modules
            .iter()
            .map(|module| module.params.clone())
            .collect::<Vec<_>>();
        let (definition, maps) = crate::definition::effect::lower_graph(
            LanguageId::WGSL,
            &envelope.passes,
            &envelope.bodies,
            &params,
            None,
        )
        .unwrap();
        assert_eq!(definition.params.len(), 1);
        assert_eq!(maps, vec![vec![0], vec![0]]);
        assert_eq!(definition.language, LanguageId::WGSL);
        assert!(
            super::super::grammar::parse_envelope(&source.replace("@@fragment", "@fragment"))
                .is_err()
        );
    }

    /// Compile the exact shipped source bytes through the production frontend,
    /// graph and SPIR-V path. GPU/AE acceptance is a separate evidence row.
    #[test]
    fn shipped_wgsl_examples_compile() {
        for (source, pass_count, param_count) in [
            (include_str!("../../examples/wgsl-field.wgsl"), 1, 7),
            (include_str!("../../examples/wgsl-multipass.wgsl"), 2, 6),
        ] {
            let envelope = super::super::grammar::parse_envelope(source).unwrap();
            let annotations = super::super::annotation::parse_annotations(source).unwrap();
            let modules = envelope
                .bodies
                .iter()
                .zip(&envelope.passes)
                .map(|(body, pass)| {
                    let module = WgslFrontend
                        .parse_module(body, &annotations, pass.inputs.len())
                        .unwrap();
                    crate::render::compile_spirv(&module.module).unwrap();
                    module
                })
                .collect::<Vec<_>>();
            assert_eq!(modules.len(), pass_count);
            if pass_count == 2 {
                assert_eq!(modules[1].extra_input_bindings, vec![3]);
            }
            let params = modules.iter().map(|m| m.params.clone()).collect::<Vec<_>>();
            let (definition, _) = crate::definition::effect::lower_graph(
                LanguageId::WGSL,
                &envelope.passes,
                &envelope.bodies,
                &params,
                None,
            )
            .unwrap();
            assert_eq!(definition.params.len(), param_count);
        }
    }
}
