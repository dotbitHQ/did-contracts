#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(all(debug_assertions))]
        ckb_std::syscalls::debug(alloc::format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        ckb_std::syscalls::debug(alloc::format!($($arg)*));
    };
}

#[macro_export]
macro_rules! code_to_error {
    ($error_code:expr, $error_msg:expr) => {
        alloc::boxed::Box::new($crate::error::Error::new(
            $error_code,
            alloc::string::String::from($error_msg),
        ))
    };
    ($error_code:expr) => {{
        alloc::boxed::Box::new($crate::error::Error::new($error_code, alloc::string::String::new()))
    }};
}

#[macro_export]
macro_rules! assert {
    ($condition:expr, $error_code:expr, $fmt:literal) => {
        if !$condition {
            ckb_std::syscalls::debug(alloc::format!($fmt));
            return core::result::Result::Err(code_to_error!($error_code).into());
        }
    };
    ($condition:expr, $error_code:expr, $fmt:literal, $($args:expr),+) => {
        if !$condition {
            ckb_std::syscalls::debug(alloc::format!($fmt, $($args), +));
            return core::result::Result::Err(code_to_error!($error_code).into());
        }
    };
}

#[macro_export]
macro_rules! das_assert {
    ($condition:expr, $error_code:expr, $fmt:literal) => {
        if !$condition {
            ckb_std::syscalls::debug(alloc::format!($fmt));
            return core::result::Result::Err(code_to_error!($error_code).into());
        }
    };
    ($condition:expr, $error_code:expr, $fmt:literal, $($args:expr),+) => {
        if !$condition {
            ckb_std::syscalls::debug(alloc::format!($fmt, $($args), +));
            return core::result::Result::Err(code_to_error!($error_code).into());
        }
    };
}
#[macro_export]
macro_rules! das_assert_custom {
    ($($condition:expr, $msg:expr),*) => {
        $(
            das_assert!(
                $condition,
                ErrorCode::InvalidTransactionStructure,
                $msg
            );
        )*
    };
}
#[macro_export]
macro_rules! assert_lock_equal {
    (($cell_a_index:expr, $cell_a_source:expr), ($cell_b_index:expr, $cell_b_source:expr), $error_code:expr, $fmt:literal) => {{
        let cell_a_lock_hash = high_level::load_cell_lock_hash($cell_a_index, $cell_a_source).map_err(Error::<ErrorCode>::from)?;
        let cell_b_lock_hash = high_level::load_cell_lock_hash($cell_b_index, $cell_b_source).map_err(Error::<ErrorCode>::from)?;

        if cell_a_lock_hash != cell_b_lock_hash {
            ckb_std::syscalls::debug(alloc::format!($fmt));
            return Err(code_to_error!($error_code));
        }
    }};
    ($condition:expr, $error_code:expr, $fmt:literal, $($args:expr),+) => {
        let cell_a_lock_hash = high_level::load_cell_lock_hash($cell_a_index, $cell_a_source).map_err(Error::<ErrorCode>::from)?;
        let cell_b_lock_hash = high_level::load_cell_lock_hash($cell_b_index, $cell_b_source).map_err(Error::<ErrorCode>::from)?;

        if cell_a_lock_hash != cell_b_lock_hash {
            ckb_std::syscalls::debug(alloc::format!($fmt, $($args), +));
            return Err(code_to_error!($error_code));
        }
    };
}
