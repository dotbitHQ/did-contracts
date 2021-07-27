use crate::util;
use crate::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;
use das_types::constants::DataType;

use super::common::init as common_init;

fn init(action: &str) -> TemplateGenerator {
    let mut template = common_init(action);
    template.push_config_cell(DataType::ConfigCellProfitRate, true, 0, Source::CellDep);

    template
}

#[test]
fn gen_income_consolidate_need_pad() {
    let mut template = init("consolidate_income");

    let capacity_of_10 = 20_000_000_000;
    let capacity_of_20 = 10_200_000_000;

    // inputs
    let records_param = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000".to_string(),
            capacity: 20_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000010".to_string(),
            capacity: capacity_of_10 / 2, // 100 CKB
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000020".to_string(),
            capacity: capacity_of_20 - 200_000_000, // 100 CKB
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000030".to_string(),
            capacity: 100_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000040".to_string(),
            capacity: 9_900_000_000,
        },
    ];
    push_income_cell!(template, records_param, 0, Source::Input);

    let records_param = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000".to_string(),
            capacity: 20_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000010".to_string(),
            capacity: capacity_of_10 / 2, // 100 CKB
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000020".to_string(),
            capacity: capacity_of_20 - 10_000_000_000, // 2 CKB
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000030".to_string(),
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
    let records_param = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000010".to_string(),
            capacity: 10_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000030".to_string(),
            capacity: 200_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000040".to_string(),
            capacity: 9_900_000_000,
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
        9_900_000_000,
        Source::Output,
    );
    template.push_signall_cell(
        "0x0000000000000000000000000000000000000020",
        10_098_000_000,
        Source::Output,
    );
    // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF can take some from user as their profit.
    template.push_signall_cell(
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        6_300_000_000,
        Source::Output,
    );

    template.write_template("income_consolidate.json");
}

test_with_template!(test_income_consolidate_need_pad, "income_consolidate.json");

test_with_generator!(test_income_consolidate_need_pad_2, || {
    let mut template = init("consolidate_income");

    // inputs
    let mut records_param = Vec::new();
    for i in 0u64..50 {
        records_param.push(IncomeRecordParam {
            belong_to: util::hex_string(&i.to_be_bytes()),
            capacity: 5_000_000_000,
        })
    }
    push_income_cell!(template, records_param, 0, Source::Input);

    let records_param = vec![
        IncomeRecordParam {
            belong_to: "0x000000000000EEEE".to_string(),
            capacity: 5_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x000000000000FFFF".to_string(),
            capacity: 20_000_000_000,
        },
    ];
    push_income_cell!(template, records_param, 1, Source::Input);

    // outputs
    let mut records_param = Vec::new();
    for i in 0u64..25 {
        records_param.push(IncomeRecordParam {
            belong_to: util::hex_string(&i.to_be_bytes()),
            capacity: 5_000_000_000,
        })
    }
    push_income_cell!(template, records_param, 0, Source::Output);

    let mut records_param = Vec::new();
    for i in 25u64..50 {
        records_param.push(IncomeRecordParam {
            belong_to: util::hex_string(&i.to_be_bytes()),
            capacity: 5_000_000_000,
        })
    }
    records_param.push(IncomeRecordParam {
        belong_to: "0x000000000000EEEE".to_string(),
        capacity: 5_000_000_000,
    });
    push_income_cell!(template, records_param, 1, Source::Output);

    template.push_signall_cell("0x000000000000FFFF", 19_800_000_000, Source::Output);

    template.as_json()
});

test_with_generator!(test_income_consolidate_no_pad, || {
    let mut template = init("consolidate_income");

    // inputs
    let records_param = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000".to_string(),
            capacity: 20_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000010".to_string(),
            capacity: 10_000_000_000,
        },
    ];
    push_income_cell!(template, records_param, 0, Source::Input);

    let records_param = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000".to_string(),
            capacity: 20_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000010".to_string(),
            capacity: 200_000_000,
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
        10_098_000_000,
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

