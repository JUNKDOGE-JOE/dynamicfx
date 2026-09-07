//! Non-shipping WGSL feasibility instrument. It deliberately imports production
//! graph, binding, layout and GPU code, without the AE SDK or a second renderer.
#![allow(dead_code)]

#[path = "../../../src/binding.rs"]
mod binding;
#[path = "../../../src/definition/mod.rs"]
mod definition;
#[path = "../../../src/frontend/mod.rs"]
mod frontend;
#[path = "../../../src/plan.rs"]
mod plan;
#[path = "../../../src/render.rs"]
mod render;

mod diag {
    pub fn log(message: &str) {
        eprintln!("{message}");
    }
}

#[cfg(test)]
use frontend::LanguageId;
use frontend::{LanguageFrontend, PassModule};
use naga::{AddressSpace, Binding, ScalarKind, ShaderStage, TypeInner, VectorSize};
use std::{collections::HashMap, path::Path};

const WGSL: &str = include_str!("../fixtures/field.wgsl");
const GLSL: &str = include_str!("../fixtures/field.glsl");
const MIX: &str = include_str!("../fixtures/mix.wgsl");
const MIX_GLSL: &str = include_str!("../fixtures/mix.glsl");

fn validate(source: &str) -> Result<naga::Module, String> {
    let module = naga::front::wgsl::parse_str(source).map_err(|e| e.emit_to_string(source))?;
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .map_err(|e| e.emit_to_string(source))?;
    Ok(module)
}

fn vector(inner: &TypeInner, size: VectorSize) -> bool {
    matches!(inner, TypeInner::Vector { size: s, scalar: naga::Scalar { kind: ScalarKind::Float, width: 4 } } if *s == size)
}

/// This limited adapter uses a generated GLSL *schema* to exercise the existing
/// production annotation/type rules. Shader code is never translated through
/// GLSL: the original validated WGSL Naga module is what goes to the GPU.
/// A shipping frontend should extract shared IR reflection from glsl.rs instead.
fn parse_wgsl(
    source: &str,
    annotations: &HashMap<String, frontend::annotation::Annotation>,
    allowed_inputs: usize,
) -> Result<PassModule, String> {
    let module = validate(source)?;
    if module.entry_points.len() != 1 {
        return Err("require exactly one entry point".into());
    }
    let entry = &module.entry_points[0];
    if entry.name != "main" || entry.stage != ShaderStage::Fragment {
        return Err("require @fragment fn main".into());
    }
    let location0 =
        |binding: &Option<Binding>| matches!(binding, Some(Binding::Location { location: 0, .. }));
    if entry.function.arguments.len() != 1
        || !location0(&entry.function.arguments[0].binding)
        || !vector(
            &module.types[entry.function.arguments[0].ty].inner,
            VectorSize::Bi,
        )
    {
        return Err("require one @location(0) vec2<f32> input".into());
    }
    let result = entry.function.result.as_ref().ok_or("missing output")?;
    if !location0(&result.binding) || !vector(&module.types[result.ty].inner, VectorSize::Quad) {
        return Err("require one @location(0) vec4<f32> output".into());
    }
    if !module.overrides.is_empty() {
        return Err("pipeline overrides are outside this spike's ABI".into());
    }
    let mut found = [false; 3];
    let mut block = None;
    let mut extra = Vec::new();
    for (_, var) in module.global_variables.iter() {
        let Some(binding) = &var.binding else {
            if !matches!(var.space, AddressSpace::Private) {
                return Err("unsupported unbound resource/address space".into());
            }
            continue;
        };
        if binding.group != 0 {
            return Err("descriptor groups >= 1 are reserved".into());
        }
        let inner = &module.types[var.ty].inner;
        let texture = matches!(
            inner,
            TypeInner::Image {
                dim: naga::ImageDimension::D2,
                arrayed: false,
                class: naga::ImageClass::Sampled {
                    kind: ScalarKind::Float,
                    multi: false
                }
            }
        ) && var.space == AddressSpace::Handle;
        match binding.binding {
            0 if texture => found[0] = true,
            1 if matches!(inner, TypeInner::Sampler { comparison: false }) => found[1] = true,
            2 if var.space == AddressSpace::Uniform => {
                found[2] = true;
                block = Some(inner);
            }
            b @ 3..=15 if texture && b <= 2 + allowed_inputs.saturating_sub(1) as u32 => {
                extra.push(b)
            }
            b => return Err(format!("unsupported resource or unfed binding {b}")),
        }
    }
    if found != [true; 3] {
        return Err("missing required ABI resource 0/1/2".into());
    }
    let Some(TypeInner::Struct { members, span }) = block else {
        return Err("uniform is not a struct".into());
    };

    let mut mirror = String::from("#version 450\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2,std140) uniform FxUniforms {\n");
    for member in members {
        let name = member.name.as_deref().ok_or("unnamed uniform member")?;
        let ty = match &module.types[member.ty].inner {
            TypeInner::Scalar(naga::Scalar {
                kind: ScalarKind::Float,
                width: 4,
            }) => "float",
            TypeInner::Scalar(naga::Scalar {
                kind: ScalarKind::Sint,
                width: 4,
            }) => "int",
            inner if vector(inner, VectorSize::Bi) => "vec2",
            inner if vector(inner, VectorSize::Tri) => "vec3",
            inner if vector(inner, VectorSize::Quad) => "vec4",
            _ => return Err(format!("{name}: type outside ABI v1")),
        };
        mirror.push_str(&format!("{ty} {name};\n"));
    }
    mirror.push_str("};\nvoid main(){outColor=vec4(v_uv,0,1);}\n");
    let mut reflected = frontend::glsl::GlslFrontend
        .parse_module(&mirror, annotations, 1)
        .map_err(|e| format!("shared parameter contract: {e:?}"))?;
    for (member, expected) in members.iter().take(3).zip([0, 8, 12]) {
        if member.offset != expected {
            return Err("WGSL builtin head offset mismatch".into());
        }
    }
    for (member, layout) in members.iter().skip(3).zip(&reflected.layout.entries) {
        if member.offset as usize != layout.offset {
            return Err(format!(
                "WGSL/std140 offset mismatch for {:?}: {} vs {}",
                member.name, member.offset, layout.offset
            ));
        }
    }
    // Use the production reflected allocation, not a guessed Rust struct size.
    // Reject layouts that would read beyond it, including explicit WGSL sizes.
    if *span as usize > reflected.layout.block_size {
        return Err("WGSL uniform span exceeds shared block allocation".into());
    }
    extra.sort_unstable();
    reflected.module = module;
    reflected.extra_input_bindings = extra;
    Ok(reflected)
}

