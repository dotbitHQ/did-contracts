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
    ($name:expr, $type_id_table:expr) => {
        $crate::debug_log!(
            "Loading {} dynamic library with type ID 0x{} ...",
            $name,
            $crate::util::hex_string($name.get_code_hash($type_id_table))
        );
    };
}

#[macro_export]
macro_rules! load_lib {
    ($context:expr, $name:expr, $type_id_table:expr) => {
        $context
            .load_by_type_id($name.get_code_hash($type_id_table))
            .expect("The shared lib should be loaded successfully.")
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
