#![allow(linker_messages)]

use after_effects as ae;
use std::cell::Cell;
use std::io::Write;

mod read;
mod idle;
mod carrier;
mod wake;
mod stage;
mod reader;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params { Time, Mode, Run, RenderPaths, Request, Original, Carrier, MaskRaster, ExplicitSource, Reader, ReaderRole }

#[derive(Default)]
struct Global {
    id: Option<ae::aegp::PluginId>,
    main_thread: Option<std::thread::ThreadId>,
    alive: std::sync::Arc<std::sync::atomic::AtomicBool>,
    wake: Option<wake::Pump>,
}

#[derive(Default)]
struct Instance { _reserved: u8 }

ae::define_effect!(Global, Instance, Params);

thread_local! { static READING: Cell<bool> = const { Cell::new(false) }; }

struct Reading;
impl Reading {
    fn enter() -> Option<Self> {
        READING.with(|active| if active.replace(true) { None } else { Some(Self) })
    }
}
impl Drop for Reading {
    fn drop(&mut self) { READING.with(|active| active.set(false)); }
}

fn log(text: &str) {
    let path = std::env::temp_dir().join("dynamicfx-host-shape-probe.log");
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map(|v| v.as_millis()).unwrap_or(0);
        let _ = writeln!(file, "[time={time} pid={} thread={:?}]\n{text}",
            std::process::id(), std::thread::current().id());
    }
}

fn checkin_all(ids: impl DoubleEndedIterator<Item = u32>,
    mut checkin: impl FnMut(u32) -> Result<(), ae::Error>) -> Result<(), ae::Error>
{
    let mut first_error = None;
    for id in ids.rev() {
        if let Err(error) = checkin(id) {
            log(&format!("SMART_CHECKIN_FAILED id={id} {error:?}"));
            if first_error.is_none() { first_error = Some(error); }
        }
    }
    match first_error { Some(error) => Err(error), None => Ok(()) }
}

impl AdobePluginGlobal for Global {
    fn params_setup(&self, params: &mut ae::Parameters<Params>, _: ae::InData, _: ae::OutData)
        -> Result<(), Error>
    {
        if cfg!(feature = "preset-seed") {
            params.add(Params::Original, "Original self source", ae::LayerDef::setup(|_| {}))?;
            params.add(Params::ExplicitSource, "Explicit source (default none)", ae::LayerDef::setup(|_| {}))?;
            return Ok(());
        }
        params.add(Params::Time, "Layer time (s)", ae::FloatSliderDef::setup(|p| {
            p.set_valid_min(-2000.0); p.set_valid_max(2000.0);
            p.set_slider_min(0.0); p.set_slider_max(10.0); p.set_default(0.0);
        }))?;
        params.add(Params::Mode, "Read mode", ae::PopupDef::setup(|p| {
            p.set_options(&["PF paths", "Vector streams", "Coverage v5 upstream", "Coverage v4 upstream",
                "Retired plain flag", "Coverage v5 layer", "Coverage v5 downstream",
                "Reference alpha 32-bit", "Reference alpha 16-bit", "Original source item",
                "PF self time variants", "Inspect layer stage", "Set source stage", "Set masks stage",
                "Set layer and source stage", "Set layer and masks stage",
                "Plain layer v4 16-bit", "Plain layer v4 32-bit",
                "Plain layer options v1 16-bit", "Plain layer options v1 32-bit",
                "Dump SmartFX native alpha", "Forward selected source", "Set first effect stage",
                "Set all effects stage", "Set first effect stage only"]);
            p.set_default(1);
        }))?;
        params.add(Params::Run, "Read host", ae::ButtonDef::setup(|p| { p.set_label("Read"); }))?;
        params.add(Params::RenderPaths, "Log PF paths and channels", ae::CheckBoxDef::setup(|p| {
            p.set_default(false);
        }))?;
        params.add_with_flags(Params::Request, "Background request serial", ae::FloatSliderDef::setup(|p| {
            p.set_valid_min(0.0); p.set_valid_max(1_000_000.0);
            p.set_slider_min(0.0); p.set_slider_max(100.0); p.set_default(0.0);
        }), ae::ParamFlag::CANNOT_TIME_VARY, ae::ParamUIFlags::empty())?;
        params.add(Params::Original, "Original self source", ae::LayerDef::setup(|p| {
            p.set_default_to_this_layer();
        }))?;
        params.add_with_flags(Params::Carrier, "Manage coverage carrier", ae::CheckBoxDef::setup(|p| {
            p.set_default(false);
        }), ae::ParamFlag::CANNOT_TIME_VARY, ae::ParamUIFlags::empty())?;
        params.add(Params::MaskRaster, "Log native mask raster", ae::CheckBoxDef::setup(|p| {
            p.set_default(false);
        }))?;
        params.add(Params::ExplicitSource, "Explicit source (default none)", ae::LayerDef::setup(|_| {}))?;
        params.add_with_flags(Params::Reader, "Manage native coverage reader", ae::CheckBoxDef::setup(|p| {
            p.set_default(false);
        }), ae::ParamFlag::CANNOT_TIME_VARY, ae::ParamUIFlags::empty())?;
        params.add_with_flags(Params::ReaderRole, "Internal native reader role", ae::CheckBoxDef::setup(|p| {
            p.set_default(false);
        }), ae::ParamFlag::CANNOT_TIME_VARY, ae::ParamUIFlags::empty())?;
        Ok(())
    }

