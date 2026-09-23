use after_effects::{self as ae, AsPtr};
use ae::aegp::{suites, CompHandle, EffectRefHandle, InstalledEffectKey, ItemHandle, LayerFlags,
    LayerHandle, LayerStream, ObjectType, PluginId, StreamValue, TimeMode};
use std::collections::{HashMap, HashSet};
use super::coverage::{Binding, Stage, StageAccess};
use super::params::{stream_index_of, ParamKey};
use super::coverage_readiness::{Permit, Publication};
use std::sync::Arc;

const READER_NAME: &str = "DynamicFx Coverage Reader";
const SOURCE_MARKER: &str = "DynamicFX coverage reader source; schema=1";
type Key = (i32, i32, u32);
pub struct Request { pub enabled: bool, pub permit: Arc<Permit> }
pub type Requests = HashMap<(u32, i32), Request>;

#[derive(Default)]
pub struct State {
    enabled: HashMap<Key, bool>,
    failed: HashSet<Key>,
    known_readers: HashMap<Key, u32>,
    removed_orphans: HashSet<Key>,
}

#[derive(Debug, PartialEq)]
enum Action { None, Create, Bind, Remove, Missing, Conflict }

fn action(previous: bool, enabled: bool, count: usize, valid: bool) -> Action {
    if count > 1 || (count == 1 && !valid) { return Action::Conflict; }
    if !previous && !enabled { return Action::None; }
    match (enabled, count, previous) {
        (true, 0, false) => Action::Create,
        (true, 0, true) => Action::Missing,
        (true, 1, _) => Action::Bind,
        (false, 1, _) => Action::Remove,
        _ => Action::None,
    }
}

struct Effect<'a> { suite: &'a suites::Effect, handle: EffectRefHandle }
impl Drop for Effect<'_> {
    fn drop(&mut self) {
        if let Err(error) = self.suite.dispose_effect(&self.handle) {
            crate::diag::log(&format!("coverage effect disposal: {error:?}"));
        }
    }
}
struct Undo(suites::Utility);
impl Undo {
    fn start() -> Result<Self, ae::Error> {
        let suite = suites::Utility::new()?;
        suite.start_undo_group("Manage DynamicFX coverage reader")?;
        Ok(Self(suite))
    }
}
impl Drop for Undo {
    fn drop(&mut self) {
        if let Err(error) = self.0.end_undo_group() { crate::diag::log(&format!("coverage undo end: {error:?}")); }
    }
}

fn failure(error: super::coverage::BindingError) -> ae::Error {
    crate::diag::log(&format!("coverage stage: {error:?}"));
    ae::Error::Generic
}
fn coverage_index() -> Result<i32, ae::Error> {
    stream_index_of(ParamKey::Pool(crate::binding::PoolKind::Coverage, 0)).ok_or(ae::Error::BadCallbackParameter)
}
fn state_index() -> Result<i32, ae::Error> {
    stream_index_of(ParamKey::CoverageState).ok_or(ae::Error::BadCallbackParameter)
}
fn write_state(basic: *const ae::sys::SPBasicSuite, id: PluginId, effect: &EffectRefHandle, desired: f64) -> Result<bool, ae::Error> {
    let streams = suites::Stream::new()?;
    let stream = streams.new_effect_stream_by_index(effect, id, state_index()?)?;
    let current = streams.new_stream_value(&stream, id, TimeMode::LayerTime, ae::Time { value: 0, scale: 1 }, true)?;
    if matches!(current, StreamValue::OneD(value) if value == desired) { return Ok(false); }
    super::idle::write_one_d(basic, id, &stream, desired)?;
    Ok(true)
}

struct Owner { layer: LayerHandle, id: u32, effects: Vec<(i32, bool, Arc<Permit>)> }
struct Reader { layer: LayerHandle, id: u32, owner: u32, valid: bool, source: Option<ItemHandle> }

