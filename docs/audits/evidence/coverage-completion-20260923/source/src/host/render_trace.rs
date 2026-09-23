use std::sync::{atomic::{AtomicU64, Ordering}, Mutex, OnceLock};

pub(crate) struct Span { id: u64, label: &'static str, frame: i32, scale: u32 }

pub(crate) fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("DYNAMICFX_RENDER_TRACE").is_ok_and(|v| v == "1"))
}

impl Span {
    pub(crate) unsafe fn enter(label: &'static str, cmd: after_effects::sys::PF_Cmd, input: *const after_effects::sys::PF_InData) -> Option<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        if cmd != after_effects::sys::PF_Cmd_SMART_RENDER as after_effects::sys::PF_Cmd || input.is_null() ||
            !enabled() { return None; }
        let span = Self { id: NEXT.fetch_add(1, Ordering::Relaxed), label, frame: unsafe { (*input).current_time }, scale: unsafe { (*input).time_scale } };
        span.write("begin");
        Some(span)
    }

    fn write(&self, event: &str) {
        static WRITER: Mutex<()> = Mutex::new(());
        let Ok(_guard) = WRITER.lock() else { return };
        let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        if let Ok(mut file) = std::fs::OpenOptions::new().append(true).create(true).open(std::env::temp_dir().join(format!("dynamicfx-render-trace-{}.log", self.label))) {
            use std::io::Write;
            let line = format!("{event}|{}|{}|{}|{:?}|{}|{}|{time}\n", self.label, std::process::id(), self.id, std::thread::current().id(), self.frame, self.scale);
            let _ = file.write_all(line.as_bytes());
        }
    }
}

impl Drop for Span { fn drop(&mut self) { self.write("end"); } }
