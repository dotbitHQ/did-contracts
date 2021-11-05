use super::common::*;
use crate::util::{
    constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::TemplateParser,
};
use ckb_testtool::context::Context;

fn before_each() -> (TemplateGenerator, &'static str, &'static str) {
    let mut template = init("redeclare_reverse_record");
    let account = "yyyyy.bit";
    let owner = "0x050000000000000000000000000000000000001111";

    // cell_deps
    push_dep_account_cell(&mut template, account);

    // inputs
    push_input_reverse_record_cell(
        &mut template,
        REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY,
        owner,
        "xxxxx.bit",
    );

    (template, account, owner)
}

test_with_generator!(test_reverse_record_redeclare, || {
    let (mut template, account, owner) = before_each();

    // outputs
    push_output_reverse_record_cell(
        &mut template,
        REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY - REVERSE_RECORD_COMMON_FEE,
        owner,
        account,
    );

    template.as_json()
});

challenge_with_generator!(
    challenge_reverse_record_redeclare_no_account_cell,
    Error::InvalidTransactionStructure,
    || {
        let mut template = init("redeclare_reverse_record");
        let account = "yyyyy.bit";
        let owner = "0x050000000000000000000000000000000000001111";

        // inputs
        push_input_reverse_record_cell(
            &mut template,
            REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY,
            owner,
            "xxxxx.bit",
        );

        // outputs
        push_output_reverse_record_cell(
            &mut template,
            REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY - REVERSE_RECORD_COMMON_FEE,
            owner,
            account,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_reverse_record_redeclare_no_reverse_record_cell,
    Error::InvalidTransactionStructure,
    || {
        let (mut template, _, owner) = before_each();

        // outputs
        push_output_balance_cell(
            &mut template,
            REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY - REVERSE_RECORD_COMMON_FEE,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_reverse_record_redeclare_multi_reverse_record_cell,
    Error::InvalidTransactionStructure,
    || {
        let (mut template, account, owner) = before_each();

        // outputs
        push_output_reverse_record_cell(
            &mut template,
            REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY,
            owner,
            account,
        );
        push_output_reverse_record_cell(
            &mut template,
            REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY,
            owner,
            account,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_reverse_record_redeclare_owner,
    Error::ReverseRecordCellLockError,
    || {
        let (mut template, account, _) = before_each();

        // outputs
        push_output_reverse_record_cell(
            &mut template,
            REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY,
            // Simulate the ReverseRecordCell.lock is not the sender's lock.
            "0x050000000000000000000000000000000000002222",
            account,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_reverse_record_redeclare_capacity,
    Error::ReverseRecordCellCapacityError,
    || {
        let (mut template, account, owner) = before_each();

        // outputs
        push_output_reverse_record_cell(
            &mut template,
            // Simulate the ReverseRecordCell.capacity is not satisfied the basic requirement.
            REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY - REVERSE_RECORD_COMMON_FEE - 1,
            owner,
            account,
        );

        template.as_json()
    }
);
