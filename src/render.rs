//! Headless GPU execution of one fragment pass over the input frame, against
//! Shader ABI v1 (ADR-0011): builtin head `u_resolution`/`u_time`/`u_frame`
//! at offsets 0/8/12, user members after it per the frontend-reflected
//! layout.
//!
//! Backend policy (ADR-0014 §3): DirectX 12 is the only supported backend on
//! Windows. `DYNAMICFX_BACKEND` exists as a diagnostic-only override and
//! never carries a support claim. The adapter identity is logged for host
//! evidence.

use crate::frontend::UniformBlockLayout;
use crate::plan::{ExecutionPlan, TexSlot};
use std::{borrow::Cow, num::NonZeroU64, sync::OnceLock};

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    /// Adapter/backend/driver line included in host evidence (ADR-0014 §6).
    pub adapter_summary: String,
    /// Optional device features actually granted; a depth whose working
    /// format needs a missing feature fails closed to pass-through with a
    /// diagnostic log (ADR-0021 §Costs).
    pub features: wgpu::Features,
}

/// Comp depth → working format, ADR-0021 §1. The whole pipeline (I/O, pass
/// targets, intermediates) uses one format per render; there is no
/// per-texture choice.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Depth {
    U8,
    U15,
    F32,
}

impl Depth {
    /// ADR-0022: 16-bpc rides Rgba32Float, not ADR-0021's Rgba16Unorm —
    /// wgpu refuses 16-bit norm formats as render attachments (measured
    /// live on the DX12 host: "Format Rgba16Unorm is not renderable").
    /// f32 holds every U15 integer exactly, so the boundary stays lossless.
    pub fn wgpu_format(self) -> wgpu::TextureFormat {
        match self {
            Depth::U8 => wgpu::TextureFormat::Rgba8Unorm,
            Depth::U15 | Depth::F32 => wgpu::TextureFormat::Rgba32Float,
        }
    }

    /// Bytes per RGBA pixel in the tightly packed working buffer (the
    /// working texel size — not the AE world's own pixel size).
    pub fn bpp(self) -> usize {
        match self {
            Depth::U8 => 4,
            Depth::U15 | Depth::F32 => 16,
        }
    }

    /// Device features the working format needs beyond core.
    fn required_features(self) -> wgpu::Features {
        match self {
            Depth::U8 => wgpu::Features::empty(),
            // Sampled through the shared filtering sampler, so filterability
            // is required, not just the format.
            Depth::U15 | Depth::F32 => wgpu::Features::FLOAT32_FILTERABLE,
        }
    }

    pub fn supported_by(self, gpu: &Gpu) -> bool {
        gpu.features.contains(self.required_features())
    }
}

fn requested_backends() -> wgpu::Backends {
    match std::env::var("DYNAMICFX_BACKEND").as_deref() {
        Ok("vulkan") => wgpu::Backends::VULKAN,
        Ok("gl") => wgpu::Backends::GL,
        Ok("all") => wgpu::Backends::all(),
        _ => wgpu::Backends::DX12,
    }
}

/// Lazily create one headless device per process. `None` is the documented
/// Unavailable state (diagnostic + pass-through, never a crash).
pub fn gpu() -> Option<&'static Gpu> {
    static GPU: OnceLock<Option<Gpu>> = OnceLock::new();
    GPU.get_or_init(|| {
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = requested_backends();
        let instance = wgpu::Instance::new(desc);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok()?;
        let info = adapter.get_info();
        let adapter_summary = format!(
            "adapter=\"{}\" backend={:?} driver=\"{} {}\"",
            info.name, info.backend, info.driver, info.driver_info
        );
        crate::diag::log(&format!("gpu ready: {adapter_summary}"));
        // Deep working formats (ADR-0021/0022) ride optional features;
        // request exactly what the adapter offers so 8-bpc still works on
        // adapters without them.
        let wanted = wgpu::Features::FLOAT32_FILTERABLE;
        let features = adapter.features() & wanted;
        if features != wanted {
            crate::diag::log(&format!(
                "gpu: adapter lacks {:?}; affected depths fail closed to pass-through",
                wanted - features
            ));
        }
        let desc = wgpu::DeviceDescriptor { required_features: features, ..Default::default() };
        let (device, queue) = pollster::block_on(adapter.request_device(&desc)).ok()?;
        Some(Gpu { device, queue, adapter_summary, features })
    })
    .as_ref()
}

