use das_types::constants::*;
use das_types::packed::*;
use serde_json::json;
use sparse_merkle_tree::H256;

use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::smt::SMTWithHistory;
use crate::util::template_common_cell::{
    push_dep_account_cell, push_input_sub_account_cell_v2, push_output_normal_cell, push_output_sub_account_cell_v2,
};
use crate::util::template_generator::*;
use crate::util::{self};

pub const SCRIPT_CODE_HASH: &str = "0x0000000000000000000000000000746573742d637573746f6d2d736372697074";
pub const SCRIPT_ARGS: &str = "0x0011223300";

pub const DUMMY_CHANNEL: &str = "0x00000000000000000000000000000000000000000000000000000000";

// total paid 100 USD
pub const TOTAL_PAID: u64 = USD_1 * 100 / CKB_QUOTE * ONE_CKB;

pub fn init(action: &str, params_opt: Option<&str>) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, params_opt.map(|raw| Bytes::from(util::hex_to_bytes(raw))));

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("balance-cell-type", ContractType::Contract);
    template.push_contract_cell("sub-account-cell-type", ContractType::Contract);

    template.push_oracle_cell(1, OracleCellType::Quote, CKB_QUOTE);
    template.push_oracle_cell(1, OracleCellType::Time, TIMESTAMP);
    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);

    template
}

pub fn init_config(action: &str, params_opt: Option<&str>) -> TemplateGenerator {
    let mut template = init(action, params_opt);

    template.push_contract_cell("account-cell-type", ContractType::Contract);

    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellSubAccount, Source::CellDep);

    template
}

pub fn init_update() -> TemplateGenerator {
    let mut template = init("update_sub_account", None);

    template.push_contract_cell("ckb_sign.so", ContractType::SharedLib);
    template.push_contract_cell("eth_sign.so", ContractType::SharedLib);
    template.push_contract_cell("tron_sign.so", ContractType::SharedLib);
    template.push_contract_cell("doge_sign.so", ContractType::SharedLib);
    template.push_contract_cell("secp256k1_data", ContractType::DeployedSharedLib);

    template.push_config_cell(DataType::ConfigCellCharSetEmoji, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetDigit, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetEn, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellUnAvailableAccount, Source::CellDep);

    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellSubAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, Source::CellDep);

    template
}

pub fn push_simple_dep_account_cell(template: &mut TemplateGenerator) {
    push_dep_account_cell(
        template,
        json!({
            "header": {
                "height": HEIGHT - 1,
                "timestamp": TIMESTAMP - DAY_SEC,
            },
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                "account": ACCOUNT_1,
                "enable_sub_account": 1,
            }
        }),
    );
}

pub fn push_simple_input_sub_account_cell(
    template: &mut TemplateGenerator,
    das_profit: u64,
    owner_profit: u64,
    flag: SubAccountConfigFlag,
) {
    push_input_sub_account_cell_v2(
        template,
        json!({
            "header": {
                "height": HEIGHT - 1,
                "timestamp": TIMESTAMP - DAY_SEC,
            },
            "data": {
                "das_profit": das_profit,
                "owner_profit": owner_profit,
                "flag": flag as u8,
            }
        }),
        ACCOUNT_1,
    );
}

pub fn push_simple_output_sub_account_cell(
    template: &mut TemplateGenerator,
    das_profit: u64,
    owner_profit: u64,
    flag: SubAccountConfigFlag,
) {
    push_output_sub_account_cell_v2(
        template,
        json!({
            "data": {
                "das_profit": das_profit,
                "owner_profit": owner_profit,
                "flag": flag as u8,
            }
        }),
        ACCOUNT_1,
    );
}

pub fn push_common_output_cells(template: &mut TemplateGenerator, total_paid_years: u64, flag: SubAccountConfigFlag) {
    let das_profit = util::gen_sub_account_register_fee(SUB_ACCOUNT_NEW_PRICE, total_paid_years);
    push_simple_output_sub_account_cell(template, das_profit, 0, flag);
    push_output_normal_cell(template, TOTAL_PAID - das_profit, OWNER);
}

pub fn calculate_sub_account_cost(new_account_count: u64) -> u64 {
    SUB_ACCOUNT_NEW_PRICE * new_account_count
}

