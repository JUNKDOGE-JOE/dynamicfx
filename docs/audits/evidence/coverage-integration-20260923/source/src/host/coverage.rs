use after_effects::sys;

pub unsafe fn supported(basic: *const sys::SPBasicSuite) -> bool {
    if basic.is_null() { return false; }
    let (Some(acquire), Some(release)) = ((*basic).AcquireSuite, (*basic).ReleaseSuite) else { return false };
    let mut table = std::ptr::null();
    // StreamSuite7 is SDK 26.5 version 12. Capability probing never dereferences its table.
    if acquire(sys::kAEGPStreamSuite.as_ptr().cast(), 12, &mut table) != 0 { return false; }
    let found = !table.is_null();
    release(sys::kAEGPStreamSuite.as_ptr().cast(), 12) == 0 && found
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
            assert!(!supported(std::ptr::null()));
            let mut basic: sys::SPBasicSuite = std::mem::zeroed();
            assert!(!supported(&basic));
            basic.AcquireSuite = Some(acquire);
            assert!(!supported(&basic));
            basic.ReleaseSuite = Some(release);
            assert!(supported(&basic));
            assert_eq!(RELEASES.load(Ordering::SeqCst), 1);
        }
    }
}
