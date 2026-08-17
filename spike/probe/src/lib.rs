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

#[derive(Default)]
struct Global;

#[derive(Default)]
struct Instance {
    last_kb: Option<u64>,
    popup_mutated: bool,
}

ae::define_effect!(Global, Instance, Params);

impl AdobePluginGlobal for Global {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _: ae::InData,
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
        params.add_with_flags(
            Params::Blob,
            "Probe Blob",
            blob,
            ae::ParamFlag::CANNOT_TIME_VARY,
            ae::ParamUIFlags::empty(),
        )?;
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
        log("params setup: complete");
        Ok(())
    }

    fn handle_command(
        &mut self,
        command: ae::Command,
        _: ae::InData,
        _: ae::OutData,
        _: &mut ae::Parameters<Params>,
    ) -> Result<(), ae::Error> {
        if let ae::Command::ArbitraryCallback { mut extra } = command {
            log(&format!("ARB_CB fn={}", extra.which_function()));
            extra.dispatch::<ProbeBlob, _>(Params::Blob)?;
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
