use after_effects::{self as ae, AsPtr, AsMutPtr};
use ae::sys as sys;
use std::{ffi::CStr, fmt::Write, marker::PhantomData, ptr, rc::Rc, time::Instant};

type Result<T> = std::result::Result<T, String>;

fn check(code: i32, operation: &str) -> Result<()> {
    if code == 0 { Ok(()) } else { Err(format!("{operation}: AE error {code}")) }
}

macro_rules! call {
    ($suite:expr, $method:ident $(, $arg:expr)* $(,)?) => {{
        let function = $suite.$method.ok_or_else(|| format!("missing {}", stringify!($method)))?;
        check(unsafe { function($($arg),*) }, stringify!($method))
    }};
}

struct Suite<T> {
    basic: *mut sys::SPBasicSuite,
    name: &'static [u8],
    version: i32,
    table: *const T,
    _main_thread: PhantomData<Rc<()>>,
}

impl<T> Suite<T> {
    // Only callers pairing an SDK table with its exact named version may acquire it.
    unsafe fn acquire(basic: *mut sys::SPBasicSuite, name: &'static [u8], version: u32) -> Result<Self> {
        if basic.is_null() { return Err("null SPBasicSuite".into()); }
        let acquire = (*basic).AcquireSuite.ok_or("missing AcquireSuite")?;
        (*basic).ReleaseSuite.ok_or("missing ReleaseSuite")?;
        let mut table = ptr::null();
        check(acquire(name.as_ptr().cast(), version as i32, &mut table), "AcquireSuite")?;
        let result = Self { basic, name, version: version as i32, table: table.cast(), _main_thread: PhantomData };
        if result.table.is_null() { return Err("null suite table".into()); }
        Ok(result)
    }
}
impl<T> std::ops::Deref for Suite<T> {
    type Target = T;
    fn deref(&self) -> &T { unsafe { &*self.table } }
}
impl<T> Drop for Suite<T> {
    fn drop(&mut self) {
        let result = unsafe { ((*self.basic).ReleaseSuite.unwrap())(self.name.as_ptr().cast(), self.version) };
        if result != 0 { super::log(&format!("CLEANUP_FAILED ReleaseSuite code={result}")); }
    }
}

struct Cleanup<F: FnOnce() -> i32>(Option<F>);
impl<F: FnOnce() -> i32> Drop for Cleanup<F> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.0.take() {
            let code = cleanup();
            if code != 0 { super::log(&format!("CLEANUP_FAILED handle code={code}")); }
        }
    }
}
fn cleanup<F: FnOnce() -> i32>(f: F) -> Cleanup<F> { Cleanup(Some(f)) }

fn nullable_expression<T>(handle: *mut T, read: impl FnOnce(*mut T) -> Result<String>) -> Result<String> {
    if handle.is_null() { Ok(String::new()) } else { read(handle) }
}

pub unsafe fn expression_text(basic: *mut sys::SPBasicSuite, id: i32,
    stream_ref: sys::AEGP_StreamRefH) -> Result<String>
{
    let stream = Suite::<sys::AEGP_StreamSuite6>::acquire(basic, sys::kAEGPStreamSuite, sys::kAEGPStreamSuiteVersion6)?;
    let mut handle = ptr::null_mut();
    call!(stream, AEGP_GetExpression, id, stream_ref, &mut handle)?;
    // AE represents an absent expression with no allocation; never lock or free it.
    nullable_expression(handle, |handle| {
        let memory = Suite::<sys::AEGP_MemorySuite1>::acquire(basic, sys::kAEGPMemorySuite, sys::kAEGPMemorySuiteVersion1)?;
        let free = memory.AEGP_FreeMemHandle.ok_or("missing FreeMemHandle")?;
        let _free = cleanup(|| free(handle));
        let mut bytes = 0;
        call!(memory, AEGP_GetMemHandleSize, handle, &mut bytes)?;
        if bytes == 0 { return Ok(String::new()); }
        if bytes > 1_048_576 || bytes % 2 != 0 { return Err("invalid expression allocation size".into()); }
        let unlock = memory.AEGP_UnlockMemHandle.ok_or("missing UnlockMemHandle")?;
        let mut data = ptr::null_mut();
        call!(memory, AEGP_LockMemHandle, handle, &mut data)?;
        let _unlock = cleanup(|| unlock(handle));
        if data.is_null() { return Err("null expression data".into()); }
        let units = std::slice::from_raw_parts(data.cast::<u16>(), bytes as usize / 2);
        let end = units.iter().position(|unit| *unit == 0).ok_or("unterminated expression")?;
        String::from_utf16(&units[..end]).map_err(|_| "invalid UTF-16 expression".into())
    })
}

#[test]
fn absent_expression_never_enters_memory_operations() {
    assert_eq!(nullable_expression::<u8>(ptr::null_mut(), |_| panic!("no allocation to access")).unwrap(), "");
    let mut allocated = 0u8;
    assert_eq!(nullable_expression(&mut allocated, |_| Ok("present".into())).unwrap(), "present");
}

pub fn time(seconds: f64) -> Result<ae::Time> {
    let ticks = (seconds * 1_000_000.0).round();
    if !ticks.is_finite() || ticks < i32::MIN as f64 || ticks > i32::MAX as f64 {
        return Err("invalid or out-of-range layer time".into());
    }
    Ok(ae::Time { value: ticks as i32, scale: 1_000_000 })
}

