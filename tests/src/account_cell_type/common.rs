use crate::util::{self, constants::*, template_generator::*};
use das_types::{
    constants::{AccountStatus, DataType},
    packed::*,
};
use serde_json::{json, Value};

pub fn init(action: &str, params_opt: Option<&str>) -> (TemplateGenerator, u64) {
    let mut template = TemplateGenerator::new(action, params_opt.map(|raw| Bytes::from(util::hex_to_bytes(raw))));
    let timestamp = 1611200000u64;

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("fake-das-lock", true);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", true);
    template.push_contract_cell("account-cell-type", false);

    template.push_oracle_cell(1, OracleCellType::Time, timestamp);

    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellAccount, true, 0, Source::CellDep);

    (template, timestamp)
}

pub fn push_input_account_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": util::gen_account_cell_capacity(8),
        "lock": {
            "owner_lock_args": "0x000000000000000000000000000000000000001111",
            "manager_lock_args": "0x000000000000000000000000000000000000001111"
        },
        "type": {
            "code_hash": "{{account-cell-type}}"
        },
        "data": {
            "account": "das00001.bit",
            "next": "das00014.bit",
            "expired_at": u64::MAX,
        },
        "witness": {
            "account": "das00001.bit",
            "registered_at": 0,
            "last_transfer_account_at": 0,
            "last_edit_manager_at": 0,
            "last_edit_records_at": 0,
            "status": (AccountStatus::Normal as u8)
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_input(cell, Some(2));
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

pub fn push_output_account_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": util::gen_account_cell_capacity(8),
        "lock": {
            "owner_lock_args": "0x000000000000000000000000000000000000001111",
            "manager_lock_args": "0x000000000000000000000000000000000000001111"
        },
        "type": {
            "code_hash": "{{account-cell-type}}"
        },
        "data": {
            "account": "das00001.bit",
            "next": "das00014.bit",
            "expired_at": u64::MAX,
        },
        "witness": {
            "account": "das00001.bit",
            "registered_at": 0,
            "last_transfer_account_at": 0,
            "last_edit_manager_at": 0,
            "last_edit_records_at": 0,
            "status": (AccountStatus::Normal as u8)
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_output(cell, Some(2));
}
