macro_rules! das_assert {
    ($condition:expr, $error_code:expr, $fmt:literal) => {
        if !$condition {
            ckb_std::syscalls::debug(alloc::format!($fmt));
            return Err($error_code);
        }
    };
    ($condition:expr, $error_code:expr, $fmt:literal, $($args:expr),+) => {
        if !$condition {
            ckb_std::syscalls::debug(alloc::format!($fmt, $($args), +));
            return Err($error_code);
        }
    };
}
