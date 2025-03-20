#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(all(debug_assertions))]
        ckb_std::syscalls::debug(alloc::format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn_log {
    ($($arg:tt)*) => {
        ckb_std::syscalls::debug(alloc::format!($($arg)*));
    };
}

#[macro_export]
macro_rules! new_context {
    () => {
        unsafe { ckb_std::dynamic_loading_c_impl::CKBDLContext::<$crate::constants::DynLibSize>::new() }
    };
}

#[macro_export]
macro_rules! log_loading {
    ($name:expr, $type_id:expr) => {
        $crate::debug_log!(
            "Loading {} dynamic library with type ID 0x{} ...",
            $name,
            $crate::util::hex_string(&$type_id)
        );
    };
}

#[macro_export]
macro_rules! load_lib {
    ($context:expr, $type_id:expr) => {
        $context
            .load_by(
                &$type_id,
                ckb_std::ckb_types::core::ScriptHashType::Type,
            )
            .expect("The shared lib should be loaded successfully.")
    };
}

#[macro_export]
macro_rules! load_3_methods {
    ($lib:expr) => {
        Some($crate::sign_lib::SignLibWith3Methods {
            c_validate: unsafe {
                $lib.get(b"validate")
                    .expect("Load function 'validate' from library failed.")
            },
            c_validate_str: unsafe {
                $lib.get(b"validate_str")
                    .expect("Load function 'validate_str' from library failed.")
            },
            c_validate_device: unsafe {
                $lib.get(b"validate_device")
                    .expect("Load function 'validate_device' from library failed.")
            },
        })
    };
}

#[macro_export]
macro_rules! load_2_methods {
    ($lib:expr) => {
        Some($crate::sign_lib::SignLibWith2Methods {
            c_validate: unsafe {
                $lib.get(b"validate")
                    .expect("Load function 'validate' from library failed.")
            },
            c_validate_str: unsafe {
                $lib.get(b"validate_str")
                    .expect("Load function 'validate_str' from library failed.")
            },
        })
    };
}

#[macro_export]
macro_rules! load_1_method {
    ($lib:expr) => {
        Some($crate::sign_lib::SignLibWith1Methods {
            c_validate: unsafe {
                $lib.get(b"validate")
                    .expect("Load function 'validate' from library failed.")
            },
        })
    };
}

#[macro_export]
macro_rules! load_and_configure_lib {
    ($sign_lib:ident, $lib_name:expr, $type_id:expr, $sign_lib_field:ident, $load_methods_macro:ident) => {
        let mut context = new_context!();
        log_loading!($lib_name, $type_id);
        let lib = load_lib!(context, $type_id);
        $sign_lib.$sign_lib_field = $load_methods_macro!(lib);
    };
}
