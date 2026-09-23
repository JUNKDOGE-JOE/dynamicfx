//! AE-facing host layer. Everything that touches the AE SDK lives here or in
//! `lib.rs`; the `definition`/`frontend`/`binding` domain layers stay
//! host-agnostic by policy (CLAUDE.md).

pub(crate) mod callback;
pub(crate) mod coverage;
pub(crate) mod entry;
pub mod idle;
pub mod params;

/// ADR-0028: modal info dialog for the Details button. Win32 directly (no
/// crate dependency); TASKMODAL so AE's own windows are blocked while the
/// message is up, exactly like AE's native alerts. UI-command contexts only.
#[cfg(target_os = "windows")]
pub fn show_info_dialog(title: &str, text: &str) {
    #[link(name = "user32")]
    extern "system" {
        fn MessageBoxW(
            hwnd: *mut core::ffi::c_void,
            text: *const u16,
            caption: *const u16,
            utype: u32,
        ) -> i32;
    }
    let wide = |s: &str| -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() };
    let text_w = wide(text);
    let title_w = wide(title);
    const MB_OK: u32 = 0x0000_0000;
    const MB_ICONINFORMATION: u32 = 0x0000_0040;
    const MB_TASKMODAL: u32 = 0x0000_2000;
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text_w.as_ptr(),
            title_w.as_ptr(),
            MB_OK | MB_ICONINFORMATION | MB_TASKMODAL,
        );
    }
}

/// Use the host's Unicode dialog on macOS; linking Win32 user32 prevents
/// even loading a native ARM plugin. Called only from the Details UI event.
#[cfg(not(target_os = "windows"))]
pub fn show_info_dialog(
    plugin_id: after_effects::aegp::PluginId,
    text: &str,
) -> Result<(), after_effects::Error> {
    after_effects::aegp::suites::Utility::new()?.report_info_unicode(plugin_id, text)
}
