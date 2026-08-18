//! The ADR-0013 AE parameter topology: fixed head parameters plus the 104
//! pool slots, declared in one deterministic order derived from
//! `binding::V1_POOLS`.
//!
//! AE matches effect parameter streams by declaration order across project
//! loads, so `declaration_order()` is a persistent contract: a released
//! index never changes kind, moves, or dies; all growth appends at the tail
//! (ADR-0013 §5).

use crate::binding::{PoolKind, GROWTH_POOLS, V1_POOLS};
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
    /// ADR-0033: how many of a gradient's eight stops are live.
    GradientCount(usize),
    /// ADR-0033: one field of one stop of one gradient. Ordinary parameters,
    /// so AE owns their persistence, undo, copy/paste and keyframes.
    GradientStop(usize, usize, GradientField),
}

/// The three ordinary parameters that make up one gradient stop (ADR-0033 §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GradientField {
    Position,
    Color,
    Alpha,
}

/// ADR-0033 §2. Capacity is a persistent contract: growth appends, never
/// renumbers.
pub const GRADIENTS: usize = 2;
pub const STOPS_PER_GRADIENT: usize = 8;

/// How many stops a freshly declared gradient has live, and therefore how many
/// stop groups the Effect Controls panel shows before the user touches
/// anything. Two, not eight: the count drives visibility now that the ADR-0031
/// §7 editor is gone, and eight would put 25 rows in the panel of every bound
/// gradient (user report, 2026-08-15).
pub const DEFAULT_LIVE_STOPS: usize = 2;

/// Default position of stop `stop`: the first stop at the start of the ramp,
/// every other stop parked at the end.
///
/// Monotone, so the declared defaults can never read back as `E54`, and
/// raising `Stops` never changes the rendered ramp — a new stop appears where
/// the ramp already ends and the user drags it inward. That is the invariant
/// the deleted editor got from sampling the ramp before inserting.
pub fn default_stop_position(stop: usize) -> f64 {
    if stop == 0 {
        0.0
    } else {
        1.0
    }
}

/// ADR-0037 §1: the *valid* range the Float and Integer pools register at
/// `PARAMS_SETUP`. This is the only range After Effects clamps a rendered
/// value to, and `PF_UpdateParamUI` cannot change it later — the SDK header
/// lists `slider_min`, `slider_max`, `precision` and `display_flags` as the
/// only slider fields it touches. Registering `0..1` here (0.0.1–0.0.3) put a
/// permanent ceiling of 1.0 under every float parameter and 10 under every
/// integer, whatever the shader's `@param min:/max:` said (public issue #5).
///
/// The magnitude is exactly representable in the `f32` the SDK stores the
/// float bounds in, fits `i32` for the integer pool, and is a finite typing
/// bound. The *slider* range stays the display default (`0..1` / `0..10`) and
/// is what the annotation reconfigures per binding.
pub const POOL_FLOAT_VALID_RANGE: (f32, f32) = (-1_000_000_000.0, 1_000_000_000.0);
pub const POOL_INT_VALID_RANGE: (i32, i32) = (-1_000_000_000, 1_000_000_000);

