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

macro_rules! read_u64_param {
    ($arg_ptr:expr) => {{
        let hex = unsafe { CStr::from_ptr($arg_ptr).to_str().unwrap() };
        let mut buf = vec![0u8; 8];
        hex::decode_to_slice(hex, &mut buf).unwrap();
        u64::from_le_bytes(buf.try_into().unwrap())
    }};
}

macro_rules! read_bytes_param {
    ($arg_ptr:expr) => {{
        let hex = unsafe { CStr::from_ptr($arg_ptr).to_str().unwrap() };
        if hex.len() == 0 {
            alloc::vec::Vec::new()
        } else {
            let mut buf = vec![0u8; hex.len() / 2];
            hex::decode_to_slice(hex, &mut buf).unwrap();

            buf
        }
    }};
}

macro_rules! read_sub_account_param {
    ($arg_ptr:expr) => {{
        let hex = unsafe { CStr::from_ptr($arg_ptr).to_str().unwrap() };
        let mut buf = vec![0u8; hex.len() / 2];
        hex::decode_to_slice(hex, &mut buf).unwrap();

        let expiration_years = u64::from_le_bytes((&buf[0..8]).try_into().unwrap());
        let sub_account_bytes = (&buf[8..]).to_vec();

        (expiration_years, sub_account_bytes)
    }};
}