    fn handle_command(&mut self, command: ae::Command, input: ae::InData, _: ae::OutData,
        _: &mut ae::Parameters<Params>) -> Result<(), Error>
    {
        if matches!(command, ae::Command::GlobalSetup) {
            self.main_thread = Some(std::thread::current().id());
            self.id = Some(ae::aegp::suites::Utility::new()?.register_with_aegp("DynamicFx Host Shape Probe")?);
            self.alive.store(true, std::sync::atomic::Ordering::Release);
            let state = idle::State::new(self.id.unwrap(), input.pica_basic_suite_ptr(), self.alive.clone());
            ae::aegp::suites::RegisterNonAegp::new()?.register_idle_hook(self.id.unwrap(), Box::new(idle::callback), state)?;
            self.wake = Some(wake::Pump::start(input.pica_basic_suite_ptr(), self.alive.clone())?);
            log(&format!("probe={} preset_seed={} initialized; diagnostic-only", env!("CARGO_PKG_VERSION"), cfg!(feature = "preset-seed")));
        }
        if matches!(command, ae::Command::GlobalSetdown) {
            self.alive.store(false, std::sync::atomic::Ordering::Release);
            self.wake.take();
        }
        Ok(())
    }
}

impl AdobePluginInstance for Instance {
    fn flatten(&self) -> Result<(u16, Vec<u8>), Error> { Ok((1, Vec::new())) }
    fn unflatten(_: u16, _: &[u8]) -> Result<Self, Error> { Ok(Self::default()) }

