use crate::util::{constants::*, template_generator::*};
use chrono::{TimeZone, Utc};
use das_types_std::constants::*;

pub fn init_without_apply(account: &str) -> (TemplateGenerator, &str, u64, u64) {
    let mut template = TemplateGenerator::new("pre_register", None);

    let timestamp = Utc.ymd(2021, 7, 7).and_hms(14, 0, 0).timestamp() as u64;
    let height = 1000000u64;

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("fake-das-lock", true);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", true);
    template.push_contract_cell("apply-register-cell-type", false);
    template.push_contract_cell("pre-account-cell-type", false);

    template.push_oracle_cell(1, OracleCellType::Height, height);
    template.push_oracle_cell(1, OracleCellType::Time, timestamp);
    template.push_oracle_cell(1, OracleCellType::Quote, 1000);

    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellApply, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetEmoji, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetDigit, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetEn, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellPrice, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellRelease, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellUnAvailableAccount, Source::CellDep);

    (template, account, timestamp, height)
}

pub fn init(account: &str) -> (TemplateGenerator, &str, u64) {
    let (mut template, account, timestamp, height) = init_without_apply(account);

    template.push_apply_register_cell(
        "0x9af92f5e690f4669ca543deb99af8385b12624cc",
        account,
        height - 4,
        timestamp - 60,
        0,
        Source::Input,
    );

    (template, account, timestamp)
}
