use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;
use das_types::constants::DataType;

macro_rules! push_income_cell {
    ( $template:expr, $records_param:expr, $index:expr, $source:expr ) => {{
        let (cell_data, entity) = $template.gen_income_cell_data(
            "0x0000000000000000000000000000000000000000",
            $records_param.clone(),
        );
        $template.push_income_cell(
            cell_data,
            Some((1, $index, entity)),
            $records_param
                .iter()
                .map(|item| item.capacity)
                .reduce(|a, b| a + b)
                .unwrap(),
            $source,
        );
    }};
}

fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("income-cell-type", false);

    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellIncome, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellProfitRate, true, 0, Source::CellDep);

    template
}

#[test]
fn gen_income_create() {
    let mut template = init("create_income");

    template.push_signall_cell(
        "0x0000000000000000000000000000000000000000",
        20_000_000_000,
        Source::Input,
    );

    let income_records = vec![IncomeRecordParam {
        belong_to: "0x0000000000000000000000000000000000000000",
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

    template.pretty_print();
}

test_with_template!(test_income_create, "income_create.json");

#[test]
fn gen_income_consolidate() {
    let mut template = init("consolidate_income");

    // inputs
    let records_param = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000",
            capacity: 20_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000010",
            capacity: 19_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000020",
            capacity: 19_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000030",
            capacity: 500_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000040",
            capacity: 500_000_000,
        },
    ];
    push_income_cell!(template, records_param, 0, Source::Input);

    let records_param = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000",
            capacity: 20_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000010",
            capacity: 1_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000020",
            capacity: 1_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000030",
            capacity: 500_000_000,
        },
    ];
    push_income_cell!(template, records_param, 1, Source::Input);

    // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF is the keeper who pushed the consolidate_income transaction.
    template.push_signall_cell(
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        6_100_000_000,
        Source::Input,
    );

    // outputs
    let records_param = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000010",
            capacity: 20_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000030",
            capacity: 1_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000040",
            capacity: 500_000_000,
        },
    ];
    push_income_cell!(template, records_param, 0, Source::Output);

    template.push_signall_cell(
        "0x0000000000000000000000000000000000000000",
        40_000_000_000,
        Source::Output,
    );
    template.push_signall_cell(
        "0x0000000000000000000000000000000000000010",
        19_800_000_000,
        Source::Output,
    );
    // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF can take some from user as their profit.
    template.push_signall_cell(
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        6_300_000_000,
        Source::Output,
    );

    template.pretty_print();
}

test_with_template!(test_income_consolidate_need_pad, "income_consolidate.json");

test_with_generator!(test_income_consolidate_no_pad, || {
    let mut template = init("consolidate_income");

    // inputs
    let records_param = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000",
            capacity: 20_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000010",
            capacity: 6_100_000_000,
        },
    ];
    push_income_cell!(template, records_param, 0, Source::Input);

    let records_param = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000",
            capacity: 20_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000010",
            capacity: 100_000_000,
        },
    ];
    push_income_cell!(template, records_param, 1, Source::Input);

    // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF is the keeper who pushed the consolidate_income transaction.
    template.push_signall_cell(
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        6_100_000_000,
        Source::Input,
    );

    // outputs
    template.push_signall_cell(
        "0x0000000000000000000000000000000000000000",
        40_000_000_000,
        Source::Output,
    );
    template.push_signall_cell(
        "0x0000000000000000000000000000000000000010",
        6_138_000_000,
        Source::Output,
    );
    // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF can take some from user as their profit.
    template.push_signall_cell(
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        6_162_000_000,
        Source::Output,
    );

    template.as_json()
});
