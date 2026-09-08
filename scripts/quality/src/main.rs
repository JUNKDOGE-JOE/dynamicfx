#![allow(dead_code)]
#[path = "../../../src/binding.rs"] mod binding;
#[path = "../../../src/definition/mod.rs"] mod definition;
#[path = "../../../src/frontend/mod.rs"] mod frontend;
#[path = "../../../src/plan.rs"] mod plan;
#[path = "../../../src/diagnostics.rs"] mod diagnostics;
// Preserve diagnostics while keeping this evidence runner's output self-contained.
mod diag { pub fn log(s: &str) { eprintln!("{s}"); } }
include!(concat!(env!("OUT_DIR"), "/render_module.rs"));

use frontend::LanguageId;
use std::{fs, path::Path};

fn render_file(args: &[String]) -> Result<(), String> {
    if args.len() != 10 && args.len() != 11 {
        return Err("usage: dynamicfx-quality SOURCE OUT.f32 WIDTH HEIGHT LOGICAL_WIDTH LOGICAL_HEIGHT TIME DEPTH INPUT_PATTERN [EXTERNALS.json]\nDEPTH: 8|16|32; INPUT_PATTERN: black|checker|ramp|rgba:PATH".into());
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
    let samples: Vec<[f32; 4]> = if let Some(path) = args[9].strip_prefix("rgba:") {
        read_rgba(Path::new(path), w, h)?
    } else { (0..w*h).map(|i| {
        let (x,y) = (i % w, i / w);
        let v = match args[9].as_str() {
            "black" => 0.,
            "checker" => ((x + y) % 2) as f32,
            "ramp" => x as f32 / (w-1).max(1) as f32,
            _ => 0.,
        };
        [v, v, v, 1.]
    }).collect() };
    let input = render::encode_samples(&samples, depth);
    let lut_samples: Vec<[f32; 4]> = (0..256).map(|i| { let v = i as f32/255.; [v,v,v,1.] }).collect();
    let lut = render::encode_samples(&lut_samples, depth);
    let external_names = plan::external_order(&env.passes);
    let manifest: serde_json::Value = if let Some(path) = args.get(10) {
        serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?
    } else { serde_json::json!({}) };
    let external_data: Vec<_> = external_names.iter().map(|name| {
        let Some(spec) = manifest.get(name) else { return Ok(None); };
        let width = spec["width"].as_u64().ok_or("external width missing")? as usize;
        let height = spec["height"].as_u64().ok_or("external height missing")? as usize;
        let path = spec["path"].as_str().ok_or("external path missing")?;
        let samples = read_rgba(Path::new(path), width, height)?;
        Ok(Some((render::encode_samples(&samples, depth), width, height)))
    }).collect::<Result<Vec<_>, String>>()?;
    let externals: Vec<_> = external_names.iter().zip(&external_data).map(|(name, data)| {
        if let Some((pixels, width, height)) = data {
            Some(render::ExternalTexture { pixels, stride: width*depth.bpp(), width: *width, height: *height, float32: false })
        } else if name == "quality_lut" {
            Some(render::ExternalTexture { pixels: &lut, stride: 256*depth.bpp(), width: 256, height: 1, float32: false })
        } else { None }
    }).collect();
    let execution = plan::build_plan(&env.passes, true);
    if env.uses_prev { return Err("quality runner only accepts stateless fixtures".into()); }
    let mut cache = None;
    render::ensure_frame_cache(gpu, &mut cache, 1, depth, w, h, execution.physical_count, false);
    let mut output = vec![0; w*h*depth.bpp()];
    render::execute_plan(gpu, &set, &execution, &values, &input, w*depth.bpp(), w, h,
        time, time*30., logical, &mut output, w*depth.bpp(), None, (0,0,w,h), &externals, cache.as_mut().unwrap())?;
    let measured_frames: usize = std::env::var("DFX_QUALITY_BENCH_FRAMES").ok()
        .map(|value| value.parse().map_err(|_| "invalid benchmark frame count"))
        .transpose()?.unwrap_or(0);
    if measured_frames > 120 { return Err("benchmark frame count exceeds 120".into()); }
    let mut steady_frame_ms = Vec::with_capacity(measured_frames);
    for _ in 0..measured_frames {
        let start = std::time::Instant::now();
        render::execute_plan(gpu, &set, &execution, &values, &input, w*depth.bpp(), w, h,
            time, time*30., logical, &mut output, w*depth.bpp(), None, (0,0,w,h), &externals, cache.as_mut().unwrap())?;
        steady_frame_ms.push(start.elapsed().as_secs_f64() * 1000.0);
    }
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
    println!("{}", serde_json::json!({"source":args[1],"language_id":language.0,"source_blake3":blake3::hash(source.as_bytes()).to_hex().as_str(),"renderer_blake3":blake3::hash(RENDER_SOURCE.as_bytes()).to_hex().as_str(),"output":args[2],"width":w,"height":h,"logical":logical,"time":time,"working_depth":args[8],"passes":env.passes.len(),"adapter":gpu.adapter_summary,"finite":true,"steady_frame_ms":steady_frame_ms}));
    Ok(())
}

fn read_rgba(path: &Path, width: usize, height: usize) -> Result<Vec<[f32; 4]>, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let expected = width.checked_mul(height).and_then(|n| n.checked_mul(16)).ok_or("RGBA dimensions overflow")?;
    if bytes.len() != expected { return Err(format!("RGBA file has {} bytes; expected {expected}", bytes.len())); }
    let samples: Vec<_> = bytes.chunks_exact(16).map(|pixel| {
        std::array::from_fn(|i| f32::from_le_bytes(pixel[i*4..i*4+4].try_into().unwrap()))
    }).collect();
    if samples.iter().flatten().any(|v| !v.is_finite()) { return Err("RGBA file contains non-finite samples".into()); }
    Ok(samples)
}

