use crate::util::{self, constants::*, template_generator::*};
use das_types::{constants::DataType, packed::*};
use serde_json::json;

pub fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, Some(Bytes::from(vec![0])));

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("fake-das-lock", true);
    template.push_contract_cell("balance-cell-type", false);
    template.push_contract_cell("offer-cell-type", false);

    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellSecondaryMarket, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellUnAvailableAccount, true, 0, Source::CellDep);

    template
}

pub fn init_with_timestamp(action: &str) -> (TemplateGenerator, u64) {
    let mut template = init(action);
    let timestamp = 1611200000u64;

    template.push_contract_cell("account-cell-type", false);
    template.push_contract_cell("income-cell-type", false);

    template.push_oracle_cell(1, OracleCellType::Time, timestamp);

    template.push_config_cell(DataType::ConfigCellAccount, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellIncome, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellProfitRate, true, 0, Source::CellDep);

    (template, timestamp)
}

pub fn push_input_offer_cell(
    template: &mut TemplateGenerator,
    capacity: u64,
    owner: &str,
    account: &str,
    price: u64,
    message: &str,
) {
    template.push_input(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner,
            },
            "type": {
                "code_hash": "{{offer-cell-type}}"
            },
            "witness": {
                "account": account,
                "price": price.to_string(),
                "message": message,
                "inviter_lock": {
                    "code_hash": "{{fake-das-lock}}",
                    "args": gen_das_lock_args("0x050000000000000000000000000000000000007777", None)
                },
                "channel_lock": {
                    "code_hash": "{{fake-das-lock}}",
                    "args": gen_das_lock_args("0x050000000000000000000000000000000000008888", None)
                }
            }
        }),
        None,
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

pub fn push_output_offer_cell(
    template: &mut TemplateGenerator,
    capacity: u64,
    owner: &str,
    account: &str,
    price: u64,
    message: &str,
) {
    template.push_output(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner,
            },
            "type": {
                "code_hash": "{{offer-cell-type}}"
            },
            "witness": {
                "account": account,
                "price": price.to_string(),
                "message": message,
                "inviter_lock": {
                    "code_hash": "{{fake-das-lock}}",
                    "args": gen_das_lock_args("0x050000000000000000000000000000000000000001", None)
                },
                "channel_lock": {
                    "code_hash": "{{fake-das-lock}}",
                    "args": gen_das_lock_args("0x050000000000000000000000000000000000000002", None)
                }
            }
        }),
        None,
    );
}
