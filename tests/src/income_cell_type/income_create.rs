use crate::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;

use super::common::init;

#[test]
fn gen_income_create() {
    let mut template = init("create_income");

    template.push_signall_cell(
        "0x0000000000000000000000000000000000000000",
        20_000_000_000,
        Source::Input,
    );

    let income_records = vec![IncomeRecordParam {
        belong_to: "0x0000000000000000000000000000000000000000".to_string(),
        capacity: 20_000_000_000,
    }];
    let (cell_data, entity) =
        template.gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
    template.push_income_cell(
        cell_data,
        Some((1, 0, entity)),
        20_000_000_000,
        Source::Output,
    );

    template.write_template("income_create.json");
}

test_with_template!(test_income_create, "income_create.json");

challenge_with_generator!(
    challenge_income_create_capacity_not_equal,
    Error::IncomeCellCapacityError,
    || {
        let mut template = init("create_income");

        template.push_signall_cell(
            "0x0000000000000000000000000000000000000000",
            20_000_000_000,
            Source::Input,
        );

        let income_records = vec![IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000".to_string(),
            capacity: 20_000_000_000,
        }];
        let (cell_data, entity) = template
            .gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
        template.push_income_cell(
            cell_data,
            Some((1, 0, entity)),
            20_000_000_001,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_income_create_more_than_one_record,
    Error::InvalidTransactionStructure,
    || {
        let mut template = init("create_income");

        template.push_signall_cell(
            "0x0000000000000000000000000000000000000000",
            40_000_000_000,
            Source::Input,
        );

        // The newly created IncomeCell should only contains one record which belongs to the creator.
        let income_records = vec![
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000000".to_string(),
                capacity: 20_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000000".to_string(),
                capacity: 20_000_000_000,
            },
        ];
        let (cell_data, entity) = template
            .gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
        template.push_income_cell(
            cell_data,
            Some((1, 0, entity)),
            40_000_000_000,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_income_create_belong_to_error,
    Error::InvalidTransactionStructure,
    || {
        let mut template = init("create_income");

        template.push_signall_cell(
            "0x0000000000000000000000000000000000000000",
            20_000_000_000,
            Source::Input,
        );

        // The belong_to of first record should equal to the creator's lock, which should be 0x0000000000000000000000000000000000000000 here.
        let income_records = vec![IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000001111".to_string(),
            capacity: 20_000_000_000,
        }];
        let (cell_data, entity) = template
            .gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
        template.push_income_cell(
            cell_data,
            Some((1, 0, entity)),
            20_000_000_000,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_income_create_capacity_error,
    Error::InvalidTransactionStructure,
    || {
        let mut template = init("create_income");

        template.push_signall_cell(
            "0x0000000000000000000000000000000000000000",
            20_000_000_001,
            Source::Input,
        );

        // The capacity of the record should equal to the ConfigCellIncome.basic_capacity.
        let income_records = vec![IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000".to_string(),
            capacity: 20_000_000_001,
        }];
        let (cell_data, entity) = template
            .gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
        template.push_income_cell(
            cell_data,
            Some((1, 0, entity)),
            20_000_000_001,
            Source::Output,
        );

        template.as_json()
    }
);