/// Emit SPIR-V for a frontend-validated module (the per-pass artifact step;
/// module identity and artifact identity stay separate per ADR-0007).
pub fn compile_spirv(module: &naga::Module) -> Result<Vec<u32>, String> {
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(module)
    .map_err(|e| format!("{e:?}"))?;
    naga::back::spv::write_vec(module, &info, &naga::back::spv::Options::default(), None)
        .map_err(|e| format!("{e:?}"))
}

pub struct FxPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    layout: UniformBlockLayout,
    /// Extra input bindings the module declares (3+); binding 2+i reads
    /// manifest input i.
    extra_input_bindings: Vec<u32>,
}

/// All pipelines for one compiled effect, keyed by its session token and the
/// working depth (per-depth pipelines cache independently, ADR-0021 §6).
pub struct PipelineSet {
    pub token: u64,
    pub depth: Depth,
    pub passes: Vec<FxPipeline>,
}

/// Internal fullscreen-triangle vertex stage; the user only writes the
/// fragment stage. UV origin is top-left, v increasing downward (ADR-0011
/// §6, pinned by the M1 gradient fixture).
const VERT_WGSL: &str = r#"
struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VsOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0)
    );
    let p = pos[i];
    var o: VsOut;
    o.pos = vec4<f32>(p, 0.0, 1.0);
    o.uv = vec2<f32>(p.x * 0.5 + 0.5, 0.5 - p.y * 0.5);
    return o;
}
"#;

