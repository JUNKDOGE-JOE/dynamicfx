use after_effects as ae;
use std::sync::{LockResult, Mutex, MutexGuard};

const LIVE_TAG: [u8; 8] = *b"DFXLIVE1";

// This tag describes an in-memory allocation only. Flattening still writes
// the existing version + DFXS snapshot, never the mutex or this header.
#[repr(C)]
pub(crate) struct LiveInstance<T> {
    tag: [u8; 8],
    value: Mutex<T>,
}

impl<T> LiveInstance<T> {
    pub(crate) fn new(value: T) -> Self {
        Self { tag: LIVE_TAG, value: Mutex::new(value) }
    }

    pub(crate) fn lock(&self) -> LockResult<MutexGuard<'_, T>> {
        self.value.lock()
    }

    #[cfg(test)]
    pub(crate) fn try_lock(&self) -> std::sync::TryLockResult<MutexGuard<'_, T>> {
        self.value.try_lock()
    }
}

impl<T: Default> Default for LiveInstance<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

fn is_live_header(size: usize, header: &[u8]) -> bool {
    size == std::mem::size_of::<crate::LocalMutex>() && header == LIVE_TAG
}

unsafe fn general_call_ready(in_data: *mut ae::sys::PF_InData) -> bool {
    if in_data.is_null() {
        return false;
    }
    let handle = unsafe { (*in_data).sequence_data };
    if handle.is_null() {
        return false;
    }
    let _pica = ae::PicaBasicSuite::from_pf_in_data_raw(in_data);
    let Ok(handles) = ae::pf::suites::Handle::new() else { return false };
    let size = handles.handle_size(handle) as usize;
    if size != std::mem::size_of::<crate::LocalMutex>() {
        return false;
    }
    // AE locks input handles for the callback. Read only the byte header;
    // forming a LocalMutex reference here would already be invalid for flat data.
    let data = unsafe { *handle } as *const u8;
    if data.is_null() {
        return false;
    }
    is_live_header(size, unsafe { std::slice::from_raw_parts(data, LIVE_TAG.len()) })
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn DynamicFxMain(
    cmd: ae::sys::PF_Cmd,
    in_data: *mut ae::sys::PF_InData,
    out_data: *mut ae::sys::PF_OutData,
    params: *mut *mut ae::sys::PF_ParamDef,
    output: *mut ae::sys::PF_LayerDef,
    extra: *mut std::ffi::c_void,
) -> ae::sys::PF_Err {
    // Idle can reach an effect before SequenceResetup during BEE_LoadProject.
    // The upstream dispatcher casts sequence_data before any instance callback.
    if cmd == ae::sys::PF_Cmd_COMPLETELY_GENERAL as ae::sys::PF_Cmd
        && !unsafe { general_call_ready(in_data) }
    {
        return ae::sys::PF_Err_NONE as ae::sys::PF_Err;
    }
    unsafe { crate::EffectMain(cmd, in_data, out_data, params, output, extra) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_sequence_bytes_never_qualify_as_a_live_mutex() {
        let size = std::mem::size_of::<crate::LocalMutex>();
        let mut flat = vec![0u8; size];
        flat[..8].copy_from_slice(&[1, 0, b'D', b'F', b'X', b'S', 1, 0]);
        let before = flat.clone();
        assert!(!is_live_header(size, &flat[..8]));
        assert_eq!(flat, before);
        assert!(!is_live_header(2, &[1, 0]));
        flat[6] = 255;
        assert!(!is_live_header(size, &flat[..8]));
        assert!(!is_live_header(size, &[0; 8]));
    }

    #[test]
    fn live_tag_is_at_the_checked_offset_and_requires_the_whole_allocation() {
        let instance = crate::LocalMutex::default();
        let header = unsafe {
            std::slice::from_raw_parts(std::ptr::from_ref(&instance).cast::<u8>(), 8)
        };
        let size = std::mem::size_of_val(&instance);
        assert!(is_live_header(size, header));
        assert!(!is_live_header(size - 1, header));
        assert!(!is_live_header(size + 1, header));
        assert!(instance.lock().is_ok());
    }
}