pub fn paths(input: &ae::InData, ticks: i32, scale: u32) -> std::result::Result<String, ae::Error> {
    let query = ae::pf::suites::PathQuery::new()?;
    let count = query.num_paths(input.effect_ref())?;
    if !(0..=1024).contains(&count) { return Err(ae::Error::BadCallbackParameter); }
    let mut report = format!("pf_path_count={count}\n");
    let mut remaining = 8192;
    for index in 0..count {
        input.interact().abort()?;
        let id = query.path_info(input.effect_ref(), index)?;
        let Some(path) = query.checkout_path(input.effect_ref(), id, ticks, 0, scale)? else {
            let _ = writeln!(report, "path[{index}]=unavailable");
            continue;
        };
        let segments = path.num_segments()?;
        if segments < 0 || segments >= remaining { return Err(ae::Error::BadCallbackParameter); }
        remaining -= segments + 1;
        let _ = writeln!(report, "path[{index}] segments={segments} open={}", path.is_open()?);
        for vertex in 0..=segments {
            if vertex % 64 == 0 { input.interact().abort()?; }
            let _ = writeln!(report, "vertex[{vertex}]={:?}", path.vertex(vertex)?);
        }
    }
    Ok(report)
}

pub fn auxiliary(input: &ae::InData) -> std::result::Result<String, ae::Error> {
    let suite = ae::pf::suites::Channel::new()?;
    let count = suite.layer_channel_count(input.effect_ref(), 0)?;
    if !(0..=64).contains(&count) { return Err(ae::Error::BadCallbackParameter); }
    let mut report = format!("auxiliary_channel_count={count}\n");
    for index in 0..count {
        input.interact().abort()?;
        match suite.layer_channel_indexed_ref_and_desc(input.effect_ref(), 0, index)? {
            Some((_, mut desc)) => {
                desc.name[desc.name.len() - 1] = 0;
                let name = unsafe { CStr::from_ptr(desc.name.as_ptr()) }.to_string_lossy();
                let _ = writeln!(report, "channel[{index}] type={} name={name}", desc.channel_type);
            }
            None => { let _ = writeln!(report, "channel[{index}]=unavailable"); }
        }
    }
    let coverage = suite.layer_channel_typed_ref_and_desc(input.effect_ref(), 0, ae::pf::ChannelType::Coverage)?;
    let _ = writeln!(report, "auxiliary_coverage_found={}", coverage.is_some());
    Ok(report)
}

pub fn raster_masks(input: &ae::InData, source: &ae::Layer) -> Result<String> {
    let (width, height) = (source.width(), source.height());
    if width == 0 || height == 0 || width.checked_mul(height).is_none_or(|n| n > 32768) {
        return Err("native mask diagnostic requires a world of at most 32768 pixels".into());
    }
    let query = ae::pf::suites::PathQuery::new().map_err(|e| format!("{e:?}"))?;
    let masks = unsafe { Suite::<sys::PF_MaskSuite1>::acquire(input.pica_basic_suite_ptr(),
        sys::kPF_MaskSuite, sys::kPF_MaskSuiteVersion1)? };
    let count = query.num_paths(input.effect_ref()).map_err(|e| format!("{e:?}"))?;
    if !(0..=64).contains(&count) { return Err("mask count exceeds diagnostic budget".into()); }
    let depth = source.bit_depth();
    let bytes = match depth { 8 => 4, 16 => 8, 32 => 16, _ => return Err("unknown source depth".into()) };
    if source.row_bytes() <= 0 { return Err("unsupported source stride".into()); }
    let mut input_alpha = Vec::with_capacity(width * height);
    let buffer = source.buffer();
    for y in 0..height {
        input.interact().abort().map_err(|e| format!("{e:?}"))?;
        for x in 0..width {
            let at = pixel_offset(width as i32, height as i32, source.row_bytes() as usize,
                bytes, x as i32, y as i32)?;
            let data = buffer.get(at..at + bytes).ok_or("source pixel outside buffer")?;
            input_alpha.push(match depth {
                8 => u32::from(data[0]),
                16 => u32::from(u16::from_ne_bytes([data[0], data[1]])),
                _ => u32::from_ne_bytes([data[0], data[1], data[2], data[3]]),
            });
        }
    }
    let mut report = format!("mask_input width={width} height={height} depth={depth}\ninput_alpha{depth}={input_alpha:?}\n");
    for index in 0..count {
        input.interact().abort().map_err(|e| format!("{e:?}"))?;
        let id = query.path_info(input.effect_ref(), index).map_err(|e| format!("{e:?}"))?;
        let Some(path) = query.checkout_path(input.effect_ref(), id, input.current_time(), 0,
            input.time_scale()).map_err(|e| format!("{e:?}"))? else { continue };
        let mode = path.mask_mode().map_err(|e| format!("{e:?}"))?;
        if !matches!(mode, ae::pf::MaskMode::Add | ae::pf::MaskMode::Subtract) { continue; }
        let inverted = path.is_inverted().map_err(|e| format!("{e:?}"))?
            ^ (mode == ae::pf::MaskMode::Subtract);
        let world_suite = ae::pf::suites::World::new().map_err(|e| format!("{e:?}"))?;
        let mut world = world_suite.new_world(input, width as i32, height as i32, true,
            ae::pf::PixelFormat::Argb64).map_err(|e| format!("{e:?}"))?;
        ae::pf::suites::FillMatte::new().and_then(|fill| fill.fill16(input.effect_ref(), &mut world,
            Some(ae::Pixel16 { alpha: 32768, red: 32768, green: 32768, blue: 32768 }), None))
            .map_err(|e| format!("{e:?}"))?;
        let mut outline = path.as_ptr();
        // This probe isolates the host rasterizer with zero feather and full opacity.
        // It does not claim the layer's animated feather, expansion or opacity semantics.
        call!(masks, PF_MaskWorldWithPath, input.effect_ref().as_ptr(), &mut outline,
            0.0, 0.0, inverted as _, 1.0, input.quality().into(), world.as_mut_ptr(), ptr::null_mut())?;
        let mut alpha = Vec::with_capacity(width * height);
        for y in 0..height {
            input.interact().abort().map_err(|e| format!("{e:?}"))?;
            for x in 0..width {
                let at = y * world.row_bytes() as usize + x * std::mem::size_of::<sys::PF_Pixel16>();
                let bytes = &world.buffer()[at..at + 2];
                alpha.push(u16::from_ne_bytes([bytes[0], bytes[1]]));
            }
        }
        let _ = writeln!(report, "mask_raster id={id} mode={mode:?} invert={inverted} width={width} height={height} feather=0 opacity=1\nalpha16={alpha:?}");
    }
    Ok(report)
}

