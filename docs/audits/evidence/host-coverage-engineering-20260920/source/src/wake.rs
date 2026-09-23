use after_effects::{self as ae, sys};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::Duration;

type Wake = unsafe extern "C" fn() -> sys::A_Err;

pub struct Pump {
    basic: *mut sys::SPBasicSuite,
    alive: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

fn run(alive: &AtomicBool, mut wake: impl FnMut() -> i32, mut wait: impl FnMut()) {
    while alive.load(Ordering::Acquire) {
        wait();
        if !alive.load(Ordering::Acquire) { break; }
        let error = wake();
        if error != 0 {
            super::log(&format!("IDLE_WAKE_FAILED code={error}"));
            break;
        }
    }
}

impl Pump {
    pub fn start(basic: *mut sys::SPBasicSuite, alive: Arc<AtomicBool>) -> Result<Self, ae::Error> {
        if basic.is_null() { return Err(ae::Error::InvalidCallback); }
        let mut suite = std::ptr::null();
        let error = unsafe {
            ((*basic).AcquireSuite.ok_or(ae::Error::MissingSuite)?)(
                sys::kAEGPUtilitySuite.as_ptr().cast(), sys::kAEGPUtilitySuiteVersion6 as i32,
                &mut suite)
        };
        if error != 0 || suite.is_null() { return Err(ae::Error::MissingSuite); }
        let mut pump = Self { basic, alive, worker: None };
        let wake: Wake = unsafe { (*(suite.cast::<sys::AEGP_UtilitySuite6>()))
            .AEGP_CauseIdleRoutinesToBeCalled.ok_or(ae::Error::MissingSuite)? };
        let alive = pump.alive.clone();
        // This SDK function alone is thread-safe. Acquire/release the suite on
        // the host thread; the worker never reads a project or enters render.
        pump.worker = Some(thread::Builder::new().name("dynamicfx-idle-wake".into())
            .spawn(move || run(&alive, || unsafe { wake() },
                || thread::park_timeout(Duration::from_millis(500))))
            .map_err(|_| ae::Error::OutOfMemory)?);
        Ok(pump)
    }
}

impl Drop for Pump {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
            if worker.join().is_err() { super::log("IDLE_WAKE_WORKER_PANICKED"); }
        }
        // Join before releasing the suite so the cached function cannot outlive it.
        let error = unsafe { ((*self.basic).ReleaseSuite.unwrap())(
            sys::kAEGPUtilitySuite.as_ptr().cast(), sys::kAEGPUtilitySuiteVersion6 as i32) };
        if error != 0 { super::log(&format!("IDLE_WAKE_RELEASE_FAILED code={error}")); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_during_wait_prevents_another_host_call() {
        let alive = AtomicBool::new(true);
        run(&alive, || panic!("host called after shutdown"),
            || alive.store(false, Ordering::Release));
    }

    #[test]
    fn host_error_stops_waking_without_retries() {
        let alive = AtomicBool::new(true);
        let mut calls = 0;
        run(&alive, || { calls += 1; -1 }, || {});
        assert_eq!(calls, 1);
    }
}
