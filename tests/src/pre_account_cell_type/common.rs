use crate::util::{constants::*, template_common_cell::*, template_generator::*};
use das_types_std::constants::*;
use serde_json::json;

pub const ACCOUNT_SP_1: &str = "✨das🎉001.bit";

pub fn init() -> TemplateGenerator {
    let mut template = TemplateGenerator::new("pre_register", None);

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("fake-das-lock", true);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", true);
    template.push_contract_cell("apply-register-cell-type", false);
    template.push_contract_cell("pre-account-cell-type", false);

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

pub fn init_for_refund() -> TemplateGenerator {
    let mut template = TemplateGenerator::new("refund_pre_register", None);

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("fake-das-lock", true);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", true);
    template.push_contract_cell("apply-register-cell-type", false);
    template.push_contract_cell("pre-account-cell-type", false);

    template.push_oracle_cell(1, OracleCellType::Time, TIMESTAMP);

    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);

    template
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
    );
}
