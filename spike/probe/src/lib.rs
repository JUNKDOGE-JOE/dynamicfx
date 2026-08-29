#![allow(linker_messages)]
#![cfg_attr(cui_nil_seq, allow(dead_code))]

//! DynamicFx Probe — throwaway M0 transport-spike instrument (ADR-0009).
//!
//! Measures After Effects host behavior only; contains no DynamicFX
//! runtime code and is never shipped:
//!
//! - TR-M0-004: hidden arbitrary-data parameter payload capacity. A
//!   `Payload KB` slider drives a deterministic pattern blob written into an
//!   arbitrary parameter; flatten/unflatten/copy traffic and checksum
//!   verification are logged from every process that touches the instance.
//! - TR-M0-006: popup menu mutation after PARAMS_SETUP. A `Mutate Popup`
//!   checkbox triggers a namesptr/num_choices rewrite plus
//!   PF_UpdateParamUI; the log records whether the host accepts it.
//! - TR-M0-005 (plugin half): parameter rename + arbitrary writes happen in
//!   UserChangedParam commits, so scripted undo/redo around them shows how
//!   plugin-published state interacts with the undo stack.
//!
//! Evidence goes to `%TEMP%\dynamicfx_probe.log` as single-line records:
//!   [unix_secs pid=NNN] EVENT key=value ...

use after_effects as ae;
use ae::serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

const KB: usize = 1024;
const SEQ_MAGIC: u32 = 0x4446_5850; // "DFXP" — probe sequence payload marker

/// Sequence-payload size for flatten(), in KB, read once from the
/// `DFX_PROBE_KB` environment variable at process start. Driving this through
/// the environment (set by the driver before AfterFX launches) avoids the
/// broken link where scripted `setValue()` never reaches the plugin as a
/// committed parameter change, so flatten() can carry a controlled size.
fn payload_kb() -> u64 {
    static KBVAL: OnceLock<u64> = OnceLock::new();
    *KBVAL.get_or_init(|| {
        std::env::var("DFX_PROBE_KB")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(0)
    })
}

/// Process-level guard so the popup-mutation attempt runs exactly once per
/// AE session, independent of the (unreadable) checkbox parameter.
static POPUP_MUTATION_TRIED: AtomicBool = AtomicBool::new(false);

fn log(msg: &str) {
    let path = std::env::temp_dir().join("dynamicfx_probe.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        use std::io::Write;
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = writeln!(f, "[{secs} pid={}] {msg}", std::process::id());
    }
}

fn fnv1a(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(leg_u3)]
const U3_WRITE_MECHANISM: &str = "FloatSliderDef::set_value + ChangeFlag::CHANGED_VALUE";

#[cfg(leg_u3)]
fn u3_normalized_x(extra: &ae::EventExtra) -> f64 {
    let point = extra.screen_point();
    let frame = extra.current_frame();
    let width = frame.width().max(1);
    (f64::from(point.h - frame.left) / f64::from(width)).clamp(0.0, 1.0)
}

#[cfg(leg_u3)]
fn u3_write_sibling(
    params: &mut ae::Parameters<Params>,
    value: f64,
) -> Result<&'static str, ae::Error> {
    params
        .get_mut(Params::U3Sibling)?
        .as_float_slider_mut()?
        .set_value(value);
    Ok(U3_WRITE_MECHANISM)
}

