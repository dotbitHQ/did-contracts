use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;
use das_types::{constants::DataType, packed::*};
use serde_json::json;

fn init(action: &str) -> (TemplateGenerator, u64) {
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("apply-register-cell-type", false);

    let height = 1_000_000u64;
    template.push_height_cell(1, height, 1000, Source::CellDep);

    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellApply, true, 0, Source::CellDep);

    (template, height)
}

#[test]
fn gen_apply_register_basic() {
    let (mut template, height) = init("apply_register");

    template.push_apply_register_cell(
        "0x0000000000000000000000000000000000000000",
        "das00001.bit",
        height,
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
        let (mut template, height) = init("apply_register");

        template.push_apply_register_cell(
            "0x0000000000000000000000000000000000001111",
            "das00001.bit",
            height,
            0,
            Source::Input,
        );
        template.push_apply_register_cell(
            "0x0000000000000000000000000000000000002222",
            "das00001.bit",
            height,
            0,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_apply_register_invalid_hash_data,
    Error::InvalidCellData,
    || {
        let (mut template, _) = init("apply_register");

        // The size of data is less than 32 bytes.
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
    challenge_apply_register_invalid_height_data,
    Error::InvalidCellData,
    || {
        let (mut template, _) = init("apply_register");

        // The size of data is greater than 32 bytes, but less than 32 + 8 bytes.
        let raw_data = [
            0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
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
    challenge_apply_register_invalid_height_data_2,
    Error::InvalidCellData,
    || {
        let (mut template, _) = init("apply_register");

        // The size of data is greater than 32 + 8 bytes.
        let raw_data = [
            0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
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
        let (mut template, height) = init("apply_register");

        template.push_apply_register_cell(
            "0x0000000000000000000000000000000000000000",
            "das00001.bit",
            height - 1,
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
        let (mut template, height) = init("apply_register");

        template.push_apply_register_cell(
            "0x0000000000000000000000000000000000000000",
            "das00001.bit",
            height + 1,
            0,
            Source::Output,
        );

        template.as_json()
    }
);

#[test]
fn gen_apply_register_refund() {
    let (mut template, height) = init("refund_apply");

    template.push_apply_register_cell(
        "0x0000000000000000000000000000000000000000",
        "das00001.bit",
        height - 5761,
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
        let (mut template, height) = init("refund_apply");

        template.push_apply_register_cell(
            "0x0000000000000000000000000000000000000000",
            "das00001.bit",
            height - 5760,
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
        let (mut template, height) = init("refund_apply");

        template.push_apply_register_cell(
            "0x0000000000000000000000000000000000000000",
            "das00001.bit",
            height - 5761,
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
