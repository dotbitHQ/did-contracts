#![allow(unused_macros)]

#[cfg(all(debug_assertions, not(feature = "std")))]
macro_rules! debug {
    ($($arg:tt)*) => {
        ckb_std::syscalls::debug(alloc::format!($($arg)*));
    };
}

#[cfg(all(debug_assertions, feature = "std"))]
macro_rules! debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*);
    };
}

#[cfg(not(feature = "std"))]
macro_rules! warn {
    ($($arg:tt)*) => {
        ckb_std::syscalls::debug(alloc::format!($($arg)*));
    };
}

#[cfg(feature = "std")]
macro_rules! warn {
    ($($arg:tt)*) => {
        log::warn!($($arg)*);
    };
}
