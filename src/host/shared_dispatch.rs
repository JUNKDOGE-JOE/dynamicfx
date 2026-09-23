// Adapted from after-effects 0.4.0, src/plugin_base.rs, under the MIT license.
// Package authors: Adrian Eddy and Moritz Moeller (upstream Cargo metadata).
// The complete license is in shared_dispatch.LICENSE.
// Shared callback state must never become an exclusive Rust reference: AE can
// invoke render callbacks concurrently even when its handle API calls this a lock.
#[macro_export]
macro_rules! define_shared_effect {
    ($global_type:ty, $sequence_type:tt, $params_type:ty) => {
        use after_effects::*;
        use std::collections::HashMap;

        struct PluginState<'main, 'global, 'params> {
            global: &'global $global_type,
            params: &'params mut after_effects::Parameters<'main, $params_type>,
            in_data: after_effects::InData,
            out_data: after_effects::OutData
        }

        // This struct **must** be thread safe
        struct GlobalData {
            params_map: std::sync::OnceLock<HashMap<$params_type, after_effects::ParamMapInfo>>,
            params_num: std::sync::atomic::AtomicUsize,
            plugin_instance: $global_type
        }

        trait AdobePluginGlobal : Default {
            fn params_setup(&self, params: &mut Parameters<$params_type>, in_data: InData, out_data: OutData) -> Result<(), Error>;

            fn handle_command(&self, command: Command, in_data: InData, out_data: OutData, params: &mut Parameters<$params_type>) -> Result<(), Error>;
        }
        trait AdobePluginInstance : Default {
            fn flatten(&self) -> Result<(u16, Vec<u8>), Error>;
            fn unflatten(version: u16, serialized: &[u8]) -> Result<Self, Error>;

            fn render(&self, plugin: &mut PluginState, in_layer: &Layer, out_layer: &mut Layer) -> Result<(), ae::Error>;

            #[cfg(does_dialog)]
            fn do_dialog(&self, plugin: &mut PluginState) -> Result<(), ae::Error>;

            fn handle_command(&self, plugin: &mut PluginState, command: Command) -> Result<(), Error>;
        }
        impl AdobePluginInstance for () {
            fn flatten(&self) -> Result<(u16, Vec<u8>), Error> { Ok((0, Vec::new())) }
            fn unflatten(_: u16, _: &[u8]) -> Result<Self, Error> { Ok(Default::default()) }
            fn render(&self, _: &mut PluginState, _: &Layer, _: &mut Layer) -> Result<(), ae::Error> { Ok(()) }
            fn handle_command(&self, _: &mut PluginState, _: Command) -> Result<(), Error> { Ok(()) }

            #[cfg(does_dialog)]
            fn do_dialog(&self, _: &mut PluginState) -> Result<(), ae::Error> { Ok(()) }
        }

        fn get_sequence_handle<'a, S: AdobePluginInstance>(cmd: RawCommand, in_data: &InData) -> Result<Option<(pf::Handle::<'a, S>, bool)>, Error> {
            // Sequence data is not available during these commands:
            const EXCLUDES: &[RawCommand] = &[RawCommand::GlobalSetup, RawCommand::GlobalSetdown, RawCommand::GpuDeviceSetup, RawCommand::GpuDeviceSetdown, RawCommand::ArbitraryCallback];
            if EXCLUDES.contains(&cmd) {
                return Ok(None);
            }
            Ok(if std::any::type_name::<S>() == "()" {
                // Don't allocate sequence data
                None
            } else if cmd == RawCommand::SequenceSetup {
                // Allocate new sequence data
                Some((pf::Handle::new(S::default())?, true))
            } else if cmd == RawCommand::SequenceResetup {
                // Restore from flat handle
                if unsafe { (*in_data.as_ptr()).sequence_data.is_null() } {
                    Some((pf::Handle::new(S::default())?, true))
                } else {
                    let instance = FlatHandle::from_raw(unsafe { (*in_data.as_ptr()).sequence_data as after_effects::sys::PF_Handle })?;
                    let bytes = instance.as_slice().ok_or(Error::InvalidIndex)?;
                    if bytes.len() < 2 {
                        return Ok(None);
                    }
                    let version = u16::from_le_bytes(bytes[0..2].try_into().unwrap());

                    let handle = pf::Handle::new(S::unflatten(version, &bytes[2..]).map_err(|_| Error::Struct)?)?;
                    Some((handle, true))
                }
            } else if unsafe { (*in_data.as_ptr()).sequence_data.is_null() } {
                // Read-only sequence data available through a suite only
                let seq_ptr = in_data.effect().const_sequence_data().unwrap_or(unsafe { (*in_data.as_ptr()).sequence_data as *const _ });
                if !seq_ptr.is_null() {
                    let instance_handle = pf::Handle::<S>::from_raw(seq_ptr as *mut _, false)?;
                    Some((instance_handle, false))
                } else {
                    after_effects::log::error!("Sequence data pointer got through EffectSequenceDataSuite is null in cmd: {:?}!", cmd);
                    None
                }
            } else {
                let should_dispose_sequence = cmd == RawCommand::SequenceSetdown || cmd == RawCommand::SequenceFlatten;
                let instance_handle = pf::Handle::<S>::from_raw(unsafe { (*in_data.as_ptr()).sequence_data }, should_dispose_sequence)?;
                Some((instance_handle, false))
            })
        }

        fn handle_effect_main<T: AdobePluginGlobal, S: AdobePluginInstance, P>(
            cmd: after_effects::sys::PF_Cmd,
            in_data_ptr: *mut after_effects::sys::PF_InData,
            out_data_ptr: *mut after_effects::sys::PF_OutData,
            params: *mut *mut after_effects::sys::PF_ParamDef,
            output: *mut after_effects::sys::PF_LayerDef,
            extra: *mut std::ffi::c_void) -> Result<(), Error>
        {
            let _pica = after_effects::PicaBasicSuite::from_pf_in_data_raw(in_data_ptr);

            let in_data = InData::from_raw(in_data_ptr);
            let out_data = OutData::from_raw(out_data_ptr);

            #[cfg(with_premiere)]
            let _pr_pica = if in_data.is_premiere() {
                Some(::premiere::PicaBasicSuite::from_sp_basic_suite_raw(in_data.pica_basic_suite_ptr() as _))
            }  else {
                None
            };

            let cmd = RawCommand::from(cmd);

            // Allocate or restore global data pointer
            let mut global_handle = if cmd == RawCommand::GlobalSetup {
                // Allocate global data
                pf::Handle::new(GlobalData {
                    params_map: std::sync::OnceLock::new(),
                    params_num: std::sync::atomic::AtomicUsize::new(1),
                    plugin_instance: <$global_type>::default()
                })?
            } else {
                if unsafe { (*in_data_ptr).global_data.is_null() } {
                    after_effects::log::error!("Global data pointer is null in cmd: {:?}!", cmd);
                    return Err(Error::BadCallbackParameter);
                }
                pf::Handle::<GlobalData>::from_raw(unsafe { (*in_data_ptr).global_data }, cmd == RawCommand::GlobalSetdown)?
            };

            // Allocate or restore sequence data pointer
            let sequence_handle = get_sequence_handle::<$sequence_type>(cmd, &in_data)?;

            let global_lock = global_handle.lock()?;
            let global_inst = global_lock.as_ref()?;

            if cmd == RawCommand::ParamsSetup {
                let mut params = Parameters::<$params_type>::new();
                params.set_in_data(in_data_ptr);
                global_inst.plugin_instance.params_setup(&mut params, InData::from_raw(in_data_ptr), OutData::from_raw(out_data_ptr))?;
                global_inst.params_num.store(params.num_params(), std::sync::atomic::Ordering::Release);
                unsafe {
                    (*out_data_ptr).num_params = params.num_params() as i32;
                    global_inst.params_map.set((*params.map).clone()).unwrap();
                }
            }

            let params_slice = if params.is_null() || global_inst.params_num.load(std::sync::atomic::Ordering::Acquire) == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(params, global_inst.params_num.load(std::sync::atomic::Ordering::Acquire)) }
            };

            let mut params_state = Parameters::<$params_type>::with_params(in_data_ptr, params_slice, global_inst.params_map.get(), global_inst.params_num.load(std::sync::atomic::Ordering::Acquire));
            let mut plugin_state = PluginState {
                global: &global_inst.plugin_instance,
                params: &mut params_state,
                in_data,
                out_data
            };

            let command = Command::from_entry_point(cmd, in_data_ptr, params, output, extra);

            let global_err = plugin_state.global.handle_command(command, in_data, out_data, plugin_state.params);
            let mut sequence_err = None;

            if let Some((mut sequence_handle, needs_lock)) = sequence_handle {
                let (lock, inst) = if needs_lock {
                    let lock = sequence_handle.lock()?;
                    let inst = lock.as_ref()?;
                    (Some(lock), inst)
                } else {
                    (None, sequence_handle.as_ref().unwrap())
                };
                let in_data = InData::from_raw(in_data_ptr);
                let command = Command::from_entry_point(cmd, in_data_ptr, params, output, extra);

                sequence_err = Some(inst.handle_command(&mut plugin_state, command));

                match cmd {
                    #[cfg(does_dialog)]
                    RawCommand::DoDialog => {
                        sequence_err = Some(inst.do_dialog(&mut plugin_state));
                    }
                    RawCommand::Render => {
                        let in_layer = after_effects::Layer::from_raw(unsafe { &mut (*(*params)).u.ld }, in_data, None);
                        let mut out_layer = after_effects::Layer::from_raw(output, in_data, None);
                        sequence_err = Some(inst.render(&mut plugin_state, &in_layer, &mut out_layer));
                    }
                    // RawCommand::UserChangedParam => {
                    //     let extra = extra as *mut after_effects::sys::PF_UserChangedParamExtra;
                    //     let param = plugin_state.params.type_at((*extra).param_index as usize);
                    //     sequence_err = Some(inst.user_changed_param(&mut plugin_state, param));
                    // }
                    _ => { }
                }

                unsafe {
                    match cmd {
                        RawCommand::SequenceSetup | RawCommand::SequenceResetup => {
                            drop(lock);
                            (*out_data_ptr).sequence_data = pf::Handle::into_raw(sequence_handle);
                        }
                        RawCommand::SequenceFlatten | RawCommand::GetFlattenedSequenceData => {
                            let serialized = inst.flatten().map_err(|_| Error::InternalStructDamaged)?;
                            drop(lock);
                            drop(sequence_handle);
                            let mut final_bytes = serialized.0.to_le_bytes().to_vec(); // version
                            final_bytes.extend(&serialized.1);
                            (*out_data_ptr).sequence_data = pf::FlatHandle::into_raw(FlatHandle::new(final_bytes)?) as *mut _;
                        }
                        RawCommand::SequenceSetdown => {
                            (*out_data_ptr).sequence_data = std::ptr::null_mut();
                            // sequence will be dropped and deallocated here
                        }
                        _ => {
                            drop(lock);
                        }
                    }
                }
            } else if std::any::type_name::<S>() == "()" && cmd == RawCommand::GetFlattenedSequenceData {
                // Even if we don't need the sequence data, AE expects us to set this pointer explicitly
                // Otherwise clicking on "Options..." in the Effect Controls panel will crash AE
                unsafe { (*out_data_ptr).sequence_data = std::ptr::null_mut(); }
            }
            drop(plugin_state);
            drop(params_state);

            unsafe {
                match cmd {
                    RawCommand::GlobalSetup => {
                        drop(global_lock);
                        (*out_data_ptr).global_data = pf::Handle::into_raw(global_handle);
                    }
                    RawCommand::GlobalSetdown => {
                        (*out_data_ptr).global_data = std::ptr::null_mut();
                        // global will be dropped and de-allocated here
                    }
                    _ => {
                        drop(global_lock);
                    }
                }
            }

            if global_err.is_err() {
                return global_err;
            }
            if sequence_err.is_some() && sequence_err.unwrap().is_err() {
                return sequence_err.unwrap();
            }

            Ok(())
        }

        #[cfg(debug_assertions)]
        static BACKTRACE_STR: std::sync::RwLock<String> = std::sync::RwLock::new(String::new());

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn PluginDataEntryFunction2(
            in_ptr: after_effects::sys::PF_PluginDataPtr,
            in_plugin_data_callback_ptr: after_effects::sys::PF_PluginDataCB2,
            _in_sp_basic_suite_ptr: *const after_effects::sys::SPBasicSuite,
            in_host_name: *const std::ffi::c_char,
            in_host_version: *const std::ffi::c_char) -> after_effects::sys::PF_Err
        {
            // let _pica = ae::PicaBasicSuite::from_sp_basic_suite_raw(in_sp_basic_suite_ptr);

            if in_host_name.is_null() || in_host_version.is_null() {
                return after_effects::sys::PF_Err_INVALID_CALLBACK as after_effects::sys::PF_Err;
            }

            if let Some(cb_ptr) = in_plugin_data_callback_ptr {
                use after_effects::cstr_literal::cstr;
                unsafe {
                    cb_ptr(in_ptr,
                        cstr!(env!("PIPL_NAME"))       .as_ptr() as *const u8, // Name
                        cstr!(env!("PIPL_MATCH_NAME")) .as_ptr() as *const u8, // Match Name
                        cstr!(env!("PIPL_CATEGORY"))   .as_ptr() as *const u8, // Category
                        cstr!(env!("PIPL_ENTRYPOINT")) .as_ptr() as *const u8, // Entry point
                        env!("PIPL_KIND")              .parse().unwrap(),
                        env!("PIPL_AE_SPEC_VER_MAJOR") .parse().unwrap(),
                        env!("PIPL_AE_SPEC_VER_MINOR") .parse().unwrap(),
                        env!("PIPL_AE_RESERVED")       .parse().unwrap(),
                        cstr!(env!("PIPL_SUPPORT_URL")).as_ptr() as *const u8, // Support url
                    )
                }
            } else {
                after_effects::sys::PF_Err_INVALID_CALLBACK as after_effects::sys::PF_Err
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn EffectMain(
            cmd: after_effects::sys::PF_Cmd,
            in_data_ptr: *mut after_effects::sys::PF_InData,
            out_data_ptr: *mut after_effects::sys::PF_OutData,
            params: *mut *mut after_effects::sys::PF_ParamDef,
            output: *mut after_effects::sys::PF_LayerDef,
            extra: *mut std::ffi::c_void) -> after_effects::sys::PF_Err
        {
            if cmd == after_effects::sys::PF_Cmd_GLOBAL_SETUP as after_effects::sys::PF_Cmd {
                unsafe {
                    (*out_data_ptr).my_version = env!("PIPL_VERSION")  .parse::<u32>().unwrap();
                    (*out_data_ptr).out_flags  = env!("PIPL_OUTFLAGS") .parse::<i32>().unwrap();
                    (*out_data_ptr).out_flags2 = env!("PIPL_OUTFLAGS2").parse::<i32>().unwrap();
                }

                #[cfg(debug_assertions)]
                {
                    #[cfg(target_os = "windows")]
                    {
                        let _ = after_effects::log::set_logger(&after_effects::win_dbg_logger::DEBUGGER_LOGGER);
                    }
                    #[cfg(target_os = "macos")]
                    {
                        let _ = after_effects::oslog::OsLogger::new(env!("CARGO_PKG_NAME")).init();
                    }
                    after_effects::log::set_max_level(after_effects::log::LevelFilter::Debug);

                    std::panic::set_hook(Box::new(|_| {
                        *BACKTRACE_STR.write().unwrap() = std::backtrace::Backtrace::force_capture().to_string();
                    }));
                }
            }

            #[cfg(threaded_rendering)]
            {
                fn assert_impl<T: Sync>() { }
                assert_impl::<$global_type>();
                assert_impl::<$sequence_type>();
            }

            define_shared_effect!(check_size: $sequence_type);

            // log::info!("EffectMain start {:?} {:?}", RawCommand::from(cmd), std::thread::current().id());
            // struct X { cmd: i32 } impl Drop for X { fn drop(&mut self) { log::info!("EffectMain end {:?} {:?}", RawCommand::from(self.cmd), std::thread::current().id()); } }
            // let _x = X { cmd: cmd as i32 };

            #[cfg(any(debug_assertions, catch_panics))]
            {
                let result = std::panic::catch_unwind(|| {
                    handle_effect_main::<$global_type, $sequence_type, $params_type>(cmd, in_data_ptr, out_data_ptr, params, output, extra)
                });
                match result {
                    Ok(Ok(_)) => after_effects::sys::PF_Err_NONE as after_effects::sys::PF_Err,
                    Ok(Err(e)) => {
                        after_effects::log::error!("EffectMain returned error: {e:?}");

                        if e != Error::InterruptCancel && !out_data_ptr.is_null() {
                            after_effects::OutData::from_raw(out_data_ptr).set_error_msg(&format!("EffectMain returned error: {e:?}"));
                        }

                        e as after_effects::sys::PF_Err
                    }
                    Err(e) => {
                        let s = if let Some(s) = e.downcast_ref::<&str>() { s.to_string() }
                           else if let Some(s) = e.downcast_ref::<String>() { s.clone() }
                           else { format!("{e:?}") };

                        let mut msg = format!("EffectMain panicked! {s}");

                        #[cfg(debug_assertions)]
                        {
                            after_effects::log::error!("{msg}, backtrace: {}", BACKTRACE_STR.read().unwrap());
                        }

                        if msg.len() > 255 {
                            msg.truncate(255);
                        }
                        if !out_data_ptr.is_null() {
                            after_effects::OutData::from_raw(out_data_ptr).set_error_msg(&msg);
                        }

                        after_effects::sys::PF_Err_INTERNAL_STRUCT_DAMAGED as after_effects::sys::PF_Err
                    }
                }
            }

            #[cfg(not(any(debug_assertions, catch_panics)))]
            match handle_effect_main::<$global_type, $sequence_type, $params_type>(cmd, in_data_ptr, out_data_ptr, params, output, extra) {
                Ok(_) => after_effects::sys::PF_Err_NONE as after_effects::sys::PF_Err,
                Err(e) => {
                    after_effects::log::error!("EffectMain returned error: {e:?}");
                    e as after_effects::sys::PF_Err
                }
            }
        }
    };
    (check_size: ()) => { };
    (check_size: $t:tt) => {
        const _: () = assert!(std::mem::size_of::<$t>() > 0, concat!("Type `", stringify!($t), "` cannot be zero-sized"));
    };
}
