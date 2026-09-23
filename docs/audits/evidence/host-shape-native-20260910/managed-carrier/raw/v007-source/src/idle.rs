use after_effects::{self as ae, AsPtr};
use std::{collections::HashMap, panic::{catch_unwind, AssertUnwindSafe},
    sync::{Arc, atomic::{AtomicBool, Ordering}}, thread::ThreadId, time::Instant};

type Key = (i32, i32, u32, i32);

pub struct State {
    id: ae::aegp::PluginId,
    basic: usize,
    thread: ThreadId,
    alive: Arc<AtomicBool>,
    last: Option<Instant>,
    key: Option<ae::aegp::InstalledEffectKey>,
    serials: HashMap<Key, u32>,
    last_error: Option<String>,
    carrier: super::carrier::State,
}

impl State {
    pub fn new(id: ae::aegp::PluginId, basic: *mut ae::sys::SPBasicSuite, alive: Arc<AtomicBool>) -> Self {
        Self { id, basic: basic as usize, thread: std::thread::current().id(), alive,
            last: None, key: None, serials: HashMap::new(), last_error: None,
            carrier: super::carrier::State::default() }
    }
}

pub fn callback(state: &mut State, max_sleep: &mut i32) -> Result<(), ae::Error> {
    if *max_sleep <= 0 || *max_sleep > 30 { *max_sleep = 30; }
    if !state.alive.load(Ordering::Acquire) || state.last.is_some_and(|t| t.elapsed().as_millis() < 1000) {
        return Ok(());
    }
    state.last = Some(Instant::now());
    let result = catch_unwind(AssertUnwindSafe(|| {
        if state.thread != std::thread::current().id() { return Err(ae::Error::WrongThread); }
        if state.basic == 0 { return Err(ae::Error::InvalidCallback); }
        let Some(_reading) = super::Reading::enter() else { return Ok(()) };
        // Non-effect idle callbacks need their own thread-local suite context.
        let _pica = ae::PicaBasicSuite::from_sp_basic_suite_raw(state.basic as *const _);
        tick(state)
    }));
    let error = match result {
        Ok(Ok(())) => None,
        Ok(Err(e)) => Some(format!("{e:?}")),
        Err(_) => Some("panic isolated in diagnostic idle callback".into()),
    };
    if error != state.last_error {
        if let Some(ref error) = error { super::log(&format!("BACKGROUND_SCAN_FAILED {error}")); }
        state.last_error = error;
    }
    Ok(())
}

struct EffectGuard<'a> { suite: &'a ae::aegp::suites::Effect, handle: ae::aegp::EffectRefHandle }
impl Drop for EffectGuard<'_> {
    fn drop(&mut self) {
        if let Err(e) = self.suite.dispose_effect(&self.handle) {
            super::log(&format!("CLEANUP_FAILED background effect={e:?}"));
        }
    }
}

fn serial(value: f64) -> Result<u32, ae::Error> {
    if !value.is_finite() || !(0.0..=1_000_000.0).contains(&value) || value.fract() != 0.0 {
        return Err(ae::Error::BadCallbackParameter);
    }
    Ok(value as u32)
}

fn changed(previous: &mut Option<u32>, current: u32) -> bool {
    // Seed from persisted state so reopening a fixture does not replay a read.
    let run = previous.is_some_and(|p| p != current) && current > 0;
    *previous = Some(current);
    run
}

fn scalar(streams: &ae::aegp::suites::Stream, effect: &ae::aegp::EffectRefHandle,
    id: ae::aegp::PluginId, index: i32, time: ae::Time) -> Result<f64, ae::Error>
{
    let stream = streams.new_effect_stream_by_index(effect, id, index)?;
    match streams.new_stream_value(&stream, id, ae::aegp::TimeMode::LayerTime, time, true)? {
        ae::aegp::StreamValue::OneD(value) if value.is_finite() => Ok(value),
        _ => Err(ae::Error::BadCallbackParameter),
    }
}

