//! The AE parameter topology and stable stream identities (ADR-0013,
//! ADR-0040).
//!
//! After Effects restores streams by the murmur3 id derived from each
//! `ParamKey`'s `Debug` rendering. Existing renderings are therefore a
//! persistence contract even when declaration order changes.

use crate::binding::{
    bank_capacity, main_pool_capacity, BindingPlan, PoolKind, SlotRef, BANK_GROUPS, BANK_POOLS,
    GROWTH_POOLS, V1_POOLS,
};
use crate::frontend;
use after_effects as ae;

/// Parameter identity for the AE dispatch macro. `Pool(kind, i)` is the
/// kind-local slot `i` of one pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParamKey {
    SetupStart,
    SetupEnd,
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
    /// ADR-0028: button that pops the full, untruncated status text (the
    /// Status name is capped at 31 chars by PF).
    Details,
    /// ADR-0033: how many of a gradient's eight stops are live.
    GradientCount(usize),
    /// ADR-0033: one field of one stop of one gradient. Ordinary parameters,
    /// so AE owns their persistence, undo, copy/paste and keyframes.
    GradientStop(usize, usize, GradientField),
    /// ADR-0038 §7: hidden primitive carrying the plan identity of the
    /// published artifact beside the StateToken, so a render clone names its
    /// own instance's entry even when its flattened copy predates the
    /// compile.
    PlanToken,
    Bank(usize, PoolKind, usize),
    MainStart,
    MainEnd,
    GradientGroupStart(usize),
    GradientGroupEnd(usize),
    #[cfg(feature = "editor")]
    GradientCanvas(usize),
    PassGroupStart(usize),
    PassGroupEnd(usize),
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
pub const LANGUAGE_STREAM_INDEX: i32 = 2;
pub const SOURCE_STREAM_INDEX: i32 = 3;
pub const STATE_TOKEN_STREAM_INDEX: i32 = 6;
const INPUT_LAYER_PARAM_COUNT: usize = 1;

/// The complete grouped declaration order. Stream identity is pinned by the
/// `ParamKey` id table; this order controls only panel topology (ADR-0040 §1).
pub fn declaration_order() -> Vec<ParamKey> {
    let mut order = vec![ParamKey::SetupStart];
    order.extend(HEAD);
    order.push(ParamKey::Details);
    order.push(ParamKey::PlanToken);
    order.push(ParamKey::SetupEnd);
    order.push(ParamKey::MainStart);
    for (kind, capacity) in V1_POOLS {
        for i in 0..*capacity {
            order.push(ParamKey::Pool(*kind, i));
        }
    }
    for (kind, capacity) in GROWTH_POOLS {
        if *kind == PoolKind::Gradient {
            continue;
        }
        for i in 0..*capacity {
            order.push(ParamKey::Pool(*kind, i));
        }
    }
    for g in 0..GRADIENTS {
        order.push(ParamKey::GradientGroupStart(g));
        #[cfg(feature = "editor")]
        order.push(ParamKey::GradientCanvas(g));
        order.push(ParamKey::Pool(PoolKind::Gradient, g));
        order.push(ParamKey::GradientCount(g));
        for stop in 0..STOPS_PER_GRADIENT {
            for field in [GradientField::Position, GradientField::Color, GradientField::Alpha] {
                order.push(ParamKey::GradientStop(g, stop, field));
            }
        }
        order.push(ParamKey::GradientGroupEnd(g));
    }
    order.push(ParamKey::MainEnd);
    for group in 0..BANK_GROUPS {
        order.push(ParamKey::PassGroupStart(group));
        for (kind, capacity) in BANK_POOLS {
            for local in 0..*capacity {
                let slot = slot_for_key(ParamKey::Bank(group, *kind, local))
                    .expect("declared bank key has a slot");
                order.push(key_for_slot(slot.kind, slot.index));
            }
        }
        order.push(ParamKey::PassGroupEnd(group));
    }
    order
}

