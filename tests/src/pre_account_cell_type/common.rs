use das_types_std::constants::*;
use lazy_static::lazy_static;
use serde_json::json;

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

pub fn init() -> TemplateGenerator {
    let mut template = TemplateGenerator::new("pre_register", None);

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("apply-register-cell-type", ContractType::Contract);
    template.push_contract_cell("pre-account-cell-type", ContractType::Contract);

    template.push_oracle_cell(1, OracleCellType::Height, HEIGHT);
    template.push_oracle_cell(1, OracleCellType::Time, TIMESTAMP);
    template.push_oracle_cell(1, OracleCellType::Quote, CKB_QUOTE);

    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellApply, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetEmoji, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetDigit, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetEn, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellPrice, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellRelease, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellUnAvailableAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, Source::CellDep);

    template
}

pub fn init_with_timestamp(timestamp: u64) -> TemplateGenerator {
    let mut template = TemplateGenerator::new("pre_register", None);

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("apply-register-cell-type", ContractType::Contract);
    template.push_contract_cell("pre-account-cell-type", ContractType::Contract);

    template.push_oracle_cell(1, OracleCellType::Height, HEIGHT);
    template.push_oracle_cell(1, OracleCellType::Time, timestamp);
    template.push_oracle_cell(1, OracleCellType::Quote, CKB_QUOTE);

    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellApply, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetEmoji, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetDigit, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetEn, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetVi, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetTr, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellPrice, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellRelease, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellUnAvailableAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, Source::CellDep);

    template
}

pub fn init_for_refund() -> TemplateGenerator {
    let mut template = TemplateGenerator::new("refund_pre_register", None);

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("apply-register-cell-type", ContractType::Contract);
    template.push_contract_cell("pre-account-cell-type", ContractType::Contract);

    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);

    template
}

pub fn before_each(account: &str) -> TemplateGenerator {
    let mut template = init();
    template.push_config_cell_derived_by_account(account, Source::CellDep);

    push_dep_simple_account_cell(&mut template);

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
            "data": {
                "account": account,
                "height": HEIGHT - 4,
                "timestamp": TIMESTAMP - 60,
            }
        }),
        None,
    );
}