#[cfg(leg_u4)]
fn u4_open_picker(params: &ae::Parameters<Params>) {
    let seed = match params.get(Params::U4Canvas) {
        Ok(param) => match param.as_color() {
            Ok(color) => color.value(),
            Err(error) => {
                log(&format!(
                    "U4_PICKER res=seed_error({error:?}) color=unavailable"
                ));
                return;
            }
        },
        Err(error) => {
            log(&format!(
                "U4_PICKER res=checkout_error({error:?}) color=unavailable"
            ));
            return;
        }
    };
    let scale = 1.0 / 255.0;
    let sample = ae::PixelF32 {
        alpha: f32::from(seed.alpha) * scale,
        red: f32::from(seed.red) * scale,
        green: f32::from(seed.green) * scale,
        blue: f32::from(seed.blue) * scale,
    };
    let result = ae::pf::suites::App::new().and_then(|suite| {
        suite.color_picker_dialog(Some("DynamicFx Probe U4"), &sample, true)
    });
    match result {
        Ok(color) => log(&format!(
            "U4_PICKER res=picked color=({:.6},{:.6},{:.6},{:.6})",
            color.red, color.green, color.blue, color.alpha
        )),
        Err(ae::Error::InterruptCancel) => log(&format!(
            "U4_PICKER res=cancelled color=({:.6},{:.6},{:.6},{:.6})",
            sample.red, sample.green, sample.blue, sample.alpha
        )),
        Err(error) => log(&format!(
            "U4_PICKER res=error({error:?}) color=({:.6},{:.6},{:.6},{:.6})",
            sample.red, sample.green, sample.blue, sample.alpha
        )),
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    /// Requested blob size in KB; 0 clears the blob.
    PayloadKb,
    /// Hidden arbitrary-data payload under test.
    Blob,
    /// Popup whose menu the mutation scenario tries to rewrite.
    PopupProbe,
    /// Checkbox that triggers the popup mutation attempt once.
    MutatePopup,
    /// Slider renamed with probe state, mirroring the prototype Status param.
    Status,
    MuteDraw,
    #[cfg(leg_u1)]
    UiCanvas,
    #[cfg(leg_u3)]
    U3Canvas,
    #[cfg(leg_u3)]
    U3Sibling,
    #[cfg(leg_u4)]
    U4Canvas,
    #[cfg(leg_u146)]
    U146Canvas,
    #[cfg(leg_u2b)]
    PodArb,
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(crate = "ae::serde")]
struct ProbeBlob {
    seed: u64,
    crc: u64,
    data: Vec<u8>,
}

impl ProbeBlob {
    fn generate(kb: u64) -> Self {
        let n = kb as usize * KB;
        let mut data = vec![0u8; n];
        for (i, b) in data.iter_mut().enumerate() {
            *b = ((i as u64).wrapping_mul(31).wrapping_add(kb) & 0xff) as u8;
        }
        Self { seed: kb, crc: fnv1a(&data), data }
    }

    fn verify(&self) -> bool {
        self.data.len() == self.seed as usize * KB && fnv1a(&self.data) == self.crc
    }
}

impl ae::ArbitraryData<ProbeBlob> for ProbeBlob {
    fn interpolate(&self, other: &ProbeBlob, value: f64) -> ProbeBlob {
        if value < 0.5 { self.clone() } else { other.clone() }
    }
}

#[cfg(leg_u2b)]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(crate = "ae::serde")]
struct PodBlob {
    vals: [f32; 8],
}

#[cfg(leg_u2b)]
impl ae::ArbitraryData<PodBlob> for PodBlob {
    fn interpolate(&self, other: &PodBlob, value: f64) -> PodBlob {
        let value = value as f32;
        let mut vals = [0.0; 8];
        for (result, (from, to)) in vals.iter_mut().zip(self.vals.iter().zip(other.vals.iter())) {
            *result = from + (to - from) * value;
        }
        PodBlob { vals }
    }
}

#[derive(Default)]
struct Global;

#[derive(Default)]
struct Instance {
    last_kb: Option<u64>,
    popup_mutated: bool,
}

#[cfg(not(cui_nil_seq))]
ae::define_effect!(Global, Instance, Params);
// Discriminant build: identical params and events, unit sequence type - the
// only variable separating the working upstream custom-UI examples from the
// crashing non-unit-sequence plug-ins.
#[cfg(cui_nil_seq)]
ae::define_effect!(Global, (), Params);

impl AdobePluginGlobal for Global {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _in_data: ae::InData,
        _: ae::OutData,
    ) -> Result<(), Error> {
        log("params setup: begin");
        params.add(Params::PayloadKb, "Payload KB", ae::FloatSliderDef::setup(|f| {
            f.set_slider_min(0.0);
            f.set_slider_max(4096.0);
            f.set_valid_min(0.0);
            f.set_valid_max(65536.0);
            f.set_precision(ae::Precision::Integer);
            f.set_default(0.0);
        }))?;
        let mut blob = ae::ArbitraryDef::new();
        blob.set_default(ProbeBlob::default())?;
        #[cfg(all(not(leg_u2), not(cui_legs)))]
        params.add_with_flags(
            Params::Blob,
            "Probe Blob",
            blob,
            ae::ParamFlag::CANNOT_TIME_VARY,
            ae::ParamUIFlags::empty(),
        )?;
        // An arb param with no drawn control is not a valid ECW row on AE 2025
        // (measured: modal "effect control not supported", wedging the bridge),
        // so CUI-leg builds keep the value but hide the row.
        #[cfg(all(not(leg_u2), cui_legs))]
        params.add_with_flags(
            Params::Blob,
            "Probe Blob",
            blob,
            ae::ParamFlag::CANNOT_TIME_VARY,
            ae::ParamUIFlags::NO_ECW_UI,
        )?;
        #[cfg(leg_u2)]
        params.add_customized(Params::Blob, "Probe Blob", blob, |param| {
            param.set_flags(ae::ParamFlag::CANNOT_TIME_VARY);
            param.set_ui_flags(
                ae::ParamUIFlags::CONTROL | ae::ParamUIFlags::DO_NOT_ERASE_CONTROL,
            );
            param.set_ui_width(200);
            param.set_ui_height(80);
            -1
        })?;
        params.add(Params::PopupProbe, "Popup Probe", ae::PopupDef::setup(|p| {
            p.set_options(&["Alpha", "Beta", "Gamma", "Delta"]);
            p.set_default(1);
        }))?;
        params.add(Params::MutatePopup, "Mutate Popup", ae::CheckBoxDef::setup(|c| {
            c.set_default(false);
        }))?;
        params.add(Params::Status, "Probe Status: idle", ae::FloatSliderDef::setup(|f| {
            f.set_slider_min(0.0);
            f.set_slider_max(1.0);
            f.set_valid_min(0.0);
            f.set_valid_max(1.0);
            f.set_default(0.0);
        }))?;
        params.add(
            Params::MuteDraw,
            "Mute Draw (U2a)",
            ae::CheckBoxDef::setup(|c| {
                c.set_default(false);
            }),
        )?;
        // Color, not Float: a Float's CONTROL canvas renders collapsed by
        // default (measured 2026-08-28) and expanding it by hand is what the
        // interactive legs must avoid; a Color+CONTROL canvas (the upstream
        // custom_ecw_ui shape) is visible immediately on apply.
        #[cfg(leg_u1)]
        params.add_customized(
            Params::UiCanvas,
            "U1 Std Canvas",
            ae::ColorDef::setup(|c| {
                c.set_default(ae::Pixel8 { alpha: 255, red: 224, green: 128, blue: 32 });
            }),
            |param| {
                param.set_ui_flags(ae::ParamUIFlags::CONTROL);
                param.set_ui_width(200);
                param.set_ui_height(80);
                -1
            },
        )?;
        #[cfg(leg_u3)]
        params.add_customized(
            Params::U3Canvas,
            "U3 Canvas",
            ae::ColorDef::setup(|c| {
                c.set_default(ae::Pixel8 {
                    alpha: 255,
                    red: 64,
                    green: 160,
                    blue: 224,
                });
            }),
            |param| {
                param.set_ui_flags(ae::ParamUIFlags::CONTROL);
                param.set_ui_width(200);
                param.set_ui_height(80);
                -1
            },
        )?;
        #[cfg(leg_u3)]
        params.add(
            Params::U3Sibling,
            "U3 Sibling",
            ae::FloatSliderDef::setup(|f| {
                f.set_slider_min(0.0);
                f.set_slider_max(1.0);
                f.set_valid_min(0.0);
                f.set_valid_max(1.0);
                f.set_default(0.0);
            }),
        )?;
        #[cfg(leg_u4)]
        params.add_customized(
            Params::U4Canvas,
            "U4 Canvas",
            ae::ColorDef::setup(|c| {
                c.set_default(ae::Pixel8 {
                    alpha: 255,
                    red: 192,
                    green: 96,
                    blue: 224,
                });
            }),
            |param| {
                param.set_ui_flags(ae::ParamUIFlags::CONTROL);
                param.set_ui_width(200);
                param.set_ui_height(80);
                -1
            },
        )?;
        #[cfg(leg_u146)]
        params.add_customized(
            Params::U146Canvas,
            "U146 Canvas",
            ae::ColorDef::setup(|c| {
                c.set_default(ae::Pixel8 {
                    alpha: 255,
                    red: 96,
                    green: 208,
                    blue: 128,
                });
            }),
            |param| {
                param.set_ui_flags(ae::ParamUIFlags::CONTROL);
                param.set_ui_width(200);
                param.set_ui_height(46);
                -1
            },
        )?;
        #[cfg(leg_u2b)]
        {
            let mut pod = ae::ArbitraryDef::new();
            pod.set_default(PodBlob::default())?;
            params.add_customized(Params::PodArb, "U2b POD Arb", pod, |param| {
                param.set_flags(ae::ParamFlag::CANNOT_TIME_VARY);
                param.set_ui_flags(
                    ae::ParamUIFlags::CONTROL | ae::ParamUIFlags::DO_NOT_ERASE_CONTROL,
                );
                param.set_ui_width(200);
                param.set_ui_height(80);
                -1
            })?;
        }
        // PF_REGISTER_UI is what makes AE deliver PF_Cmd_EVENT to effect-window
        // custom controls at all: every unregistered leg logged zero EVT lines
        // while upstream's registered samples drew and clicked on the same host
        // (measured 2026-08-28). DFX_PROBE_NO_REGUI=1 skips the call so one
        // byte-identical artifact serves both arms of that comparison.
        #[cfg(cui_legs)]
        {
            if std::env::var_os("DFX_PROBE_NO_REGUI").is_some() {
                log("REGISTER_UI skipped (DFX_PROBE_NO_REGUI)");
            } else {
                let res = _in_data
                    .interact()
                    .register_ui(ae::CustomUIInfo::new().events(ae::CustomEventFlags::EFFECT));
                log(&format!("REGISTER_UI res={res:?}"));
            }
        }
        log("params setup: complete");
        Ok(())
    }

    fn handle_command(
        &mut self,
        command: ae::Command,
        _: ae::InData,
        _: ae::OutData,
        params: &mut ae::Parameters<Params>,
    ) -> Result<(), ae::Error> {
        if let ae::Command::ArbitraryCallback { mut extra } = command {
            log(&format!("ARB_CB fn={}", extra.which_function()));
            extra.dispatch::<ProbeBlob, _>(Params::Blob)?;
            #[cfg(leg_u2b)]
            extra.dispatch::<PodBlob, _>(Params::PodArb)?;
            return Ok(());
        }
        if let ae::Command::Event { mut extra } = command {
            let event = extra.event();
            let event_name = match event {
                ae::Event::None => "None",
                ae::Event::NewContext => "NewContext",
                ae::Event::Activate => "Activate",
                ae::Event::Click(_) => "Click",
                ae::Event::Drag(_) => "Drag",
                ae::Event::Draw(_) => "Draw",
                ae::Event::Deactivate => "Deactivate",
                ae::Event::CloseContext => "CloseContext",
                ae::Event::Idle => "Idle",
                ae::Event::AdjustCursor(_) => "AdjustCursor",
                ae::Event::Keydown(_) => "Keydown",
                ae::Event::MouseExited => "MouseExited",
            };
            log(&format!("EVT {event_name}"));

            match event {
                ae::Event::Draw(_) if extra.effect_area() == ae::EffectArea::Control => {
                    let param_index = extra.param_index();
                    let muted = params.get(Params::MuteDraw)?.as_checkbox()?.value();
                    if muted {
                        log(&format!("DRAW muted idx={param_index}"));
                        return Ok(());
                    }

                    log(&format!("DRAW begin idx={param_index}"));
                    let drawbot = extra.context_handle().drawing_reference()?;
                    log("DRAW drawbot_acquired");
                    let surface = drawbot.surface()?;
                    let frame = extra.current_frame();
                    let rect = ae::drawbot::RectF32 {
                        left: frame.left as f32,
                        top: frame.top as f32,
                        width: frame.width() as f32,
                        height: frame.height() as f32,
                    };
                    let color = match param_index % 3 {
                        0 => ae::drawbot::ColorRgba {
                            red: 0.85,
                            green: 0.15,
                            blue: 0.15,
                            alpha: 1.0,
                        },
                        1 => ae::drawbot::ColorRgba {
                            red: 0.15,
                            green: 0.70,
                            blue: 0.20,
                            alpha: 1.0,
                        },
                        _ => ae::drawbot::ColorRgba {
                            red: 0.15,
                            green: 0.30,
                            blue: 0.90,
                            alpha: 1.0,
                        },
                    };
                    surface.paint_rect(&color, &rect)?;
                    log("DRAW painted");
                    extra.set_event_out_flags(ae::EventOutFlags::HANDLED_EVENT);
                }
                // Click/Drag measure the two facts the 2026-08-15 editor relied on
                // without host evidence: drag tracking via send_drag, and whether
                // screen_point shares current_frame's coordinate space.
                ae::Event::Click(_) => {
                    let idx = extra.param_index();
                    if extra.effect_area() == ae::EffectArea::Control {
                        #[cfg(leg_u3)]
                        if Some(idx) == params.index(Params::U3Canvas) {
                            let value = u3_normalized_x(&extra);
                            let res = u3_write_sibling(params, value);
                            log(&format!("U3_CLICK_WRITE value={value:.6} res={res:?}"));
                        }
                        #[cfg(leg_u4)]
                        if Some(idx) == params.index(Params::U4Canvas) {
                            u4_open_picker(params);
                        }
                        let sp = extra.screen_point();
                        let cf = extra.current_frame();
                        extra.set_send_drag(true);
                        extra.set_event_out_flags(ae::EventOutFlags::HANDLED_EVENT);
                        log(&format!(
                            "CLICK idx={idx} sp=({},{}) frame=({},{},{},{})",
                            sp.h, sp.v, cf.left, cf.top, cf.right, cf.bottom
                        ));
                    } else {
                        log(&format!("EVT Click idx={idx} area=title"));
                    }
                }
                ae::Event::Drag(_) => {
                    let idx = extra.param_index();
                    let sp = extra.screen_point();
                    let last = extra.last_time();
                    #[cfg(leg_u3)]
                    if last
                        && extra.effect_area() == ae::EffectArea::Control
                        && Some(idx) == params.index(Params::U3Canvas)
                    {
                        let value = u3_normalized_x(&extra);
                        let res = u3_write_sibling(params, value);
                        log(&format!("U3_DRAG_WRITE value={value:.6} res={res:?}"));
                    }
                    extra.set_send_drag(true);
                    extra.set_event_out_flags(ae::EventOutFlags::HANDLED_EVENT);
                    log(&format!(
                        "DRAG idx={} sp=({},{}) last={}",
                        idx, sp.h, sp.v, last
                    ));
                }
                _ => {}
            }
            return Ok(());
        }
        if matches!(command, ae::Command::GlobalSetup) {
            log(&format!("GLOBAL_SETUP probe v{}", env!("CARGO_PKG_VERSION")));
        }
        Ok(())
    }
}

