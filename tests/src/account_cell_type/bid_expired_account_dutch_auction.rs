use das_types::constants::{DataType, Source};
use serde_json::json;

use crate::util;
use crate::util::accounts::{DP_TRANSFER_WHITELIST_1, SENDER};
use crate::util::constants::{
    OracleCellType, ACCOUNT_EXPIRATION_AUCTION_PERIOD, ACCOUNT_EXPIRATION_GRACE_PERIOD, TIMESTAMP,
};
use crate::util::error::{AccountCellErrorCode, DPointCellErrorCode, ErrorCode};
use crate::util::template_common_cell::{
    push_input_account_cell, push_input_dpoint_cell_float, push_input_normal_cell, push_output_account_cell,
    push_output_balance_cell, push_output_dpoint_cell_float, push_output_normal_cell,
};
use crate::util::template_generator::{ContractType, TemplateGenerator};
use crate::util::template_parser::{challenge_tx, test_tx};
const ACCOUNT_FOUR_LETTER: &str = "1234.bit";
const ACCOUNT_FIVE_LETTER: &str = "12345.bit";
const SECONDS_ONE_DAY: u64 = 24 * 3600;
const SECONDS_ONE_YEAR: u64 = 365 * SECONDS_ONE_DAY;
const DURATION_AFTER_EXPIRED: u64 = ACCOUNT_EXPIRATION_GRACE_PERIOD + 20 * SECONDS_ONE_DAY; //day20, $95.367432
const ACCOUNT_EXPIRED_AT: u64 = TIMESTAMP - DURATION_AFTER_EXPIRED;
const ACCOUNT_REGISTERED_AT: u64 = TIMESTAMP - DURATION_AFTER_EXPIRED - SECONDS_ONE_YEAR;
const BIDDER: &str = "0x050000000000000000000000000000000000008888";
const DP_SVR: &str = "0x050000000000000000000000000000000000009991";
const DID_SVR: &str = "0x050000000000000000000000000000000000009992";
const DECIMAL_PRECISION: u64 = 1000000;
const SHANNON: u64 = 100000000;
fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, None);
    //oracle
    template.push_oracle_cell(1, OracleCellType::Time, TIMESTAMP);
    template.push_oracle_cell(1, OracleCellType::Quote, 3788);

    //type
    template.push_contract_cell("balance-cell-type", ContractType::Contract);
    template.push_contract_cell("always-success", ContractType::Contract);
    template.push_contract_cell("account-cell-type", ContractType::Contract);
    template.push_contract_cell("dpoint-cell-type", ContractType::Contract);

    //lock
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("eth_sign.so", ContractType::SharedLib);
    template.push_contract_cell("secp256k1_data", ContractType::DeployedSharedLib);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("eip712-lib", ContractType::Contract);

    //config
    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellDPoint, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellPrice, Source::CellDep);

    template
}

#[test]
fn test_bid_expired_account_auction_success_normal() {
    let mut template = init("bid_expired_account_dutch_auction");

    //push inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER,
            },
            "data": {
                "expired_at": ACCOUNT_EXPIRED_AT,
            },
            "witness": {
                "registered_at": ACCOUNT_REGISTERED_AT,
                "last_transfer_account_at": ACCOUNT_REGISTERED_AT + 123 * SECONDS_ONE_DAY,
                "last_edit_manager_at": ACCOUNT_REGISTERED_AT + 124 * SECONDS_ONE_DAY,
                "last_edit_records_at": ACCOUNT_REGISTERED_AT + 125 * SECONDS_ONE_DAY,
                "records": [
                        {
                            "type": "address",
                            "key": "eth",
                            "label": "Personal",
                            "value": "0x0000000000000000000000000000000000000000",
                    }
            ]
            }
        }),
    );
    push_input_dpoint_cell_float(&mut template, 1000 * DECIMAL_PRECISION, BIDDER);
    push_input_normal_cell(&mut template, 100 * SHANNON, DID_SVR);

    //push outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BIDDER,
                "manager_lock_args": BIDDER,
            },
            "data": {
                "expired_at": TIMESTAMP + SECONDS_ONE_YEAR,
            },
            "witness": {
                "registered_at": TIMESTAMP,
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "records": [
                        {
                            "type": "address",
                            "key": "eth",
                            "label": "Personal",
                            "value": "0x0000000000000000000000000000000000000000",
                    }]
            }
        }),
    );
    push_output_dpoint_cell_float(&mut template, 101185640, DP_TRANSFER_WHITELIST_1);
    push_output_dpoint_cell_float(&mut template, 898814360, BIDDER);

    push_output_normal_cell(&mut template, 10 * SHANNON, DP_SVR);
    push_output_normal_cell(&mut template, 90 * SHANNON, DID_SVR);

    push_output_balance_cell(&mut template, util::gen_account_cell_capacity(5), SENDER);

    test_tx(template.as_json());
}