    fn render(&self, plugin: &mut PluginState, input: &ae::Layer, output: &mut ae::Layer)
        -> Result<(), Error>
    {
        if cfg!(feature = "preset-seed") { return output.copy_from(input, None, None); }
        if plugin.params.get(Params::RenderPaths)?.as_checkbox()?.value() {
            for index in [0, plugin.params.index(Params::Original).ok_or(Error::BadCallbackParameter)? as i32,
                plugin.params.index(Params::ExplicitSource).ok_or(Error::BadCallbackParameter)? as i32] {
                log(&format!("PF_ORIGINAL_BEGIN index={index}"));
                match read::original_pixels(&plugin.in_data, index) {
                    Ok(report) => log(&format!("PF_ORIGINAL index={index}\n{report}")),
                    Err(error) => { log(&format!("PF_ORIGINAL_FAILED index={index} error={error:?}")); return Err(error); }
                }
            }
            if plugin.params.get(Params::Mode)?.as_popup()?.value() == 11 {
                let index = plugin.params.index(Params::ExplicitSource).ok_or(Error::BadCallbackParameter)? as i32;
                let tick = plugin.in_data.current_time();
                let scale = plugin.in_data.time_scale();
                let step = plugin.in_data.time_step();
                let variants = [
                    ("zero_step", Some(tick), Some(0), Some(scale)),
                    ("equivalent_time", tick.checked_mul(2), step.checked_mul(2), scale.checked_mul(2)),
                    ("next_frame", tick.checked_add(step), Some(step), Some(scale)),
                ];
                for (label, tick, step, scale) in variants {
                    if let (Some(tick), Some(step), Some(scale)) = (tick, step, scale) {
                        match read::original_pixels_at(&plugin.in_data, index, tick, step, scale) {
                            Ok(report) => log(&format!("PF_TIME_VARIANT {label}\n{report}")),
                            Err(error) => log(&format!("PF_TIME_VARIANT_FAILED {label} {error:?}")),
                        }
                    }
                }
            }
            let result = read::paths(&plugin.in_data, plugin.in_data.current_time(),
                plugin.in_data.time_scale());
            match result {
                Ok(report) => log(&format!("PF_RENDER\n{report}")),
                Err(error) => { log(&format!("PF_RENDER error={error:?}")); return Err(error); }
            }
            match read::auxiliary(&plugin.in_data) {
                Ok(report) => log(&format!("PF_AUXILIARY\n{report}")),
                Err(error) => { log(&format!("PF_AUXILIARY error={error:?}")); return Err(error); }
            }
        }
        if plugin.params.get(Params::MaskRaster)?.as_checkbox()?.value() {
            match read::raster_masks(&plugin.in_data, input) {
                Ok(report) => log(&format!("PF_MASK_RASTER\n{report}")),
                Err(error) => log(&format!("PF_MASK_RASTER_FAILED {error}")),
            }
        }
        output.copy_from(input, None, None)
    }