fn identity_value(stream: LayerStream, value: StreamValue, width: u32, height: u32) -> bool {
    let expected = match stream {
        LayerStream::AnchorPoint | LayerStream::Position => [width as f64 / 2.0, height as f64 / 2.0, 0.0],
        LayerStream::Scale => [100.0; 3],
        LayerStream::RotateZ => [0.0; 3],
        LayerStream::Opacity => [100.0; 3],
        _ => return false,
    };
    match value {
        StreamValue::OneD(x) => x == expected[0],
        StreamValue::TwoD { x, y } | StreamValue::TwoDSpatial { x, y } => [x, y] == expected[..2],
        StreamValue::ThreeD { x, y, z } | StreamValue::ThreeDSpatial { x, y, z } => [x, y, z] == expected,
        _ => false,
    }
}

fn reader_identity(layer: &LayerHandle, source: &ItemHandle, comp_item: &ItemHandle, id: PluginId) -> Result<bool, ae::Error> {
    let items = suites::Item::new()?;
    let footage = suites::Footage::new()?;
    if items.item_type(source)? != ae::aegp::ItemType::Footage ||
        footage.footage_signature(footage.main_footage_from_item(source)?)? != ae::aegp::FootageSignature::Solid {
        return Ok(false);
    }
    let (width, height) = items.item_dimensions(source)?;
    if width == 0 || height == 0 { return Ok(false); }
    let layers = suites::Layer::new()?;
    let stretch = layers.layer_stretch(layer)?;
    let offset = layers.layer_offset(layer)?;
    let start = layers.layer_in_point(layer, TimeMode::CompTime)?;
    let duration = layers.layer_duration(layer, TimeMode::CompTime)?;
    let comp_duration = items.item_duration(comp_item)?;
    if stretch.den == 0 || stretch.num as i64 != stretch.den as i64 || offset.value != 0 || start.value != 0 ||
        duration.scale == 0 || comp_duration.scale == 0 ||
        (duration.value as i64) * (comp_duration.scale as i64) < (comp_duration.value as i64) * (duration.scale as i64) {
        return Ok(false);
    }
    let streams = suites::Stream::new()?;
    let dynamic = suites::DynamicStream::new()?;
    let keys = suites::Keyframe::new()?;
    for kind in [LayerStream::AnchorPoint, LayerStream::Position, LayerStream::Scale, LayerStream::RotateZ, LayerStream::Opacity] {
        let stream = streams.new_layer_stream(layer, id, kind)?;
        if keys.stream_num_kfs(&stream)? > 0 || streams.expression_state(&stream, id)? ||
            (dynamic.is_separation_leader(&stream)? && dynamic.are_dimensions_separated(&stream)?) {
            return Ok(false);
        }
        let value = streams.new_stream_value(&stream, id, TimeMode::LayerTime, ae::Time { value: 0, scale: 1 }, true)?;
        if !identity_value(kind, value, width, height) { return Ok(false); }
    }
    Ok(true)
}

fn installed_reader(effects: &suites::Effect) -> Result<Option<InstalledEffectKey>, ae::Error> {
    let mut current = InstalledEffectKey::None;
    for _ in 0..effects.num_installed_effects()? {
        current = effects.next_installed_effect(current)?;
        if current == InstalledEffectKey::None { break; }
        if effects.effect_match_name(current)? == READER_NAME { return Ok(Some(current)); }
    }
    Ok(None)
}

impl State {
    fn forget_missing_owners(&mut self, project: i32, comp: i32, present: &HashSet<u32>) {
        let keep = |key: &Key| key.0 != project || key.1 != comp || present.contains(&key.2);
        self.enabled.retain(|key, _| keep(key));
        self.failed.retain(keep);
    }
    pub fn needs_scan(&self, project: i32, comp_id: i32, requests: &Requests) -> bool {
        requests.values().any(|value| value.enabled) || self.enabled.keys().any(|key| key.0 == project && key.1 == comp_id)
    }

