use das_types::constants::*;
use lazy_static::lazy_static;
use serde_json::{json, Value};

use crate::util::constants::*;
use crate::util::since_util::SinceFlag;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;

pub const ACCOUNT_SP_1: &str = "✨das🎉001.bit";
pub const INPUT_CAPACITY_OF_REFUND_LOCK: u64 = 6_100_000_000;

lazy_static! {
    pub static ref SINCE_1_D: Option<u64> = gen_since(SinceFlag::Relative, SinceFlag::Timestamp, DAY_SEC);
    pub static ref SINCE_1_H: Option<u64> = gen_since(SinceFlag::Relative, SinceFlag::Timestamp, HOUR_SEC);
}

pub fn init(args: Value) -> TemplateGenerator {
    let action = args["action"].as_str().unwrap_or("pre_register");
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("apply-register-cell-type", ContractType::Contract);
    template.push_contract_cell("pre-account-cell-type", ContractType::Contract);

    if action == "pre_register" {
        if !args["has_super_lock"].as_bool().unwrap_or(false) {
            template.push_oracle_cell(1, OracleCellType::Height, HEIGHT);
            template.push_oracle_cell(1, OracleCellType::Time, args["timestamp"].as_u64().unwrap_or(TIMESTAMP));
            template.push_oracle_cell(1, OracleCellType::Quote, CKB_QUOTE);
        }
    }

    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);
    if action == "pre_register" {
        template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);
        template.push_config_cell(DataType::ConfigCellApply, Source::CellDep);
        template.push_config_cell(DataType::ConfigCellCharSetEmoji, Source::CellDep);
        template.push_config_cell(DataType::ConfigCellCharSetDigit, Source::CellDep);
        template.push_config_cell(DataType::ConfigCellCharSetEn, Source::CellDep);
        template.push_config_cell(DataType::ConfigCellCharSetVi, Source::CellDep);
        template.push_config_cell(DataType::ConfigCellCharSetTr, Source::CellDep);
        template.push_config_cell(DataType::ConfigCellPrice, Source::CellDep);
        template.push_config_cell(DataType::ConfigCellRelease, Source::CellDep);
        template.push_config_cell(DataType::ConfigCellUnAvailableAccount, Source::CellDep);
        template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, Source::CellDep);

        if let Some(account) = args["account"].as_str() {
            template.push_config_cell_derived_by_account(account, Source::CellDep);
        }

        if !args["has_custom_dep_account_cell"].as_bool().unwrap_or(false) {
            push_dep_simple_account_cell(&mut template);
        }
    }

    template
}

pub fn push_dep_simple_account_cell(template: &mut TemplateGenerator) {
    push_dep_account_cell(
        template,
        json!({
            "data": {
                "id": "0x0000000000000000000000000000000000000000",
                "next": "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
            }
        }),
    );
}

pub fn push_input_simple_apply_register_cell(template: &mut TemplateGenerator, account: &str) {
    push_input_apply_register_cell(
        template,
        json!({
            "header": {
                "height": HEIGHT - 1,
                "timestamp": TIMESTAMP_20221018,
            },
            "data": {
                "account": account
            }
        }),
        gen_since(SinceFlag::Relative, SinceFlag::Height, 1),
    );
}
