#![allow(linker_messages)]

use after_effects as ae;
use std::cell::Cell;
use std::io::Write;

mod read;
mod idle;
mod carrier;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params { Time, Mode, Run, RenderPaths, Request, Original, Carrier }

#[derive(Default)]
struct Global {
    id: Option<ae::aegp::PluginId>,
    main_thread: Option<std::thread::ThreadId>,
    alive: std::sync::Arc<std::sync::atomic::AtomicBool>,
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

impl AdobePluginGlobal for Global {
    fn params_setup(&self, params: &mut ae::Parameters<Params>, _: ae::InData, _: ae::OutData)
        -> Result<(), Error>
    {
        params.add(Params::Time, "Layer time (s)", ae::FloatSliderDef::setup(|p| {
            p.set_valid_min(-2000.0); p.set_valid_max(2000.0);
            p.set_slider_min(0.0); p.set_slider_max(10.0); p.set_default(0.0);
        }))?;
        params.add(Params::Mode, "Read mode", ae::PopupDef::setup(|p| {
            p.set_options(&["PF paths", "Vector streams", "Coverage v5 upstream", "Coverage v4 upstream",
                "Retired plain flag", "Coverage v5 layer", "Coverage v5 downstream"]);
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
            log("probe=0.0.9 initialized; managed carrier is diagnostic-only");
        }
        if matches!(command, ae::Command::GlobalSetdown) {
            self.alive.store(false, std::sync::atomic::Ordering::Release);
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
        if plugin.params.get(Params::RenderPaths)?.as_checkbox()?.value() {
            for index in [0, plugin.params.index(Params::Original).ok_or(Error::BadCallbackParameter)? as i32] {
                log(&format!("PF_ORIGINAL_BEGIN index={index}"));
                match read::original_pixels(&plugin.in_data, index) {
                    Ok(report) => log(&format!("PF_ORIGINAL index={index}\n{report}")),
                    Err(error) => { log(&format!("PF_ORIGINAL_FAILED index={index} error={error:?}")); return Err(error); }
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
        output.copy_from(input, None, None)
    }

    fn handle_command(&mut self, plugin: &mut PluginState, command: ae::Command)
        -> Result<(), Error>
    {
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
    fn nested_read_does_not_clear_the_outer_guard() {
        let first = Reading::enter().unwrap();
        assert!(Reading::enter().is_none());
        assert!(Reading::enter().is_none());
        drop(first);
        assert!(Reading::enter().is_some());
    }
}
