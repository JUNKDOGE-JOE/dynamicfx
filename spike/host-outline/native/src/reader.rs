use after_effects::{self as ae, AsPtr};
use ae::aegp::{suites, CompHandle, EffectRefHandle, InstalledEffectKey, ItemHandle, LayerFlags,
    LayerHandle, ObjectType, PluginId, StreamValue, TimeMode};
use std::collections::{HashMap, HashSet};

const SOURCE_MARKER: &str = "DynamicFX native coverage reader source; diagnostic schema=1";
type Key = (i32, i32, u32);

#[derive(Default)]
pub struct State {
    enabled: HashMap<Key, bool>,
    failed: HashSet<Key>,
    reports: HashMap<Key, String>,
    reader_owners: HashMap<Key, u32>,
    removed_orphans: HashSet<Key>,
}

#[derive(Debug, PartialEq)]
enum Action { None, Create, Bind, Remove, Missing, Conflict }

fn action(previous: bool, enabled: bool, readers: usize, valid: bool) -> Action {
    if readers > 1 || (readers == 1 && !valid) { return Action::Conflict; }
    if !previous && !enabled { return Action::None; }
    match (enabled, readers, previous) {
        (true, 0, false) => Action::Create,
        (true, 0, true) => Action::Missing,
        (true, 1, _) => Action::Bind,
        (false, 1, _) => Action::Remove,
        _ => Action::None,
    }
}

fn remove_orphan(owner_exists: bool, known: bool, removed: bool) -> bool {
    !owner_exists && known && !removed
}

struct Effect<'a> { suite: &'a suites::Effect, handle: EffectRefHandle }
impl Drop for Effect<'_> {
    fn drop(&mut self) {
        if let Err(error) = self.suite.dispose_effect(&self.handle) {
            super::log(&format!("READER_EFFECT_DISPOSE_FAILED {error:?}"));
        }
    }
}
struct Undo(suites::Utility);
impl Undo {
    fn start() -> Result<Self, ae::Error> {
        let suite = suites::Utility::new()?;
        suite.start_undo_group("Manage DynamicFX native coverage reader")?;
        Ok(Self(suite))
    }
}
impl Drop for Undo {
    fn drop(&mut self) {
        if let Err(error) = self.0.end_undo_group() { super::log(&format!("READER_UNDO_FAILED {error:?}")); }
    }
}

fn value(streams: &suites::Stream, id: PluginId, effect: &EffectRefHandle, index: i32) -> Result<StreamValue, ae::Error> {
    let stream = streams.new_effect_stream_by_index(effect, id, index)?;
    streams.new_stream_value(&stream, id, TimeMode::LayerTime, ae::Time { value: 0, scale: 1 }, true)
}
fn number(streams: &suites::Stream, id: PluginId, effect: &EffectRefHandle, index: i32) -> Result<f64, ae::Error> {
    match value(streams, id, effect, index)? {
        StreamValue::OneD(v) if v.is_finite() => Ok(v),
        _ => Err(ae::Error::BadCallbackParameter),
    }
}
fn layer_id(streams: &suites::Stream, id: PluginId, effect: &EffectRefHandle, index: i32) -> Result<u32, ae::Error> {
    match value(streams, id, effect, index)? {
        StreamValue::LayerId(v) => Ok(v as u32),
        _ => Err(ae::Error::BadCallbackParameter),
    }
}
fn write_number(basic: *mut ae::sys::SPBasicSuite, streams: &suites::Stream, id: PluginId,
    effect: &EffectRefHandle, index: i32, number: f64) -> Result<(), ae::Error> {
    let stream = streams.new_effect_stream_by_index(effect, id, index)?;
    unsafe { super::read::set_primitive(basic, id, stream.as_ptr(), StreamValue::OneD(number)) }
        .map_err(failure)
}
fn failure(error: String) -> ae::Error {
    super::log(&format!("READER_OPERATION_FAILED {error}"));
    ae::Error::Generic
}
fn bind(basic: *mut ae::sys::SPBasicSuite, id: PluginId, effect: &EffectRefHandle,
    index: i32, layer: u32, stage: i32) -> Result<bool, ae::Error> {
    unsafe { super::stage::bind(basic, id, effect.as_ptr(), index, layer as i32, stage, true) }.map_err(failure)
}

