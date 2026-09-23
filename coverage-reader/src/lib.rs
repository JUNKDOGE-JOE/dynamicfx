#![allow(linker_messages)]
use after_effects as ae;
mod pixels;
#[path = "../../src/host/shared_dispatch.rs"]
mod shared_dispatch;
#[path = "../../src/host/render_trace.rs"]
mod render_trace;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum Params { Original }
#[derive(Default)]
struct Global;
#[derive(Default)]
struct Instance { _reserved: u8 }
crate::define_shared_effect!(Global, Instance, Params);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn CoverageReaderMain(cmd: ae::sys::PF_Cmd, input: *mut ae::sys::PF_InData,
    output: *mut ae::sys::PF_OutData, params: *mut *mut ae::sys::PF_ParamDef,
    world: *mut ae::sys::PF_LayerDef, extra: *mut std::ffi::c_void) -> ae::sys::PF_Err {
    let _trace = unsafe { render_trace::Span::enter("reader", cmd, input) };
    unsafe { EffectMain(cmd, input, output, params, world, extra) }
}

impl AdobePluginGlobal for Global {
    fn params_setup(&self, params: &mut ae::Parameters<Params>, _: ae::InData, _: ae::OutData) -> Result<(), Error> {
        params.add(Params::Original, "Original", ae::LayerDef::setup(|_| {}))
    }
    fn handle_command(&self, _: ae::Command, _: ae::InData, _: ae::OutData,
        _: &mut ae::Parameters<Params>) -> Result<(), Error> { Ok(()) }
}

fn checkin_all(ids: impl DoubleEndedIterator<Item = u32>, mut checkin: impl FnMut(u32) -> Result<(), Error>) -> Result<(), Error> {
    let mut result = Ok(());
    for id in ids.rev() { let next = checkin(id); if result.is_ok() { result = next; } }
    result
}

fn copy(source: &ae::Layer, output: &mut ae::Layer) -> Result<(), Error> {
    if source.world_type() != output.world_type() { return Err(Error::BadCallbackParameter); }
    let size = match source.bit_depth() { 8 => 4, 16 => 8, 32 => 16, _ => return Err(Error::BadCallbackParameter) };
    let plane = |world: &ae::Layer| {
        let origin = world.origin();
        pixels::Plane { width: world.width(), height: world.height(), stride: world.buffer_stride(), x: origin.h, y: origin.v }
    };
    let (src, dst) = (plane(source), plane(output));
    pixels::copy(source.buffer(), output.buffer_mut(), src, dst, size).map_err(|_| Error::BadCallbackParameter)
}

fn union(a: ae::sys::PF_LRect, b: ae::sys::PF_LRect) -> ae::sys::PF_LRect {
    ae::sys::PF_LRect { left: a.left.min(b.left), top: a.top.min(b.top), right: a.right.max(b.right), bottom: a.bottom.max(b.bottom) }
}

fn requested_extent(available: ae::sys::PF_LRect, request: ae::sys::PF_LRect) -> ae::sys::PF_LRect {
    let left = available.left.max(request.left);
    let top = available.top.max(request.top);
    ae::sys::PF_LRect { left, top, right: available.right.min(request.right).max(left), bottom: available.bottom.min(request.bottom).max(top) }
}

impl AdobePluginInstance for Instance {
    fn flatten(&self) -> Result<(u16, Vec<u8>), Error> { Ok((1, Vec::new())) }
    fn unflatten(_: u16, _: &[u8]) -> Result<Self, Error> { Ok(Self::default()) }
    fn render(&self, _: &mut PluginState, _: &ae::Layer, _: &mut ae::Layer) -> Result<(), Error> {
        Err(Error::BadCallbackParameter)
    }
    fn handle_command(&self, plugin: &mut PluginState, command: ae::Command) -> Result<(), Error> {
        match command {
            ae::Command::SmartPreRender { mut extra } => {
                let input = plugin.in_data;
                let request = extra.output_request();
                let cb = extra.callbacks();
                let base = cb.checkout_layer(0, 0, &request, input.current_time(), input.time_step(), input.time_scale())?;
                let source = cb.checkout_layer(1, 1, &request, input.current_time(), input.time_step(), input.time_scale())?;
                extra.set_result_rect(requested_extent(union(base.result_rect, source.result_rect), request.rect).into());
                extra.set_max_result_rect(union(base.max_result_rect, source.max_result_rect).into());
            }
            ae::Command::SmartRender { extra } => {
                let cb = extra.callbacks();
                let mut checked = Vec::new();
                let result = (|| {
                    let _base = cb.checkout_layer_pixels(0)?;
                    checked.push(0);
                    let source = cb.checkout_layer_pixels(1)?;
                    checked.push(1);
                    let source = source.ok_or(Error::BadCallbackParameter)?;
                    let mut output = cb.checkout_output()?.ok_or(Error::BadCallbackParameter)?;
                    copy(&source, &mut output)
                })();
                let cleanup = checkin_all(checked.into_iter(), |id| cb.checkin_layer_pixels(id));
                return result.and(cleanup);
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checkin_failure_still_releases_every_successful_checkout() {
        let mut released = Vec::new();
        let result = checkin_all([0, 1].into_iter(), |id| { released.push(id); Err(Error::Generic) });
        assert!(result.is_err());
        assert_eq!(released, [1, 0]);
    }

    #[test]
    fn source_beyond_comp_is_preserved_while_roi_stays_within_the_request() {
        let rect = |left, top, right, bottom| ae::sys::PF_LRect { left, top, right, bottom };
        let maximum = union(rect(0, 0, 800, 600), rect(-80, 40, 880, 560));
        assert_eq!((maximum.left, maximum.top, maximum.right, maximum.bottom), (-80, 0, 880, 600));
        let region = requested_extent(maximum, rect(0, 0, 64, 64));
        assert_eq!((region.left, region.top, region.right, region.bottom), (0, 0, 64, 64));
    }
}
