use alloc::vec;
use super::error::Error;
use ckb_std::{cstr_core::CStr, debug};
use core::convert::TryInto;
use core::slice::from_raw_parts;
use das_types::{packed::SubAccount, prelude::Entity};

pub fn main(argc: usize, argv: *const *const u8) -> Result<(), Error> {
    debug!("====== Running test-custom-script ======");

    das_assert!(
        argc >= 6,
        Error::InvalidArgument,
        "The param argc must be greater than or equal to 4."
    );

    let args = unsafe { from_raw_parts(argv, argc as usize) };
    let action = unsafe { CStr::from_ptr(args[0]).to_str().unwrap() };
    let quote = read_u64_param!(args[1]);
    let owner_profit = read_u64_param!(args[2]);
    let das_profit = read_u64_param!(args[3]);
    let script_args = read_bytes_param!(args[4]);

    debug!("quote = {:?}", quote);
    debug!("owner_profit = {:?}", owner_profit);
    debug!("das_profit = {:?}", das_profit);
    debug!("script_args = 0x{}", hex::encode(&script_args));

    das_assert!(
        action == "create_sub_account",
        Error::InvalidAction,
        "The param action should be create_sub_account . (current: {})",
        action
    );

    das_assert!(
        owner_profit == 24_000_000_000u64,
        Error::InvalidOwnerProfit,
        "The param owner_profit should be 24_000_000_000u64 . (current: {})",
        owner_profit
    );

    das_assert!(
        das_profit == 6_000_000_000u64,
        Error::InvalidDasProfit,
        "The param das_profit should be 6_000_000_000u64 . (current: {})",
        das_profit
    );

    das_assert!(
        &script_args == &[0, 17, 34, 51, 0] || &script_args == &[],
        Error::InvalidScriptArgs,
        "The param script_args should be 0x0011223300 . (current: 0x{})",
        hex::encode(&script_args)
    );

    for i in 5..argc {
        let (expiration_years, sub_account_bytes) = read_sub_account_param!(args[i]);
        debug!("expiration_years = {:?}", expiration_years);

        das_assert!(
            expiration_years == 1,
            Error::InvalidSubAccount,
            "The param expiration_years should be 1 ."
        );

        match SubAccount::from_slice(&sub_account_bytes) {
            Ok(_sub_account) => {
                let account_len = _sub_account.account().len();
                das_assert!(
                    account_len > 0,
                    Error::InvalidSubAccount,
                    "The param sub_account should be parsed successfully"
                );
            }
            Err(_err) => {
                debug!("Decoding SubAccount from slice failed: {}", _err);
                return Err(Error::InvalidSubAccount);
            }
        }
    }

    Ok(())
}