/// Translate a binding-plan slot into its physical topology key.
pub fn key_for_slot(kind: PoolKind, index: usize) -> ParamKey {
    let main = main_pool_capacity(kind);
    if index < main {
        return ParamKey::Pool(kind, index);
    }
    let capacity = bank_capacity(kind);
    assert!(capacity > 0, "Main-only pool index {index} is out of range for {kind:?}");
    let offset = index - main;
    let group = offset / capacity;
    assert!(group < BANK_GROUPS, "pool index {index} is out of range for {kind:?}");
    ParamKey::Bank(group, kind, offset % capacity)
}

/// Inverse of `key_for_slot`; group markers and ordinary controls have no
/// binding slot.
pub fn slot_for_key(key: ParamKey) -> Option<SlotRef> {
    match key {
        ParamKey::Pool(kind, index) if index < main_pool_capacity(kind) => {
            Some(SlotRef { kind, index })
        }
        ParamKey::Bank(group, kind, local)
            if group < BANK_GROUPS && local < bank_capacity(kind) =>
        {
            Some(SlotRef {
                kind,
                index: main_pool_capacity(kind) + group * bank_capacity(kind) + local,
            })
        }
        _ => None,
    }
}

/// AEGP stream index of the plan token. It is derived because only the five
/// fixed heads have direct observer constants.
pub fn plan_token_stream_index() -> i32 {
    stream_index_of(ParamKey::PlanToken).expect("plan token is declared")
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
    match key_for_slot(kind, index) {
        ParamKey::Pool(_, local) => format!("{} {:02}", kind_label(kind), local + 1),
        ParamKey::Bank(group, _, local) => default_bank_slot_name(group, kind, local),
        _ => unreachable!("slot keys are Pool or Bank"),
    }
}

fn default_bank_slot_name(group: usize, kind: PoolKind, index: usize) -> String {
    format!("P{:02} {} {:02}", group + 1, kind_label(kind), index + 1)
}

/// AEGP effect stream index of a declared parameter (declaration position
/// + 1; the implicit input layer occupies stream 0).
pub fn stream_index_of(key: ParamKey) -> Option<i32> {
    param_index_of(key).map(|index| index as i32)
}

pub fn param_index_of(key: ParamKey) -> Option<usize> {
    declaration_order()
        .iter()
        .position(|k| *k == key)
        .map(|position| position + INPUT_LAYER_PARAM_COUNT)
}

pub fn key_for_param_index(index: usize) -> Option<ParamKey> {
    declaration_order()
        .get(index.checked_sub(INPUT_LAYER_PARAM_COUNT)?)
        .copied()
}

/// Declare the full topology. Iterates `declaration_order()` so the declared
/// indexes and the contract remain one source.
pub fn setup(params: &mut ae::Parameters<ParamKey>) -> Result<(), ae::Error> {
    let order = declaration_order();
    let mut cursor = 0;
    declare_range(params, &order, &mut cursor, None)?;
    assert_eq!(cursor, order.len(), "all parameter declarations were consumed");
    Ok(())
}

/// The single declaration source for custom-control dimensions and the
/// registration boundary that makes those controls safe for AE to paint.
pub fn control_surface(key: ParamKey) -> Option<(u16, u16)> {
    match key {
        #[cfg(feature = "editor")]
        ParamKey::GradientCanvas(_) => Some((200, 80)),
        _ => None,
    }
}

pub fn requires_custom_ui() -> bool {
    declaration_order().into_iter().any(|key| control_surface(key).is_some())
}

fn declare_range(
    params: &mut ae::Parameters<ParamKey>,
    order: &[ParamKey],
    cursor: &mut usize,
    end: Option<ParamKey>,
) -> Result<(), ae::Error> {
    while *cursor < order.len() {
        let key = order[*cursor];
        *cursor += 1;
        if Some(key) == end {
            return Ok(());
        }
        match key {
            ParamKey::SetupStart => params.add_group(
                key,
                ParamKey::SetupEnd,
                "Setup",
                group_starts_collapsed(key),
                |inner| declare_range(inner, order, cursor, Some(ParamKey::SetupEnd)),
            )?,
            ParamKey::MainStart => params.add_group(
                key,
                ParamKey::MainEnd,
                "Main",
                group_starts_collapsed(key),
                |inner| declare_range(inner, order, cursor, Some(ParamKey::MainEnd)),
            )?,
            ParamKey::GradientGroupStart(g) => params.add_group(
                key,
                ParamKey::GradientGroupEnd(g),
                &format!("Gradient {:02}", g + 1),
                group_starts_collapsed(key),
                |inner| {
                    declare_range(inner, order, cursor, Some(ParamKey::GradientGroupEnd(g)))
                },
            )?,
            ParamKey::PassGroupStart(group) => params.add_group(
                key,
                ParamKey::PassGroupEnd(group),
                &default_pass_group_name(group),
                group_starts_collapsed(key),
                |inner| {
                    declare_range(inner, order, cursor, Some(ParamKey::PassGroupEnd(group)))
                },
            )?,
            ParamKey::SetupEnd
            | ParamKey::MainEnd
            | ParamKey::GradientGroupEnd(_)
            | ParamKey::PassGroupEnd(_) => {
                panic!("unexpected group end marker {key:?}")
            }
            _ => declare_one(params, key)?,
        }
    }
    assert!(end.is_none(), "missing group end marker {end:?}");
    Ok(())
}

