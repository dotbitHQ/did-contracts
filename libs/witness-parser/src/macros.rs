macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(all(debug_assertions))]
        ckb_std::syscalls::debug(alloc::format!($($arg)*));
    };
}

macro_rules! err_assert {
    ($condition:expr, $error_code:expr) => {
        if !$condition {
            return core::result::Result::Err($error_code);
        }
    };
}