    fn handle_command(&mut self, plugin: &mut PluginState, command: ae::Command)
        -> Result<(), Error>
    {
        if cfg!(feature = "preset-seed") {
            match command {
                ae::Command::SmartPreRender { mut extra } => {
                    let input = plugin.in_data;
                    let result = extra.callbacks().checkout_layer(0, 0, &extra.output_request(),
                        input.current_time(), input.time_step(), input.time_scale())?;
                    extra.set_result_rect(result.result_rect.into());
                    extra.set_max_result_rect(result.max_result_rect.into());
                }
                ae::Command::SmartRender { extra } => {
                    let cb = extra.callbacks();
                    let input = cb.checkout_layer_pixels(0)?;
                    let result = (|| {
                        if let (Some(input), Some(mut output)) = (input, cb.checkout_output()?) {
                            output.copy_from(&input, None, None)?;
                        }
                        Ok(())
                    })();
                    return result.and(cb.checkin_layer_pixels(0));
                }
                _ => {}
            }
            return Ok(());
        }
        match command {
            ae::Command::SmartPreRender { mut extra } => {
                let cb = extra.callbacks();
                let request = extra.output_request();
                let input = plugin.in_data;
                let indexes = [0, plugin.params.index(Params::Original).ok_or(Error::BadCallbackParameter)? as i32,
                    plugin.params.index(Params::ExplicitSource).ok_or(Error::BadCallbackParameter)? as i32];
                for (id, index) in indexes.into_iter().enumerate() {
                    let result = cb.checkout_layer(index, id as i32, &request,
                        input.current_time(), input.time_step(), input.time_scale())?;
                    if id == 0 {
                        extra.set_result_rect(result.result_rect.into());
                        extra.set_max_result_rect(result.max_result_rect.into());
                    }
                }
                return Ok(());
            }
            ae::Command::SmartRender { extra } => {
                let cb = extra.callbacks();
                let logging = plugin.params.get(Params::RenderPaths)?.as_checkbox()?.value();
                let dump = plugin.params.get(Params::Mode)?.as_popup()?.value() == 21;
                let forward = plugin.params.get(Params::Mode)?.as_popup()?.value() == 22;
                let mut checked = Vec::new();
                let result = (|| {
                    let mut worlds = Vec::new();
                    for id in 0..3 {
                        let world = cb.checkout_layer_pixels(id)?;
                        checked.push(id);
                        if logging {
                            match &world {
                                Some(world) => match read::world_samples(world) {
                                    Ok(report) => log(&format!("SMART_INPUT id={id}\n{report}")),
                                    Err(error) => log(&format!("SMART_INPUT_FAILED id={id} {error}")),
                                },
                                None => log(&format!("SMART_INPUT id={id} unavailable")),
                            }
                        }
                        if dump {
                            if let Some(world) = &world {
                                match read::world_alpha(world) {
                                    Ok(report) => log(&format!("SMART_ALPHA id={id}\n{report}")),
                                    Err(error) => log(&format!("SMART_ALPHA_FAILED id={id} {error}")),
                                }
                            }
                        }
                        worlds.push(world);
                    }
                    if let Some(mut output) = cb.checkout_output()? {
                        if forward {
                            if let Some(source) = &worlds[1] {
                                read::copy_aligned(source, &mut output).map_err(|_| Error::BadCallbackParameter)?;
                            } else { return Err(Error::BadCallbackParameter); }
                        } else if let Some(input) = &worlds[0] {
                            output.copy_from(input, None, None)?;
                        } else {
                            let fill = ae::pf::suites::FillMatte::new()?;
                            match output.bit_depth() {
                                8 => fill.fill(plugin.in_data.effect_ref(), &mut output, None, None)?,
                                16 => fill.fill16(plugin.in_data.effect_ref(), &mut output, None, None)?,
                                _ => fill.fill_float(plugin.in_data.effect_ref(), &mut output, None, None)?,
                            }
                        }
                    }
                    Ok(())
                })();
                // Every successful pixels checkout, including an empty world, is balanced.
                let cleanup = checkin_all(checked.into_iter(), |id| cb.checkin_layer_pixels(id));
                return result.and(cleanup);
            }
            _ => {}
        }
        let ae::Command::UserChangedParam { param_index } = command else { return Ok(()) };
        if Some(param_index) != plugin.params.index(Params::Run) { return Ok(()) }
        if plugin.global.main_thread != Some(std::thread::current().id()) {
            log("READ refused: callback is not on the setup thread");
            return Err(Error::InvalidCallback);
        }
        let Some(_reading) = Reading::enter() else {
            log("READ deferred: nested read");
            return Ok(());
        };
        let seconds = plugin.params.get(Params::Time)?.as_float_slider()?.value();
        let time = match read::time(seconds) {
            Ok(time) => time,
            Err(error) => { log(&format!("READ error={error}")); return Ok(()) }
        };
        let mode = plugin.params.get(Params::Mode)?.as_popup()?.value();
        let Some(id) = plugin.global.id else { return Err(Error::InvalidCallback) };
        let result = if mode == 1 {
            read::paths(&plugin.in_data, time.value, time.scale).map_err(|e| format!("{e:?}"))
        } else {
            read::host(&plugin.in_data, id, mode, time)
        };
        match result {
            Ok(report) => log(&format!("READ mode={mode} layer_time={}/{}\n{report}\nREAD_COMPLETE", time.value, time.scale)),
            Err(error) => log(&format!("READ_FAILED mode={mode} layer_time={}/{} error={error}", time.value, time.scale)),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_checkin_does_not_leave_other_worlds_checked_out() {
        let mut called = Vec::new();
        let result = checkin_all([0, 1, 2].into_iter(), |id| {
            called.push(id);
            if id == 2 { Err(ae::Error::Generic) } else { Ok(()) }
        });
        assert_eq!(called, [2, 1, 0]);
        assert!(result.is_err());
    }
    #[test]
    fn nested_read_does_not_clear_the_outer_guard() {
        let first = Reading::enter().unwrap();
        assert!(Reading::enter().is_none());
        assert!(Reading::enter().is_none());
        drop(first);
        assert!(Reading::enter().is_some());
    }
}