fn group_starts_collapsed(key: ParamKey) -> bool {
    match key {
        ParamKey::SetupStart => false,
        ParamKey::MainStart
        | ParamKey::GradientGroupStart(_)
        | ParamKey::PassGroupStart(_) => true,
        _ => unreachable!("{key:?} is not a group start"),
    }
}

pub fn default_pass_group_name(group: usize) -> String {
    format!("Pass {}", group + 1)
}

pub fn pass_group_name(group: usize, live_name: Option<&str>) -> String {
    live_name
        .filter(|name| !name.trim().is_empty())
        .map(|name| name.chars().take(31).collect())
        .unwrap_or_else(|| default_pass_group_name(group))
}

/// Returns the desired Hidden flag for presentation-only group and canvas
/// rows. Setup and Main are topology anchors and have no dynamic visibility.
pub fn group_hidden(plan: Option<&BindingPlan>, key: ParamKey) -> Option<bool> {
    let bound = |candidate: ParamKey| {
        plan.is_some_and(|plan| {
            plan.bindings.iter().flat_map(|binding| &binding.slots).any(|slot| {
                let slot_key = key_for_slot(slot.kind, slot.index);
                match (candidate, slot_key) {
                    (ParamKey::PassGroupStart(group), ParamKey::Bank(bound_group, _, _))
                    | (ParamKey::PassGroupEnd(group), ParamKey::Bank(bound_group, _, _)) => {
                        group == bound_group
                    }
                    (
                        ParamKey::GradientGroupStart(group),
                        ParamKey::Pool(PoolKind::Gradient, bound_group),
                    )
                    | (
                        ParamKey::GradientGroupEnd(group),
                        ParamKey::Pool(PoolKind::Gradient, bound_group),
                    ) => group == bound_group,
                    #[cfg(feature = "editor")]
                    (
                        ParamKey::GradientCanvas(group),
                        ParamKey::Pool(PoolKind::Gradient, bound_group),
                    ) => group == bound_group,
                    _ => false,
                }
            })
        })
    };

    match key {
        #[cfg(feature = "editor")]
        ParamKey::GradientCanvas(_) => Some(!bound(key)),
        ParamKey::PassGroupStart(_)
        | ParamKey::PassGroupEnd(_)
        | ParamKey::GradientGroupStart(_)
        | ParamKey::GradientGroupEnd(_) => Some(!bound(key)),
        ParamKey::SetupStart | ParamKey::SetupEnd | ParamKey::MainStart | ParamKey::MainEnd => None,
        _ => None,
    }
}

