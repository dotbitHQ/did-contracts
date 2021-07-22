use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;
use das_types::{constants::DataType, packed::*};
use serde_json::json;

fn init(action: &str) -> (TemplateGenerator, u64, u64) {
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("apply-register-cell-type", false);

    let height = 1_000_000u64;
    template.push_oracle_cell(1, OracleCellType::Height, height);
    let timestamp = 1611200000u64;
    template.push_oracle_cell(1, OracleCellType::Time, timestamp);

    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellApply, true, 0, Source::CellDep);

    (template, height, timestamp)
}

#[test]
fn gen_apply_register_basic() {
    let (mut template, height, timestamp) = init("apply_register");

    template.push_apply_register_cell(
        "0x0000000000000000000000000000000000000000",
        "das00001.bit",
        height,
        timestamp,
        0,
        Source::Output,
    );

    template.write_template("apply_register_basic.json");
}

test_with_template!(test_apply_register_basic, "apply_register_basic.json");

challenge_with_generator!(
    challenge_apply_register_consuming_cell,
    Error::InvalidTransactionStructure,
    || {
        let (mut template, height, timestamp) = init("apply_register");

        template.push_apply_register_cell(
            "0x0000000000000000000000000000000000001111",
            "das00001.bit",
            height,
            timestamp,
            0,
            Source::Input,
        );
        template.push_apply_register_cell(
            "0x0000000000000000000000000000000000002222",
            "das00001.bit",
            height,
            timestamp,
            0,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_apply_register_data_too_small,
    Error::InvalidCellData,
    || {
        let (mut template, _, _) = init("apply_register");

        // The size of data is less than 48 bytes.
        let raw_data = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let cell_data = Bytes::from(raw_data.as_ref());
        let lock_script = json!({
            "code_hash": "{{always_success}}",
            "args": "0x0000000000000000000000000000000000001111"
        });
        let type_script = json!({
            "code_hash": "{{apply-register-cell-type}}"
        });
        template.push_cell(0, lock_script, type_script, Some(cell_data), Source::Output);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_apply_register_data_too_big,
    Error::InvalidCellData,
    || {
        let (mut template, _, _) = init("apply_register");

        // The size of data is less than 48 bytes.
        let raw_data = [
            0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let cell_data = Bytes::from(raw_data.as_ref());
        let lock_script = json!({
            "code_hash": "{{always_success}}",
            "args": "0x0000000000000000000000000000000000001111"
        });
        let type_script = json!({
            "code_hash": "{{apply-register-cell-type}}"
        });
        template.push_cell(0, lock_script, type_script, Some(cell_data), Source::Output);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_apply_register_height_equal_to_height_cell,
    Error::ApplyRegisterCellHeightInvalid,
    || {
        let (mut template, height, timestamp) = init("apply_register");

        template.push_apply_register_cell(
            "0x0000000000000000000000000000000000000000",
            "das00001.bit",
            height - 1,
            timestamp,
            0,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_apply_register_height_equal_to_height_cell_2,
    Error::ApplyRegisterCellHeightInvalid,
    || {
        let (mut template, height, timestamp) = init("apply_register");

        template.push_apply_register_cell(
            "0x0000000000000000000000000000000000000000",
            "das00001.bit",
            height + 1,
            timestamp,
            0,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_apply_register_time_equal_to_time_cell,
    Error::ApplyRegisterCellTimeInvalid,
    || {
        let (mut template, height, timestamp) = init("apply_register");

        template.push_apply_register_cell(
            "0x0000000000000000000000000000000000000000",
            "das00001.bit",
            height,
            timestamp - 1,
            0,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_apply_register_time_equal_to_time_cell_2,
    Error::ApplyRegisterCellTimeInvalid,
    || {
        let (mut template, height, timestamp) = init("apply_register");

        template.push_apply_register_cell(
            "0x0000000000000000000000000000000000000000",
            "das00001.bit",
            height,
            timestamp + 1,
            0,
            Source::Output,
        );

        template.as_json()
    }
);

#[test]
fn gen_apply_register_refund() {
    let (mut template, height, timestamp) = init("refund_apply");

    template.push_apply_register_cell(
        "0x0000000000000000000000000000000000000000",
        "das00001.bit",
        height - 5761,
        timestamp,
        20_000_000_000,
        Source::Input,
    );
    template.push_signall_cell(
        "0x0000000000000000000000000000000000000000",
        19_900_000_000,
        Source::Output,
    );

    template.write_template("apply_register_refund.json");
}

test_with_template!(test_apply_register_refund, "apply_register_refund.json");

challenge_with_generator!(
    challenge_apply_register_refund_too_early,
    Error::ApplyRegisterRefundNeedWaitLonger,
    || {
        let (mut template, height, timestamp) = init("refund_apply");

        template.push_apply_register_cell(
            "0x0000000000000000000000000000000000000000",
            "das00001.bit",
            height - 5760,
            timestamp,
            20_000_000_000,
            Source::Input,
        );
        template.push_signall_cell(
            "0x0000000000000000000000000000000000000000",
            19_900_000_000,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_apply_register_refund_capacity_error,
    Error::ApplyRegisterRefundCapacityError,
    || {
        let (mut template, height, timestamp) = init("refund_apply");

        template.push_apply_register_cell(
            "0x0000000000000000000000000000000000000000",
            "das00001.bit",
            height - 5761,
            timestamp,
            20_000_000_000,
            Source::Input,
        );
        template.push_signall_cell(
            "0x0000000000000000000000000000000000000000",
            19_800_000_000,
            Source::Output,
        );

        template.as_json()
    }
);