pub fn build_pipeline(
    gpu: &Gpu,
    spirv: &[u32],
    layout: &UniformBlockLayout,
    extra_input_bindings: &[u32],
    depth: Depth,
) -> Result<FxPipeline, String> {
    let device = &gpu.device;
    let block_size = layout.block_size.max(16);
    // wgpu validation failures panic through the uncaptured-error handler by
    // default; inside AE that means a modal error dialog that also poisons
    // scripted harness runs. Scope them into an Err -> diagnostic log +
    // pass-through instead (measured live with ADR-0021's Rgba16Unorm).
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);

    let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("dynamicfx-vs"),
        source: wgpu::ShaderSource::Wgsl(VERT_WGSL.into()),
    });
    let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("dynamicfx-fs"),
        source: wgpu::ShaderSource::SpirV(Cow::Borrowed(spirv)),
    });

    let texture_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let mut entries = vec![
        texture_entry(0),
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(block_size as u64),
            },
            count: None,
        },
    ];
    for &binding in extra_input_bindings {
        entries.push(texture_entry(binding));
    }
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("dynamicfx-bgl"),
        entries: &entries,
    });

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("dynamicfx-uniforms"),
        size: block_size as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("dynamicfx-pl"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("dynamicfx-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vs,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &fs,
            entry_point: Some("main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: depth.wgpu_format(),
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    if let Some(e) = pollster::block_on(error_scope.pop()) {
        return Err(format!("wgpu validation: {e}"));
    }

    Ok(FxPipeline {
        pipeline,
        bind_group_layout,
        uniform_buffer,
        layout: layout.clone(),
        extra_input_bindings: extra_input_bindings.to_vec(),
    })
}

fn align256(n: usize) -> usize {
    (n + 255) & !255
}

/// Execute one complete plan on one RGBA frame in the working encoding of
/// `set.depth`: upload the input, allocate the plan's physical intermediates
/// (working format per ADR-0019/ADR-0021), run every step in order, and read
/// the final output back.
/// `per_pass_values[p][i]` carries up to 4 words for pipeline `p`'s layout
/// entry `i` (scalars use [0]; vec4-color alpha rides in [3]).
/// Per-instance GPU frame resources reused across renders (M7 optimization
/// item 1: resource churn measured as the dominant span at baseline).
/// Thread safety is by construction: the render path holds the instance
/// lock for the whole execute, so one cache is never touched by two renders
/// at once (MFR concurrency is across sequence clones, each with its own
/// instance state — measured at M6).
pub struct FrameCache {
    key: (u64, Depth, usize, usize, usize),
    input_tex: wgpu::Texture,
    physical: Vec<wgpu::Texture>,
    out_tex: wgpu::Texture,
    readback: wgpu::Buffer,
    sampler: wgpu::Sampler,
    /// Never written: fresh textures read as zeros, so this is the History
    /// basis of iteration 0 — cached ping/pong never needs clearing and
    /// every frame still starts from black (ADR-0025 §2).
    zero_tex: wgpu::Texture,
    /// Ping/pong pair for windowed re-simulation; allocated only for
    /// temporal definitions, reused across frames.
    temporal: Option<(wgpu::Texture, wgpu::Texture)>,
    /// Reused staging for the 256-alignment repack fallback.
    staging: Vec<u8>,
}

/// GPU bytes a frame cache would pin for this shape: input + output + zero
/// textures, the plan's physical intermediates, the optional temporal pair,
/// and the 256-aligned readback buffer (M7 budget enforcement).
pub fn frame_cache_bytes(
    depth: Depth,
    width: usize,
    height: usize,
    physical_count: usize,
    temporal: bool,
) -> usize {
    let texel = width * height * depth.bpp();
    let textures = 3 + physical_count + if temporal { 2 } else { 0 };
    textures * texel + align256(width * depth.bpp()) * height
}

/// Budget gate: instances whose resource set would exceed the cap render
/// with transient resources instead (allocate + drop per render — the
/// pre-cache behavior), logged upstream. Cap: DYNAMICFX_CACHE_CAP_MB,
/// default 2048.
pub fn cache_within_budget(
    depth: Depth,
    width: usize,
    height: usize,
    physical_count: usize,
    temporal: bool,
) -> bool {
    static CAP_MB: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    let cap = *CAP_MB.get_or_init(|| {
        std::env::var("DYNAMICFX_CACHE_CAP_MB")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(2048)
    });
    frame_cache_bytes(depth, width, height, physical_count, temporal) <= cap * 1024 * 1024
}

#[cfg(test)]
mod budget_tests {
    use super::*;

    #[test]
    fn cache_bytes_shape_math() {
        // 4K float, 3 physical, temporal: (3+3+2) textures * w*h*16 + readback.
        let w = 3840;
        let h = 2160;
        let tex = w * h * 16;
        assert_eq!(
            frame_cache_bytes(Depth::F32, w, h, 3, true),
            8 * tex + align256(w * 16) * h
        );
        // U8 single-pass non-temporal: 3 textures + readback.
        let tex8 = w * h * 4;
        assert_eq!(
            frame_cache_bytes(Depth::U8, w, h, 0, false),
            3 * tex8 + align256(w * 4) * h
        );
    }

    #[test]
    fn budget_thresholds() {
        // Default cap 2048 MB: 4K float 6-pass temporal (~1.3 GiB) fits;
        // 8K float with the same shape (~5.3 GiB) must be refused.
        assert!(cache_within_budget(Depth::F32, 3840, 2160, 3, true));
        assert!(!cache_within_budget(Depth::F32, 7680, 4320, 3, true));
        // The benchmark shapes all fit.
        assert!(cache_within_budget(Depth::U8, 3840, 2160, 3, false));
        assert!(cache_within_budget(Depth::U8, 1280, 720, 0, true));
    }
}

/// Ensure `cache` matches (token, depth, size, plan shape); rebuild on any
/// mismatch (dropping the old resources), add the temporal pair on demand.
pub fn ensure_frame_cache(
    gpu: &Gpu,
    cache: &mut Option<FrameCache>,
    token: u64,
    depth: Depth,
    width: usize,
    height: usize,
    physical_count: usize,
    temporal: bool,
) {
    let key = (token, depth, width, height, physical_count);
    if cache.as_ref().is_none_or(|c| c.key != key) {
        let device = &gpu.device;
        let base = wgpu::TextureDescriptor {
            label: Some("dynamicfx-cached"),
            size: wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: depth.wgpu_format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };
        let intermediate = wgpu::TextureDescriptor {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ..base
        };
        let padded = align256(width * depth.bpp());
        *cache = Some(FrameCache {
            key,
            input_tex: device.create_texture(&base),
            physical: (0..physical_count)
                .map(|_| device.create_texture(&intermediate))
                .collect(),
            out_tex: device.create_texture(&wgpu::TextureDescriptor {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                ..base
            }),
            readback: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("dynamicfx-readback"),
                size: (padded * height) as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            sampler: device.create_sampler(&wgpu::SamplerDescriptor::default()),
            zero_tex: device.create_texture(&wgpu::TextureDescriptor {
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                ..base
            }),
            temporal: None,
            staging: Vec::new(),
        });
    }
    let entry = cache.as_mut().expect("just ensured");
    if temporal && entry.temporal.is_none() {
        entry.temporal = Some((
            create_history_texture(gpu, depth, width, height),
            create_history_texture(gpu, depth, width, height),
        ));
    }
}

/// Wall-clock spans of one execute (M7 measurement plan). Collection is a
/// handful of `Instant` reads — always on; logging is env-gated upstream.
#[derive(Default, Clone, Copy)]
pub struct PerfBreakdown {
    pub upload_ms: f32,
    pub gpu_ms: f32,
    pub readback_ms: f32,
}

/// Allocate one history texture: working format, zero-initialized (wgpu
/// guarantees new textures read as zeros — the ADR-0023 initial state).
pub fn create_history_texture(gpu: &Gpu, depth: Depth, width: usize, height: usize) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("dynamicfx-history"),
        size: wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: depth.wgpu_format(),
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

#[allow(clippy::too_many_arguments)]
pub fn execute_plan(
    gpu: &Gpu,
    set: &PipelineSet,
    plan: &ExecutionPlan,
    per_pass_values: &[Vec<[f32; 4]>],
    input: &[u8],
    in_stride: usize,
    width: usize,
    height: usize,
    time: f32,
    frame: f32,
    // ADR-0029: the u_resolution the SHADER sees — the logical full-
    // resolution frame size, invariant under AE preview downsampling.
    // Geometry (buffers, rects, taps) stays physical.
    logical_res: (f32, f32),
    output: &mut [u8],
    out_stride: usize,
    window: Option<(u32, f32)>,
    // Requested output window in input-world coords (x, y, w, h) — ROI
    // (M7): the FINAL pass is scissored to it and only it is read back.
    // Safe for arbitrary shaders by construction: fragments are pure
    // per-pixel functions, uv mapping and every sampled texture (input,
    // intermediates, history) stay full-frame, and pixels outside the
    // rect are never returned to AE. Intermediate passes and intermediate
    // window iterations always render full-frame (downstream passes and
    // the next iteration may sample anywhere).
    out_rect: (usize, usize, usize, usize),
    cache: &mut FrameCache,
) -> Result<PerfBreakdown, String> {
    let mut perf = PerfBreakdown::default();
    let t_start = std::time::Instant::now();
    let device = &gpu.device;
    let queue = &gpu.queue;
    let (rx0, ry0, rw_out, rh_out) = out_rect;
    if rw_out == 0 || rh_out == 0 || rx0 + rw_out > width || ry0 + rh_out > height {
        return Err(format!(
            "out_rect ({rx0},{ry0},{rw_out},{rh_out}) outside {width}x{height}"
        ));
    }
    let row_bytes = width * set.depth.bpp();
    let padded = align256(row_bytes);
    let rect_row_bytes = rw_out * set.depth.bpp();
    let rect_padded = align256(rect_row_bytes);
    let extent = wgpu::Extent3d {
        width: width as u32,
        height: height as u32,
        depth_or_array_layers: 1,
    };
    let n = window.map(|(w, _)| w.max(1)).unwrap_or(1);
    let dt = window.map(|(_, d)| d).unwrap_or(0.0);

    // Upload the input ONCE per render — window iterations re-read the same
    // current-frame input (ADR-0025 §4 v1). Rows already 256-aligned go up
    // directly; odd strides repack into the cache's reusable staging.
    let copy_dst = wgpu::TexelCopyTextureInfo {
        texture: &cache.input_tex,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
    };
    if in_stride % (wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize) == 0 {
        queue.write_texture(
            copy_dst,
            input,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(in_stride as u32),
                rows_per_image: Some(height as u32),
            },
            extent,
        );
    } else {
        cache.staging.resize(padded * height, 0);
        for y in 0..height {
            let src = &input[y * in_stride..y * in_stride + row_bytes];
            cache.staging[y * padded..y * padded + row_bytes].copy_from_slice(src);
        }
        queue.write_texture(
            copy_dst,
            &cache.staging,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded as u32),
                rows_per_image: Some(height as u32),
            },
            extent,
        );
    }

    let input_view = cache.input_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let physical_views: Vec<wgpu::TextureView> = cache
        .physical
        .iter()
        .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()))
        .collect();
    let out_view = cache.out_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let zero_view = cache.zero_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let temporal_pair = match (&cache.temporal, window.is_some()) {
        (Some((a, b)), true) => Some((
            a.create_view(&wgpu::TextureViewDescriptor::default()),
            b.create_view(&wgpu::TextureViewDescriptor::default()),
        )),
        (None, true) => return Err("temporal render without cached history pair".to_string()),
        _ => None,
    };
    perf.upload_ms = t_start.elapsed().as_secs_f32() * 1000.0;

    for it in 0..n {
        let t_prep = std::time::Instant::now();
        let back = (n - 1 - it) as f32;
        let iter_time = time - back * dt;
        let iter_frame = frame - back;
        // Ping/pong across iterations; iteration 0 samples the never-written
        // zero texture so a reused pair still starts every frame from black.
        let (basis_view, target_view): (&wgpu::TextureView, &wgpu::TextureView) =
            match &temporal_pair {
                Some((ping, pong)) => {
                    let (basis, target) = if it % 2 == 0 { (ping, pong) } else { (pong, ping) };
                    (if it == 0 { &zero_view } else { basis }, target)
                }
                None => (&zero_view, &out_view),
            };
        let view_of = |slot: TexSlot| -> Result<&wgpu::TextureView, String> {
            match slot {
                TexSlot::EffectInput => Ok(&input_view),
                TexSlot::FinalOutput => Ok(target_view),
                TexSlot::History => {
                    if temporal_pair.is_some() {
                        Ok(basis_view)
                    } else {
                        Err("history slot without temporal textures".to_string())
                    }
                }
                TexSlot::Physical(i) => {
                    physical_views.get(i).ok_or_else(|| format!("physical slot {i} out of range"))
                }
            }
        };

        // One encoder per iteration; wgpu inserts barriers between passes,
        // and queue order makes iteration i's writes visible to i+1.
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        for step in &plan.steps {
            let fx = set
                .passes
                .get(step.pass_index)
                .ok_or_else(|| format!("pipeline {} missing", step.pass_index))?;
            let values = per_pass_values
                .get(step.pass_index)
                .ok_or_else(|| format!("values {} missing", step.pass_index))?;

            // ABI v1 builtin head at fixed offsets 0/8/12, then user members
            // at their reflected std140 offsets.
            let mut ubuf = vec![0u8; fx.layout.block_size.max(16)];
            ubuf[0..4].copy_from_slice(&logical_res.0.to_le_bytes());
            ubuf[4..8].copy_from_slice(&logical_res.1.to_le_bytes());
            ubuf[8..12].copy_from_slice(&iter_time.to_le_bytes());
            ubuf[12..16].copy_from_slice(&iter_frame.to_le_bytes());
            for (i, entry) in fx.layout.entries.iter().enumerate() {
                let (Some(vals), Some(dst)) = (
                    values.get(i),
                    ubuf.get_mut(entry.offset..entry.offset + 4 * entry.words),
                ) else {
                    continue;
                };
                for w in 0..entry.words {
                    let bytes: [u8; 4] = if entry.int {
                        (vals[w] as i32).to_le_bytes()
                    } else {
                        vals[w].to_le_bytes()
                    };
                    dst[w * 4..w * 4 + 4].copy_from_slice(&bytes);
                }
            }
            queue.write_buffer(&fx.uniform_buffer, 0, &ubuf);

            let primary = view_of(*step.inputs.first().ok_or("step has no inputs")?)?;
            let mut entries = vec![
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(primary),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&cache.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: fx.uniform_buffer.as_entire_binding(),
                },
            ];
            for &binding in &fx.extra_input_bindings {
                // Binding 2+i carries manifest input i (ADR-0018 §5).
                let input_index = (binding - 2) as usize;
                let slot = *step
                    .inputs
                    .get(input_index)
                    .ok_or_else(|| format!("binding {binding} has no step input"))?;
                entries.push(wgpu::BindGroupEntry {
                    binding,
                    resource: wgpu::BindingResource::TextureView(view_of(slot)?),
                });
            }
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("dynamicfx-bg"),
                entries: &entries,
                layout: &fx.bind_group_layout,
            });

            let target = view_of(step.output)?;
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("dynamicfx-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&fx.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            // ROI: only the delivered final image is narrowed. Fragment
            // uv/inputs are untouched, so covered pixels are bit-identical
            // to a full render (equivalence gated by DYNAMICFX_NO_ROI A/B).
            if matches!(step.output, TexSlot::FinalOutput) && it + 1 == n {
                pass.set_scissor_rect(rx0 as u32, ry0 as u32, rw_out as u32, rh_out as u32);
            }
            pass.draw(0..3, 0..1);
            drop(pass);
        }

        // The final iteration carries the readback copy in the same submit.
        if it + 1 == n {
            let src_tex: &wgpu::Texture = match &cache.temporal {
                Some((a, b)) if window.is_some() => {
                    if it % 2 == 0 {
                        b
                    } else {
                        a
                    }
                }
                _ => &cache.out_tex,
            };
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: src_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: rx0 as u32,
                        y: ry0 as u32,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &cache.readback,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(rect_padded as u32),
                        rows_per_image: Some(rh_out as u32),
                    },
                },
                wgpu::Extent3d {
                    width: rw_out as u32,
                    height: rh_out as u32,
                    depth_or_array_layers: 1,
                },
            );
        }
        perf.upload_ms += t_prep.elapsed().as_secs_f32() * 1000.0;
        let t_submit = std::time::Instant::now();
        queue.submit([encoder.finish()]);
        perf.gpu_ms += t_submit.elapsed().as_secs_f32() * 1000.0;
    }

    let t_gpu = std::time::Instant::now();
    let slice = cache.readback.slice(0..(rect_padded * rh_out) as u64);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = tx.send(res);
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .map_err(|e| format!("device poll failed: {e:?}"))?;
    rx.recv()
        .map_err(|e| format!("map channel failed: {e:?}"))?
        .map_err(|e| format!("buffer map failed: {e:?}"))?;
    perf.gpu_ms += t_gpu.elapsed().as_secs_f32() * 1000.0;

    let t_read = std::time::Instant::now();
    {
        let data = slice.get_mapped_range();
        for y in 0..rh_out {
            output[y * out_stride..y * out_stride + rect_row_bytes]
                .copy_from_slice(&data[y * rect_padded..y * rect_padded + rect_row_bytes]);
        }
    }
    cache.readback.unmap();
    perf.readback_ms = t_read.elapsed().as_secs_f32() * 1000.0;
    Ok(perf)
}