test_with_generator!(test_income_consolidate_free_fee, || {
    let mut template = init("consolidate_income");

    // inputs
    let records_param = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000".to_string(),
            capacity: 20_000_000_000,
        },
        IncomeRecordParam {
            belong_to: "0x0300000000000000000000000000000000000000".to_string(),
            capacity: 10_000_000_000,
        },
    ];
    push_income_cell!(template, records_param, 0, Source::Input);

    let records_param = vec![IncomeRecordParam {
        belong_to: "0x0300000000000000000000000000000000000000".to_string(),
        capacity: 10_000_000_000,
    }];
    push_income_cell!(template, records_param, 1, Source::Input);

    // outputs
    template.push_signall_cell(
        "0x0000000000000000000000000000000000000000",
        20_000_000_000,
        Source::Output,
    );
    // DAS should be free from consolidating fee.
    template.push_signall_cell(
        "0x0300000000000000000000000000000000000000",
        20_000_000_000,
        Source::Output,
    );

    template.as_json()
});

test_with_generator!(test_income_consolidate_big_capacity, || {
    let mut template = init("consolidate_income");

    let capacity_of_10 = 1000_000_000_000_000_000;

    // inputs
    let records_param = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000010".to_string(),
            capacity: capacity_of_10,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000020".to_string(),
            capacity: 500_000_000,
        },
    ];
    push_income_cell!(template, records_param, 0, Source::Input);

    let records_param = vec![IncomeRecordParam {
        belong_to: "0x0000000000000000000000000000000000000010".to_string(),
        capacity: capacity_of_10,
    }];
    push_income_cell!(template, records_param, 1, Source::Input);

    // outputs
    let records_param = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000010".to_string(),
            capacity: capacity_of_10,
        },
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000020".to_string(),
            capacity: 500_000_000,
        },
    ];
    push_income_cell!(template, records_param, 0, Source::Output);

    template.push_signall_cell(
        "0x0000000000000000000000000000000000000010",
        capacity_of_10 / RATE_BASE * (RATE_BASE - CONSOLIDATING_FEE),
        Source::Output,
    );
    template.push_signall_cell(
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        6_162_000_000,
        Source::Output,
    );

    template.as_json()
});