struct CheckedParam<'a> { input: &'a ae::InData, param: sys::PF_ParamDef }
impl Drop for CheckedParam<'_> {
    fn drop(&mut self) {
        if let Err(error) = self.input.interact().checkin_param(&self.param) {
            super::log(&format!("CLEANUP_FAILED original checkout={error:?}"));
        }
    }
}

fn probe_points(width: i32, height: i32) -> Result<[(i32, i32); 3]> {
    if width <= 0 || height <= 0 { return Err("empty probe world".into()); }
    Ok([(210i64, 490i64), (400, 300), (50, 50)].map(|(x, y)| (
        (x * i64::from(width) / 800) as i32,
        (y * i64::from(height) / 600) as i32,
    )))
}

pub fn world_samples(world: &ae::Layer) -> Result<String> {
    let (width, height) = (world.width() as i32, world.height() as i32);
    let depth = world.bit_depth();
    let size = match depth { 8 => 4, 16 => 8, 32 => 16, _ => return Err("unknown depth".into()) };
    if world.row_bytes() <= 0 { return Err("invalid world stride".into()); }
    let mut report = format!("world={width}x{height} depth={depth} origin={:?}\n", world.origin());
    for (x, y) in probe_points(width, height)? {
        let at = pixel_offset(width, height, world.row_bytes() as usize, size, x, y)?;
        let bytes = world.buffer().get(at..at + size).ok_or("world buffer bounds")?;
        let rgba = unsafe {
            match depth {
                8 => {
                    let p = ptr::read_unaligned(bytes.as_ptr().cast::<sys::PF_Pixel8>());
                    [p.red, p.green, p.blue, p.alpha].map(|v| f64::from(v) / 255.0)
                }
                16 => {
                    let p = ptr::read_unaligned(bytes.as_ptr().cast::<sys::PF_Pixel16>());
                    [p.red, p.green, p.blue, p.alpha].map(|v| f64::from(v) / 32768.0)
                }
                _ => {
                    let p = ptr::read_unaligned(bytes.as_ptr().cast::<sys::PF_PixelFloat>());
                    [p.red, p.green, p.blue, p.alpha].map(f64::from)
                }
            }
        };
        let _ = writeln!(report, "rgba[{x},{y}]={rgba:?}");
    }
    Ok(report)
}

pub fn original_pixels(input: &ae::InData, index: i32) -> std::result::Result<String, ae::Error> {
    original_pixels_at(input, index, input.current_time(), input.time_step(), input.time_scale())
}

pub fn original_pixels_at(input: &ae::InData, index: i32, ticks: i32, step: i32, scale: u32)
    -> std::result::Result<String, ae::Error>
{
    input.interact().abort()?;
    let checked = CheckedParam { input, param: input.interact().checkout_param(index,
        ticks, step, scale)? };
    if checked.param.param_type != sys::PF_Param_LAYER { return Err(ae::Error::BadCallbackParameter); }
    let mut world = unsafe { checked.param.u.ld };
    if world.data.is_null() { return Ok("original_world=unavailable".into()); }
    if world.width <= 0 || world.height <= 0 || world.rowbytes <= 0 { return Err(ae::Error::BadCallbackParameter); }
    let layer = ae::Layer::from_raw(&mut world, input, None);
    let depth = layer.bit_depth();
    let bytes = match depth { 8 => 4, 16 => 8, 32 => 16, _ => return Err(ae::Error::BadCallbackParameter) };
    let mut report = format!("original_world={}x{} depth={depth} rowbytes={} origin={:?} time={}/{}\n",
        world.width, world.height, world.rowbytes, layer.origin(), ticks, scale);
    for (x, y) in probe_points(world.width, world.height)
        .map_err(|_| ae::Error::BadCallbackParameter)? {
        input.interact().abort()?;
        let at = pixel_offset(world.width, world.height, world.rowbytes as usize, bytes, x, y)
            .map_err(|_| ae::Error::BadCallbackParameter)?;
        let alpha = unsafe {
            let pixel = world.data.cast::<u8>().add(at);
            match depth {
                8 => f64::from(ptr::read_unaligned(pixel.cast::<sys::PF_Pixel8>()).alpha) / 255.0,
                16 => f64::from(ptr::read_unaligned(pixel.cast::<sys::PF_Pixel16>()).alpha) / 32768.0,
                _ => f64::from(ptr::read_unaligned(pixel.cast::<sys::PF_PixelFloat>()).alpha),
            }
        };
        let _ = writeln!(report, "original_alpha[{x},{y}]={alpha}");
    }
    Ok(report)
}

