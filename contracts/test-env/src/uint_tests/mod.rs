use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::str::FromStr;

use das_core::constants::ONE_CKB;
use das_core::error::*;
use das_core::{code_to_error, das_assert, debug};
use primitive_types::U256;
use rust_decimal::Decimal;

fn to_u256(s: &str) -> U256 {
    let dec_str = s.replace("_", "");
    U256::from_dec_str(&dec_str).unwrap()
}

fn to_dec_str(s: &str) -> String {
    s.replace("_", "")
}

// Testing the basic interface of U256 type.
pub fn test_basic_interface() -> Result<(), Box<dyn ScriptError>> {
    let a = U256::from(999999);
    das_assert!(
        a.to_string() == "999999",
        ErrorCode::UnittestError,
        "U256::from(999999) failed"
    );

    let a = U256::from_dec_str("999999").unwrap();
    das_assert!(
        a.to_string() == "999999",
        ErrorCode::UnittestError,
        "U256::from_str(\"999999\").unwrap() failed"
    );

    Ok(())
}

pub fn test_safty() -> Result<(), Box<dyn ScriptError>> {
    let a = to_u256("1_000_000_000_000_000");
    let b = to_u256("1_000_000_000_000_000");
    let c = a + b;
    das_assert!(
        c.to_string() == to_dec_str("2_000_000_000_000_000"),
        ErrorCode::UnittestError,
        "U256::add failed"
    );

    let a = to_u256("1_000_000_000_000_000");
    let b = to_u256("1_000_000_000_000_000");
    let c = a - b;
    das_assert!(c.to_string() == "0", ErrorCode::UnittestError, "U256::sub failed");

    let a = to_u256("1_000_000_000_000_000");
    let b = to_u256("100_000_000");
    let c = a * b;
    das_assert!(
        c.to_string() == to_dec_str("1_000_000_000_000_000_00_000_000"),
        ErrorCode::UnittestError,
        "U256::mul failed"
    );

    let a = to_u256("1_000_000_000_000_000_00_000_000");
    let b = to_u256("1000");
    let c = a / b;
    das_assert!(
        c.to_string() == to_dec_str("1_000_000_000_000_000_00_000"),
        ErrorCode::UnittestError,
        "U256::div failed"
    );

    Ok(())
}

// Testing the security of U256 type within the range of CKB exchange rate from 0.0001CKB/USD to 100CKB/USD.
pub fn perf_price_formula() -> Result<(), Box<dyn ScriptError>> {
    let one_billion = "1_000_000_000_000_000".replace("_", "");
    let yearly_prices = [
        U256::from(5_000_000),                     // 5$ per year
        U256::from_dec_str(&one_billion).unwrap(), // 1_000_000_000$ per year
    ];

    for yearly_price in yearly_prices {
        let ckb = U256::from(ONE_CKB);
        let quote = U256::from(1_000); // 0.0001

        let expect_yearly_price = Decimal::from_str("5_000_000").unwrap();
        let expect_ckb = Decimal::from(ONE_CKB);
        let expect_quote = Decimal::from(1_000); // 0.0001

        // The full execution takes too long to run, so we only run the first 1000 iterations.
        let mut print_at = U256::from(1_000);
        for i in 0u64..1000 {
            let quote = quote * U256::from(i + 1);
            let expect_quote = expect_quote * Decimal::from(i + 1);

            let total_price = yearly_price * ckb / quote;
            let total_price = Decimal::from_str(total_price.to_string().as_str()).unwrap();
            let expect_total_price = expect_yearly_price * expect_ckb / expect_quote;

            if quote == print_at {
                print_at = print_at * U256::from(10);
                debug!("i: {}, total_price: {}, quote: {}", i, total_price, quote);
            }

            das_assert!(
                expect_total_price - total_price < Decimal::from(1),
                ErrorCode::UnittestError,
                "The calculation error exceeded 1000 shannon."
            );
        }

        // An the last 1000 iterations.
        print_at = U256::from(99_001_000);
        for i in 99000u64..100000 {
            let quote = quote * U256::from(i + 1);
            let expect_quote = expect_quote * Decimal::from(i + 1);

            let total_price = yearly_price * ckb / quote;
            let total_price = Decimal::from_str(total_price.to_string().as_str()).unwrap();
            let expect_total_price = expect_yearly_price * expect_ckb / expect_quote;

            if quote == print_at {
                print_at = U256::from(100_000_000);
                debug!("i: {}, total_price: {}, quote: {}", i, total_price, quote);
            }

            das_assert!(
                expect_total_price - total_price < Decimal::from(1),
                ErrorCode::UnittestError,
                "The calculation error exceeded 1000 shannon."
            );
        }
    }

    Ok(())
}
