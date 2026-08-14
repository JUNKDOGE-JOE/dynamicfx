//! The ADR-0013 AE parameter topology: fixed head parameters plus the 104
//! pool slots, declared in one deterministic order derived from
//! `binding::V1_POOLS`.
//!
//! AE matches effect parameter streams by declaration order across project
//! loads, so `declaration_order()` is a persistent contract: a released
//! index never changes kind, moves, or dies; all growth appends at the tail
//! (ADR-0013 §5).

use crate::binding::{PoolKind, V1_POOLS};
use crate::frontend;
use after_effects as ae;

/// Parameter identity for the AE dispatch macro. `Pool(kind, i)` is the
/// kind-local slot `i` of one pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParamKey {
    /// Non-time-varying language selector (ADR-0010).
    Language,
    /// Float slider whose committed expression carries the source (ADR-0001).
    Source,
    /// Button: explicit re-observe/compile request.
    Compile,
    /// Read-only style status line, renamed with the current diagnostic text.
    Status,
    /// Hidden primitive reserved for the UI→render StateToken. The fixed
    /// topology declares it now; its value stays 0 until the M3 StateToken
    /// ADR fixes the layout — interim encodings must not persist (ADR-0009).
    StateToken,
    Pool(PoolKind, usize),
    /// ADR-0028: appended after every pool slot (append-only growth per
    /// ADR-0013) — a button that pops the full, untruncated status text
    /// (the Status name is capped at 31 chars by PF).
    Details,
}

/// Head parameters in declaration order, after AE's implicit input layer.
const HEAD: [ParamKey; 5] = [
    ParamKey::Language,
    ParamKey::Source,
    ParamKey::Compile,
    ParamKey::Status,
    ParamKey::StateToken,
];

/// AEGP effect stream indexes (the implicit input layer occupies 0, so a
/// declared parameter's stream index is its declaration position + 1).
pub const LANGUAGE_STREAM_INDEX: i32 = 1;
pub const SOURCE_STREAM_INDEX: i32 = 2;
pub const STATE_TOKEN_STREAM_INDEX: i32 = 5;

/// The complete declaration order — the persistent index contract.
pub fn declaration_order() -> Vec<ParamKey> {
    let mut order: Vec<ParamKey> = HEAD.to_vec();
    for (kind, capacity) in V1_POOLS {
        for i in 0..*capacity {
            order.push(ParamKey::Pool(*kind, i));
        }
    }
    // ADR-0028 append-only growth: Details rides after every pool slot so
    // all 0.0.1 indexes stay stable.
    order.push(ParamKey::Details);
    order
}

/// Placeholder pool label; real names arrive when a definition binds a slot.
fn kind_label(kind: PoolKind) -> &'static str {
    match kind {
        PoolKind::Float => "Float",
        PoolKind::Integer => "Int",
        PoolKind::Bool => "Bool",
        PoolKind::Color => "Color",
        PoolKind::Point2D => "Point",
        PoolKind::Angle => "Angle",
    }
}

/// Default (unbound) display name of a pool slot — the single source for
/// PARAMS_SETUP and for restoring a slot's label when a binding goes away.
pub fn default_slot_name(kind: PoolKind, index: usize) -> String {
    format!("{} {:02}", kind_label(kind), index + 1)
}

/// AEGP effect stream index of a declared parameter (declaration position
/// + 1; the implicit input layer occupies stream 0).
pub fn stream_index_of(key: ParamKey) -> Option<i32> {
    declaration_order()
        .iter()
        .position(|k| *k == key)
        .map(|position| position as i32 + 1)
}