    pub fn reconcile(&mut self, project: i32, comp_id: i32, comp: &CompHandle, item: &ItemHandle,
        id: PluginId, basic: *const ae::sys::SPBasicSuite, main_thread: std::thread::ThreadId,
        installed: InstalledEffectKey, requests: &Requests) -> Result<bool, ae::Error> {
        if !self.needs_scan(project, comp_id, requests) {
            return Ok(false);
        }
        let _quiet = suites::Utility::new()?.start_quiet_errors(false)?;
        let effects = suites::Effect::new()?;
        let layers = suites::Layer::new()?;
        let items = suites::Item::new()?;
        let masks = suites::Mask::new()?;
        if basic.is_null() { return Err(ae::Error::InvalidCallback); }
        let stage = unsafe { StageAccess::new(&*basic, id, main_thread) }.map_err(failure)?;
        let helper = installed_reader(&effects)?;
        let mut owners = Vec::new();
        let mut readers = Vec::new();
        let count = layers.comp_num_layers(comp)?;
        if count > 4096 { return Err(ae::Error::BadCallbackParameter); }
        for index in 0..count {
            let layer = layers.comp_layer_by_index(comp, index)?;
            let layer_id = layers.layer_id(&layer)?;
            let total = effects.layer_num_effects(&layer)?;
            if total > 256 { return Err(ae::Error::BadCallbackParameter); }
            let mut own = Vec::new();
            let mut reader_binding = None;
            let mut reader_active = false;
            for effect_index in 0..total {
                let effect = Effect { suite: &effects, handle: effects.layer_effect_by_index(&layer, id, effect_index)? };
                let key = effects.installed_key_from_layer_effect(&effect.handle)?;
                if Some(key) == helper {
                    reader_binding = Some(unsafe { stage.read(effect.handle.as_ptr(), 1) }.map_err(failure)?);
                    reader_active = effects.effect_flags(&effect.handle)?.contains(ae::aegp::EffectFlags::ACTIVE);
                } else if key == installed {
                    let request = requests.get(&(layer_id, effect_index)).ok_or(ae::Error::BadCallbackParameter)?;
                    own.push((effect_index, request.enabled, Arc::clone(&request.permit)));
                }
            }
            if let Some(binding) = reader_binding {
                let flags = layers.layer_flags(&layer)?;
                let source = if layers.layer_object_type(&layer)? == ObjectType::AudioVideo {
                    Some(layers.layer_source_item(&layer)?)
                } else { None };
                let marked = source.as_ref().map(|s| items.item_comment(s)).transpose()?.is_some_and(|c| c == SOURCE_MARKER);
                let valid = total == 1 && own.is_empty() && binding.stage == -2 && marked && reader_active &&
                    flags.contains(LayerFlags::EFFECTS_ACTIVE) &&
                    !flags.intersects(LayerFlags::VIDEO_ACTIVE | LayerFlags::LAYER_IS_3D | LayerFlags::ADJUSTMENT_LAYER | LayerFlags::TIME_REMAPPING) &&
                    flags.contains(LayerFlags::LOCKED | LayerFlags::SHY) && masks.layer_num_masks(&layer)? == 0 &&
                    layers.layer_parent(&layer)?.is_none() &&
                    !layers.does_layer_have_track_matte(&layer)? &&
                    source.as_ref().map(|source| reader_identity(&layer, source, item, id)).transpose()?.unwrap_or(false);
                readers.push(Reader { layer, id: layer_id, owner: binding.layer as u32, valid, source });
            } else { owners.push(Owner { layer, id: layer_id, effects: own }); }
        }
        self.forget_missing_owners(project, comp_id, &owners.iter().map(|owner| owner.id).collect());
        let Some(helper) = helper else {
            for owner in &owners { self.mark(owner, id, basic, 0.0)?; }
            return Ok(false);
        };
        for reader in &readers {
            let key = (project, comp_id, reader.id);
            let exists = owners.iter().any(|o| o.id == reader.owner);
            if exists && reader.valid {
                self.known_readers.insert(key, reader.owner);
                self.removed_orphans.remove(&key);
            }
            let known = self.known_readers.get(&key).copied();
            if reader.valid && !exists && known.is_some_and(|previous| !owners.iter().any(|o| o.id == previous)) &&
                !self.removed_orphans.contains(&key) &&
                !foreign_references(comp, known.unwrap(), reader.id, id, installed, helper)? {
                let _undo = Undo::start()?;
                delete_reader(reader)?;
                self.removed_orphans.insert(key);
                return Ok(true);
            }
        }
        for owner in &owners {
            let key = (project, comp_id, owner.id);
            let enabled = owner.effects.iter().any(|(_, value, _)| *value);
            let previous = self.enabled.get(&key).copied().unwrap_or(false);
            if !enabled { self.failed.remove(&key); }
            if enabled && self.failed.contains(&key) { self.mark(owner, id, basic, 0.0)?; continue; }
            if self.enabled.len() >= 8192 && !self.enabled.contains_key(&key) { return Err(ae::Error::BadCallbackParameter); }
            self.enabled.insert(key, enabled);
            let owned: Vec<_> = readers.iter().filter(|reader| reader.owner == owner.id).collect();
            let decision = action(previous, enabled, owned.len(), owned.iter().all(|reader| reader.valid));
            let result = match decision {
                Action::Create => self.create(project, comp_id, comp, item, owner, id, basic, &stage, helper),
                Action::Bind => self.bind_owner(owner, owned[0], id, basic, &stage, false),
                Action::Remove => {
                    self.mark(owner, id, basic, 0.0)?;
                    if foreign_references(comp, owner.id, owned[0].id, id, installed, helper)? { Ok(false) }
                    else {
                        let _undo = Undo::start()?;
                        self.bind_owner(owner, owned[0], id, basic, &stage, true)?;
                        delete_reader(owned[0])?;
                        Ok(true)
                    }
                }
                Action::Missing => self.mark(owner, id, basic, 0.0),
                Action::Conflict => self.mark(owner, id, basic, 3.0),
                Action::None => Ok(false),
            };
            if result.is_err() {
                self.failed.insert(key);
                let _ = self.mark(owner, id, basic, 0.0);
            }
            if result? { return Ok(true); }
        }
        Ok(false)
    }