fn declare_one(params: &mut ae::Parameters<ParamKey>, key: ParamKey) -> Result<(), ae::Error> {
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
            ParamKey::StateToken | ParamKey::PlanToken => {
                params.add_with_flags(
                    key,
                    if key == ParamKey::StateToken {
                        "State Token (internal)"
                    } else {
                        "Plan Token (internal)"
                    },
                    ae::FloatSliderDef::setup(|f| {
                        f.set_slider_min(0.0);
                        f.set_slider_max(1.0);
                        f.set_valid_min(0.0);
                        // Room for an exactly representable f64 integer word
                        // (ADR-0015 token layout; ADR-0038 §7 plan word).
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
            #[cfg(feature = "editor")]
            ParamKey::GradientCanvas(_) => {
                let (width, height) = control_surface(key).expect("canvas has a control surface");
                params.add_customized(
                    key,
                    "Preview",
                    ae::ColorDef::setup(|c| {
                        c.set_default(ae::Pixel8 {
                            alpha: 255,
                            red: 128,
                            green: 128,
                            blue: 128,
                        });
                    }),
                    |param| {
                        param.set_flags(ae::ParamFlag::CANNOT_TIME_VARY);
                        param.set_ui_flags(ae::ParamUIFlags::CONTROL);
                        param.set_ui_width(width);
                        param.set_ui_height(height);
                        -1
                    },
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
            key @ (ParamKey::Pool(..) | ParamKey::Bank(..)) => {
                let (kind, name) = match key {
                    ParamKey::Pool(kind, index) => (kind, default_slot_name(kind, index)),
                    ParamKey::Bank(group, kind, index) => {
                        (kind, default_bank_slot_name(group, kind, index))
                    }
                    _ => unreachable!(),
                };
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
                    // Keep the binding anchor inert and invisible. The optional
                    // editor canvas is a separate presentation-only stream, so
                    // neither rendering nor binding identity can depend on it.
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
            ParamKey::SetupStart
            | ParamKey::SetupEnd
            | ParamKey::MainStart
            | ParamKey::MainEnd
            | ParamKey::GradientGroupStart(_)
            | ParamKey::GradientGroupEnd(_)
            | ParamKey::PassGroupStart(_)
            | ParamKey::PassGroupEnd(_) => {
                unreachable!("group markers are declared by add_group")
            }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shipped_declaration_order() -> Vec<ParamKey> {
        let mut order = vec![ParamKey::SetupStart];
        order.extend(HEAD);
        order.push(ParamKey::Details);
        order.push(ParamKey::PlanToken);
        order.push(ParamKey::SetupEnd);
        order.push(ParamKey::MainStart);
        for (kind, capacity) in V1_POOLS {
            for index in 0..*capacity {
                order.push(ParamKey::Pool(*kind, index));
            }
        }
        for (kind, capacity) in GROWTH_POOLS {
            if *kind == PoolKind::Gradient {
                continue;
            }
            for index in 0..*capacity {
                order.push(ParamKey::Pool(*kind, index));
            }
        }
        for gradient in 0..GRADIENTS {
            order.push(ParamKey::GradientGroupStart(gradient));
            order.push(ParamKey::Pool(PoolKind::Gradient, gradient));
            order.push(ParamKey::GradientCount(gradient));
            for stop in 0..STOPS_PER_GRADIENT {
                for field in [
                    GradientField::Position,
                    GradientField::Color,
                    GradientField::Alpha,
                ] {
                    order.push(ParamKey::GradientStop(gradient, stop, field));
                }
            }
            order.push(ParamKey::GradientGroupEnd(gradient));
        }
        order.push(ParamKey::MainEnd);
        for group in 0..BANK_GROUPS {
            order.push(ParamKey::PassGroupStart(group));
            for (kind, capacity) in BANK_POOLS {
                for local in 0..*capacity {
                    order.push(ParamKey::Bank(group, *kind, local));
                }
            }
            order.push(ParamKey::PassGroupEnd(group));
        }
        order
    }

    #[test]
    fn editor_boundary_is_one_control_surface_set() {
        let surfaces: std::collections::HashSet<_> = declaration_order()
            .into_iter()
            .filter(|key| control_surface(*key).is_some())
            .collect();
        #[cfg(feature = "editor")]
        let expected = std::collections::HashSet::from([
            ParamKey::GradientCanvas(0),
            ParamKey::GradientCanvas(1),
        ]);
        #[cfg(not(feature = "editor"))]
        let expected = std::collections::HashSet::new();

        assert_eq!(surfaces, expected);
        assert_eq!(requires_custom_ui(), !surfaces.is_empty());
        for key in surfaces {
            assert_eq!(control_surface(key), Some((200, 80)));
        }
    }

    #[test]
    fn editor_topology_is_the_shipped_order_plus_first_row_canvases() {
        let order = declaration_order();
        let shipped = shipped_declaration_order();
        #[cfg(not(feature = "editor"))]
        assert_eq!(order, shipped);
        #[cfg(feature = "editor")]
        {
            let without_canvases: Vec<_> = order
                .iter()
                .copied()
                .filter(|key| !matches!(key, ParamKey::GradientCanvas(_)))
                .collect();
            assert_eq!(without_canvases, shipped);
            for gradient in 0..GRADIENTS {
                let start = order
                    .iter()
                    .position(|key| *key == ParamKey::GradientGroupStart(gradient))
                    .unwrap();
                assert_eq!(order[start + 1], ParamKey::GradientCanvas(gradient));
                assert_eq!(order[start + 2], ParamKey::Pool(PoolKind::Gradient, gradient));
            }
        }
        for key in order {
            let index = param_index_of(key).unwrap();
            assert_eq!(key_for_param_index(index), Some(key));
        }
        assert_eq!(key_for_param_index(0), None);
    }

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
    fn grouped_topology_has_balanced_nested_markers() {
        let order = declaration_order();
        assert_eq!(order[0], ParamKey::SetupStart);
        assert_eq!(&order[1..6], &HEAD);
        assert_eq!(order[6], ParamKey::Details);
        assert_eq!(order[7], ParamKey::PlanToken);
        assert_eq!(order[8], ParamKey::SetupEnd);
        assert_eq!(order[9], ParamKey::MainStart);

        let v1_slots: usize = V1_POOLS.iter().map(|(_, capacity)| capacity).sum();
        let main_growth: usize = GROWTH_POOLS
            .iter()
            .filter(|(kind, _)| *kind != PoolKind::Gradient)
            .map(|(_, capacity)| capacity)
            .sum();
        let gradient_rows = GRADIENTS
            * (1 + 1 + STOPS_PER_GRADIENT * 3 + 2 + usize::from(cfg!(feature = "editor")));
        let bank_slots: usize = BANK_POOLS.iter().map(|(_, capacity)| capacity).sum();
        assert_eq!(v1_slots, 104);
        assert_eq!(main_growth, 14);
        assert_eq!(bank_slots, 18);
        assert_eq!(
            BANK_POOLS,
            &[
                (PoolKind::Float, 8),
                (PoolKind::Integer, 2),
                (PoolKind::Bool, 2),
                (PoolKind::Color, 3),
                (PoolKind::Point2D, 2),
                (PoolKind::Angle, 1),
            ]
        );
        let expected = HEAD.len()
            + 2
            + 2
            + 2
            + v1_slots
            + main_growth
            + gradient_rows
            + BANK_GROUPS * (bank_slots + 2);
        assert_eq!(order.len(), expected);

        let mut stack = Vec::new();
        for key in &order {
            match *key {
                ParamKey::SetupStart => {
                    assert!(stack.is_empty(), "Setup must be top-level");
                    stack.push(*key);
                }
                ParamKey::MainStart => {
                    assert!(stack.is_empty(), "Main must be top-level");
                    stack.push(*key);
                }
                ParamKey::GradientGroupStart(_) => {
                    assert_eq!(stack.last(), Some(&ParamKey::MainStart));
                    stack.push(*key);
                }
                ParamKey::PassGroupStart(_) => {
                    assert!(stack.is_empty(), "pass groups must be top-level");
                    stack.push(*key);
                }
                ParamKey::SetupEnd => assert_eq!(stack.pop(), Some(ParamKey::SetupStart)),
                ParamKey::MainEnd => assert_eq!(stack.pop(), Some(ParamKey::MainStart)),
                ParamKey::GradientGroupEnd(g) => {
                    assert_eq!(stack.pop(), Some(ParamKey::GradientGroupStart(g)))
                }
                ParamKey::PassGroupEnd(group) => {
                    assert_eq!(stack.pop(), Some(ParamKey::PassGroupStart(group)))
                }
                _ => {}
            }
        }
        assert!(stack.is_empty(), "every group marker must be balanced");
    }

    #[test]
    fn setup_is_the_only_group_declared_expanded() {
        assert!(!group_starts_collapsed(ParamKey::SetupStart));
        assert!(group_starts_collapsed(ParamKey::MainStart));
        for group in 0..BANK_GROUPS {
            assert!(group_starts_collapsed(ParamKey::PassGroupStart(group)));
        }
        for gradient in 0..GRADIENTS {
            assert!(group_starts_collapsed(ParamKey::GradientGroupStart(gradient)));
        }
    }

    /// Released `Debug` renderings are persisted AE stream identities. The
    /// former declaration-index freeze is superseded by this id contract
    /// (ADR-0040 §1).
    #[test]
    fn debug_renderings_and_murmur3_ids_are_golden_and_collision_free() {
        use std::hash::{Hash, Hasher};

        let golden = vec![
            (ParamKey::SetupStart, "SetupStart"),
            (ParamKey::SetupEnd, "SetupEnd"),
            (ParamKey::Language, "Language"),
            (ParamKey::Source, "Source"),
            (ParamKey::Compile, "Compile"),
            (ParamKey::Status, "Status"),
            (ParamKey::StateToken, "StateToken"),
            (ParamKey::Details, "Details"),
            (ParamKey::PlanToken, "PlanToken"),
            (ParamKey::Pool(PoolKind::Float, 0), "Pool(Float, 0)"),
            (ParamKey::Pool(PoolKind::Float, 47), "Pool(Float, 47)"),
            (ParamKey::Pool(PoolKind::Gradient, 0), "Pool(Gradient, 0)"),
            (ParamKey::GradientCount(0), "GradientCount(0)"),
            (
                ParamKey::GradientStop(0, 0, GradientField::Position),
                "GradientStop(0, 0, Position)",
            ),
            (ParamKey::MainStart, "MainStart"),
            (ParamKey::MainEnd, "MainEnd"),
            (ParamKey::GradientGroupStart(0), "GradientGroupStart(0)"),
            (ParamKey::GradientGroupEnd(0), "GradientGroupEnd(0)"),
            (ParamKey::PassGroupStart(0), "PassGroupStart(0)"),
            (ParamKey::PassGroupEnd(0), "PassGroupEnd(0)"),
            (ParamKey::Bank(0, PoolKind::Float, 0), "Bank(0, Float, 0)"),
        ];
        #[cfg(feature = "editor")]
        let golden = {
            let mut golden = golden;
            golden.extend([
                (ParamKey::GradientCanvas(0), "GradientCanvas(0)"),
                (ParamKey::GradientCanvas(1), "GradientCanvas(1)"),
            ]);
            golden
        };
        for (key, expected) in golden {
            assert_eq!(
                format!("{key:?}"),
                expected,
                "ParamKey Debug rendering changed; this breaks persisted AE stream identity"
            );
        }

        let mut renderings = std::collections::HashSet::new();
        let mut ids = std::collections::HashMap::new();
        for key in declaration_order() {
            let rendering = format!("{key:?}");
            assert!(
                renderings.insert(rendering.clone()),
                "duplicate ParamKey Debug rendering: {rendering}"
            );
            let mut hasher = hash32::Murmur3Hasher::default();
            rendering.hash(&mut hasher);
            let id = hasher.finish() as i32;
            assert!(
                ids.insert(id, rendering.clone()).is_none(),
                "murmur3 id collision for {rendering}: {id}"
            );
        }
    }

    #[test]
    fn slot_key_mapping_is_bijective() {
        for (kind, capacity) in crate::binding::all_pools() {
            for index in 0..capacity {
                let slot = SlotRef { kind, index };
                assert_eq!(slot_for_key(key_for_slot(kind, index)), Some(slot));
            }
        }
        assert_eq!(
            default_slot_name(PoolKind::Float, main_pool_capacity(PoolKind::Float)),
            "P01 Float 01"
        );
    }

    #[test]
    fn every_pool_kind_has_a_label() {
        for (kind, _) in crate::binding::all_pools() {
            assert!(!kind_label(kind).is_empty());
        }
    }

    /// The host harness addresses the new pools by hard-coded AE property
    /// index (`scripts/f003/*.jsx`), because ExtendScript has no way to ask
    /// for a slot by kind. Pin them here so a future append cannot silently
    /// repoint what those scripts probe.
    #[test]
    fn growth_pool_property_indexes_match_the_harness() {
        let order = declaration_order();
        let editor_offset = usize::from(cfg!(feature = "editor"));
        let property_index = |key: ParamKey| {
            order.iter().position(|k| *k == key).map(|p| p + 1)
        };
        assert_eq!(property_index(ParamKey::Pool(PoolKind::Layer, 0)), Some(115));
        assert_eq!(
            property_index(ParamKey::Pool(PoolKind::Gradient, 0)),
            Some(130 + editor_offset)
        );
        assert_eq!(
            property_index(ParamKey::Pool(PoolKind::Point3D, 0)),
            Some(119)
        );
        assert_eq!(property_index(ParamKey::Pool(PoolKind::Path, 0)), Some(127));
        assert_eq!(
            property_index(ParamKey::GradientCount(0)),
            Some(131 + editor_offset)
        );
        assert_eq!(property_index(ParamKey::Pool(PoolKind::Float, 0)), Some(11));
        assert_eq!(property_index(ParamKey::Pool(PoolKind::Float, 1)), Some(12));
        assert_eq!(property_index(ParamKey::Pool(PoolKind::Integer, 0)), Some(59));
        assert_eq!(
            property_index(ParamKey::Bank(0, PoolKind::Float, 0)),
            Some(187 + 2 * editor_offset)
        );
    }

    #[test]
    fn group_presentation_decisions_follow_the_binding_plan() {
        use crate::binding::{ParamBinding, SlotRef};
        use crate::definition::param::ParamId;

        for group in 0..BANK_GROUPS {
            assert_eq!(group_hidden(None, ParamKey::PassGroupStart(group)), Some(true));
            assert_eq!(group_hidden(None, ParamKey::PassGroupEnd(group)), Some(true));
        }
        for gradient in 0..GRADIENTS {
            assert_eq!(
                group_hidden(None, ParamKey::GradientGroupStart(gradient)),
                Some(true)
            );
            assert_eq!(group_hidden(None, ParamKey::GradientGroupEnd(gradient)), Some(true));
            #[cfg(feature = "editor")]
            assert_eq!(group_hidden(None, ParamKey::GradientCanvas(gradient)), Some(true));
        }
        assert_eq!(group_hidden(None, ParamKey::SetupStart), None);
        assert_eq!(group_hidden(None, ParamKey::MainStart), None);

        let pass_slot = slot_for_key(ParamKey::Bank(2, PoolKind::Float, 0)).unwrap();
        let plan = BindingPlan {
            bindings: vec![
                ParamBinding {
                    id: ParamId::new("pass_value").unwrap(),
                    slots: vec![pass_slot],
                    inherited: false,
                },
                ParamBinding {
                    id: ParamId::new("gradient_value").unwrap(),
                    slots: vec![SlotRef { kind: PoolKind::Gradient, index: 1 }],
                    inherited: false,
                },
            ],
        };
        assert_eq!(group_hidden(Some(&plan), ParamKey::PassGroupStart(2)), Some(false));
        assert_eq!(group_hidden(Some(&plan), ParamKey::PassGroupEnd(2)), Some(false));
        assert_eq!(group_hidden(Some(&plan), ParamKey::PassGroupStart(1)), Some(true));
        assert_eq!(
            group_hidden(Some(&plan), ParamKey::GradientGroupStart(1)),
            Some(false)
        );
        #[cfg(feature = "editor")]
        assert_eq!(group_hidden(Some(&plan), ParamKey::GradientCanvas(1)), Some(false));
        assert_eq!(group_hidden(Some(&plan), ParamKey::GradientGroupEnd(1)), Some(false));
        assert_eq!(group_hidden(Some(&plan), ParamKey::GradientGroupStart(0)), Some(true));
    }

    #[test]
    fn pass_group_names_use_live_names_with_pf_fallbacks() {
        assert_eq!(pass_group_name(0, Some("blur_h")), "blur_h");
        assert_eq!(pass_group_name(1, None), "Pass 2");
        assert_eq!(pass_group_name(2, Some("   ")), "Pass 3");
        let long = "abcdefghijklmnopqrstuvwxyz0123456789";
        assert_eq!(pass_group_name(0, Some(long)), "abcdefghijklmnopqrstuvwxyz01234");
        assert_eq!(pass_group_name(0, Some(long)).chars().count(), 31);
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
        assert_eq!(stream_of(ParamKey::PlanToken), Some(plan_token_stream_index()));
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
