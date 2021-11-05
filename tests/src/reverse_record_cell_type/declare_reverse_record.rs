use super::common::*;
use crate::util::{
    constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::TemplateParser,
};
use ckb_testtool::context::Context;

fn before_each() -> (TemplateGenerator, &'static str, &'static str, u64) {
    let mut template = init("declare_reverse_record");
    let account = "xxxxx.bit";
    let owner = "0x050000000000000000000000000000000000001111";

    // cell_deps
    push_dep_account_cell(&mut template, account);

    // inputs
    let total_input = 100_000_000_000;
    push_input_balance_cell(&mut template, total_input, owner);

    (template, account, owner, total_input)
}

test_with_generator!(test_reverse_record_declare, || {
    let (mut template, account, owner, total_input) = before_each();

    // outputs
    push_output_reverse_record_cell(
        &mut template,
        REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY,
        owner,
        account,
    );
    push_output_balance_cell(
        &mut template,
        total_input - REVERSE_RECORD_BASIC_CAPACITY - REVERSE_RECORD_PREPARED_FEE_CAPACITY - REVERSE_RECORD_COMMON_FEE,
        owner,
    );

    template.as_json()
});

test_with_generator!(test_reverse_record_declare_multiple_balance_cells, || {
    let mut template = init("declare_reverse_record");
    let account = "xxxxx.bit";
    let owner = "0x050000000000000000000000000000000000001111";

    // cell_deps
    push_dep_account_cell(&mut template, account);

    // inputs
    let total_input = 100_000_000_000;
    push_input_balance_cell(&mut template, 100_000_000_000 / 4, owner);
    push_input_balance_cell(&mut template, 100_000_000_000 / 4, owner);
    push_input_balance_cell(&mut template, 100_000_000_000 / 4, owner);
    push_input_balance_cell(&mut template, 100_000_000_000 / 4, owner);

    // outputs
    push_output_reverse_record_cell(
        &mut template,
        REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY,
        owner,
        account,
    );
    let change =
        total_input - REVERSE_RECORD_BASIC_CAPACITY - REVERSE_RECORD_PREPARED_FEE_CAPACITY - REVERSE_RECORD_COMMON_FEE;
    push_output_balance_cell(&mut template, change / 2, owner);
    push_output_balance_cell(&mut template, change / 2, owner);

    template.as_json()
});