impl Instance {
    fn read_kb(plugin: &mut PluginState) -> u64 {
        match plugin.params.get(Params::PayloadKb) {
            Ok(p) => match p.as_param() {
                Ok(ae::Param::FloatSlider(f)) => f.value().round().max(0.0) as u64,
                _ => 0,
            },
            Err(_) => 0,
        }
    }

    fn read_blob_state(plugin: &mut PluginState) -> String {
        match plugin.params.get(Params::Blob) {
            Ok(p) => match p.as_param() {
                Ok(ae::Param::Arbitrary(a)) => match a.value::<ProbeBlob>() {
                    Ok(b) => format!("kb={} bytes={} ok={}", b.seed, b.data.len(), b.verify()),
                    Err(e) => format!("read_err={e:?}"),
                },
                _ => "not_arb".to_string(),
            },
            Err(e) => format!("checkout_err={e:?}"),
        }
    }

    fn sync(&mut self, plugin: &mut PluginState, commit: bool) {
        let kb = Self::read_kb(plugin);
        #[cfg(leg_u3)]
        match plugin.params.get(Params::U3Sibling) {
            Ok(param) => match param.as_float_slider() {
                Ok(slider) => log(&format!(
                    "U3_READ value={:.6} commit={commit}",
                    slider.value()
                )),
                Err(error) => log(&format!(
                    "U3_READ value=unavailable commit={commit} res={error:?}"
                )),
            },
            Err(error) => log(&format!(
                "U3_READ value=unavailable commit={commit} res={error:?}"
            )),
        }
        // Record kb unconditionally on commit so flatten() carries the
        // intended sequence-payload size. The arb-blob write below is a
        // separate (negative) probe of the ParamDef write path and must not
        // gate last_kb.
        if commit {
            self.last_kb = Some(kb);
        }

        if commit {
            let blob = ProbeBlob::generate(kb);
            let (bytes, crc) = (blob.data.len(), blob.crc);
            match plugin.params.get_mut(Params::Blob) {
                Ok(mut p) => {
                    let res = match p.as_param_mut() {
                        Ok(ae::Param::Arbitrary(mut a)) => a.set_value(blob).map(|_| ()),
                        Ok(_) => Err(ae::Error::BadCallbackParameter),
                        Err(e) => Err(e),
                    };
                    log(&format!("BLOB_SET kb={kb} bytes={bytes} crc={crc:016x} res={res:?}"));
                }
                Err(e) => log(&format!("BLOB_SET kb={kb} checkout_err={e:?}")),
            }
        }

        log(&format!("BLOB_UI_READ {}", Self::read_blob_state(plugin)));

        // Attempt the popup mutation once per process, not gated on the
        // checkbox parameter (scripted setValue never reaches the plugin as a
        // committed change, so the checkbox reads stale). This isolates the
        // host question: does set_options + PF_UpdateParamUI at runtime grow
        // the menu declared at PARAMS_SETUP?
        let mutate = !POPUP_MUTATION_TRIED.load(Ordering::Relaxed);
        if mutate {
            POPUP_MUTATION_TRIED.store(true, Ordering::Relaxed);
            log(&format!("MUTATE_TRY commit={commit}"));
        }
        if mutate && !self.popup_mutated {
            match plugin.params.get_mut(Params::PopupProbe) {
                Ok(mut pdef) => {
                    let value_before = match pdef.as_param_mut() {
                        Ok(ae::Param::Popup(p)) => {
                            let v = p.value();
                            // Reading options() would dereference namesptr,
                            // which the checkout may leave null; value only.
                            std::mem::forget(p);
                            v
                        }
                        _ => -1,
                    };
                    let rewrote = match pdef.as_param_mut() {
                        Ok(ae::Param::Popup(mut p)) => {
                            p.set_options(&["Ren-A", "Ren-B", "Ren-C", "Ren-D", "Extra-E"]);
                            // Deliberate leak: keeps the options CString (and
                            // therefore namesptr) alive for the host's later
                            // reads. A throwaway probe pays ~64 bytes once.
                            std::mem::forget(p);
                            true
                        }
                        Ok(_) => false,
                        Err(_) => false,
                    };
                    let name_res = pdef.set_name("Popup Probe MUTATED");
                    let ui_res = pdef.update_param_ui();
                    log(&format!(
                        "POPUP_MUTATE value_before={value_before} rewrote={rewrote} name={name_res:?} ui={ui_res:?}"
                    ));
                    self.popup_mutated = true;
                }
                Err(e) => log(&format!("POPUP_MUTATE checkout_err={e:?}")),
            }
        }

        let label = format!("Probe: kb={kb} mut={}", self.popup_mutated);
        if let Ok(mut p) = plugin.params.get_mut(Params::Status) {
            let name_res = p.set_name(&label);
            let ui_res = p.update_param_ui();
            log(&format!("STATUS_RENAME name={name_res:?} ui={ui_res:?}"));
        }
    }
}