pub fn host(input: &ae::InData, id: ae::aegp::PluginId, mode: i32, time: ae::Time) -> Result<String> {
    let interface = ae::aegp::suites::PFInterface::new().map_err(|e| format!("{e:?}"))?;
    if mode == 2 {
        let layer = interface.effect_layer(input.effect_ref()).map_err(|e| format!("{e:?}"))?;
        unsafe { streams(input.pica_basic_suite_ptr(), id, layer.as_ptr(), time.into()) }
    } else if (3..=10).contains(&mode) {
        unsafe { coverage(input, id, mode, time.into()) }
    } else { Err("unknown mode".into()) }
}

unsafe fn streams(input: *mut sys::SPBasicSuite, id: i32, layer: sys::AEGP_LayerH, time: sys::A_Time) -> Result<String> {
    let dynamic = Suite::<sys::AEGP_DynamicStreamSuite4>::acquire(input, sys::kAEGPDynamicStreamSuite, sys::kAEGPDynamicStreamSuiteVersion4)?;
    let stream = Suite::<sys::AEGP_StreamSuite6>::acquire(input, sys::kAEGPStreamSuite, sys::kAEGPStreamSuiteVersion6)?;
    let mask = Suite::<sys::AEGP_MaskOutlineSuite3>::acquire(input, sys::kAEGPMaskOutlineSuite, sys::kAEGPMaskOutlineSuiteVersion3)?;
    let dispose = stream.AEGP_DisposeStream.ok_or("missing DisposeStream")?;
    let mut root = ptr::null_mut();
    call!(dynamic, AEGP_GetNewStreamRefForLayer, id, layer, &mut root)?;
    if root.is_null() { return Err("null root stream".into()); }
    let _root = cleanup(|| dispose(root));
    let mut report = String::from("raw post-expression streams; geometry has not been transformed or rasterized\n");
    let mut budget = Budget { nodes: 1024, vertices: 8192, started: Instant::now() };
    for name in [c"ADBE Root Vectors Group", c"ADBE Transform Group"] {
        let mut group = ptr::null_mut();
        call!(dynamic, AEGP_GetNewStreamRefByMatchname, id, root, name.as_ptr(), &mut group)?;
        if group.is_null() { return Err("null group stream".into()); }
        let _group = cleanup(|| dispose(group));
        walk(&dynamic, &stream, &mask, id, group, time, 0, &mut budget, &mut report)?;
    }
    Ok(report)
}