fn tick(state: &mut State) -> Result<(), ae::Error> {
    let projects = ae::aegp::suites::Project::new()?;
    let items = ae::aegp::suites::Item::new()?;
    let comps = ae::aegp::suites::Comp::new()?;
    let layers = ae::aegp::suites::Layer::new()?;
    let effects = ae::aegp::suites::Effect::new()?;
    let streams = ae::aegp::suites::Stream::new()?;
    if state.key.is_none() {
        let mut key = ae::aegp::InstalledEffectKey::None;
        for _ in 0..16384 {
            key = effects.next_installed_effect(key)?;
            if key == ae::aegp::InstalledEffectKey::None { return Ok(()); }
            if effects.effect_match_name(key)? == "DynamicFx Host Shape Probe" {
                state.key = Some(key); break;
            }
        }
    }
    let Some(target) = state.key else { return Err(ae::Error::BadCallbackParameter) };
    let mut remaining = 8192usize;
    for project_index in 0..projects.num_projects()? {
        let project = projects.project_by_index(project_index)?;
        let mut current = Some(items.first_proj_item(&project)?);
        while let Some(item) = current {
            remaining = remaining.checked_sub(1).ok_or(ae::Error::BadCallbackParameter)?;
            current = items.next_proj_item(&project, &item)?;
            if items.item_type(&item)? != ae::aegp::ItemType::Comp { continue; }
            let item_id = items.item_id(&item)?;
            let manage_carriers = items.item_name(&item, state.id)?.starts_with("HS_auto_");
            let Some(comp) = comps.comp_from_item(&item)? else { continue };
            for layer_index in 0..layers.comp_num_layers(&comp)? {
                remaining = remaining.checked_sub(1).ok_or(ae::Error::BadCallbackParameter)?;
                let layer = layers.comp_layer_by_index(&comp, layer_index)?;
                let mut carrier_enabled = false;
                for effect_index in 0..effects.layer_num_effects(&layer)? {
                    remaining = remaining.checked_sub(1).ok_or(ae::Error::BadCallbackParameter)?;
                    let effect = EffectGuard { suite: &effects,
                        handle: effects.layer_effect_by_index(&layer, state.id, effect_index)? };
                    if effects.installed_key_from_layer_effect(&effect.handle)? != target { continue; }
                    let time = layers.layer_current_time(&layer, ae::aegp::TimeMode::LayerTime)?;
                    if manage_carriers {
                        carrier_enabled |= scalar(&streams, &effect.handle, state.id, 7, time)? != 0.0;
                    }
                    let key = (project_index as i32, item_id, layers.layer_id(&layer)?, effect_index);
                    let current_serial = serial(scalar(&streams, &effect.handle, state.id, 5, time)?)?;
                    let mut previous = state.serials.get(&key).copied();
                    let first = previous.is_none();
                    let run = changed(&mut previous, current_serial);
                    if first && state.serials.len() >= 8192 { return Err(ae::Error::BadCallbackParameter); }
                    state.serials.insert(key, current_serial);
                    if first {
                        super::log(&format!("BACKGROUND_READY comp={item_id} layer={} effect={} serial={current_serial} carrier_guard={manage_carriers} carrier_enabled={carrier_enabled}", key.2, effect_index + 1));
                    }
                    if !run { continue; }
                    let seconds = scalar(&streams, &effect.handle, state.id, 1, time)?;
                    let mode = serial(scalar(&streams, &effect.handle, state.id, 2, time)?)? as i32;
                    super::log(&format!("BACKGROUND_BEGIN comp={item_id} layer={} effect={} serial={current_serial} mode={mode} layer_seconds={seconds}", key.2, effect_index + 1));
                    let result = super::read::time(seconds).and_then(|sample_time| unsafe {
                        super::read::background(state.basic as *mut _, state.id, layer.as_ptr(),
                            effect.handle.as_ptr(), mode, sample_time)
                    });
                    match result {
                        Ok(report) => super::log(&format!("{report}\nBACKGROUND_COMPLETE comp={item_id} serial={current_serial}")),
                        Err(error) => super::log(&format!("BACKGROUND_FAILED comp={item_id} serial={current_serial} error={error}")),
                    }
                    return Ok(());
                }
                if manage_carriers {
                    state.carrier.reconcile((project_index as i32, item_id, layers.layer_id(&layer)?),
                        state.id, &layer, carrier_enabled)?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persisted_requests_do_not_replay_and_each_change_runs_once() {
        let mut state = None;
        assert!(!changed(&mut state, 7));
        assert!(!changed(&mut state, 7));
        assert!(changed(&mut state, 8));
        assert!(!changed(&mut state, 8));
        assert!(!changed(&mut state, 0));
        assert!(changed(&mut state, 1));
    }
    #[test]
    fn request_serial_rejects_fractional_nonfinite_and_out_of_range_values() {
        for value in [-1.0, 0.5, f64::NAN, f64::INFINITY, 1_000_001.0] { assert!(serial(value).is_err()); }
        assert_eq!(serial(1_000_000.0).unwrap(), 1_000_000);
    }
}