/// ADR-0029: logical (full-resolution) size of a downsampled buffer.
/// AE hands effects buffers scaled by downsample_x/y (num/den ratios); the
/// shader-visible u_resolution reports the full-resolution frame so pixel-
/// based shader math is invariant across preview resolutions.
pub fn logical_size(physical: usize, num: i32, den: u32) -> f32 {
    if num <= 0 || den == 0 {
        return physical as f32;
    }
    physical as f32 * den as f32 / num as f32
}

#[cfg(test)]
mod logical_size_tests {
    use super::logical_size;

    #[test]
    fn downsample_ratios() {
        // Full res: ratio 1/1.
        assert_eq!(logical_size(1280, 1, 1), 1280.0);
        // Half res: AE hands a 640 buffer with ratio 1/2 -> logical 1280.
        assert_eq!(logical_size(640, 1, 2), 1280.0);
        // Third res.
        assert_eq!(logical_size(427, 1, 3), 1281.0);
        // Degenerate ratios fall back to physical.
        assert_eq!(logical_size(640, 0, 2), 640.0);
        assert_eq!(logical_size(640, 1, 0), 640.0);
    }
}

/// U15 (0..=32768, white = 32768) -> working f32. Exact: every U15 integer
/// fits f32's 24-bit mantissa and /2^15 only shifts the exponent
/// (ADR-0022; exhaustive unit sweep below).
#[inline]
pub fn u15_to_f32(v: u16) -> f32 {
    v as f32 / 32768.0
}