fn parse(source: &str) -> Result<PassModule, String> {
    let annotations =
        frontend::annotation::parse_annotations(source).map_err(|e| format!("{e:?}"))?;
    parse_wgsl(source, &annotations, 1)
}

fn emit_msl(module: &naga::Module) -> Result<String, String> {
    use naga::back::msl;
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(module)
    .map_err(|e| format!("{e:?}"))?;
    let mut resources = msl::EntryPointResources::default();
    for (_, var) in module.global_variables.iter() {
        if let Some(binding) = &var.binding {
            let slot = binding.binding as u8;
            let target = match module.types[var.ty].inner {
                TypeInner::Image { .. } => msl::BindTarget {
                    texture: Some(slot),
                    ..Default::default()
                },
                TypeInner::Sampler { .. } => msl::BindTarget {
                    sampler: Some(msl::BindSamplerTarget::Resource(slot)),
                    ..Default::default()
                },
                _ => msl::BindTarget {
                    buffer: Some(slot),
                    ..Default::default()
                },
            };
            resources.resources.insert(*binding, target);
        }
    }
    let mut options = msl::Options {
        lang_version: (2, 0),
        fake_missing_bindings: false,
        ..Default::default()
    };
    options.per_entry_point_map.insert("main".into(), resources);
    let (source, translated) =
        msl::write_string(module, &info, &options, &msl::PipelineOptions::default())
            .map_err(|e| format!("{e:?}"))?;
    for entry in translated.entry_point_names {
        entry.map_err(|e| format!("{e:?}"))?;
    }
    Ok(source)
}

fn escape_body(source: &str) -> String {
    source
        .lines()
        .map(|line| {
            let content = line.trim_start();
            if content.starts_with('@') {
                format!("{}@{content}\n", &line[..line.len() - content.len()])
            } else {
                format!("{line}\n")
            }
        })
        .collect()
}