#[test]
fn test_bid_expired_success_four_letters_account() {
    let mut template = init("bid_expired_account_dutch_auction");

    //push inputs
    push_input_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(4),
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER,
            },
            "data": {
                "account": ACCOUNT_FOUR_LETTER,
                "expired_at": ACCOUNT_EXPIRED_AT,
            },
            "witness": {
                "account": ACCOUNT_FOUR_LETTER,

                "registered_at": ACCOUNT_REGISTERED_AT,
                "last_transfer_account_at": ACCOUNT_REGISTERED_AT + 123 * SECONDS_ONE_DAY,
                "last_edit_manager_at": ACCOUNT_REGISTERED_AT + 124 * SECONDS_ONE_DAY,
                "last_edit_records_at": ACCOUNT_REGISTERED_AT + 125 * SECONDS_ONE_DAY,
                "records": [
                        {
                            "type": "address",
                            "key": "eth",
                            "label": "Personal",
                            "value": "0x0000000000000000000000000000000000000000",
                    }
            ]
            }
        }),
    );
    push_input_dpoint_cell_float(&mut template, 1000 * DECIMAL_PRECISION, BIDDER);
    push_input_normal_cell(&mut template, 100 * SHANNON, DID_SVR);

    //push outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BIDDER,
                "manager_lock_args": BIDDER,
            },
            "data": {
                "account": ACCOUNT_FOUR_LETTER,
                "expired_at": TIMESTAMP + SECONDS_ONE_YEAR,
            },
            "witness": {
                "account": ACCOUNT_FOUR_LETTER,
                "registered_at": TIMESTAMP,
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "records": [
                        {
                            "type": "address",
                            "key": "eth",
                            "label": "Personal",
                            "value": "0x0000000000000000000000000000000000000000",
                    }
            ]
            }
        }),
    );

    //note: the basic price is 160,
    push_output_dpoint_cell_float(&mut template, 256181852, DP_TRANSFER_WHITELIST_1);
    push_output_dpoint_cell_float(&mut template, 743818148, BIDDER);

    push_output_normal_cell(&mut template, 10 * SHANNON, DP_SVR);
    push_output_normal_cell(&mut template, 90 * SHANNON, DID_SVR);

    push_output_balance_cell(&mut template, util::gen_account_cell_capacity(4), SENDER);

    test_tx(template.as_json());
}

fn common_when_auction_have_started(account_expired_at: u64, premium: u64) -> TemplateGenerator {
    let mut template = init("bid_expired_account_dutch_auction");

    let account_expired_at = account_expired_at;
    let registered_at = account_expired_at - SECONDS_ONE_YEAR;
    let last_transfer_account_at = registered_at + 123 * SECONDS_ONE_DAY;
    let last_edit_manager_at = registered_at + 124 * SECONDS_ONE_DAY;
    let last_edit_records_at = registered_at + 125 * SECONDS_ONE_DAY;

    let basic_price_five_letters_account = 5814420;
    let outputs_user_dp_amount = 1 * DECIMAL_PRECISION;
    let outputs_das_dp_amount = premium * DECIMAL_PRECISION + basic_price_five_letters_account;
    let inputs_user_dp_amount = outputs_das_dp_amount + outputs_user_dp_amount;
    //push inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER,
            },
            "data": {
                "expired_at": account_expired_at,
            },
            "witness": {
                "registered_at": registered_at,
                "last_transfer_account_at": last_transfer_account_at,
                "last_edit_manager_at": last_edit_manager_at,
                "last_edit_records_at": last_edit_records_at,
                 "records": [
                        {
                            "type": "address",
                            "key": "eth",
                            "label": "Personal",
                            "value": "0x0000000000000000000000000000000000000000",
                    }
                    ]
            }
        }),
    );
    push_input_dpoint_cell_float(&mut template, inputs_user_dp_amount, BIDDER);
    push_input_normal_cell(&mut template, 100 * SHANNON, DID_SVR);

    //push outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BIDDER,
                "manager_lock_args": BIDDER,
            },
            "data": {
                "expired_at": TIMESTAMP + SECONDS_ONE_YEAR,
            },
            "witness": {
                "registered_at": TIMESTAMP,
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                 "records": [
                        {
                            "type": "address",
                            "key": "eth",
                            "label": "Personal",
                            "value": "0x0000000000000000000000000000000000000000",
                    }
                    ]
            }
        }),
    );
    push_output_dpoint_cell_float(&mut template, outputs_das_dp_amount, DP_TRANSFER_WHITELIST_1);
    push_output_dpoint_cell_float(&mut template, outputs_user_dp_amount, BIDDER);

    push_output_normal_cell(&mut template, 10 * SHANNON, DP_SVR);
    push_output_normal_cell(&mut template, 90 * SHANNON, DID_SVR);

    push_output_balance_cell(&mut template, util::gen_account_cell_capacity(5), SENDER);

    template
}

