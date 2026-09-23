use after_effects::sys;

pub unsafe fn supported(basic: *const sys::SPBasicSuite) -> bool {
    cfg!(coverage_stage_sdk) && suite_available(basic)
}

unsafe fn suite_available(basic: *const sys::SPBasicSuite) -> bool {
    if basic.is_null() { return false; }
    let (Some(acquire), Some(release)) = ((*basic).AcquireSuite, (*basic).ReleaseSuite) else { return false };
    let mut table = std::ptr::null();
    // StreamSuite7 is SDK 26.5 version 12. Capability probing never dereferences its table.
    if acquire(sys::kAEGPStreamSuite.as_ptr().cast(), 12, &mut table) != 0 { return false; }
    let found = !table.is_null();
    release(sys::kAEGPStreamSuite.as_ptr().cast(), 12) == 0 && found
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage { Source, Masks, Effects }

impl Stage {
    fn value(self) -> i32 {
        match self { Self::Source => 0, Self::Masks => -2, Self::Effects => -1 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Binding { pub layer: i32, pub stage: i32 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingError { BridgeUnavailable, WrongThread, InvalidArgument, Host(i32) }

pub struct StageAccess<'a> {
    basic: &'a sys::SPBasicSuite,
    plugin_id: i32,
    _main_thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

#[cfg(coverage_stage_sdk)]
extern "C" {
    fn dynamicfx_coverage_binding(basic: *const sys::SPBasicSuite, id: i32,
        effect: sys::AEGP_EffectRefH, index: i32, write: i32, layer: i32, stage: i32,
        actual_layer: *mut i32, actual_stage: *mut i32, changed: *mut i32) -> i32;
}

impl<'a> StageAccess<'a> {
    /// The suite must belong to the live host and `main_thread` must be the
    /// thread recorded at GlobalSetup. The guard cannot cross threads.
    pub unsafe fn new(basic: &'a sys::SPBasicSuite, plugin_id: i32,
        main_thread: std::thread::ThreadId) -> Result<Self, BindingError> {
        if main_thread != std::thread::current().id() { return Err(BindingError::WrongThread); }
        if !cfg!(coverage_stage_sdk) { return Err(BindingError::BridgeUnavailable); }
        if basic.AcquireSuite.is_none() || basic.ReleaseSuite.is_none() {
            return Err(BindingError::InvalidArgument);
        }
        Ok(Self { basic, plugin_id, _main_thread: std::marker::PhantomData })
    }

    /// `effect` must be a live main-thread effect reference in this host.
    pub unsafe fn read(&self, effect: sys::AEGP_EffectRefH, index: i32) -> Result<Binding, BindingError> {
        self.call(effect, index, None).map(|(binding, _)| binding)
    }

    /// Call inside the manager's undo group, after ownership has been checked.
    pub unsafe fn set(&self, effect: sys::AEGP_EffectRefH, index: i32,
        layer: i32, stage: Stage) -> Result<bool, BindingError> {
        self.call(effect, index, Some(Binding { layer, stage: stage.value() })).map(|(_, changed)| changed)
    }

    unsafe fn call(&self, effect: sys::AEGP_EffectRefH, index: i32,
        desired: Option<Binding>) -> Result<(Binding, bool), BindingError> {
        if effect.is_null() || index <= 0 { return Err(BindingError::InvalidArgument); }
        #[cfg(coverage_stage_sdk)]
        {
            let mut actual = Binding { layer: 0, stage: 0 };
            let mut changed = 0;
            let value = desired.unwrap_or(Binding { layer: 0, stage: 0 });
            let error = dynamicfx_coverage_binding(self.basic, self.plugin_id, effect, index,
                i32::from(desired.is_some()), value.layer, value.stage,
                &mut actual.layer, &mut actual.stage, &mut changed);
            if error != 0 { return Err(BindingError::Host(error)); }
            Ok((actual, changed != 0))
        }
        #[cfg(not(coverage_stage_sdk))]
        {
            let _ = (self.basic, self.plugin_id, desired);
            Err(BindingError::BridgeUnavailable)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static RELEASES: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn acquire(name: *const i8, version: i32, table: *mut *const std::ffi::c_void) -> i32 {
        if version != 12 || std::ffi::CStr::from_ptr(name).to_bytes() != b"AEGP Stream Suite" { return 1; }
        *table = std::ptr::dangling::<u8>().cast();
        0
    }
    unsafe extern "C" fn release(_: *const i8, _: i32) -> i32 {
        RELEASES.fetch_add(1, Ordering::SeqCst);
        0
    }
    #[test]
    fn probing_checks_callbacks_and_balances_the_suite() {
        unsafe {
            assert!(!suite_available(std::ptr::null()));
            let mut basic: sys::SPBasicSuite = std::mem::zeroed();
            assert!(!suite_available(&basic));
            basic.AcquireSuite = Some(acquire);
            assert!(!suite_available(&basic));
            basic.ReleaseSuite = Some(release);
            assert!(suite_available(&basic));
            assert_eq!(RELEASES.load(Ordering::SeqCst), 1);
        }
    }

    #[test]
    fn stage_guard_refuses_a_different_thread_before_host_access() {
        let other = std::thread::spawn(|| std::thread::current().id()).join().unwrap();
        let basic = unsafe { std::mem::zeroed() };
        assert!(matches!(unsafe { StageAccess::new(&basic, 1, other) }, Err(BindingError::WrongThread)));
    }

    #[test]
    fn stage_guard_requires_the_compiled_bridge_and_live_callbacks() {
        let basic = unsafe { std::mem::zeroed() };
        let result = unsafe { StageAccess::new(&basic, 1, std::thread::current().id()) };
        let expected = if cfg!(coverage_stage_sdk) { BindingError::InvalidArgument } else { BindingError::BridgeUnavailable };
        assert!(matches!(result, Err(error) if error == expected));
    }
}