struct Budget { nodes: usize, vertices: usize, started: Instant }
impl Budget {
    fn node(&mut self, depth: usize) -> Result<()> {
        if depth > 32 || self.nodes == 0 || self.started.elapsed().as_secs() >= 10 {
            return Err("stream traversal limit exceeded".into());
        }
        self.nodes -= 1;
        Ok(())
    }
    fn vertices(&mut self, segments: i32) -> Result<usize> {
        let count = usize::try_from(segments).ok().and_then(|n| n.checked_add(1))
            .filter(|n| *n <= self.vertices).ok_or("vertex limit exceeded")?;
        self.vertices -= count;
        Ok(count)
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn walk(dynamic: &sys::AEGP_DynamicStreamSuite4, stream: &sys::AEGP_StreamSuite6,
    mask: &sys::AEGP_MaskOutlineSuite3, id: i32, handle: sys::AEGP_StreamRefH,
    time: sys::A_Time, depth: usize, budget: &mut Budget, report: &mut String) -> Result<()>
{
    budget.node(depth)?;
    let mut name = [0; 256];
    call!(dynamic, AEGP_GetMatchName, handle, name.as_mut_ptr())?;
    name[255] = 0;
    let name = CStr::from_ptr(name.as_ptr()).to_string_lossy();
    let mut grouping = 0;
    call!(dynamic, AEGP_GetStreamGroupingType, handle, &mut grouping)?;
    let _ = writeln!(report, "{}match={name} group={grouping}", "  ".repeat(depth));
    if grouping == sys::AEGP_StreamGroupingType_LEAF {
        let mut kind = 0;
        call!(stream, AEGP_GetStreamType, handle, &mut kind)?;
        if !matches!(kind, sys::AEGP_StreamType_OneD | sys::AEGP_StreamType_TwoD |
            sys::AEGP_StreamType_TwoD_SPATIAL | sys::AEGP_StreamType_ThreeD |
            sys::AEGP_StreamType_ThreeD_SPATIAL | sys::AEGP_StreamType_MASK) {
            let _ = writeln!(report, "type={kind} not sampled");
            return Ok(());
        }
        let dispose = stream.AEGP_DisposeStreamValue.ok_or("missing DisposeStreamValue")?;
        let mut value: sys::AEGP_StreamValue2 = std::mem::zeroed();
        call!(stream, AEGP_GetNewStreamValue, id, handle, sys::AEGP_LTimeMode_LayerTime as _, &time, 0, &mut value)?;
        // The outline is borrowed from this stream value, which must outlive vertex reads.
        let value_ptr = &mut value as *mut _;
        let _value = cleanup(|| dispose(value_ptr));
        match kind {
            sys::AEGP_StreamType_OneD => { let _ = writeln!(report, "value={}", value.val.one_d); }
            sys::AEGP_StreamType_TwoD | sys::AEGP_StreamType_TwoD_SPATIAL => {
                let _ = writeln!(report, "value={:?}", value.val.two_d);
            }
            sys::AEGP_StreamType_ThreeD | sys::AEGP_StreamType_ThreeD_SPATIAL => {
                let _ = writeln!(report, "value={:?}", value.val.three_d);
            }
            sys::AEGP_StreamType_MASK => {
                let outline = value.val.mask;
                if outline.is_null() { return Err("null outline value".into()); }
                let mut segments = 0;
                let mut open = 0;
                call!(mask, AEGP_GetMaskOutlineNumSegments, outline, &mut segments)?;
                call!(mask, AEGP_IsMaskOutlineOpen, outline, &mut open)?;
                let count = budget.vertices(segments)?;
                let _ = writeln!(report, "outline segments={segments} open={open}");
                for index in 0..count {
                    let mut vertex = std::mem::zeroed();
                    call!(mask, AEGP_GetMaskOutlineVertexInfo, outline, index as i32, &mut vertex)?;
                    let _ = writeln!(report, "vertex[{index}]={vertex:?}");
                }
            }
            _ => unreachable!(),
        }
    } else {
        let mut count = 0;
        call!(dynamic, AEGP_GetNumStreamsInGroup, handle, &mut count)?;
        if count < 0 || count as usize > budget.nodes { return Err("stream count limit exceeded".into()); }
        let dispose = stream.AEGP_DisposeStream.ok_or("missing DisposeStream")?;
        for index in 0..count {
            let mut child = ptr::null_mut();
            call!(dynamic, AEGP_GetNewStreamRefByIndex, id, handle, index, &mut child)?;
            if child.is_null() { return Err("null child stream".into()); }
            let _child = cleanup(|| dispose(child));
            let _ = writeln!(report, "index={index}");
            walk(dynamic, stream, mask, id, child, time, depth + 1, budget, report)?;
        }
    }
    Ok(())
}

unsafe extern "C" fn cancel(refcon: *mut std::ffi::c_void, cancelled: *mut sys::A_Boolean) -> sys::A_Err {
    if refcon.is_null() || cancelled.is_null() { return 1; }
    *cancelled = ((*(refcon as *const Instant)).elapsed().as_secs() >= 10) as _;
    0
}

unsafe fn coverage(input: &ae::InData, id: i32, mode: i32, time: sys::A_Time) -> Result<String> {
    let basic = input.pica_basic_suite_ptr();
    let pf = Suite::<sys::AEGP_PFInterfaceSuite1>::acquire(basic, sys::kAEGPPFInterfaceSuite, sys::kAEGPPFInterfaceSuiteVersion1)?;
    let effects = Suite::<sys::AEGP_EffectSuite5>::acquire(basic, sys::kAEGPEffectSuite, sys::kAEGPEffectSuiteVersion5)?;
    let dispose_effect = effects.AEGP_DisposeEffect.ok_or("missing DisposeEffect")?;
    let mut effect = ptr::null_mut();
    call!(pf, AEGP_GetNewEffectForEffect, id, input.effect_ref().as_ptr(), &mut effect)?;
    if effect.is_null() { return Err("null effect reference".into()); }
    let _effect = cleanup(|| dispose_effect(effect));
    let mut layer = ptr::null_mut();
    call!(pf, AEGP_GetEffectLayer, input.effect_ref().as_ptr(), &mut layer)?;
    coverage_for_effect(basic, id, layer, effect, mode, time)
}

pub unsafe fn background(input: *mut sys::SPBasicSuite, id: i32, layer: sys::AEGP_LayerH,
    effect: sys::AEGP_EffectRefH, mode: i32, time: ae::Time) -> Result<String>
{
    match mode {
        2 => streams(input, id, layer, time.into()),
        3..=10 => coverage_for_effect(input, id, layer, effect, mode, time.into()),
        12..=16 => super::stage::inspect(input, id, effect, mode - 12),
        17..=18 => plain_layer(input, id, layer, time.into(), mode == 18),
        _ => Err("background mode must be 2 through 10 or 12 through 18; PF paths use rendering".into()),
    }
}

unsafe fn coverage_for_effect(input: *mut sys::SPBasicSuite, id: i32,
    layer: sys::AEGP_LayerH, effect: sys::AEGP_EffectRefH, mode: i32, time: sys::A_Time) -> Result<String>
{
    if mode == 5 { return Err("retired plain flag: incompatible with upstream layer options".into()); }
    if mode == 10 { return source_item(input, id, layer, time); }
    let options = Suite::<sys::AEGP_LayerRenderOptionsSuite2>::acquire(input, sys::kAEGPLayerRenderOptionsSuite, sys::kAEGPLayerRenderOptionsSuiteVersion2)?;
    let dispose_options = options.AEGP_Dispose.ok_or("missing Dispose options")?;
    let mut opts = ptr::null_mut();
    let origin = if mode == 6 {
        if layer.is_null() { return Err("null effect layer".into()); }
        call!(options, AEGP_NewFromLayer, id, layer, &mut opts)?;
        "layer (all effects)"
    } else if mode == 7 {
        call!(options, AEGP_NewFromDownstreamOfEffect, id, effect, &mut opts)?;
        "downstream of diagnostic"
    } else {
        call!(options, AEGP_NewFromUpstreamOfEffect, id, effect, &mut opts)?;
        "upstream of diagnostic"
    };
    if opts.is_null() { return Err("null render options".into()); }
    let _opts = cleanup(|| dispose_options(opts));
    call!(options, AEGP_SetTime, opts, time)?;
    call!(options, AEGP_SetWorldType, opts,
        if mode == 8 { sys::AEGP_WorldType_32 } else { sys::AEGP_WorldType_16 })?;
    call!(options, AEGP_SetDownsampleFactor, opts, 1, 1)?;
    let mut start = Instant::now();
    let refcon = (&mut start as *mut Instant).cast();
    let mut receipt = ptr::null_mut();
    super::log(&format!("COVERAGE_CHECKOUT_BEGIN mode={mode} options={origin}"));
    if mode != 4 {
        let render = Suite::<sys::AEGP_RenderSuite5>::acquire(input, sys::kAEGPRenderSuite, sys::kAEGPRenderSuiteVersion5)?;
        let checkin = render.AEGP_CheckinFrame.ok_or("missing CheckinFrame")?;
        call!(render, AEGP_RenderAndCheckoutLayerFrame, opts, Some(cancel), refcon, &mut receipt)?;
        if receipt.is_null() { return Err("null frame receipt".into()); }
        let _receipt = cleanup(|| checkin(receipt));
        let mut world = ptr::null_mut();
        let mut region = std::mem::zeroed();
        call!(render, AEGP_GetReceiptWorld, receipt, &mut world)?;
        call!(render, AEGP_GetRenderedRegion, receipt, &mut region)?;
        if mode >= 8 { reference_alpha(input, world, region) } else { pixels(input, world, region) }
    } else {
        let render = Suite::<sys::AEGP_RenderSuite4>::acquire(input, sys::kAEGPRenderSuite, sys::kAEGPRenderSuiteVersion4)?;
        let checkin = render.AEGP_CheckinFrame.ok_or("missing CheckinFrame")?;
        call!(render, AEGP_RenderAndCheckoutLayerFrame, opts, 0, Some(cancel), refcon, &mut receipt)?;
        if receipt.is_null() { return Err("null frame receipt".into()); }
        let _receipt = cleanup(|| checkin(receipt));
        let mut world = ptr::null_mut();
        let mut region = std::mem::zeroed();
        call!(render, AEGP_GetReceiptWorld, receipt, &mut world)?;
        call!(render, AEGP_GetRenderedRegion, receipt, &mut region)?;
        pixels(input, world, region)
    }
}

unsafe fn plain_layer(basic: *mut sys::SPBasicSuite, id: i32, layer: sys::AEGP_LayerH,
    time: sys::A_Time, float: bool) -> Result<String>
{
    if layer.is_null() { return Err("null plain source layer".into()); }
    let _quiet = ae::aegp::suites::Utility::new().and_then(|s| s.start_quiet_errors(false))
        .map_err(|e| format!("{e:?}"))?;
    let options = Suite::<sys::AEGP_LayerRenderOptionsSuite2>::acquire(basic,
        sys::kAEGPLayerRenderOptionsSuite, sys::kAEGPLayerRenderOptionsSuiteVersion2)?;
    let render = Suite::<sys::AEGP_RenderSuite4>::acquire(basic,
        sys::kAEGPRenderSuite, sys::kAEGPRenderSuiteVersion4)?;
    let dispose = options.AEGP_Dispose.ok_or("missing Dispose options")?;
    let checkin = render.AEGP_CheckinFrame.ok_or("missing CheckinFrame")?;
    let mut opts = ptr::null_mut();
    // The plain-layer flag requires layer options, never upstream-effect options.
    call!(options, AEGP_NewFromLayer, id, layer, &mut opts)?;
    if opts.is_null() { return Err("null plain layer options".into()); }
    let _opts = cleanup(|| dispose(opts));
    call!(options, AEGP_SetTime, opts, time)?;
    call!(options, AEGP_SetWorldType, opts, if float { sys::AEGP_WorldType_32 } else { sys::AEGP_WorldType_16 })?;
    call!(options, AEGP_SetDownsampleFactor, opts, 1, 1)?;
    let mut start = Instant::now();
    let mut receipt = ptr::null_mut();
    super::log("PLAIN_LAYER_BEGIN options=NewFromLayer render_suite=4 plain=true");
    call!(render, AEGP_RenderAndCheckoutLayerFrame, opts, 1, Some(cancel),
        (&mut start as *mut Instant).cast(), &mut receipt)?;
    if receipt.is_null() { return Err("null plain frame receipt".into()); }
    let _receipt = cleanup(|| checkin(receipt));
    let mut world = ptr::null_mut();
    let mut region = std::mem::zeroed();
    call!(render, AEGP_GetReceiptWorld, receipt, &mut world)?;
    call!(render, AEGP_GetRenderedRegion, receipt, &mut region)?;
    let samples = pixels(basic, world, region)?;
    let raw = reference_alpha(basic, world, region)?;
    Ok(format!("PLAIN_LAYER_COMPLETE\n{samples}\n{raw}"))
}

unsafe fn source_item(basic: *mut sys::SPBasicSuite, id: i32, layer: sys::AEGP_LayerH,
    time: sys::A_Time) -> Result<String>
{
    if layer.is_null() { return Err("null source layer".into()); }
    let _quiet = ae::aegp::suites::Utility::new().and_then(|s| s.start_quiet_errors(false))
        .map_err(|e| format!("{e:?}"))?;
    let layers = Suite::<sys::AEGP_LayerSuite9>::acquire(basic, sys::kAEGPLayerSuite,
        sys::kAEGPLayerSuiteVersion9)?;
    let mut item = ptr::null_mut();
    let mut object_type = 0;
    call!(layers, AEGP_GetLayerObjectType, layer, &mut object_type)?;
    call!(layers, AEGP_GetLayerSourceItem, layer, &mut item)?;
    if item.is_null() {
        return Ok(format!("source_item=none layer_object_type={object_type}"));
    }
    let items = Suite::<sys::AEGP_ItemSuite9>::acquire(basic, sys::kAEGPItemSuite,
        sys::kAEGPItemSuiteVersion9)?;
    let (mut item_id, mut item_type) = (0, 0);
    call!(items, AEGP_GetItemID, item, &mut item_id)?;
    call!(items, AEGP_GetItemType, item, &mut item_type)?;
    let options = Suite::<sys::AEGP_RenderOptionsSuite4>::acquire(basic,
        sys::kAEGPRenderOptionsSuite, sys::kAEGPRenderOptionsSuiteVersion4)?;
    let dispose = options.AEGP_Dispose.ok_or("missing source options disposal")?;
    let mut opts = ptr::null_mut();
    call!(options, AEGP_NewFromItem, id, item, &mut opts)?;
    if opts.is_null() { return Err("null source-item options".into()); }
    let _options = cleanup(|| dispose(opts));
    call!(options, AEGP_SetTime, opts, time)?;
    call!(options, AEGP_SetWorldType, opts, sys::AEGP_WorldType_16)?;
    call!(options, AEGP_SetDownsampleFactor, opts, 1, 1)?;
    let render = Suite::<sys::AEGP_RenderSuite5>::acquire(basic, sys::kAEGPRenderSuite,
        sys::kAEGPRenderSuiteVersion5)?;
    let checkin = render.AEGP_CheckinFrame.ok_or("missing source frame checkin")?;
    let mut started = Instant::now();
    let mut receipt = ptr::null_mut();
    call!(render, AEGP_RenderAndCheckoutFrame, opts, Some(cancel),
        (&mut started as *mut Instant).cast(), &mut receipt)?;
    if receipt.is_null() { return Err("null source-item receipt".into()); }
    let _receipt = cleanup(|| checkin(receipt));
    let mut world = ptr::null_mut();
    let mut region = std::mem::zeroed();
    call!(render, AEGP_GetReceiptWorld, receipt, &mut world)?;
    call!(render, AEGP_GetRenderedRegion, receipt, &mut region)?;
    let report = pixels(basic, world, region)?;
    Ok(format!("source_item={item_id} item_type={item_type} layer_object_type={object_type}\n{report}"))
}

unsafe fn reference_alpha(input: *mut sys::SPBasicSuite, world: sys::AEGP_WorldH,
    region: sys::A_LRect) -> Result<String>
{
    if world.is_null() { return Err("null reference world".into()); }
    let suite = Suite::<sys::AEGP_WorldSuite3>::acquire(input, sys::kAEGPWorldSuite,
        sys::kAEGPWorldSuiteVersion3)?;
    let (mut width, mut height, mut stride, mut kind) = (0, 0, 0, 0);
    call!(suite, AEGP_GetSize, world, &mut width, &mut height)?;
    call!(suite, AEGP_GetRowBytes, world, &mut stride)?;
    call!(suite, AEGP_GetType, world, &mut kind)?;
    let count = reference_pixel_count(width, height)?;
    if stride <= 0 { return Err("invalid reference stride".into()); }
    let (base, size, depth) = if kind == sys::AEGP_WorldType_32 {
        let mut pixels = ptr::null_mut();
        call!(suite, AEGP_GetBaseAddr32, world, &mut pixels)?;
        (pixels.cast::<u8>(), std::mem::size_of::<sys::PF_PixelFloat>(), 32)
    } else if kind == sys::AEGP_WorldType_16 {
        let mut pixels = ptr::null_mut();
        call!(suite, AEGP_GetBaseAddr16, world, &mut pixels)?;
        (pixels.cast::<u8>(), std::mem::size_of::<sys::PF_Pixel16>(), 16)
    } else { return Err(format!("unexpected reference world type {kind}")); };
    if base.is_null() { return Err("null reference pixels".into()); }
    let mut values = Vec::with_capacity(count);
    for y in 0..height {
        for x in 0..width {
            let at = pixel_offset(width, height, stride as usize, size, x, y)?;
            values.push(if depth == 32 {
                ptr::read_unaligned(base.add(at).cast::<sys::PF_PixelFloat>()).alpha.to_bits()
            } else {
                u32::from(ptr::read_unaligned(base.add(at).cast::<sys::PF_Pixel16>()).alpha)
            });
        }
    }
    Ok(format!("reference width={width} height={height} depth={depth} region={region:?}\nreference_alpha{depth}={values:?}"))
}

fn reference_pixel_count(width: i32, height: i32) -> Result<usize> {
    let count = i64::from(width) * i64::from(height);
    if width <= 0 || height <= 0 || count > 524288 {
        return Err("reference world exceeds diagnostic budget".into());
    }
    Ok(count as usize)
}

fn offset(width: i32, height: i32, stride: usize, x: i32, y: i32) -> Result<usize> {
    pixel_offset(width, height, stride, std::mem::size_of::<sys::PF_Pixel16>(), x, y)
}

fn pixel_offset(width: i32, height: i32, stride: usize, size: usize, x: i32, y: i32) -> Result<usize> {
    if width <= 0 || height <= 0 || x < 0 || y < 0 || x >= width || y >= height {
        return Err("sample outside world".into());
    }
    let packed = (width as usize).checked_mul(size).ok_or("row overflow")?;
    let bytes = stride.checked_mul(height as usize).filter(|n| *n <= isize::MAX as usize).ok_or("world overflow")?;
    if stride < packed { return Err("invalid row stride".into()); }
    (y as usize).checked_mul(stride).and_then(|n| n.checked_add(x as usize * size))
        .filter(|n| n.checked_add(size).is_some_and(|end| end <= bytes)).ok_or_else(|| "sample overflow".into())
}

unsafe fn pixels(input: *mut sys::SPBasicSuite, world: sys::AEGP_WorldH, region: sys::A_LRect) -> Result<String> {
    if world.is_null() { return Err("null receipt world".into()); }
    let suite = Suite::<sys::AEGP_WorldSuite3>::acquire(input, sys::kAEGPWorldSuite, sys::kAEGPWorldSuiteVersion3)?;
    let (mut width, mut height, mut stride, mut kind) = (0, 0, 0, 0);
    call!(suite, AEGP_GetSize, world, &mut width, &mut height)?;
    call!(suite, AEGP_GetRowBytes, world, &mut stride)?;
    call!(suite, AEGP_GetType, world, &mut kind)?;
    if kind != sys::AEGP_WorldType_16 { return Err(format!("expected 16-bit world, got {kind}")); }
    let mut base = ptr::null_mut();
    call!(suite, AEGP_GetBaseAddr16, world, &mut base)?;
    if base.is_null() { return Err("null pixel buffer".into()); }
    let mut report = format!("world={width}x{height} rowbytes={stride} type={kind} region={region:?}\ncoordinates=world-local; compare only identity baseline until origin mapping is proven\n");
    for (x, y) in probe_points(width, height)? {
        match offset(width, height, stride as usize, x, y) {
            Ok(offset) => {
                let pixel = ptr::read_unaligned(base.cast::<u8>().add(offset).cast::<sys::PF_Pixel16>());
                let _ = writeln!(report, "alpha[{x},{y}]={}", f64::from(pixel.alpha) / 32768.0);
                let _ = writeln!(report, "rgba[{x},{y}]={:?}", [pixel.red, pixel.green, pixel.blue, pixel.alpha]
                    .map(|v| f64::from(v) / 32768.0));
            }
            Err(error) => { let _ = writeln!(report, "alpha[{x},{y}]=UNAVAILABLE reason={error}"); }
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reference_budget_accepts_full_fixture_but_rejects_unbounded_worlds() {
        assert_eq!(reference_pixel_count(800, 600).unwrap(), 480000);
        for (width, height) in [(0, 600), (-1, 1), (1920, 1080), (i32::MAX, i32::MAX)] {
            assert!(reference_pixel_count(width, height).is_err());
        }
    }
    #[test]
    fn probe_points_preserve_the_reference_fixture_and_fit_small_worlds() {
        assert_eq!(probe_points(800, 600).unwrap(), [(210, 490), (400, 300), (50, 50)]);
        assert_eq!(probe_points(128, 96).unwrap(), [(33, 78), (64, 48), (8, 8)]);
        for (width, height) in [(1, 1), (1, 96), (128, 1), (1920, 1080), (i32::MAX, i32::MAX)] {
            for (x, y) in probe_points(width, height).unwrap() {
                assert!((0..width).contains(&x) && (0..height).contains(&y));
            }
        }
        assert!(probe_points(0, 96).is_err());
        assert!(probe_points(128, -1).is_err());
    }
    #[test]
    fn retired_plain_mode_never_enters_host_suites() {
        let result = unsafe { coverage_for_effect(ptr::null_mut(), 0, ptr::null_mut(),
            ptr::null_mut(), 5, sys::A_Time { value: 0, scale: 1 }) };
        assert_eq!(result.unwrap_err(), "retired plain flag: incompatible with upstream layer options");
    }
    #[test]
    fn invalid_time_never_saturates_into_a_valid_sample() {
        for input in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 2148.0, -2148.0] { assert!(time(input).is_err()); }
        assert_eq!(time(-0.25).unwrap().value, -250000);
    }
    #[test]
    fn source_sample_offsets_respect_each_depth() {
        for size in [4, 8, 16] {
            assert_eq!(pixel_offset(2, 2, size * 3, size, 1, 1).unwrap(), size * 4);
            assert!(pixel_offset(2, 2, size, size, 0, 0).is_err());
        }
    }
    #[test]
    fn sample_bounds_include_stride_padding_and_exclude_overflow() {
        assert_eq!(offset(2, 2, 24, 1, 1).unwrap(), 32);
        for (w, h, stride, x, y) in [(2, 2, 8, 0, 0), (2, 2, 24, 2, 0), (2, 2, 24, 0, -1),
            (0, 2, 24, 0, 0), (2, 2, usize::MAX, 0, 0)] { assert!(offset(w, h, stride, x, y).is_err()); }
    }
    #[test]
    fn traversal_limits_reject_deep_and_large_geometry() {
        let mut budget = Budget { nodes: 1, vertices: 4, started: Instant::now() };
        assert!(budget.node(33).is_err());
        assert!(budget.node(0).is_ok());
        assert!(budget.node(0).is_err());
        assert!(budget.vertices(-1).is_err());
        assert_eq!(budget.vertices(3).unwrap(), 4);
        assert!(budget.vertices(0).is_err());
    }
    #[test]
    fn cleanup_runs_on_early_error() {
        let released = std::cell::Cell::new(false);
        let result: Result<()> = (|| {
            let _guard = cleanup(|| { released.set(true); 0 });
            Err("read failed".into())
        })();
        assert!(result.is_err());
        assert!(released.get());
    }
}
