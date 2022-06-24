use super::error::Error;
use alloc::vec::Vec;
use ckb_std::{cstr_core::CStr, debug};
use core::convert::TryInto;
use core::slice::from_raw_parts;
use das_types::{packed::SubAccount, prelude::Entity};

pub fn main(argc: usize, argv: *const *const u8) -> Result<(), Error> {
    debug!("====== Running test-custom-script ======");

    das_assert!(
        argc >= 4,
        Error::InvalidArgument,
        "The param argc must be greater than or equal to 4."
    );

    let args = unsafe { from_raw_parts(argv, argc as usize) };
    let action = unsafe { CStr::from_ptr(args[0]).to_str().unwrap() };
    let owner_profit_bytes: Vec<u8> = unsafe { hex::decode(CStr::from_ptr(args[1]).to_str().unwrap()).unwrap() };
    let das_profit_bytes: Vec<u8> = unsafe { hex::decode(CStr::from_ptr(args[2]).to_str().unwrap()).unwrap() };
    let owner_profit = u64::from_le_bytes(owner_profit_bytes.try_into().unwrap());
    let das_profit = u64::from_le_bytes(das_profit_bytes.try_into().unwrap());

    das_assert!(
        action == "create_sub_account",
        Error::InvalidAction,
        "The param action should be create_sub_account ."
    );

    das_assert!(
        owner_profit == 24_000_000_000u64,
        Error::InvalidOwnerProfit,
        "The param owner_profit should be 24_000_000_000u64 ."
    );

    das_assert!(
        das_profit == 6_000_000_000u64,
        Error::InvalidDasProfit,
        "The param das_profit should be 6_000_000_000u64 ."
    );

    for i in 3..argc {
        let param_bytes: Vec<u8> = unsafe { hex::decode(CStr::from_ptr(args[i]).to_str().unwrap()).unwrap() };
        let expiration_years = u64::from_le_bytes((&param_bytes[0..8]).try_into().unwrap());

        das_assert!(
            expiration_years == 1,
            Error::InvalidSubAccount,
            "The param expiration_years should be 1 ."
        );

        let sub_account_bytes = &param_bytes[8..];
        match SubAccount::from_slice(sub_account_bytes) {
            Ok(_sub_account) => {
                // use das_types::prettier::Prettier;
                // debug!("sub_account = {}", _sub_account.as_prettier());
            }
            Err(_err) => {
                debug!("Decoding SubAccount from slice failed: {}", _err);
                return Err(Error::InvalidSubAccount);
            }
        }
    }

    Ok(())
}
