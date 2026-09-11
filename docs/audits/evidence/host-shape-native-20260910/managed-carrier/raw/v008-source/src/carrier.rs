use after_effects as ae;
use std::collections::{HashMap, HashSet};

const EXPRESSION: &str = concat!(
    "// DynamicFX internal coverage raster carrier; schema=1\n",
    include_str!("../../coverage-blocks-expression.js")
);
type Key = (i32, i32, u32);

#[derive(Default)]
pub struct State {
    enabled: HashMap<Key, bool>,
    results: HashMap<Key, String>,
    failed: HashSet<Key>,
}

#[derive(Debug, PartialEq)]
enum Action { Skip, Create, Reuse, Remove, Missing, Conflict }

fn action(previous: bool, enabled: bool, owned: usize, valid: bool) -> Action {
    if owned > 1 || (owned == 1 && !valid) { return Action::Conflict; }
    match (enabled, owned) {
        (false, 0) => Action::Skip,
        (false, _) => Action::Remove,
        (true, 1) => Action::Reuse,
        (true, 0) if previous => Action::Missing,
        _ => Action::Create,
    }
}

struct Undo(ae::aegp::suites::Utility);

fn phase<T>(name: &str, result: Result<T, ae::Error>) -> Result<T, ae::Error> {
    result.map_err(|error| {
        super::log(&format!("CARRIER_OPERATION_FAILED {name} {error:?}"));
        error
    })
}
impl Undo {
    fn start() -> Result<Self, ae::Error> {
        let utility = ae::aegp::suites::Utility::new()?;
        utility.start_undo_group("Manage DynamicFX coverage data")?;
        Ok(Self(utility))
    }
}
impl Drop for Undo {
    fn drop(&mut self) {
        if let Err(error) = self.0.end_undo_group() {
            super::log(&format!("CARRIER_UNDO_CLEANUP_FAILED {error:?}"));
        }
    }
}

impl State {
    pub fn reconcile(&mut self, key: Key, id: ae::aegp::PluginId,
        basic: *mut ae::sys::SPBasicSuite, layer: &ae::aegp::LayerHandle, enabled: bool) -> Result<(), ae::Error>
    {
        let previous = self.enabled.get(&key).copied().unwrap_or(false);
        if !previous && !enabled { return Ok(()); }
        if enabled && self.failed.contains(&key) { return Ok(()); }
        if !enabled { self.failed.remove(&key); }
        if !self.enabled.contains_key(&key) && self.enabled.len() >= 8192 {
            return Err(ae::Error::BadCallbackParameter);
        }
        // Remember the attempt before entering host code so Undo is not auto-replayed.
        self.enabled.insert(key, enabled);
        let result = self.apply(id, basic, layer, previous, enabled);
        if result.is_err() { self.failed.insert(key); }
        let report = match &result {
            Ok(text) => text.clone(),
            Err(error) => format!("error={error:?}"),
        };
        if self.results.get(&key) != Some(&report) {
            super::log(&format!("CARRIER comp={} layer={} {report}", key.1, key.2));
            self.results.insert(key, report);
        }
        result.map(|_| ())
    }

    fn apply(&self, id: ae::aegp::PluginId, basic: *mut ae::sys::SPBasicSuite, layer: &ae::aegp::LayerHandle,
        previous: bool, enabled: bool) -> Result<String, ae::Error>
    {
        let _quiet = ae::aegp::suites::Utility::new()?.start_quiet_errors(false)?;
        let masks = ae::aegp::suites::Mask::new()?;
        let streams = ae::aegp::suites::Stream::new()?;
        let count = masks.layer_num_masks(layer)?;
        if !(0..=256).contains(&count) { return Err(ae::Error::BadCallbackParameter); }
        let mut owned = Vec::new();
        let mut valid = true;
        for index in 0..count {
            let mask = masks.layer_mask_by_index(layer, index)?;
            let outline = streams.new_mask_stream(&mask, id, ae::aegp::MaskStream::Outline)?;
            let expression_enabled = streams.expression_state(&outline, id)?;
            let expression = unsafe { super::read::expression_text(basic, id, outline.as_ptr()) }
                .map_err(|error| {
                    super::log(&format!("CARRIER_EXPRESSION_READ_FAILED {error}"));
                    ae::Error::InvalidCallback
                })?;
            if expression.replace("\r\n", "\n") == EXPRESSION.replace("\r\n", "\n") {
                valid &= masks.mask_mode(&mask)? == ae::aegp::MaskMode::None &&
                    expression_enabled;
                owned.push(mask);
            }
        }
        let decision = action(previous, enabled, owned.len(), valid);
        match decision {
            Action::Create => {
                let _undo = Undo::start()?;
                let (mask, _) = phase("create_mask", masks.create_new_mask(layer))?;
                let result = (|| {
                    phase("set_mode", masks.set_mask_mode(&mask, ae::aegp::MaskMode::None))?;
                    let dynamic = ae::aegp::suites::DynamicStream::new()?;
                    let root = phase("mask_root", dynamic.new_stream_ref_for_mask(&mask, id))?;
                    phase("rename", dynamic.set_stream_name(&root, "DynamicFX Coverage Data"))?;
                    let outline = phase("mask_outline", streams.new_mask_stream(&mask, id, ae::aegp::MaskStream::Outline))?;
                    phase("set_expression", streams.set_expression_string(&outline, id, EXPRESSION))?;
                    phase("enable_expression", streams.set_expression_state(&outline, id, true))?;
                    Ok::<_, ae::Error>(format!("created mask_id={}", masks.mask_id(&mask)?))
                })();
                if result.is_err() {
                    if let Err(error) = masks.delete_mask_from_layer(&mask) {
                        super::log(&format!("CARRIER_ROLLBACK_FAILED {error:?}"));
                    }
                }
                result
            }
            Action::Remove => {
                let _undo = Undo::start()?;
                masks.delete_mask_from_layer(&owned[0])?;
                Ok("removed".into())
            }
            Action::Reuse => Ok(format!("reused mask_id={}", masks.mask_id(&owned[0])?)),
            Action::Missing => Ok("missing; explicit re-enable required after Undo or deletion".into()),
            Action::Conflict => Ok("conflict; carrier is duplicated, disabled, or has a changed mask mode".into()),
            Action::Skip => Ok("inactive".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creation_is_not_replayed_after_undo() {
        assert_eq!(action(false, true, 0, true), Action::Create);
        assert_eq!(action(true, true, 0, true), Action::Missing);
        assert_eq!(action(false, true, 0, true), Action::Create);
    }
    #[test]
    fn multiple_effects_share_one_valid_carrier() {
        assert_eq!(action(false, true, 1, true), Action::Reuse);
        assert_eq!(action(true, true, 1, true), Action::Reuse);
    }
    #[test]
    fn removal_requires_no_remaining_owner_and_valid_data() {
        assert_eq!(action(true, false, 1, true), Action::Remove);
        assert_eq!(action(true, false, 1, false), Action::Conflict);
        assert_eq!(action(true, true, 2, true), Action::Conflict);
    }
}
