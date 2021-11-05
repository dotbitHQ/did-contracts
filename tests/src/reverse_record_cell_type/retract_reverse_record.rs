use super::common::*;
use crate::util::{error::Error, template_common_cell::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;

fn before_each() -> (TemplateGenerator, &'static str) {
    let mut template = init("retract_reverse_record");
    let owner = "0x050000000000000000000000000000000000001111";

    // inputs
    push_input_reverse_record_cell(&mut template, 20_100_000_000, owner, "xxxxx.bit");
    push_input_reverse_record_cell(&mut template, 20_100_000_000, owner, "yyyyy.bit");
    push_input_reverse_record_cell(&mut template, 20_100_000_000, owner, "zzzzz.bit");

    (template, owner)
}

test_with_generator!(test_reverse_record_retract, || {
    let (mut template, owner) = before_each();

    // outputs
    push_output_balance_cell(&mut template, 20_100_000_000 * 3 - 10_000, owner);

    template.as_json()
});

challenge_with_generator!(
    challenge_reverse_record_retract_redundant_cells,
    Error::InvalidTransactionStructure,
    || {
        let (mut template, owner) = before_each();

        // inputs
        // Simulate containing redundant cells in inputs.
        push_input_balance_cell(&mut template, 10_000_000_000, owner);

        // outputs
        push_output_balance_cell(&mut template, 20_100_000_000 * 3 - 10_000, owner);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_reverse_record_retract_reverse_record_cell_of_multi_lock,
    Error::InvalidTransactionStructure,
    || {
        let (mut template, owner) = before_each();

        // inputs
        // Simulate containing ReverseRecordCell with different lock script in inputs.
        push_input_reverse_record_cell(
            &mut template,
            20_100_000_000,
            "0x050000000000000000000000000000000000002222",
            "aaaaa.bit",
        );

        // outputs
        push_output_balance_cell(&mut template, 20_100_000_000 * 4 - 10_000, owner);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_reverse_record_retract_change_owner,
    Error::ChangeError,
    || {
        let (mut template, _) = before_each();

        // outputs
        push_output_balance_cell(
            &mut template,
            20_100_000_000 * 3 - 10_000,
            // Simulate transfer change to another lock.
            "0x050000000000000000000000000000000000002222",
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_reverse_record_retract_change_capacity,
    Error::ChangeError,
    || {
        let (mut template, owner) = before_each();

        // outputs
        push_output_balance_cell(
            &mut template,
            // Simulate transfer changes less than the user should get.
            20_100_000_000 * 3 - 10_000 - 1,
            owner,
        );

        template.as_json()
    }
);