challenge_with_generator!(
    challenge_income_consolidate_newly_created,
    Error::IncomeCellConsolidateConditionNotSatisfied,
    || {
        let mut template = init("consolidate_income");

        // inputs
        // This IncomeCell only contains one record of the creator, it should not be consolidated.
        let records_param = vec![IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000".to_string(),
            capacity: 20_000_000_000,
        }];
        push_income_cell!(template, records_param, 0, Source::Input);

        let records_param = vec![
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000000".to_string(),
                capacity: 20_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000010".to_string(),
                capacity: 1_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000020".to_string(),
                capacity: 1_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000030".to_string(),
                capacity: 500_000_000,
            },
        ];
        push_income_cell!(template, records_param, 1, Source::Input);

        // outputs
        let records_param = vec![
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000000".to_string(),
                capacity: 20_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000010".to_string(),
                capacity: 1_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000020".to_string(),
                capacity: 1_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000030".to_string(),
                capacity: 500_000_000,
            },
        ];
        push_income_cell!(template, records_param, 0, Source::Output);

        template.push_signall_cell(
            "0x0000000000000000000000000000000000000000",
            20_000_000_000,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_income_consolidate_no_redundant_records,
    Error::IncomeCellConsolidateError,
    || {
        let mut template = init("consolidate_income");

        // inputs
        let records_param = vec![
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000000".to_string(),
                capacity: 20_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000010".to_string(),
                capacity: 1_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000020".to_string(),
                capacity: 1_000_000_000,
            },
        ];
        push_income_cell!(template, records_param, 0, Source::Input);

        let records_param = vec![
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000000".to_string(),
                capacity: 20_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000010".to_string(),
                capacity: 1_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000020".to_string(),
                capacity: 1_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000030".to_string(),
                capacity: 500_000_000,
            },
        ];
        push_income_cell!(template, records_param, 1, Source::Input);

        // outputs
        let records_param = vec![
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000000".to_string(),
                capacity: 20_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000010".to_string(),
                capacity: 2_000_000_000,
            },
            // The consolidated records should not contain more than one record which has the same belong_to field.
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000020".to_string(),
                capacity: 1_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000020".to_string(),
                capacity: 1_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000030".to_string(),
                capacity: 500_000_000,
            },
        ];
        push_income_cell!(template, records_param, 0, Source::Output);

        template.push_signall_cell(
            "0x0000000000000000000000000000000000000000",
            20_000_000_000,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_income_consolidate_no_redundant_cells,
    Error::IncomeCellConsolidateConditionNotSatisfied,
    || {
        let mut template = init("consolidate_income");

        // inputs
        let records_param = vec![
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000000".to_string(),
                capacity: 20_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000010".to_string(),
                capacity: 1_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000030".to_string(),
                capacity: 21_000_000_000,
            },
        ];
        push_income_cell!(template, records_param, 0, Source::Input);

        let records_param = vec![
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000000".to_string(),
                capacity: 20_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000020".to_string(),
                capacity: 1_000_000_000,
            },
        ];
        push_income_cell!(template, records_param, 1, Source::Input);

        // outputs
        let records_param = vec![
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000000".to_string(),
                capacity: 20_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000010".to_string(),
                capacity: 1_000_000_000,
            },
        ];
        push_income_cell!(template, records_param, 0, Source::Output);

        let records_param = vec![
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000000".to_string(),
                capacity: 20_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000020".to_string(),
                capacity: 1_000_000_000,
            },
        ];
        push_income_cell!(template, records_param, 1, Source::Output);

        let records_param = vec![IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000030".to_string(),
            capacity: 21_000_000_000,
        }];
        push_income_cell!(template, records_param, 2, Source::Output);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_income_consolidate_missing_some_records,
    Error::IncomeCellConsolidateError,
    || {
        let mut template = init("consolidate_income");

        // inputs
        let mut records_param = Vec::new();
        for i in 0u64..50 {
            records_param.push(IncomeRecordParam {
                belong_to: util::hex_string(&i.to_be_bytes()),
                capacity: 5_000_000_000,
            })
        }
        push_income_cell!(template, records_param, 0, Source::Input);

        let records_param = vec![
            IncomeRecordParam {
                belong_to: "0x000000000000EEEE".to_string(),
                capacity: 5_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x000000000000FFFF".to_string(),
                capacity: 20_000_000_000,
            },
        ];
        push_income_cell!(template, records_param, 1, Source::Input);

        // outputs
        let mut records_param = Vec::new();
        for i in 0u64..25 {
            records_param.push(IncomeRecordParam {
                belong_to: util::hex_string(&i.to_be_bytes()),
                capacity: 5_000_000_000,
            })
        }
        push_income_cell!(template, records_param, 0, Source::Output);

        let mut records_param = Vec::new();
        for i in 25u64..50 {
            records_param.push(IncomeRecordParam {
                belong_to: util::hex_string(&i.to_be_bytes()),
                capacity: 5_000_000_000,
            })
        }
        push_income_cell!(template, records_param, 1, Source::Output);

        template.push_signall_cell("0x000000000000FFFF", 19_800_000_000, Source::Output);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_income_consolidate_wasted_capacity,
    Error::IncomeCellConsolidateWaste,
    || {
        let mut template = init("consolidate_income");

        let capacity_of_10 = 20_000_000_000;
        let capacity_of_20 = 10_200_000_000;

        // inputs
        let records_param = vec![
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000010".to_string(),
                capacity: capacity_of_10 / 2, // 100 CKB
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000020".to_string(),
                capacity: capacity_of_20 - 200_000_000, // 100 CKB
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000040".to_string(),
                capacity: 9_900_000_000,
            },
        ];
        push_income_cell!(template, records_param, 0, Source::Input);

        let records_param = vec![
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000010".to_string(),
                capacity: capacity_of_10 / 2, // 100 CKB
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000020".to_string(),
                capacity: capacity_of_20 - 10_000_000_000, // 2 CKB
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000030".to_string(),
                capacity: 200_000_000,
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
                belong_to: "0x0000000000000000000000000000000000000010".to_string(),
                capacity: 10_000_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000030".to_string(),
                capacity: 200_000_000,
            },
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000040".to_string(),
                capacity: 9_900_000_000,
            },
        ];
        push_income_cell!(template, records_param, 0, Source::Output);

        template.push_signall_cell(
            "0x0000000000000000000000000000000000000010",
            9_900_000_000,
            Source::Output,
        );
        // Waste 198_000_000 CKBytes
        template.push_signall_cell(
            "0x0000000000000000000000000000000000000020",
            9_900_000_000,
            Source::Output,
        );
        // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF can take some from user as their profit.
        template.push_signall_cell(
            "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
            6_300_000_000,
            Source::Output,
        );

        template.as_json()
    }
);