/// Working f32 -> U15 with the AE-range clamp (16-bpc holds no over-white
/// or negatives; NaN clamps to 0 via f32::max's NaN handling).
#[inline]
pub fn f32_to_u15(v: f32) -> u16 {
    (v.max(0.0).min(1.0) * 32768.0).round() as u16
}

/// Convert AE 16bpc (U15) ARGB pixels to tightly packed RGBA f32 for the
/// Rgba32Float working texture (ADR-0022). `out` must be width*height*16
/// bytes.
pub fn argb_u15_to_rgba_f32(input: &[u8], in_stride: usize, width: usize, height: usize, out: &mut [u8]) {
    for y in 0..height {
        let row_in = &input[y * in_stride..y * in_stride + width * 8];
        let row_out = &mut out[y * width * 16..(y + 1) * width * 16];
        for x in 0..width {
            let i = x * 8;
            let ch = |o: usize| u16::from_le_bytes([row_in[i + o], row_in[i + o + 1]]);
            let o = x * 16;
            row_out[o..o + 4].copy_from_slice(&u15_to_f32(ch(2)).to_le_bytes()); // R
            row_out[o + 4..o + 8].copy_from_slice(&u15_to_f32(ch(4)).to_le_bytes()); // G
            row_out[o + 8..o + 12].copy_from_slice(&u15_to_f32(ch(6)).to_le_bytes()); // B
            row_out[o + 12..o + 16].copy_from_slice(&u15_to_f32(ch(0)).to_le_bytes()); // A
        }
    }
}