fn main() {
    if let Err(e) = render_file(&std::env::args().collect::<Vec<_>>()) { eprintln!("{e}"); std::process::exit(1); }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a DX12 adapter and the Windows FXC compiler"]
    fn backend_compile_error_returns_and_next_pipeline_succeeds() {
        let gpu = render::gpu().expect("DX12 GPU");
        let prefix = "#version 450\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_in;\nlayout(set=0,binding=1) uniform sampler u_s;\nlayout(set=0,binding=2) uniform FxUniforms {vec2 u_resolution;float u_time;float u_frame;};\n";
        let invalid = format!("{prefix}void main() {{ vec4 values[5000]; for(int i=0;i<5000;i++) {{ values[i]=vec4(sin(u_time+float(i))); }} int i=int(clamp(v_uv.x,0.0,1.0)*4999.0); outColor=values[i]+values[4999-i]; }}");
        let compile = |source: &str| {
            let annotations = frontend::annotation::parse_annotations(source).unwrap();
            let compiler = frontend::frontend_for(LanguageId::GLSL).unwrap();
            let module = compiler.parse_module(source, &annotations, 1).unwrap();
            let spv = render::compile_spirv(&module.module).unwrap();
            render::build_pipeline(gpu, &spv, &module.layout, &module.extra_input_bindings, render::Depth::U8)
        };
        let instance = std::sync::Mutex::new(());
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = instance.lock().unwrap();
            compile(&invalid)
        }));
        eprintln!("host unwind={}, instance lock poisoned={}", failed.is_err(), instance.is_poisoned());
        let result = failed.expect("a backend rejection must not unwind into the host");
        assert!(instance.lock().is_ok(), "later render and flatten calls must still acquire the instance");
        let error = result.err().expect("FXC must reject the oversized temporary array");
        eprintln!("captured compiler diagnostic: {error}");
        assert!(error.contains("FXC") && error.contains("Internal error"), "{error}");
        assert!(error.starts_with("E58 "), "backend rejection keeps a stable diagnostic: {error}");
        let valid = format!("{prefix}void main() {{ outColor=vec4(v_uv,0.0,1.0); }}");
        assert!(compile(&valid).is_ok(), "the error scopes must be balanced after failure");
    }
}
