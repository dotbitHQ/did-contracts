macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(all(debug_assertions))]
        ckb_std::syscalls::debug(alloc::format!($($arg)*));
    };
}

macro_rules! warn_log {
    ($($arg:tt)*) => {
        ckb_std::syscalls::debug(alloc::format!($($arg)*));
    };
}

pub(crate) use debug;
pub(crate) use warn_log;