/// Convert tightly packed RGBA f32 back to AE 16bpc (U15) ARGB pixels.
pub fn rgba_f32_to_argb_u15(input: &[u8], in_stride: usize, width: usize, height: usize, out: &mut [u8], out_stride: usize) {
    for y in 0..height {
        let row_in = &input[y * in_stride..y * in_stride + width * 16];
        let row_out = &mut out[y * out_stride..y * out_stride + width * 8];
        for x in 0..width {
            let i = x * 16;
            let ch = |o: usize| {
                f32::from_le_bytes([row_in[i + o], row_in[i + o + 1], row_in[i + o + 2], row_in[i + o + 3]])
            };
            let w = |v: f32| f32_to_u15(v).to_le_bytes();
            let o = x * 8;
            row_out[o..o + 2].copy_from_slice(&w(ch(12))); // A
            row_out[o + 2..o + 4].copy_from_slice(&w(ch(0))); // R
            row_out[o + 4..o + 6].copy_from_slice(&w(ch(4))); // G
            row_out[o + 6..o + 8].copy_from_slice(&w(ch(8))); // B
        }
    }
}

/// Convert AE 32bpc float ARGB pixels to tightly packed RGBA f32. Pure
/// 32-bit lane reorder — over-white, negatives, NaN payloads, and infinities
/// pass through bit-exact (ADR-0021 §2). `out` must be width*height*16 bytes.
pub fn argb_f32_to_rgba_f32(input: &[u8], in_stride: usize, width: usize, height: usize, out: &mut [u8]) {
    let row_bytes = width * 16;
    for y in 0..height {
        let row_in = &input[y * in_stride..y * in_stride + row_bytes];
        let row_out = &mut out[y * row_bytes..(y + 1) * row_bytes];
        for x in 0..width {
            let i = x * 16;
            row_out[i..i + 12].copy_from_slice(&row_in[i + 4..i + 16]); // RGB
            row_out[i + 12..i + 16].copy_from_slice(&row_in[i..i + 4]); // A
        }
    }
}