/*

   auction_start_timestamp = expired_at + grace_period;
   auction_end_timestamp = expired_at + grace_period + auction_period;
   auction during [auction_start_timestamp, auction_end_timestamp]

*/
#[test]
fn test_bid_expired_success_when_auction_started_00_00() {
    let account_expired_at = TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD;
    let template = common_when_auction_have_started(account_expired_at, 100000000);

    //note: The value of each new DPointCell should be 0 < x <= 10 000 000 000 000.(current: 100000005818208)
    challenge_tx(template.as_json(), DPointCellErrorCode::InitialDataError);
    //test_tx(template.as_json());
}

#[test]
fn challenge_bid_expired_failed_when_auction_has_not_started() {
    let account_expired_at = TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD + 1;
    let template = common_when_auction_have_started(account_expired_at, 0);

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::AccountCellInExpirationGracePeriod,
    );
    //test_tx(template.as_json());
}
// the premium will not decrease to 0 in 27 days
// #[test]
// fn test_bid_expired_success_when_auction_started_27_days_00_00() {
//     let account_expired_at = TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - ACCOUNT_EXPIRATION_AUCTION_PERIOD;
//     let template = common_when_auction_have_started(account_expired_at, 0);
//     test_tx(template.as_json());
// }
#[test]
fn challenge_bid_expired_failed_when_auction_started_27_days_00_01() {
    let account_expired_at = TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - ACCOUNT_EXPIRATION_AUCTION_PERIOD - 1;
    let template = common_when_auction_have_started(account_expired_at, 0);
    //test_tx(template.as_json());
    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellHasExpired);
}

#[test]
fn challenge_bid_failed_account_auction_registered_at() {
    let mut template = init("bid_expired_account_dutch_auction");

    //push inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER,
            },
            "data": {
                "expired_at": ACCOUNT_EXPIRED_AT,
            },
            "witness": {
                "registered_at": ACCOUNT_REGISTERED_AT,
                "last_transfer_account_at": ACCOUNT_REGISTERED_AT + 123 * SECONDS_ONE_DAY,
                "last_edit_manager_at": ACCOUNT_REGISTERED_AT + 124 * SECONDS_ONE_DAY,
                "last_edit_records_at": ACCOUNT_REGISTERED_AT + 125 * SECONDS_ONE_DAY,
                                 "records": [
                        {
                            "type": "address",
                            "key": "eth",
                            "label": "Personal",
                            "value": "0x0000000000000000000000000000000000000000",
                    }
                    ]
            }
        }),
    );
    push_input_dpoint_cell_float(&mut template, 1000 * DECIMAL_PRECISION, BIDDER);
    push_input_normal_cell(&mut template, 100 * SHANNON, DID_SVR);

    //push outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BIDDER,
                "manager_lock_args": BIDDER,
            },
            "data": {
                "expired_at": TIMESTAMP + SECONDS_ONE_YEAR,
            },
            "witness": {
                //"registered_at": TIMESTAMP,
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                                 "records": [
                        {
                            "type": "address",
                            "key": "eth",
                            "label": "Personal",
                            "value": "0x0000000000000000000000000000000000000000",
                    }
                    ]
            }
        }),
    );
    push_output_dpoint_cell_float(&mut template, 101185640, DP_TRANSFER_WHITELIST_1);
    push_output_dpoint_cell_float(&mut template, 898814360, BIDDER);

    push_output_normal_cell(&mut template, 10 * SHANNON, DP_SVR);
    push_output_normal_cell(&mut template, 90 * SHANNON, DID_SVR);

    push_output_balance_cell(&mut template, util::gen_account_cell_capacity(5), SENDER);
    challenge_tx(template.as_json(), ErrorCode::InvalidTransactionStructure);
}