impl AdobePluginInstance for Instance {
    fn flatten(&self) -> Result<(u16, Vec<u8>), Error> {
        // Carry a kb-sized checksummed payload so save/reopen exercises the
        // sequence transport at a controlled size (TR-M0-004). Sequence
        // flatten — not the arb parameter value — is the architecture's
        // primary persistence carrier (sequence schema v1).
        let kb = payload_kb();
        let n = kb as usize * KB;
        let mut body = vec![0u8; n];
        for (i, b) in body.iter_mut().enumerate() {
            *b = ((i as u64).wrapping_mul(31).wrapping_add(kb) & 0xff) as u8;
        }
        let crc = fnv1a(&body);
        let mut data = Vec::with_capacity(16 + n);
        data.extend_from_slice(&SEQ_MAGIC.to_le_bytes());
        data.extend_from_slice(&(kb as u32).to_le_bytes());
        data.extend_from_slice(&crc.to_le_bytes());
        data.extend_from_slice(&body);
        log(&format!(
            "SEQ_FLATTEN env_kb={kb} param_last_kb={:?} total_bytes={} crc={crc:016x}",
            self.last_kb,
            data.len()
        ));
        Ok((1, data))
    }

    fn unflatten(version: u16, bytes: &[u8]) -> Result<Self, Error> {
        let mut inst = Self::default();
        if bytes.len() >= 16 {
            let magic = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
            let kb = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
            let crc = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
            let body = &bytes[16..];
            let len_ok = body.len() == kb as usize * KB;
            let crc_ok = fnv1a(body) == crc;
            let magic_ok = magic == SEQ_MAGIC;
            log(&format!(
                "SEQ_UNFLATTEN version={version} kb={kb} body_bytes={} magic_ok={magic_ok} len_ok={len_ok} crc_ok={crc_ok}",
                body.len()
            ));
            if magic_ok && len_ok && crc_ok {
                inst.last_kb = Some(kb as u64);
            }
        } else {
            log(&format!("SEQ_UNFLATTEN version={version} bytes={} (empty/legacy)", bytes.len()));
        }
        Ok(inst)
    }

