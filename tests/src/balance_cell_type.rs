use super::util::{constants::*, hex_to_bytes, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;
use das_types::{constants::DataType, packed::*};
use serde_json::json;

fn init(action: &str, params_opt: Option<&str>) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, params_opt.map(|raw| Bytes::from(hex_to_bytes(raw))));

    template.push_contract_cell("fake-das-lock", true);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", true);
    template.push_contract_cell("balance-cell-type", false);

    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);

    template
}

test_with_generator!(test_balance_only_handle_type_5, || {
    let mut template = init("transfer", None);

    // Testing verification for typed data hash.

    // inputs
    template.push_das_lock_cell(
        "0x000000000000000000000000000000000000001111",
        10_000_000_000,
        Source::Input,
        None,
    );
    template.push_das_lock_cell(
        "0x050000000000000000000000000000000000004444",
        10_000_000_000,
        Source::Input,
        Some("0xbdaab2df14275f411bf0bc41e5e92360731fc78e1441f7752000463ca9686c7f"),
    );
    template.push_das_lock_cell(
        "0x050000000000000000000000000000000000004444",
        10_000_000_000,
        Source::Input,
        None,
    );
    template.push_signall_cell(
        "0x0000000000000000000000000000000000005555",
        10_000_000_000,
        Source::Input,
    );

    // outputs
    template.push_das_lock_cell(
        "0x000000000000000000000000000000000000009999",
        20_000_000_000,
        Source::Output,
        None,
    );
    template.push_signall_cell(
        "0x0000000000000000000000000000000000009999",
        20_000_000_000,
        Source::Output,
    );

    template.as_json()
});

test_with_generator!(test_balance_skip_all, || {
    let mut template = init("transfer", None);

    // Testing das-lock with types should be skipped.

    // inputs
    template.push_das_lock_cell(
        "0x000000000000000000000000000000000000001111",
        10_000_000_000,
        Source::Input,
        None,
    );
    template.push_das_lock_cell(
        "0x030000000000000000000000000000000000002222",
        10_000_000_000,
        Source::Input,
        None,
    );
    template.push_das_lock_cell(
        "0x040000000000000000000000000000000000003333",
        10_000_000_000,
        Source::Input,
        None,
    );
    template.push_signall_cell(
        "0x0000000000000000000000000000000000005555",
        10_000_000_000,
        Source::Input,
    );

    // outputs
    template.push_das_lock_cell(
        "0x000000000000000000000000000000000000009999",
        20_000_000_000,
        Source::Output,
        None,
    );
    template.push_signall_cell(
        "0x0000000000000000000000000000000000009999",
        20_000_000_000,
        Source::Output,
    );

    template.as_json()
});

challenge_with_generator!(
    challenge_balance_without_type_in_outputs,
    Error::BalanceCellFoundSomeOutputsLackOfType,
    || {
        let mut template = init("transfer", None);

        // Challenge cells in outputs without das-lock which they should have.

        // inputs
        template.push_das_lock_cell(
            "0x000000000000000000000000000000000000001111",
            10_000_000_000,
            Source::Input,
            None,
        );
        template.push_signall_cell(
            "0x0000000000000000000000000000000000005555",
            10_000_000_000,
            Source::Input,
        );

        // outputs
        let lock_script = json!({
          "code_hash": "{{fake-das-lock}}",
          "args": "0x000000000000000000000000000000000000009999"
        });
        let type_script = json!(null);
        template.push_cell(20_000_000_000, lock_script, type_script, None, Source::Output);

        template.push_signall_cell(
            "0x0000000000000000000000000000000000009999",
            20_000_000_000,
            Source::Output,
        );

        template.as_json()
    }
);

fn init_with_account_cell_type(action: &str, params_opt: Option<&str>) -> (TemplateGenerator, u64) {
    let mut template = TemplateGenerator::new(action, params_opt.map(|raw| Bytes::from(hex_to_bytes(raw))));
    let timestamp = 1611200000u64;

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("fake-das-lock", true);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", true);
    template.push_contract_cell("account-cell-type", false);
    template.push_contract_cell("balance-cell-type", false);

    template.push_oracle_cell(1, OracleCellType::Time, timestamp);

    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellAccount, true, 0, Source::CellDep);

    (template, timestamp)
}

test_with_generator!(test_balance_work_with_other_type, || {
    let (mut template, timestamp) = init_with_account_cell_type("transfer_account", Some("0x00"));

    let account = "das00001.bit";
    let next_account = "das00014.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;

    // inputs
    let (cell_data, old_entity) =
        template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
    template.push_account_cell::<AccountCellData>(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000001111",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
        Source::Input,
    );
    template.push_das_lock_witness("4b16a36e0d544d42966106702e07eacf673b0ce5bd3764d701db3101294f7ecd");

    template.push_das_lock_cell(
        "0x000000000000000000000000000000000000001111",
        10_000_000_000,
        Source::Input,
        None,
    );

    // outputs
    let (cell_data, new_entity) =
        template.gen_account_cell_data(account, next_account, registered_at, expired_at, timestamp, 0, 0, None);
    template.push_account_cell::<AccountCellData>(
        "0x0000000000000000000000000000000000002222",
        "0x0000000000000000000000000000000000002222",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY - ACCOUNT_OPERATE_FEE,
        Source::Output,
    );

    template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
        DataType::AccountCellData,
        Some((2, 0, new_entity)),
        Some((2, 0, old_entity)),
        None,
    );

    template.as_json()
});