/// Display range of an unbound / un-annotated Float slot (ADR-0037 §1). A
/// bound slot's `@param min:/max:` replaces it through `PF_UpdateParamUI`.
pub const POOL_FLOAT_SLIDER_RANGE: (f32, f32) = (0.0, 1.0);
/// Display range of an unbound / un-annotated Integer slot.
pub const POOL_INT_SLIDER_RANGE: (i32, i32) = (0, 10);

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
    // ADR-0030/0031 growth: after Details, never inside the V1 loop above —
    // Details is index 109 in every project saved by 0.0.2, and widening the
    // V1 pools would slide it. See `binding::GROWTH_POOLS`.
    for (kind, capacity) in GROWTH_POOLS {
        for i in 0..*capacity {
            order.push(ParamKey::Pool(*kind, i));
        }
    }
    // ADR-0033: a gradient's stops are ordinary parameters. `Pool(Gradient, g)`
    // above is the preview/canvas row; the value lives in these. Declared last
    // so every index before them — including Details at 109 and the ADR-0030
    // Layer slots — keeps its position.
    for g in 0..GRADIENTS {
        order.push(ParamKey::GradientCount(g));
        for stop in 0..STOPS_PER_GRADIENT {
            for field in [GradientField::Position, GradientField::Color, GradientField::Alpha] {
                order.push(ParamKey::GradientStop(g, stop, field));
            }
        }
    }
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
        PoolKind::Layer => "Layer",
        PoolKind::Gradient => "Gradient",
        PoolKind::Point3D => "Point 3D",
        PoolKind::Path => "Mask",
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
            // ADR-0033: gradient stops as ordinary rows. Two stops are live by
            // default and the other six park at the far end, so a freshly bound
            // gradient reads as a clean black->white ramp on sight AND raising
            // the count never changes the picture — a stop appears where the
            // ramp already ends, and the user drags it inward. The same
            // invariant the deleted editor got from sampling the ramp before
            // inserting.
            ParamKey::GradientCount(g) => {
                params.add_with_flags(
                    key,
                    &format!("Gradient {:02} Stops", g + 1),
                    ae::FloatSliderDef::setup(|f| {
                        f.set_slider_min(1.0);
                        f.set_slider_max(STOPS_PER_GRADIENT as f32);
                        f.set_valid_min(1.0);
                        f.set_valid_max(STOPS_PER_GRADIENT as f32);
                        f.set_precision(ae::Precision::Integer);
                        f.set_default(DEFAULT_LIVE_STOPS as f64);
                    }),
                    ae::ParamFlag::START_COLLAPSED,
                    ae::ParamUIFlags::empty(),
                )?;
            }
            ParamKey::GradientStop(g, stop, field) => {
                let even = default_stop_position(stop);
                let name = format!("G{:02} Stop {:02} ", g + 1, stop + 1);
                match field {
                    // Precision is explicit for the same reason ADR-0028
                    // made it explicit on the pool sliders: a float slider
                    // left at AE's default displays and rounds to whole
                    // numbers, so an evenly spread ramp came back as
                    // 0,0,0,0,1,1,1,1 (measured 2026-08-15).
                    GradientField::Position => params.add_with_flags(
                        key,
                        &(name + "Pos"),
                        ae::FloatSliderDef::setup(|f| {
                            f.set_slider_min(0.0);
                            f.set_slider_max(1.0);
                            f.set_valid_min(0.0);
                            f.set_valid_max(1.0);
                            f.set_precision(ae::Precision::Thousandths);
                            f.set_default(even);
                        }),
                        ae::ParamFlag::START_COLLAPSED,
                        ae::ParamUIFlags::empty(),
                    )?,
                    // Black at the start, white everywhere else — derived from
                    // the same curve as the position, so the two cannot drift.
                    GradientField::Color => params.add_with_flags(
                        key,
                        &(name + "Color"),
                        ae::ColorDef::setup(|c| {
                            let level = (even * 255.0).round() as u8;
                            c.set_default(ae::Pixel8 { alpha: 255, red: level, green: level, blue: level });
                        }),
                        ae::ParamFlag::START_COLLAPSED,
                        ae::ParamUIFlags::empty(),
                    )?,
                    GradientField::Alpha => params.add_with_flags(
                        key,
                        &(name + "Alpha"),
                        ae::FloatSliderDef::setup(|f| {
                            f.set_slider_min(0.0);
                            f.set_slider_max(1.0);
                            f.set_valid_min(0.0);
                            f.set_valid_max(1.0);
                            f.set_precision(ae::Precision::Thousandths);
                            f.set_default(1.0);
                        }),
                        ae::ParamFlag::START_COLLAPSED,
                        ae::ParamUIFlags::empty(),
                    )?,
                }
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
                    // ADR-0037: the valid range is wide and fixed here; the
                    // slider range is the display default a binding replaces.
                    PoolKind::Float => params.add_with_flags(
                        key,
                        &name,
                        ae::FloatSliderDef::setup(|f| {
                            f.set_slider_min(POOL_FLOAT_SLIDER_RANGE.0);
                            f.set_slider_max(POOL_FLOAT_SLIDER_RANGE.1);
                            f.set_valid_min(POOL_FLOAT_VALID_RANGE.0);
                            f.set_valid_max(POOL_FLOAT_VALID_RANGE.1);
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
                            s.set_slider_min(POOL_INT_SLIDER_RANGE.0);
                            s.set_slider_max(POOL_INT_SLIDER_RANGE.1);
                            s.set_valid_min(POOL_INT_VALID_RANGE.0);
                            s.set_valid_max(POOL_INT_VALID_RANGE.1);
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
                    // ADR-0034. x/y mirror the Point 2D default exactly, and
                    // z is 0 — the plane the layer already occupies, so a
                    // fresh Point 3D starts where the user is looking rather
                    // than off in depth.
                    //
                    // Whether PF reads a Point 3D's declared x/y as a
                    // percentage of the frame (as it does for Point 2D) or as
                    // absolute pixels is NOT established here: the SDK header
                    // is not vendored with the crate, and no host leg has
                    // measured it. Both readings put the default somewhere
                    // visible and draggable, so this is safe to ship and is
                    // recorded as a host-verification item rather than
                    // asserted from memory.
                    PoolKind::Point3D => params.add_with_flags(
                        key,
                        &name,
                        ae::Point3DDef::setup(|p| {
                            p.set_default((50.0, 50.0, 0.0));
                        }),
                        ae::ParamFlag::START_COLLAPSED,
                        ae::ParamUIFlags::empty(),
                    )?,
                    // ADR-0035. The default is 0 = NONE: an unassigned
                    // selector binds the documented zero texture (§5), which is
                    // honest about the user not having picked a mask.
                    // Defaulting to path 1 would silently pick whichever mask
                    // happened to be drawn first.
                    PoolKind::Path => params.add_with_flags(
                        key,
                        &name,
                        ae::PathDef::setup(|p| {
                            p.set_default(0);
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
                    // ADR-0030. The default is deliberately *not*
                    // `PF_LayerDefault_MYSELF`: a layer input that silently
                    // pointed at the effect's own layer would render something
                    // plausible instead of the documented all-zeros, hiding
                    // the fact that the user never picked a source.
                    PoolKind::Layer => params.add_with_flags(
                        key,
                        &name,
                        ae::LayerDef::setup(|_| {}),
                        ae::ParamFlag::START_COLLAPSED,
                        ae::ParamUIFlags::empty(),
                    )?,
                    // ADR-0031 §7's custom-UI editor is gone (removed
                    // 2026-08-15 by decision, after the canvas parameter took
                    // the host down in every configuration tried: arbitrary
                    // data with no callbacks, arbitrary data with callbacks,
                    // and a float slider substituted to dodge both). ADR-0033
                    // §6 anticipated exactly this — "the preview/editor may
                    // therefore be [...] dropped entirely without making the
                    // feature unusable" — because the gradient VALUE lives in
                    // ordinary stop parameters that AE persists and keyframes
                    // by itself.
                    //
                    // The slot survives as an inert, permanently invisible
                    // float: it is the binding anchor `hint:gradient` resolves
                    // to, and it holds its declaration index so the ADR-0013 §5
                    // append-only topology contract still holds for every
                    // parameter declared after it. It is never shown and never
                    // read.
                    PoolKind::Gradient => params.add_with_flags(
                        key,
                        &name,
                        ae::FloatSliderDef::setup(|f| {
                            f.set_slider_min(0.0);
                            f.set_slider_max(1.0);
                            f.set_valid_min(0.0);
                            f.set_valid_max(1.0);
                            f.set_default(0.0);
                        }),
                        ae::ParamFlag::CANNOT_TIME_VARY,
                        ae::ParamUIFlags::INVISIBLE,
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

    /// The declared defaults must read back as a legal gradient. A default
    /// that fails `validate` would put every freshly bound gradient into `E54`
    /// before the user had touched it.
    #[test]
    fn declared_stop_defaults_are_a_legal_black_to_white_ramp() {
        let stops: Vec<crate::gradient::Stop> = (0..DEFAULT_LIVE_STOPS)
            .map(|stop| {
                let position = default_stop_position(stop) as f32;
                crate::gradient::Stop { position, rgba: [position, position, position, 1.0] }
            })
            .collect();
        let value = crate::gradient::Gradient { stops };
        value.validate().expect("declared defaults are a legal gradient");
        assert_eq!(value.sample(0.0), [0.0, 0.0, 0.0, 1.0], "starts black");
        assert_eq!(value.sample(1.0), [1.0, 1.0, 1.0, 1.0], "ends white");

        // Every spare stop parks at the end, so the whole eight stay monotone
        // and raising `Stops` cannot author an out-of-order value.
        let all: Vec<crate::gradient::Stop> = (0..STOPS_PER_GRADIENT)
            .map(|stop| {
                let position = default_stop_position(stop) as f32;
                crate::gradient::Stop { position, rgba: [position, position, position, 1.0] }
            })
            .collect();
        let widened = crate::gradient::Gradient { stops: all };
        widened.validate().expect("all eight declared defaults stay monotone");
        // ...and raising the count leaves the picture alone.
        for i in 0..=16 {
            let t = i as f32 / 16.0;
            assert_eq!(widened.sample(t), value.sample(t), "raising Stops changed the ramp at {t}");
        }
    }

    #[test]
    fn topology_has_five_heads_104_pool_slots_and_details() {
        let order = declaration_order();
        assert_eq!(&order[..5], &HEAD);
        // The 0.0.2 prefix is frozen: 5 heads + 104 V1 pool slots, then
        // Details at 109. Every project saved by a released build binds its
        // parameter streams to these positions, so this assertion may only
        // ever be *extended* past index 109 — never renumbered.
        assert_eq!(order[109], ParamKey::Details);
        // ADR-0030/0031/0034/0035 pool growth, then the ADR-0033 stop
        // parameters, all strictly after Details.
        let pools: usize = GROWTH_POOLS.iter().map(|(_, capacity)| capacity).sum();
        let stops = GRADIENTS * (1 + STOPS_PER_GRADIENT * 3);
        assert_eq!(order.len(), 110 + pools + stops);
        // 4 Layer + 2 Gradient anchors + 8 Point 3D + 2 Path, then
        // 2 x (1 count + 8 x 3 stop fields).
        assert_eq!(pools, 16);
        assert_eq!(stops, 50);
        assert!(
            order[110..110 + pools].iter().all(|k| matches!(k, ParamKey::Pool(..))),
            "the pool segment carries pool slots only"
        );
        assert!(
            order[110 + pools..].iter().all(|k| matches!(
                k,
                ParamKey::GradientCount(_) | ParamKey::GradientStop(..)
            )),
            "the tail carries ADR-0033 stop parameters only"
        );
    }

    /// The released prefix cannot move. Reconstructing it from the frozen
    /// `V1_POOLS` table and comparing against the live order is what makes a
    /// later append fail loudly if someone widens a V1 pool instead of
    /// appending to `GROWTH_POOLS`.
    #[test]
    fn released_prefix_is_frozen_through_details() {
        let order = declaration_order();
        let mut expected: Vec<ParamKey> = HEAD.to_vec();
        for (kind, capacity) in V1_POOLS {
            for i in 0..*capacity {
                expected.push(ParamKey::Pool(*kind, i));
            }
        }
        expected.push(ParamKey::Details);
        assert_eq!(expected.len(), 110);
        assert_eq!(&order[..110], &expected[..]);
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
        for (kind, _) in crate::binding::all_pools() {
            assert!(!kind_label(*kind).is_empty());
        }
    }

    /// The host harness addresses the new pools by hard-coded AE property
    /// index (`scripts/f003/*.jsx`), because ExtendScript has no way to ask
    /// for a slot by kind. Pin them here so a future append cannot silently
    /// repoint what those scripts probe.
    #[test]
    fn growth_pool_property_indexes_match_the_harness() {
        let order = declaration_order();
        let property_index = |key: ParamKey| {
            order.iter().position(|k| *k == key).map(|p| p + 1)
        };
        assert_eq!(
            property_index(ParamKey::Pool(PoolKind::Layer, 0)),
            Some(111),
            "f003a_layer.jsx probes property 111"
        );
        assert_eq!(
            property_index(ParamKey::Pool(PoolKind::Gradient, 0)),
            Some(115),
            "f003b_gradient.jsx probes property 115"
        );
        // ADR-0034/0035 appended ten more pool slots between the gradient
        // anchors and the stop parameters, so the stop block starts ten later.
        assert_eq!(
            property_index(ParamKey::Pool(PoolKind::Point3D, 0)),
            Some(117),
            "f003f_point3d.jsx probes property 117"
        );
        assert_eq!(
            property_index(ParamKey::Pool(PoolKind::Path, 0)),
            Some(125),
            "f003g_path.jsx probes property 125"
        );
        assert_eq!(
            property_index(ParamKey::GradientCount(0)),
            Some(127),
            "f003b_gradient.jsx reads the stop block from property 127"
        );
        // TR-0037-001 (f003h_range.jsx) drives the first two Float slots and
        // the first Integer slot; these are frozen V1 positions.
        assert_eq!(property_index(ParamKey::Pool(PoolKind::Float, 0)), Some(6), "f003h: wide");
        assert_eq!(property_index(ParamKey::Pool(PoolKind::Float, 1)), Some(7), "f003h: neg");
        assert_eq!(property_index(ParamKey::Pool(PoolKind::Integer, 0)), Some(54), "f003h: count");
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

    /// ADR-0037 §1. The registered valid range is the only range After Effects
    /// clamps a rendered value to, and it cannot be changed after
    /// `PARAMS_SETUP`; a narrow one here is the public-issue-#5 defect (every
    /// float above 1 and every int above 10 reached the shader clamped). The
    /// pins: wide enough for any plausible shader value, symmetric so negatives
    /// pass, exactly representable in the `f32` the SDK stores the float bounds
    /// in, and the display (slider) defaults still the modest `0..1` / `0..10`
    /// that an un-annotated slot shows.
    #[test]
    fn pool_valid_ranges_are_wide_symmetric_and_exact() {
        let (fmin, fmax) = POOL_FLOAT_VALID_RANGE;
        assert_eq!(fmin, -fmax, "float valid range is symmetric");
        assert!(fmax >= 1_000_000_000.0, "float valid range covers ±1e9");
        // Exact in f32: 1e9 = 2^9 · 1953125 and 1953125 < 2^24, so the
        // conversion the SDK performs loses nothing.
        assert_eq!(fmax as f64, 1_000_000_000.0_f64);
        assert_eq!(fmax as i64, 1_000_000_000_i64);

        let (imin, imax) = POOL_INT_VALID_RANGE;
        assert_eq!(imin, -imax, "int valid range is symmetric");
        assert!(imax >= 1_000_000_000, "int valid range covers ±1e9");

        // The display defaults are unchanged from 0.0.1: a modest range an
        // un-annotated slot drags over, replaced per binding by @param.
        assert_eq!(POOL_FLOAT_SLIDER_RANGE, (0.0, 1.0));
        assert_eq!(POOL_INT_SLIDER_RANGE, (0, 10));

        // The slider default lies inside the valid range (AE rejects a
        // definition whose display range exceeds its valid range).
        assert!(fmin <= POOL_FLOAT_SLIDER_RANGE.0 && POOL_FLOAT_SLIDER_RANGE.1 <= fmax);
        assert!(imin <= POOL_INT_SLIDER_RANGE.0 && POOL_INT_SLIDER_RANGE.1 <= imax);
    }
}