fn manifest(two: bool) -> Vec<frontend::grammar::ManifestPass> {
    let src = if two {
        format!("@dynamicfx 1\n@graph\npass field: input -> field_out\npass mix: field_out, input -> output\n@end\n@pass field\n{}@endpass\n@pass mix\n{}@endpass\n", escape_body(WGSL), escape_body(MIX))
    } else {
        format!(
            "@dynamicfx 1\n@graph\npass field: input -> output\n@end\n@pass field\n{}@endpass\n",
            escape_body(WGSL)
        )
    };
    frontend::grammar::parse_envelope(&src).unwrap().passes
}

fn gpu_render(
    gpu: &render::Gpu,
    modules: &[PassModule],
    depth: render::Depth,
) -> Result<Vec<u8>, String> {
    let (width, height) = (256, 128);
    let passes = modules
        .iter()
        .map(|pm| {
            let spv = render::compile_spirv(&pm.module)?;
            render::build_pipeline(gpu, &spv, &pm.layout, &pm.extra_input_bindings, depth)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let set = render::PipelineSet {
        token: 1,
        depth,
        passes,
    };
    let plan = plan::build_plan(&manifest(modules.len() == 2), true);
    let samples: Vec<[f32; 4]> = (0..width * height)
        .map(|i| {
            [
                (i % width) as f32 / (width - 1) as f32,
                (i / width) as f32 / (height - 1) as f32,
                0.25,
                0.5,
            ]
        })
        .collect();
    let input = render::encode_samples(&samples, depth);
    let mut output = vec![0; input.len()];
    let values: Vec<Vec<[f32; 4]>> = modules
        .iter()
        .map(|pm| {
            pm.params
                .iter()
                .map(|p| match p.id.as_str() {
                    "gain" => [0.72, 0., 0., 0.],
                    "tint" => [0.25, 0.7, 1.2, 0.],
                    "enabled" => [1., 0., 0., 0.],
                    _ => panic!("unexpected parameter"),
                })
                .collect()
        })
        .collect();
    let mut cache = None;
    render::ensure_frame_cache(
        gpu,
        &mut cache,
        1,
        depth,
        width,
        height,
        plan.physical_count,
        false,
    );
    render::execute_plan(
        gpu,
        &set,
        &plan,
        &values,
        &input,
        width * depth.bpp(),
        width,
        height,
        1.25,
        30.,
        (width as f32, height as f32),
        &mut output,
        width * depth.bpp(),
        None,
        (0, 0, width, height),
        &[],
        cache.as_mut().unwrap(),
    )?;
    Ok(output)
}

fn main() -> Result<(), String> {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "spike/wgsl/out".into());
    let out = Path::new(&out);
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let annotations = frontend::annotation::parse_annotations(WGSL).unwrap();
    let wgsl = parse(WGSL)?;
    let glsl = frontend::glsl::GlslFrontend
        .parse_module(GLSL, &annotations, 1)
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(wgsl.params.len(), glsl.params.len());
    for (a, b) in wgsl.params.iter().zip(&glsl.params) {
        assert_eq!(
            (&a.id, a.ty, &a.aliases, &a.ui, a.canvas),
            (&b.id, b.ty, &b.aliases, &b.ui, b.canvas)
        );
    }
    assert_eq!(wgsl.layout, glsl.layout);
    println!(
        "CONTRACT uniform_layout={:?} params={:?}",
        wgsl.layout, wgsl.params
    );
    let mix = parse_wgsl(MIX, &HashMap::new(), 2)?;
    let mix_glsl = frontend::glsl::GlslFrontend
        .parse_module(MIX_GLSL, &HashMap::new(), 2)
        .map_err(|e| format!("{e:?}"))?;
    for (name, module) in [("field", &wgsl.module), ("mix", &mix.module)] {
        let msl = emit_msl(module)?;
        println!("MSL name={name} bytes={} RESULT=PASS", msl.len());
        std::fs::write(out.join(format!("{name}.metal")), msl).map_err(|e| e.to_string())?;
    }
    let gpu = render::gpu()
        .ok_or("no GPU adapter; run on a host with Metal/DX12 or explicit diagnostic backend")?;
    println!("GPU {} features={:?}", gpu.adapter_summary, gpu.features);
    if cfg!(target_os = "macos") && !gpu.adapter_summary.contains("backend=Metal") {
        return Err("macOS result did not select Metal".into());
    }
    for depth in [render::Depth::U8, render::Depth::U15, render::Depth::F32] {
        if !depth.supported_by(gpu) {
            return Err(format!("unsupported depth {depth:?}"));
        }
        for two in [false, true] {
            let wm = if two {
                vec![wgsl.clone(), mix.clone()]
            } else {
                vec![wgsl.clone()]
            };
            let gm = if two {
                vec![glsl.clone(), mix_glsl.clone()]
            } else {
                vec![glsl.clone()]
            };
            let actual = gpu_render(gpu, &wm, depth)?;
            let expected = gpu_render(gpu, &gm, depth)?;
            let max_error = if depth == render::Depth::U8 {
                actual
                    .iter()
                    .zip(&expected)
                    .map(|(a, b)| a.abs_diff(*b) as f32)
                    .fold(0., f32::max)
            } else {
                actual
                    .chunks_exact(4)
                    .zip(expected.chunks_exact(4))
                    .map(|(a, b)| {
                        let a = f32::from_le_bytes(a.try_into().unwrap());
                        let b = f32::from_le_bytes(b.try_into().unwrap());
                        assert!(a.is_finite() && b.is_finite());
                        (a - b).abs()
                    })
                    .fold(0., f32::max)
            };
            let tolerance = if depth == render::Depth::U8 {
                1.0
            } else {
                0.00002
            };
            assert!(max_error <= tolerance, "WGSL/GLSL mismatch {max_error}");
            let pixel_size = depth.bpp();
            let non_constant = actual
                .chunks_exact(pixel_size)
                .any(|pixel| pixel != &actual[..pixel_size]);
            assert!(
                non_constant && actual != vec![0; actual.len()],
                "black/constant output is not success"
            );
            println!("PIXELS depth={depth:?} passes={} extent=256x128 max_abs_error={max_error:.9} tolerance={tolerance} bytes_equal={} RESULT=PASS", wm.len(), actual == expected);
            let stem = format!("{depth:?}-{}pass", wm.len());
            std::fs::write(out.join(format!("{stem}.rgba")), &actual).map_err(|e| e.to_string())?;
            if depth != render::Depth::U8 {
                let max = actual
                    .chunks_exact(4)
                    .map(|v| f32::from_le_bytes(v.try_into().unwrap()))
                    .fold(f32::NEG_INFINITY, f32::max);
                println!(
                    "RANGE depth={depth:?} passes={} max_component={max}",
                    wm.len()
                );
            }
        }
    }
    println!("RESULT=PASS WGSL parsing/reflection/SPIR-V/common-renderer/Metal readback; AE frontend is not enabled");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wgsl_all_supported_members_have_glsl_offsets_and_annotations() {
        let source = WGSL
            .replace("gain: f32,", "gain: f32,\n count: i32,\n point: vec2<f32>,")
            .replace(
                "enabled: i32,",
                "enabled: i32,\n rgba: vec4<f32>,\n direction: vec3<f32>,\n tail: f32,",
            );
        let pm = parse(&source).unwrap();
        assert_eq!(
            pm.layout
                .entries
                .iter()
                .map(|e| e.offset)
                .collect::<Vec<_>>(),
            vec![16, 20, 24, 32, 44, 48, 64, 76]
        );
        assert_eq!(pm.layout.block_size, 80);
        assert_eq!(
            pm.params
                .iter()
                .find(|p| p.id.as_str() == "enabled")
                .unwrap()
                .ty,
            definition::param::ShaderParamType::Bool
        );
    }

    #[test]
    fn wgsl_envelope_reuses_explicit_at_escape() {
        let escaped = format!(
            "@dynamicfx 1\n@graph\npass field: input -> output\n@end\n@pass field\n{}@endpass\n",
            escape_body(WGSL)
        );
        let env = frontend::grammar::parse_envelope(&escaped).unwrap();
        assert_eq!(env.bodies[0], WGSL);
        parse(&env.bodies[0]).unwrap();
        let err = frontend::grammar::parse_envelope(&escaped.replace("@@", "@")).unwrap_err();
        assert!(err.message.contains("unknown directive"));
    }

    #[test]
    fn wgsl_bool_storage_and_unsupported_abi_fail() {
        let bool_source = WGSL
            .replace("enabled: i32", "enabled: bool")
            .replace("fx.enabled!=0", "fx.enabled");
        let bool_error = validate(&bool_source).unwrap_err();
        assert!(bool_error.contains("not host-shareable"), "{bool_error}");
        assert!(parse(&WGSL.replace("@fragment", "@compute @workgroup_size(1)")).is_err());
        assert!(parse(&WGSL.replace("@group(0)", "@group(1)")).is_err());
        assert!(parse(&WGSL.replace(
            "gain: f32,",
            "@align(16) gain: f32,\n @align(16) offset: f32,"
        ))
        .is_err());
        assert!(parse(&WGSL.replace("gain: f32,", "gain: f32,\n samples: array<f32,4>,")).is_err());
    }

    #[test]
    fn wgsl_natural_tail_span_matches_production_reflection() {
        let source = WGSL
            .replace("    tint: vec3<f32>,\n", "")
            .replace("    enabled: i32,\n", "")
            .replace("fx.tint", "vec3<f32>(1.0)")
            .replace("fx.enabled!=0", "true");
        let pm = parse(&source).unwrap();
        assert_eq!(pm.layout.block_size, 24);
        assert_eq!(pm.layout.entries[0].offset, 16);
        let block = pm
            .module
            .global_variables
            .iter()
            .find(|(_, var)| var.space == AddressSpace::Uniform)
            .unwrap()
            .1;
        assert!(matches!(
            pm.module.types[block.ty].inner,
            TypeInner::Struct { span: 24, .. }
        ));
    }

    #[test]
    fn wgsl_parse_error_preserves_source_location() {
        let error = validate(&WGSL.replace("return mix", "return missing_function")).unwrap_err();
        assert!(
            error.contains("missing_function") && error.contains("wgsl:"),
            "{error}"
        );
    }

    #[test]
    fn wgsl_resource_reflection_is_bound_to_manifest() {
        assert!(parse_wgsl(MIX, &HashMap::new(), 1).is_err());
        assert_eq!(
            parse_wgsl(MIX, &HashMap::new(), 2)
                .unwrap()
                .extra_input_bindings,
            vec![3]
        );
        assert!(
            parse(&WGSL.replace("u_input: texture_2d<f32>", "u_input: texture_cube<f32>")).is_err()
        );
    }

    #[test]
    fn wgsl_language_switch_and_alias_keep_param_slots() {
        let annotations = frontend::annotation::parse_annotations(WGSL).unwrap();
        let glsl = frontend::glsl::GlslFrontend
            .parse_module(GLSL, &annotations, 1)
            .unwrap();
        let wgsl = parse(WGSL).unwrap();
        let first = binding::build_fresh(&glsl.params).unwrap();
        let second = binding::build_with_reuse(&wgsl.params, &first).unwrap();
        assert!(second.bindings.iter().all(|b| b.inherited));
        assert_eq!(
            first.mapping().collect::<Vec<_>>(),
            second.mapping().collect::<Vec<_>>()
        );
        let renamed = WGSL
            .replace("gain", "strength")
            .replace("label:\"Gain\"", "label:\"Gain\" alias:gain");
        let third = binding::build_with_reuse(&parse(&renamed).unwrap().params, &second).unwrap();
        assert!(third.bindings.iter().all(|b| b.inherited));
    }

    #[test]
    fn wgsl_multi_pass_lowers_through_existing_definition() {
        let modules = [
            parse(WGSL).unwrap(),
            parse_wgsl(MIX, &HashMap::new(), 2).unwrap(),
        ];
        let (def, maps) = definition::effect::lower_graph(
            LanguageId::WGSL,
            &manifest(true),
            &[WGSL.into(), MIX.into()],
            &modules.iter().map(|m| m.params.clone()).collect::<Vec<_>>(),
            None,
        )
        .unwrap();
        assert_eq!(def.language, LanguageId(2));
        assert_eq!(def.graph.passes.len(), 2);
        assert_eq!(maps[0].len(), 3);
        assert!(maps[1].is_empty());
        assert_eq!(plan::build_plan(&manifest(true), true).physical_count, 1);
        // The production language menu remains GLSL-only throughout this spike.
        assert_eq!(frontend::popup_menu(), vec!["GLSL"]);
        assert!(frontend::frontend_for(LanguageId::WGSL).is_none());
    }
}