pub fn push_commen_mint_sign_witness(template: &mut TemplateGenerator) -> SMTWithHistory {
    let smt = template.push_sub_account_mint_sign_witness(
        DataType::SubAccountMintSign,
        json!({
            "version": 1,
            "expired_at": TIMESTAMP + DAY_SEC,
            "account_list_smt_root": [
                [SUB_ACCOUNT_1, gen_das_lock_args(OWNER_1, Some(MANAGER_1))],
                [SUB_ACCOUNT_2, gen_das_lock_args(OWNER_2, Some(MANAGER_2))],
                [SUB_ACCOUNT_3, gen_das_lock_args(OWNER_3, Some(MANAGER_3))],
                [SUB_ACCOUNT_4, gen_das_lock_args(OWNER_4, Some(MANAGER_4))],
            ]
        }),
    );

    smt
}

pub fn push_commen_renew_sign_witness(template: &mut TemplateGenerator) -> SMTWithHistory {
    let smt = template.push_sub_account_mint_sign_witness(
        DataType::SubAccountRenewSign,
        json!({
            "version": 1,
            "expired_at": TIMESTAMP + DAY_SEC,
            "account_list_smt_root": [
                [SUB_ACCOUNT_1, gen_das_lock_args(OWNER_1, Some(MANAGER_1))],
                [SUB_ACCOUNT_2, gen_das_lock_args(OWNER_2, Some(MANAGER_2))],
                [SUB_ACCOUNT_3, gen_das_lock_args(OWNER_3, Some(MANAGER_3))],
                [SUB_ACCOUNT_4, gen_das_lock_args(OWNER_4, Some(MANAGER_4))],
            ]
        }),
    );

    smt
}

pub fn get_compiled_proof(smt: &SMTWithHistory, key: &str) -> String {
    let key = H256::from(util::gen_smt_key_from_account(key));
    let proof = smt.get_compiled_proof(vec![key]);

    format!("0x{}", hex::encode(proof))
}

pub fn push_simple_input_sub_account_cell_with_custom_script(
    template: &mut TemplateGenerator,
    das_profit: u64,
    owner_profit: u64,
    script_args: &str,
) {
    push_input_sub_account_cell_v2(
        template,
        json!({
            "header": {
                "height": HEIGHT - 1,
                "timestamp": TIMESTAMP - DAY_SEC,
            },
            "data": {
                "das_profit": das_profit,
                "owner_profit": owner_profit,
                "flag": SubAccountConfigFlag::CustomScript as u8,
                // 0x0000000000000000000000000000746573742d637573746f6d2d7363726970740011223300 means args of type ID 0x0c133a395b06d1bdb953f4a7f02bbd0d2eba99d3eb50de9de80ac7c741ed11e7 of custom script.
                "custom_script": "0x0000000000000000000000000000746573742d637573746f6d2d736372697074",
                "script_args": script_args
            }
        }),
        ACCOUNT_1,
    );
}

pub fn push_simple_output_sub_account_cell_with_custom_script(
    template: &mut TemplateGenerator,
    das_profit: u64,
    owner_profit: u64,
    script_args: &str,
) {
    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell_v2(
        template,
        json!({
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "das_profit": das_profit,
                "owner_profit": owner_profit,
                "flag": SubAccountConfigFlag::CustomScript as u8,
                // 0x0000000000000000000000000000746573742d637573746f6d2d7363726970740011223300 means args of type ID 0x0c133a395b06d1bdb953f4a7f02bbd0d2eba99d3eb50de9de80ac7c741ed11e7 of custom script.
                "custom_script": "0x0000000000000000000000000000746573742d637573746f6d2d736372697074",
                "script_args": script_args
            }
        }),
        ACCOUNT_1,
    );
}

pub fn get_profit_of_each_role(total_profit: u64, account_count: u64) -> (u64, u64) {
    let minimal_das_profit = util::gen_sub_account_register_fee(SUB_ACCOUNT_NEW_PRICE, account_count);
    let mut das_profit = total_profit * SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE / RATE_BASE;
    if das_profit < minimal_das_profit {
        das_profit = minimal_das_profit;
    }
    let owner_profit = total_profit - das_profit;

    (das_profit, owner_profit)
}

pub fn push_common_output_cells_with_custom_script(template: &mut TemplateGenerator, account_count: u64) {
    let minimal_das_profit = util::gen_sub_account_register_fee(SUB_ACCOUNT_NEW_PRICE, account_count);
    let total_profit = util::gen_sub_account_register_fee(SUB_ACCOUNT_NEW_CUSTOM_PRICE, account_count);
    let (das_profit, owner_profit) = get_profit_of_each_role(total_profit, account_count);
    push_simple_output_sub_account_cell_with_custom_script(template, das_profit, owner_profit, SCRIPT_ARGS);
    push_output_normal_cell(template, TOTAL_PAID - minimal_das_profit, OWNER);
}
