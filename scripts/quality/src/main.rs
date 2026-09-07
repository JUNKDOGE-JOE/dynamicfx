#![allow(dead_code)]
#[path = "../../../src/binding.rs"] mod binding;
#[path = "../../../src/definition/mod.rs"] mod definition;
#[path = "../../../src/frontend/mod.rs"] mod frontend;
#[path = "../../../src/plan.rs"] mod plan;
// Preserve diagnostics while keeping this evidence runner's output self-contained.
mod diag { pub fn log(s: &str) { eprintln!("{s}"); } }
include!(concat!(env!("OUT_DIR"), "/render_module.rs"));

use frontend::LanguageId;
use std::{fs, path::Path};

fn render_file(args: &[String]) -> Result<(), String> {
    if args.len() != 10 {
        return Err("usage: dynamicfx-quality SOURCE OUT.f32 WIDTH HEIGHT LOGICAL_WIDTH LOGICAL_HEIGHT TIME DEPTH INPUT_PATTERN\nDEPTH: 8|16|32; INPUT_PATTERN: black|checker|ramp".into());
    }
    let source = fs::read_to_string(&args[1]).map_err(|e| e.to_string())?;
    // CLI-only convenience. AE always uses the explicit Language popup.
    let language = if Path::new(&args[1]).extension().is_some_and(|e| e == "wgsl") {
        LanguageId::WGSL
    } else { LanguageId::GLSL };
    let compiler = frontend::frontend_for(language).ok_or("language unavailable")?;
    let env = frontend::grammar::parse_envelope(&source).map_err(|e| format!("{e:?}"))?;
    let annotations = frontend::annotation::parse_annotations(&source).map_err(|e| format!("{e:?}"))?;
    let modules: Vec<_> = env.bodies.iter().zip(&env.passes)
        .map(|(b, p)| compiler.parse_module(b, &annotations, p.inputs.len()))
        .collect::<Result<_, _>>().map_err(|e| format!("{e:?}"))?;
    let parse = |i: usize| args[i].parse::<usize>().map_err(|e| e.to_string());
    let (w, h) = (parse(3)?, parse(4)?);
    let logical = (parse(5)? as f32, parse(6)? as f32);
    let time = args[7].parse::<f32>().map_err(|e| e.to_string())?;
    let depth = match args[8].as_str() { "8" => render::Depth::U8, "16" => render::Depth::U15, "32" => render::Depth::F32, _ => return Err("depth must be 8, 16, or 32".into()) };
    let gpu = render::gpu().ok_or("GPU unavailable")?;
    if !depth.supported_by(gpu) { return Err("GPU lacks requested working-depth features".into()); }
    let passes = modules.iter().map(|m| {
        let spv = render::compile_spirv(&m.module)?;
        render::build_pipeline(gpu, &spv, &m.layout, &m.extra_input_bindings, depth)
    }).collect::<Result<Vec<_>, String>>()?;
    let set = render::PipelineSet { token: 1, depth, passes };
    let values: Vec<Vec<[f32; 4]>> = modules.iter().map(|m| m.params.iter().map(|p| {
        let mut value = [0.; 4];
        if let Some(default) = &p.ui.default {
            for (out, x) in value.iter_mut().zip(default) { *out = *x; }
        }
        value
    }).collect()).collect();
    let samples: Vec<[f32; 4]> = (0..w*h).map(|i| {
        let (x,y) = (i % w, i / w);
        let v = match args[9].as_str() {
            "black" => 0.,
            "checker" => ((x + y) % 2) as f32,
            "ramp" => x as f32 / (w-1).max(1) as f32,
            _ => 0.,
        };
        [v, v, v, 1.]
    }).collect();
    let input = render::encode_samples(&samples, depth);
    let lut_samples: Vec<[f32; 4]> = (0..256).map(|i| { let v = i as f32/255.; [v,v,v,1.] }).collect();
    let lut = render::encode_samples(&lut_samples, depth);
    let external_names = plan::external_order(&env.passes);
    let externals: Vec<_> = external_names.iter().map(|name| (name == "quality_lut").then_some(render::ExternalTexture {
        pixels: &lut, stride: 256*depth.bpp(), width: 256, height: 1, float32: false,
    })).collect();
    let execution = plan::build_plan(&env.passes, true);
    if env.uses_prev { return Err("quality runner only accepts stateless fixtures".into()); }
    let mut cache = None;
    render::ensure_frame_cache(gpu, &mut cache, 1, depth, w, h, execution.physical_count, false);
    let mut output = vec![0; w*h*depth.bpp()];
    render::execute_plan(gpu, &set, &execution, &values, &input, w*depth.bpp(), w, h,
        time, time*30., logical, &mut output, w*depth.bpp(), None, (0,0,w,h), &externals, cache.as_mut().unwrap())?;
    // A stable RGBA f32 dump, regardless of working depth, is convenient for
    // numerical analysis. U15 is still the production f32 working buffer;
    // AE's final U15 quantization is explicitly outside this headless test.
    let floats: Vec<f32> = if depth == render::Depth::U8 {
        output.iter().map(|b| *b as f32 / 255.).collect()
    } else { output.chunks_exact(4).map(|v| f32::from_ne_bytes(v.try_into().unwrap())).collect() };
    if floats.iter().any(|x| !x.is_finite()) { return Err("non-finite output".into()); }
    let bytes: Vec<u8> = floats.iter().flat_map(|f| f.to_le_bytes()).collect();
    let out = Path::new(&args[2]);
    if let Some(dir) = out.parent() { fs::create_dir_all(dir).map_err(|e| e.to_string())?; }
    fs::write(out, bytes).map_err(|e| e.to_string())?;
    println!("{}", serde_json::json!({"source":args[1],"language_id":language.0,"source_blake3":blake3::hash(source.as_bytes()).to_hex().as_str(),"renderer_blake3":blake3::hash(RENDER_SOURCE.as_bytes()).to_hex().as_str(),"output":args[2],"width":w,"height":h,"logical":logical,"time":time,"working_depth":args[8],"passes":env.passes.len(),"adapter":gpu.adapter_summary,"finite":true}));
    Ok(())
}

fn main() {
    if let Err(e) = render_file(&std::env::args().collect::<Vec<_>>()) { eprintln!("{e}"); std::process::exit(1); }
}