    fn mark(&self, owner: &Owner, id: PluginId, basic: *const ae::sys::SPBasicSuite, state: f64) -> Result<bool, ae::Error> {
        let _publication = Publication::enter();
        let effects = suites::Effect::new()?;
        let mut changed = false;
        for (index, _, permit) in &owner.effects {
            permit.revoke();
            let effect = Effect { suite: &effects, handle: effects.layer_effect_by_index(&owner.layer, id, *index)? };
            changed |= write_state(basic, id, &effect.handle, state)?;
        }
        Ok(changed)
    }

    fn bind_owner(&self, owner: &Owner, reader: &Reader, id: PluginId, basic: *const ae::sys::SPBasicSuite,
        stage: &StageAccess<'_>, remove: bool) -> Result<bool, ae::Error> {
        let _publication = Publication::enter();
        let effects = suites::Effect::new()?;
        let mut changed = false;
        let mut pending = Vec::new();
        for (index, enabled, permit) in &owner.effects {
            let effect = Effect { suite: &effects, handle: effects.layer_effect_by_index(&owner.layer, id, *index)? };
            let current = unsafe { stage.read(effect.handle.as_ptr(), coverage_index()?) }.map_err(failure)?;
            let release = remove || !enabled;
            let desired = if release { Binding { layer: 0, stage: 0 } } else { Binding { layer: reader.id as i32, stage: -1 } };
            if current != desired && (!release || current.layer == reader.id as i32) {
                permit.revoke();
                changed |= write_state(basic, id, &effect.handle, 0.0)?;
                pending.push((*index, desired));
            }
        }
        if !pending.is_empty() {
            let _undo = Undo::start()?;
            for (index, desired) in pending {
                let effect = Effect { suite: &effects, handle: effects.layer_effect_by_index(&owner.layer, id, index)? };
                changed |= unsafe { stage.set(effect.handle.as_ptr(), coverage_index()?, desired.layer,
                    if desired.layer == 0 { Stage::Source } else { Stage::Effects }) }.map_err(failure)?;
            }
        }
        for (index, enabled, permit) in &owner.effects {
            let effect = Effect { suite: &effects, handle: effects.layer_effect_by_index(&owner.layer, id, *index)? };
            let ready = *enabled && !remove;
            if !ready { permit.revoke(); }
            changed |= write_state(basic, id, &effect.handle, if ready { permit.grant() as f64 } else { 0.0 })?;
        }
        Ok(changed)
    }

