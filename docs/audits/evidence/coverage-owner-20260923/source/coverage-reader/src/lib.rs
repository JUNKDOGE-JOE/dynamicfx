#![allow(linker_messages)]
use after_effects as ae;
mod pixels;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum Params { Original }
#[derive(Default)]
struct Global;
#[derive(Default)]
struct Instance { _reserved: u8 }
ae::define_effect!(Global, Instance, Params);

impl AdobePluginGlobal for Global {
    fn params_setup(&self, params: &mut ae::Parameters<Params>, _: ae::InData, _: ae::OutData) -> Result<(), Error> {
        params.add(Params::Original, "Original", ae::LayerDef::setup(|_| {}))
    }
    fn handle_command(&mut self, _: ae::Command, _: ae::InData, _: ae::OutData,
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

impl AdobePluginInstance for Instance {
    fn flatten(&self) -> Result<(u16, Vec<u8>), Error> { Ok((1, Vec::new())) }
    fn unflatten(_: u16, _: &[u8]) -> Result<Self, Error> { Ok(Self::default()) }
    fn render(&self, _: &mut PluginState, _: &ae::Layer, _: &mut ae::Layer) -> Result<(), Error> {
        Err(Error::BadCallbackParameter)
    }
    fn handle_command(&mut self, plugin: &mut PluginState, command: ae::Command) -> Result<(), Error> {
        match command {
            ae::Command::SmartPreRender { mut extra } => {
                let input = plugin.in_data;
                let request = extra.output_request();
                let cb = extra.callbacks();
                let base = cb.checkout_layer(0, 0, &request, input.current_time(), input.time_step(), input.time_scale())?;
                cb.checkout_layer(1, 1, &request, input.current_time(), input.time_step(), input.time_scale())?;
                extra.set_result_rect(base.result_rect.into());
                extra.set_max_result_rect(base.max_result_rect.into());
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
}