struct Owner { layer: LayerHandle, id: u32, effects: Vec<(i32, bool, u32)> }
struct Reader { layer: LayerHandle, id: u32, owner: u32, valid: bool, source: Option<ItemHandle> }

impl State {
    pub fn reconcile(&mut self, project: i32, comp_id: i32, comp: &CompHandle, item: &ItemHandle,
        id: PluginId, basic: *mut ae::sys::SPBasicSuite, installed: InstalledEffectKey) -> Result<bool, ae::Error>
    {
        let _quiet = suites::Utility::new()?.start_quiet_errors(false)?;
        let layers = suites::Layer::new()?;
        let effects = suites::Effect::new()?;
        let streams = suites::Stream::new()?;
        let items = suites::Item::new()?;
        let masks = suites::Mask::new()?;
        let mut owners = Vec::new();
        let mut readers = Vec::new();
        let count = layers.comp_num_layers(comp)?;
        if count > 4096 { return Err(ae::Error::BadCallbackParameter); }
        for index in 0..count {
            let layer = layers.comp_layer_by_index(comp, index)?;
            let layer_id_value = layers.layer_id(&layer)?;
            let total = effects.layer_num_effects(&layer)?;
            if total > 256 { return Err(ae::Error::BadCallbackParameter); }
            let mut own = Vec::new();
            let mut reader_owner = None;
            let mut reader_mode = false;
            for index in 0..total {
                let effect = Effect { suite: &effects, handle: effects.layer_effect_by_index(&layer, id, index)? };
                if effects.installed_key_from_layer_effect(&effect.handle)? != installed { continue; }
                if number(&streams, id, &effect.handle, 11)? != 0.0 {
                    reader_owner = Some(layer_id(&streams, id, &effect.handle, 6)?);
                    reader_mode = number(&streams, id, &effect.handle, 2)? == 22.0 &&
                        number(&streams, id, &effect.handle, 10)? == 0.0;
                } else {
                    own.push((index, number(&streams, id, &effect.handle, 10)? != 0.0,
                        layer_id(&streams, id, &effect.handle, 9)?));
                }
            }
            if let Some(owner) = reader_owner {
                let flags = layers.layer_flags(&layer)?;
                let source = if layers.layer_object_type(&layer)? == ObjectType::AudioVideo {
                    Some(layers.layer_source_item(&layer)?)
                } else { None };
                let marked = source.as_ref().map(|s| items.item_comment(s)).transpose()?.is_some_and(|c| c == SOURCE_MARKER);
                let valid = total == 1 && own.is_empty() && reader_mode && marked &&
                    !flags.contains(LayerFlags::VIDEO_ACTIVE) && flags.contains(LayerFlags::LOCKED | LayerFlags::SHY) &&
                    masks.layer_num_masks(&layer)? == 0 && layers.layer_parent(&layer)?.is_none();
                readers.push(Reader { layer, id: layer_id_value, owner, valid, source });
            } else {
                owners.push(Owner { layer, id: layer_id_value, effects: own });
            }
        }
        for reader in &readers {
            let key = (project, comp_id, reader.id);
            let owner_exists = owners.iter().any(|owner| owner.id == reader.owner);
            if owner_exists && reader.valid {
                if self.reader_owners.len() >= 8192 && !self.reader_owners.contains_key(&key) {
                    return Err(ae::Error::BadCallbackParameter);
                }
                self.reader_owners.insert(key, reader.owner);
                self.removed_orphans.remove(&key);
            }
            let previous_owner_exists = self.reader_owners.get(&key)
                .is_some_and(|id| owners.iter().any(|owner| owner.id == *id));
            if reader.valid && remove_orphan(owner_exists || previous_owner_exists, self.reader_owners.contains_key(&key),
                self.removed_orphans.contains(&key)) {
                let previous_owner = self.reader_owners[&key];
                if foreign_references(comp, previous_owner, reader.id, id, installed)? { continue; }
                let _undo = Undo::start()?;
                let source = reader.source.as_ref().ok_or(ae::Error::BadCallbackParameter)?;
                let remove_source = source_references(items.item_id(source)?)? == 1;
                layers.set_layer_flag(&reader.layer, LayerFlags::LOCKED, false)?;
                layers.delete_layer(&reader.layer)?;
                if remove_source { items.delete_item(source)?; }
                self.removed_orphans.insert(key);
                super::log(&format!("READER comp={comp_id} owner={previous_owner} removed orphan={} source_removed={remove_source}", reader.id));
                return Ok(true);
            }
        }
        for owner in &owners {
            let key = (project, comp_id, owner.id);
            let enabled = owner.effects.iter().any(|(_, enabled, _)| *enabled);
            let previous = self.enabled.get(&key).copied().unwrap_or(false);
            let owned: Vec<_> = readers.iter().filter(|reader| reader.owner == owner.id).collect();
            if !enabled { self.failed.remove(&key); }
            if enabled && self.failed.contains(&key) { continue; }
            if self.enabled.len() >= 8192 && !self.enabled.contains_key(&key) { return Err(ae::Error::BadCallbackParameter); }
            self.enabled.insert(key, enabled);
            let decision = action(previous, enabled, owned.len(), owned.iter().all(|r| r.valid));
            let result = match decision {
                Action::Create => self.create(project, comp_id, comp, item, owner, id, basic, installed),
                Action::Bind => self.bind_owner(owner, owned[0], id, basic, false),
                Action::Remove => {
                    if foreign_references(comp, owner.id, owned[0].id, id, installed)? {
                        Ok((false, "in use by an external effect; preserved".into()))
                    } else {
                        let _undo = Undo::start()?;
                        self.bind_owner(owner, owned[0], id, basic, true)?;
                        let source = owned[0].source.as_ref().ok_or(ae::Error::BadCallbackParameter)?;
                        let references = source_references(items.item_id(source)?)?;
                        let remove_source = references == 1;
                        layers.set_layer_flag(&owned[0].layer, LayerFlags::LOCKED, false)?;
                        layers.delete_layer(&owned[0].layer)?;
                        if remove_source { items.delete_item(source)?; }
                        Ok((true, format!("removed reader={} source_references={references} source_removed={remove_source}", owned[0].id)))
                    }
                }
                Action::Missing => Ok((false, "missing after Undo or deletion; re-enable required".into())),
                Action::Conflict => Ok((false, "ownership conflict; preserved".into())),
                Action::None => Ok((false, "inactive".into())),
            };
            if result.is_err() { self.failed.insert(key); }
            let report = match &result { Ok((_, report)) => report.clone(), Err(e) => format!("error={e:?}") };
            if self.reports.get(&key) != Some(&report) {
                super::log(&format!("READER comp={comp_id} owner={} {report}", owner.id));
                self.reports.insert(key, report);
            }
            if result?.0 { return Ok(true); }
        }
        Ok(false)
    }