    fn create(&mut self, project: i32, comp_id: i32, comp: &CompHandle, item: &ItemHandle, owner: &Owner,
        id: PluginId, basic: *const ae::sys::SPBasicSuite, stage: &StageAccess<'_>, helper: InstalledEffectKey) -> Result<bool, ae::Error> {
        self.mark(owner, id, basic, 0.0)?;
        let _undo = Undo::start()?;
        let layers = suites::Layer::new()?;
        let effects = suites::Effect::new()?;
        let items = suites::Item::new()?;
        let (width, height) = items.item_dimensions(item)?;
        let layer = suites::Comp::new()?.create_solid_in_comp(comp, "DynamicFX internal coverage reader",
            width as i32, height as i32, ae::sys::AEGP_ColorVal { alphaF: 1.0, redF: 0.0, greenF: 0.0, blueF: 0.0 },
            Some(items.item_duration(item)?))?;
        let source = layers.layer_source_item(&layer)?;
        let result = (|| {
            items.set_item_comment(&source, SOURCE_MARKER)?;
            layers.set_layer_flag(&layer, LayerFlags::VIDEO_ACTIVE, false)?;
            layers.set_layer_flag(&layer, LayerFlags::SHY, true)?;
            {
                let effect = Effect { suite: &effects, handle: effects.apply_effect(&layer, id, helper)? };
                unsafe { stage.set(effect.handle.as_ptr(), 1, owner.id as i32, Stage::Masks) }.map_err(failure)?;
            }
            layers.set_layer_flag(&layer, LayerFlags::LOCKED, true)?;
            let reader = Reader { layer, id: layers.layer_id(&layer)?, owner: owner.id, valid: true, source: Some(source) };
            self.bind_owner(owner, &reader, id, basic, stage, false)?;
            self.known_readers.insert((project, comp_id, reader.id), owner.id);
            crate::diag::log(&format!("coverage reader created: owner={} reader={}", owner.id, reader.id));
            Ok(true)
        })();
        if result.is_err() {
            let _ = layers.set_layer_flag(&layer, LayerFlags::LOCKED, false);
            if layers.delete_layer(&layer).is_ok() { let _ = items.delete_item(&source); }
        }
        result
    }
}

fn delete_reader(reader: &Reader) -> Result<(), ae::Error> {
    let items = suites::Item::new()?;
    let layers = suites::Layer::new()?;
    let source = reader.source.as_ref().ok_or(ae::Error::BadCallbackParameter)?;
    let remove_source = source_references(items.item_id(source)?)? == 1;
    layers.set_layer_flag(&reader.layer, LayerFlags::LOCKED, false)?;
    layers.delete_layer(&reader.layer)?;
    if remove_source { items.delete_item(source)?; }
    Ok(())
}

