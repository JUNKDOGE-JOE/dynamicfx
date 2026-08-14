//! Debug diagnostics: append lines to %TEMP%\dynamicfx.log so we can see
//! what happens inside AE without a debugger attached.

use std::sync::{Mutex, OnceLock};

pub fn log(msg: &str) {
    // MFR (ADR-0023 §4): renders log from multiple threads; serialize
    // appends so lines never interleave mid-write.
    static WRITER: Mutex<()> = Mutex::new(());
    let _guard = WRITER.lock();
    let path = std::env::temp_dir().join("dynamicfx.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        use std::io::Write;
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = writeln!(f, "[{secs}] {msg}");
    }
}

/// Per-frame diagnostics are opt-in: opening and appending a file several
/// times per render materially slows previews, especially in large comps.
pub fn verbose(msg: &str) {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    if *ENABLED.get_or_init(|| {
        std::env::var("DYNAMICFX_VERBOSE_LOG")
            .ok()
            .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "on" | "ON"))
    }) {
        log(msg);
    }
}
