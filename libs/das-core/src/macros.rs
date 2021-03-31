#[macro_export]
macro_rules! debug {
    ($fmt:literal) => {
        #[cfg(not(feature = "mainnet"))]
        ckb_std::syscalls::debug(alloc::format!($fmt));
    };
    ($fmt:literal, $($args:expr),+) => {
        #[cfg(not(feature = "mainnet"))]
        ckb_std::syscalls::debug(alloc::format!($fmt, $($args), +));
    };
}

#[macro_export]
macro_rules! warn {
    ($fmt:literal) => {
        ckb_std::syscalls::debug(alloc::format!($fmt));
    };
    ($fmt:literal, $($args:expr),+) => {
        ckb_std::syscalls::debug(alloc::format!($fmt, $($args), +));
    };
}

#[macro_export]
macro_rules! assert {
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