fn foreign_references(comp: &CompHandle, owner: u32, reader: u32, id: PluginId,
    installed: InstalledEffectKey, helper: InstalledEffectKey) -> Result<bool, ae::Error> {
    let layers = suites::Layer::new()?;
    let effects = suites::Effect::new()?;
    let streams = suites::Stream::new()?;
    let mut budget = 200000usize;
    for index in 0..layers.comp_num_layers(comp)? {
        let layer = layers.comp_layer_by_index(comp, index)?;
        for reference in [layers.layer_parent(&layer)?, layers.track_matte_layer(&layer)?].into_iter().flatten() {
            if layers.layer_id(&reference)? == reader { return Ok(true); }
        }
        for index in 0..effects.layer_num_effects(&layer)? {
            let effect = Effect { suite: &effects, handle: effects.layer_effect_by_index(&layer, id, index)? };
            let key = effects.installed_key_from_layer_effect(&effect.handle)?;
            let owner_effect = layers.layer_id(&layer)? == owner && key == installed;
            let internal_reader = layers.layer_id(&layer)? == reader && key == helper;
            for index in 1..streams.effect_num_param_streams(&effect.handle)? {
                budget = budget.checked_sub(1).ok_or(ae::Error::BadCallbackParameter)?;
                if (owner_effect && index == coverage_index()?) || (internal_reader && index == 1) { continue; }
                let stream = streams.new_effect_stream_by_index(&effect.handle, id, index)?;
                if streams.stream_type(&stream)? == ae::aegp::StreamType::LayerId &&
                    matches!(streams.new_stream_value(&stream, id, TimeMode::LayerTime,
                        ae::Time { value: 0, scale: 1 }, true)?, StreamValue::LayerId(value) if value as u32 == reader) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn source_references(source: i32) -> Result<usize, ae::Error> {
    let projects = suites::Project::new()?;
    let items = suites::Item::new()?;
    let comps = suites::Comp::new()?;
    let layers = suites::Layer::new()?;
    let mut count = 0;
    let mut budget = 16384usize;
    for p in 0..projects.num_projects()? {
        let project = projects.project_by_index(p)?;
        let mut current = Some(items.first_proj_item(&project)?);
        while let Some(item) = current {
            current = items.next_proj_item(&project, &item)?;
            budget = budget.checked_sub(1).ok_or(ae::Error::BadCallbackParameter)?;
            if items.item_type(&item)? != ae::aegp::ItemType::Comp { continue; }
            let Some(comp) = comps.comp_from_item(&item)? else { continue };
            for index in 0..layers.comp_num_layers(&comp)? {
                budget = budget.checked_sub(1).ok_or(ae::Error::BadCallbackParameter)?;
                let layer = layers.comp_layer_by_index(&comp, index)?;
                if layers.layer_object_type(&layer)? == ObjectType::AudioVideo &&
                    items.item_id(layers.layer_source_item(&layer)?)? == source { count += 1; }
            }
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reader_transform_identity_rejects_offsets_scale_opacity_and_nonfinite_values() {
        assert!(identity_value(LayerStream::Position, StreamValue::ThreeDSpatial { x: 400.0, y: 300.0, z: 0.0 }, 800, 600));
        assert!(!identity_value(LayerStream::Position, StreamValue::TwoDSpatial { x: 435.0, y: 300.0 }, 800, 600));
        assert!(!identity_value(LayerStream::Scale, StreamValue::ThreeD { x: 100.0, y: 90.0, z: 100.0 }, 800, 600));
        assert!(!identity_value(LayerStream::Opacity, StreamValue::OneD(99.0), 800, 600));
        assert!(!identity_value(LayerStream::RotateZ, StreamValue::OneD(f64::NAN), 800, 600));
        assert!(identity_value(LayerStream::AnchorPoint, StreamValue::TwoDSpatial { x: 400.5, y: 300.5 }, 801, 601));
    }
    #[test]
    fn undo_does_not_recreate_a_reader_until_opt_in_changes() {
        assert_eq!(action(false, true, 0, true), Action::Create);
        assert_eq!(action(true, true, 0, true), Action::Missing);
        assert_eq!(action(true, false, 0, true), Action::None);
        assert_eq!(action(false, false, 1, true), Action::None);
    }
    #[test]
    fn duplicates_and_modified_readers_are_conflicts() {
        assert_eq!(action(true, false, 2, true), Action::Conflict);
        assert_eq!(action(true, false, 1, false), Action::Conflict);
        assert_eq!(action(false, true, 1, true), Action::Bind);
        assert_eq!(action(true, false, 1, true), Action::Remove);
    }

    #[test]
    fn observed_absence_clears_reused_layer_ids_without_defeating_undo_on_existing_owners() {
        let mut state = State::default();
        state.enabled.insert((0, 30, 44), true);
        state.enabled.insert((0, 30, 47), true);
        state.enabled.insert((0, 1, 13), true);
        state.failed.insert((0, 30, 47));
        state.forget_missing_owners(0, 30, &HashSet::from([44]));
        assert!(!state.enabled.contains_key(&(0, 30, 47)));
        assert!(!state.failed.contains(&(0, 30, 47)));
        assert!(state.enabled[&(0, 30, 44)]);
        assert!(state.enabled[&(0, 1, 13)]);
        assert_eq!(action(state.enabled.get(&(0, 30, 47)).copied().unwrap_or(false), true, 0, true), Action::Create);
        assert_eq!(action(state.enabled[&(0, 30, 44)], true, 0, true), Action::Missing);
    }
}