    fn bind_owner(&self, owner: &Owner, reader: &Reader, id: PluginId,
        basic: *mut ae::sys::SPBasicSuite, remove: bool) -> Result<(bool, String), ae::Error> {
        let effects = suites::Effect::new()?;
        let mut pending = Vec::new();
        for (index, enabled, linked) in &owner.effects {
            let release = remove || !enabled;
            if release && *linked != reader.id { continue; }
            let effect = Effect { suite: &effects, handle: effects.layer_effect_by_index(&owner.layer, id, *index)? };
            let (target, stage) = if release { (0, 0) } else { (reader.id, -1) };
            if unsafe { super::stage::bind(basic, id, effect.handle.as_ptr(), 9, target as i32, stage, false) }.map_err(failure)? {
                pending.push((*index, target, stage));
            }
        }
        let changed = !pending.is_empty();
        if changed {
            let _undo = Undo::start()?;
            for (index, target, stage) in pending {
                let effect = Effect { suite: &effects, handle: effects.layer_effect_by_index(&owner.layer, id, index)? };
                bind(basic, id, &effect.handle, 9, target, stage)?;
            }
        }
        Ok((changed, format!("bound reader={} users={}", reader.id, owner.effects.iter().filter(|(_, e, _)| *e).count())))
    }

    fn create(&mut self, project: i32, comp_id: i32, comp: &CompHandle, item: &ItemHandle, owner: &Owner, id: PluginId,
        basic: *mut ae::sys::SPBasicSuite, installed: InstalledEffectKey) -> Result<(bool, String), ae::Error> {
        let _undo = Undo::start()?;
        let layers = suites::Layer::new()?;
        let effects = suites::Effect::new()?;
        let streams = suites::Stream::new()?;
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
                let effect = Effect { suite: &effects, handle: effects.apply_effect(&layer, id, installed)? };
                write_number(basic, &streams, id, &effect.handle, 2, 22.0)?;
                write_number(basic, &streams, id, &effect.handle, 11, 1.0)?;
                bind(basic, id, &effect.handle, 6, owner.id, -2)?;
                bind(basic, id, &effect.handle, 9, 0, 0)?;
            }
            layers.set_layer_flag(&layer, LayerFlags::LOCKED, true)?;
            let reader = Reader { id: layers.layer_id(&layer)?, layer, owner: owner.id, valid: true, source: Some(source) };
            self.bind_owner(owner, &reader, id, basic, false)?;
            self.reader_owners.insert((project, comp_id, reader.id), owner.id);
            Ok((true, format!("created reader={}", reader.id)))
        })();
        if result.is_err() {
            let _ = layers.set_layer_flag(&layer, LayerFlags::LOCKED, false);
            if layers.delete_layer(&layer).is_ok() { let _ = items.delete_item(&source); }
        }
        result
    }
}