/// Convert tightly packed RGBA f32 back to AE 32bpc float ARGB pixels.
pub fn rgba_f32_to_argb_f32(input: &[u8], in_stride: usize, width: usize, height: usize, out: &mut [u8], out_stride: usize) {
    let row_bytes = width * 16;
    for y in 0..height {
        let row_in = &input[y * in_stride..y * in_stride + row_bytes];
        let row_out = &mut out[y * out_stride..y * out_stride + row_bytes];
        for x in 0..width {
            let i = x * 16;
            row_out[i..i + 4].copy_from_slice(&row_in[i + 12..i + 16]); // A
            row_out[i + 4..i + 16].copy_from_slice(&row_in[i..i + 12]); // RGB
        }
    }
}

#[cfg(test)]
mod depth_tests {
    use super::*;

    // ADR-0022 goldens: exact powers of two and the endpoints.
    #[test]
    fn u15_f32_goldens() {
        assert_eq!(u15_to_f32(0), 0.0);
        assert_eq!(u15_to_f32(16384), 0.5);
        assert_eq!(u15_to_f32(32768), 1.0);
        assert_eq!(f32_to_u15(1.0), 32768);
        // AE-range clamp: 16-bpc has no over-white/negatives; NaN -> 0.
        assert_eq!(f32_to_u15(2.0), 32768);
        assert_eq!(f32_to_u15(-0.5), 0);
        assert_eq!(f32_to_u15(f32::NAN), 0);
    }

    // ADR-0022: the mapping must be lossless over the whole U15 domain.
    #[test]
    fn u15_f32_exhaustive_round_trip() {
        let mut prev = None;
        for v in 0..=32768u16 {
            let f = u15_to_f32(v);
            assert_eq!(f32_to_u15(f), v, "round trip failed at {v}");
            if let Some(p) = prev {
                assert!(f > p, "not strictly increasing at {v}");
            }
            prev = Some(f);
        }
    }

    #[test]
    fn u15_pixel_round_trip_with_strides() {
        // One 2x2 ARGB U15 frame with a padded stride.
        let px: [[u16; 4]; 4] = [
            [32768, 0, 1, 16384],
            [0, 32768, 32767, 2],
            [12345, 23456, 3, 30000],
            [1, 32768, 0, 9999],
        ];
        let stride = 2 * 8 + 16;
        let mut argb = vec![0u8; stride * 2];
        for (n, p) in px.iter().enumerate() {
            let (x, y) = (n % 2, n / 2);
            for (c, v) in p.iter().enumerate() {
                argb[y * stride + x * 8 + c * 2..y * stride + x * 8 + c * 2 + 2]
                    .copy_from_slice(&v.to_le_bytes());
            }
        }
        let mut rgba = vec![0u8; 2 * 2 * 16];
        argb_u15_to_rgba_f32(&argb, stride, 2, 2, &mut rgba);
        let mut back = vec![0u8; stride * 2];
        rgba_f32_to_argb_u15(&rgba, 2 * 16, 2, 2, &mut back, stride);
        for (n, p) in px.iter().enumerate() {
            let (x, y) = (n % 2, n / 2);
            for (c, v) in p.iter().enumerate() {
                let o = y * stride + x * 8 + c * 2;
                assert_eq!(u16::from_le_bytes([back[o], back[o + 1]]), *v, "px {n} ch {c}");
            }
        }
    }