    fn render(
        &self,
        plugin: &mut PluginState,
        in_layer: &ae::Layer,
        out_layer: &mut ae::Layer,
    ) -> Result<(), ae::Error> {
        log(&format!("RENDER_READ {}", Self::read_blob_state(plugin)));
        // Passthrough only 8/16-bpc; any other depth (e.g. 32-bpc float) is
        // left untouched rather than erroring. A throwaway probe must never
        // turn an unsupported render format into a modal host error that
        // blocks the driving script.
        let extent = plugin.in_data.extent_hint();
        let res = in_layer.iterate_with(
            out_layer,
            0,
            extent.height(),
            Some(extent),
            |_x: i32, _y: i32, pixel: ae::GenericPixel, out_pixel: ae::GenericPixelMut| -> Result<(), Error> {
                match (pixel, out_pixel) {
                    (ae::GenericPixel::Pixel8(p), ae::GenericPixelMut::Pixel8(o)) => *o = *p,
                    (ae::GenericPixel::Pixel16(p), ae::GenericPixelMut::Pixel16(o)) => *o = *p,
                    _ => {} // unsupported depth: leave output as-is, never error
                }
                Ok(())
            },
        );
        if let Err(e) = res {
            log(&format!("RENDER_ITER_SKIP err={e:?}"));
        }
        Ok(())
    }

    fn handle_command(&mut self, plugin: &mut PluginState, command: ae::Command) -> Result<(), Error> {
        match command {
            ae::Command::SequenceSetup => log("SEQ_SETUP"),
            ae::Command::SequenceResetup => log("SEQ_RESETUP"),
            ae::Command::SequenceSetdown => log("SEQ_SETDOWN"),
            command @ (ae::Command::UserChangedParam { .. } | ae::Command::UpdateParamsUi) => {
                let commit = matches!(command, ae::Command::UserChangedParam { .. });
                self.sync(plugin, commit);
            }
            _ => {}
        }
        Ok(())
    }
}