fn foreign_references(comp: &CompHandle, owner: u32, reader: u32, id: PluginId,
    installed: InstalledEffectKey) -> Result<bool, ae::Error> {
    let layers = suites::Layer::new()?;
    let effects = suites::Effect::new()?;
    let streams = suites::Stream::new()?;
    let mut budget = 200000usize;
    for index in 0..layers.comp_num_layers(comp)? {
        let layer = layers.comp_layer_by_index(comp, index)?;
        for index in 0..effects.layer_num_effects(&layer)? {
            let effect = Effect { suite: &effects, handle: effects.layer_effect_by_index(&layer, id, index)? };
            let is_probe = effects.installed_key_from_layer_effect(&effect.handle)? == installed;
            let managed_owner = layers.layer_id(&layer)? == owner && is_probe;
            let internal_reader = layers.layer_id(&layer)? == reader && is_probe &&
                number(&streams, id, &effect.handle, 11)? != 0.0;
            for index in 1..streams.effect_num_param_streams(&effect.handle)? {
                budget = budget.checked_sub(1).ok_or(ae::Error::BadCallbackParameter)?;
                if managed_owner && index == 9 { continue; }
                if internal_reader && index == 6 { continue; }
                let stream = streams.new_effect_stream_by_index(&effect.handle, id, index)?;
                if streams.stream_type(&stream)? == ae::aegp::StreamType::LayerId &&
                    matches!(streams.new_stream_value(&stream, id, TimeMode::LayerTime,
                        ae::Time { value: 0, scale: 1 }, true)?, StreamValue::LayerId(v) if v as u32 == reader) {
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
                if layers.layer_object_type(&layer)? == ObjectType::AudioVideo {
                    let item = layers.layer_source_item(&layer)?;
                    if items.item_id(&item)? == source { count += 1; }
                }
            }
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn undo_does_not_recreate_a_reader_until_opt_in_changes() {
        assert_eq!(action(false, true, 0, true), Action::Create);
        assert_eq!(action(true, true, 0, true), Action::Missing);
        assert_eq!(action(true, false, 0, true), Action::None);
        assert_eq!(action(false, false, 1, true), Action::None);
    }
    #[test]
    fn duplication_and_modified_ownership_are_never_deleted() {
        assert_eq!(action(true, false, 2, true), Action::Conflict);
        assert_eq!(action(true, false, 1, false), Action::Conflict);
        assert_eq!(action(false, true, 1, true), Action::Bind);
        assert_eq!(action(true, false, 1, true), Action::Remove);
    }
    #[test]
    fn orphan_cleanup_requires_prior_ownership_and_does_not_fight_undo() {
        assert!(remove_orphan(false, true, false));
        assert!(!remove_orphan(true, true, false));
        assert!(!remove_orphan(false, false, false));
        assert!(!remove_orphan(false, true, true));
    }
}