/// Declare the full topology. Iterates `declaration_order()` so the declared
/// indexes and the contract remain one source.
pub fn setup(params: &mut ae::Parameters<ParamKey>) -> Result<(), ae::Error> {
    for key in declaration_order() {
        match key {
            ParamKey::Language => {
                let menu = frontend::popup_menu();
                params.add_with_flags(
                    key,
                    "Language",
                    ae::PopupDef::setup(|p| {
                        p.set_options(&menu);
                        p.set_default(1);
                    }),
                    ae::ParamFlag::CANNOT_TIME_VARY,
                    ae::ParamUIFlags::empty(),
                )?;
            }
            ParamKey::Source => {
                params.add(
                    key,
                    "Source (use expression)",
                    ae::FloatSliderDef::setup(|f| {
                        f.set_slider_min(0.0);
                        f.set_slider_max(1.0);
                        f.set_valid_min(0.0);
                        f.set_valid_max(1.0);
                        f.set_default(0.0);
                    }),
                )?;
            }
            ParamKey::Compile => {
                params.add(
                    key,
                    "Compile",
                    ae::ButtonDef::setup(|b| {
                        b.set_label("Compile");
                    }),
                )?;
            }
            ParamKey::Status => {
                // Collapsed: the row is a text carrier (the name IS the
                // status); its expanded slider widget means nothing.
                params.add_with_flags(
                    key,
                    "Status: idle",
                    ae::FloatSliderDef::setup(|f| {
                        f.set_slider_min(0.0);
                        f.set_slider_max(1.0);
                        f.set_valid_min(0.0);
                        f.set_valid_max(1.0);
                        f.set_default(0.0);
                    }),
                    ae::ParamFlag::START_COLLAPSED,
                    ae::ParamUIFlags::empty(),
                )?;
            }
            ParamKey::StateToken => {
                params.add_with_flags(
                    key,
                    "State Token (internal)",
                    ae::FloatSliderDef::setup(|f| {
                        f.set_slider_min(0.0);
                        f.set_slider_max(1.0);
                        f.set_valid_min(0.0);
                        // Room for an exactly representable f64 integer word;
                        // the M3 ADR fixes the actual token layout.
                        f.set_valid_max(9_007_199_254_740_992.0);
                        f.set_precision(ae::Precision::Integer);
                        f.set_default(0.0);
                    }),
                    ae::ParamFlag::CANNOT_TIME_VARY,
                    ae::ParamUIFlags::INVISIBLE,
                )?;
            }
            ParamKey::Details => {
                params.add(
                    key,
                    "Details",
                    ae::ButtonDef::setup(|b| {
                        b.set_label("Show Full Status");
                    }),
                )?;
            }
            ParamKey::Pool(kind, i) => {
                let name = default_slot_name(kind, i);
                match kind {
                    PoolKind::Float => params.add_with_flags(
                        key,
                        &name,
                        ae::FloatSliderDef::setup(|f| {
                            f.set_slider_min(0.0);
                            f.set_slider_max(1.0);
                            f.set_valid_min(0.0);
                            f.set_valid_max(1.0);
                            f.set_default(0.0);
                            // Unset precision is the zeroed field = integer
                            // stepping — floats dragged like ints (0.0.1
                            // user report). Hundredths matches AE's own
                            // float sliders.
                            f.set_precision(ae::Precision::Hundredths);
                        }),
                        ae::ParamFlag::START_COLLAPSED,
                        ae::ParamUIFlags::empty(),
                    )?,
                    PoolKind::Integer => params.add_with_flags(
                        key,
                        &name,
                        ae::SliderDef::setup(|s| {
                            s.set_slider_min(0);
                            s.set_slider_max(10);
                            s.set_valid_min(0);
                            s.set_valid_max(10);
                            s.set_default(0);
                        }),
                        ae::ParamFlag::START_COLLAPSED,
                        ae::ParamUIFlags::empty(),
                    )?,
                    PoolKind::Bool => params.add_with_flags(
                        key,
                        &name,
                        ae::CheckBoxDef::setup(|c| {
                            c.set_default(false);
                        }),
                        ae::ParamFlag::START_COLLAPSED,
                        ae::ParamUIFlags::empty(),
                    )?,
                    PoolKind::Color => params.add_with_flags(
                        key,
                        &name,
                        ae::ColorDef::setup(|c| {
                            c.set_default(ae::sys::PF_Pixel {
                                alpha: 255,
                                red: 255,
                                green: 255,
                                blue: 255,
                            });
                        }),
                        ae::ParamFlag::START_COLLAPSED,
                        ae::ParamUIFlags::empty(),
                    )?,
                    PoolKind::Point2D => params.add_with_flags(
                        key,
                        &name,
                        ae::PointDef::setup(|p| {
                            p.set_default((50.0, 50.0));
                        }),
                        ae::ParamFlag::START_COLLAPSED,
                        ae::ParamUIFlags::empty(),
                    )?,
                    PoolKind::Angle => params.add_with_flags(
                        key,
                        &name,
                        ae::AngleDef::setup(|a| {
                            a.set_default(0.0);
                        }),
                        ae::ParamFlag::START_COLLAPSED,
                        ae::ParamUIFlags::empty(),
                    )?,
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
    fn topology_has_five_heads_104_pool_slots_and_details() {
        let order = declaration_order();
        // ADR-0013 base (5 heads + 104 pools) + ADR-0028 Details appended.
        assert_eq!(order.len(), 110);
        assert_eq!(&order[..5], &HEAD);
        assert_eq!(order[109], ParamKey::Details);
    }

    /// The pool segment mirrors V1_POOLS exactly: table order, kind-local
    /// indexes 0..capacity, no gaps. This is the index contract a future
    /// append must extend at the tail only.
    #[test]
    fn pool_segment_mirrors_the_configuration_source() {
        let order = declaration_order();
        let mut expected = Vec::new();
        for (kind, capacity) in V1_POOLS {
            for i in 0..*capacity {
                expected.push(ParamKey::Pool(*kind, i));
            }
        }
        assert_eq!(&order[5..109], &expected[..]);
    }

    #[test]
    fn every_pool_kind_has_a_label() {
        for (kind, _) in V1_POOLS {
            assert!(!kind_label(*kind).is_empty());
        }
    }

    /// Stream indexes are declaration positions + 1 (input layer at 0).
    #[test]
    fn stream_index_constants_track_declaration_order() {
        let order = declaration_order();
        let stream_of = |key: ParamKey| {
            order.iter().position(|k| *k == key).map(|p| p as i32 + 1)
        };
        assert_eq!(stream_of(ParamKey::Language), Some(LANGUAGE_STREAM_INDEX));
        assert_eq!(stream_of(ParamKey::Source), Some(SOURCE_STREAM_INDEX));
        assert_eq!(stream_of(ParamKey::StateToken), Some(STATE_TOKEN_STREAM_INDEX));
    }
}