challenge_with_generator!(
    challenge_reverse_record_declare_no_account_cell,
    Error::InvalidTransactionStructure,
    || {
        let mut template = init("declare_reverse_record");
        let account = "xxxxx.bit";
        let owner = "0x050000000000000000000000000000000000001111";

        // inputs
        let total_input = 100_000_000_000;
        push_input_balance_cell(&mut template, total_input, owner);

        // outputs
        push_output_reverse_record_cell(
            &mut template,
            REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY,
            owner,
            account,
        );
        push_output_balance_cell(
            &mut template,
            total_input
                - REVERSE_RECORD_BASIC_CAPACITY
                - REVERSE_RECORD_PREPARED_FEE_CAPACITY
                - REVERSE_RECORD_COMMON_FEE,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_reverse_record_declare_no_reverse_record_cell,
    Error::InvalidTransactionStructure,
    || {
        let (mut template, _, owner, total_input) = before_each();

        // outputs
        push_output_balance_cell(
            &mut template,
            REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY,
            owner,
        );
        push_output_balance_cell(
            &mut template,
            total_input
                - REVERSE_RECORD_BASIC_CAPACITY
                - REVERSE_RECORD_PREPARED_FEE_CAPACITY
                - REVERSE_RECORD_COMMON_FEE,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_reverse_record_declare_multi_reverse_record_cell,
    Error::InvalidTransactionStructure,
    || {
        let (mut template, account, owner, total_input) = before_each();

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
        push_output_balance_cell(
            &mut template,
            total_input
                - REVERSE_RECORD_BASIC_CAPACITY
                - REVERSE_RECORD_PREPARED_FEE_CAPACITY
                - REVERSE_RECORD_COMMON_FEE,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_reverse_record_declare_account,
    Error::ReverseRecordCellAccountError,
    || {
        let (mut template, _, owner, total_input) = before_each();

        // outputs
        push_output_reverse_record_cell(
            &mut template,
            REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY,
            owner,
            // Simulate the ReverseRecordCell.data.account is not the same as the AccountCell.data.account.
            "yyyyy.bit",
        );
        push_output_balance_cell(
            &mut template,
            total_input
                - REVERSE_RECORD_BASIC_CAPACITY
                - REVERSE_RECORD_PREPARED_FEE_CAPACITY
                - REVERSE_RECORD_COMMON_FEE,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_reverse_record_declare_owner,
    Error::ReverseRecordCellLockError,
    || {
        let (mut template, account, owner, total_input) = before_each();

        // outputs
        push_output_reverse_record_cell(
            &mut template,
            REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY,
            // Simulate the ReverseRecordCell.lock is not the sender's lock.
            "0x050000000000000000000000000000000000002222",
            account,
        );
        push_output_balance_cell(
            &mut template,
            total_input
                - REVERSE_RECORD_BASIC_CAPACITY
                - REVERSE_RECORD_PREPARED_FEE_CAPACITY
                - REVERSE_RECORD_COMMON_FEE,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_reverse_record_declare_capacity,
    Error::ReverseRecordCellCapacityError,
    || {
        let (mut template, account, owner, total_input) = before_each();

        // outputs
        push_output_reverse_record_cell(
            &mut template,
            // Simulate the ReverseRecordCell.capacity is not satisfied the basic requirement.
            REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY - 1,
            owner,
            account,
        );
        push_output_balance_cell(
            &mut template,
            total_input
                - REVERSE_RECORD_BASIC_CAPACITY
                - REVERSE_RECORD_PREPARED_FEE_CAPACITY
                - REVERSE_RECORD_COMMON_FEE,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_reverse_record_declare_payment_not_enough,
    Error::InvalidTransactionStructure,
    || {
        let mut template = init("declare_reverse_record");
        let account = "xxxxx.bit";
        let owner = "0x050000000000000000000000000000000000001111";

        // cell_deps
        push_dep_account_cell(&mut template, account);

        // inputs
        let total_input = REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY - 1;
        push_input_balance_cell(&mut template, total_input, owner);

        // outputs
        push_output_reverse_record_cell(
            &mut template,
            // Simulate the ReverseRecordCell.capacity is not satisfied the basic requirement.
            REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY - 1,
            owner,
            account,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_reverse_record_declare_change_owner,
    Error::ChangeError,
    || {
        let (mut template, account, owner, total_input) = before_each();

        // outputs
        push_output_reverse_record_cell(
            &mut template,
            REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY,
            owner,
            account,
        );
        push_output_balance_cell(
            &mut template,
            total_input
                - REVERSE_RECORD_BASIC_CAPACITY
                - REVERSE_RECORD_PREPARED_FEE_CAPACITY
                - REVERSE_RECORD_COMMON_FEE,
            // Simulate transfer change to another lock.
            "0x050000000000000000000000000000000000002222",
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_reverse_record_declare_change_capacity,
    Error::ChangeError,
    || {
        let (mut template, account, owner, total_input) = before_each();

        // outputs
        push_output_reverse_record_cell(
            &mut template,
            REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY,
            owner,
            account,
        );
        push_output_balance_cell(
            &mut template,
            // Simulate transfer changes less than the user should get.
            total_input
                - REVERSE_RECORD_BASIC_CAPACITY
                - REVERSE_RECORD_PREPARED_FEE_CAPACITY
                - REVERSE_RECORD_COMMON_FEE
                - 1,
            owner,
        );

        template.as_json()
    }
);
