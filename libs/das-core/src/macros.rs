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