    // ADR-0021 §2: F32 is a bit-exact lane reorder — over-white, negatives,
    // NaN payloads, infinities all survive.
    #[test]
    fn f32_reorder_bit_exact() {
        let vals: [u32; 8] = [
            2.0f32.to_bits(),
            (-0.5f32).to_bits(),
            f32::NEG_INFINITY.to_bits(),
            0x7FC0_0001, // NaN with payload
            0x0000_0001, // subnormal
            1.0f32.to_bits(),
            f32::INFINITY.to_bits(),
            0x8000_0000, // -0.0
        ];
        let mut argb = vec![0u8; 2 * 16];
        for (i, v) in vals.iter().enumerate() {
            argb[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        let mut rgba = vec![0u8; 2 * 16];
        argb_f32_to_rgba_f32(&argb, 2 * 16, 2, 1, &mut rgba);
        // ARGB word i maps to RGBA: A->3, R->0, G->1, B->2 per pixel.
        for px in 0..2 {
            let src = |c: usize| &argb[px * 16 + c * 4..px * 16 + c * 4 + 4];
            let dst = |c: usize| &rgba[px * 16 + c * 4..px * 16 + c * 4 + 4];
            assert_eq!(dst(0), src(1));
            assert_eq!(dst(1), src(2));
            assert_eq!(dst(2), src(3));
            assert_eq!(dst(3), src(0));
        }
        let mut back = vec![0u8; 2 * 16];
        rgba_f32_to_argb_f32(&rgba, 2 * 16, 2, 1, &mut back, 2 * 16);
        assert_eq!(argb, back, "round trip must be bit-exact");
    }

    // ADR-0021 §1 as amended by ADR-0022: depth -> working format and
    // buffer layout selection (16-bpc rides f32; Rgba16Unorm is not
    // renderable in wgpu).
    #[test]
    fn depth_format_selection() {
        assert_eq!(Depth::U8.wgpu_format(), wgpu::TextureFormat::Rgba8Unorm);
        assert_eq!(Depth::U15.wgpu_format(), wgpu::TextureFormat::Rgba32Float);
        assert_eq!(Depth::F32.wgpu_format(), wgpu::TextureFormat::Rgba32Float);
        assert_eq!(Depth::U8.bpp(), 4);
        assert_eq!(Depth::U15.bpp(), 16);
        assert_eq!(Depth::F32.bpp(), 16);
        assert_eq!(Depth::U8.required_features(), wgpu::Features::empty());
        assert!(Depth::U15.required_features().contains(wgpu::Features::FLOAT32_FILTERABLE));
        assert!(Depth::F32.required_features().contains(wgpu::Features::FLOAT32_FILTERABLE));
    }
}

/// Convert AE 8bpc ARGB pixels to tightly packed 8bpc RGBA.
pub fn argb8_to_rgba8(input: &[u8], in_stride: usize, width: usize, height: usize, out: &mut [u8]) {
    let row_bytes = width * 4;
    for y in 0..height {
        let row_in = &input[y * in_stride..y * in_stride + row_bytes];
        let row_out = &mut out[y * row_bytes..(y + 1) * row_bytes];
        for x in 0..width {
            let i = x * 4;
            row_out[i] = row_in[i + 1]; // R
            row_out[i + 1] = row_in[i + 2]; // G
            row_out[i + 2] = row_in[i + 3]; // B
            row_out[i + 3] = row_in[i]; // A
        }
    }
}

/// Convert tightly packed 8bpc RGBA back to AE 8bpc ARGB pixels.
pub fn rgba8_to_argb8(input: &[u8], in_stride: usize, width: usize, height: usize, out: &mut [u8], out_stride: usize) {
    let row_bytes = width * 4;
    for y in 0..height {
        let row_in = &input[y * in_stride..y * in_stride + row_bytes];
        let row_out = &mut out[y * out_stride..y * out_stride + row_bytes];
        for x in 0..width {
            let i = x * 4;
            row_out[i] = row_in[i + 3]; // A
            row_out[i + 1] = row_in[i]; // R
            row_out[i + 2] = row_in[i + 1]; // G
            row_out[i + 3] = row_in[i + 2]; // B
        }
    }
}
