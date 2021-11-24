use super::common::{push_dep_account_cell, *};
use crate::util::{constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*};

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

#[test]
fn test_reverse_record_redeclare() {
    let (mut template, account, owner) = before_each();

    // outputs
    push_output_reverse_record_cell(
        &mut template,
        REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY - REVERSE_RECORD_COMMON_FEE,
        owner,
        account,
    );

    test_tx(template.as_json());
}

#[test]
fn challenge_reverse_record_redeclare_no_account_cell() {
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

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_reverse_record_redeclare_no_reverse_record_cell() {
    let (mut template, _, owner) = before_each();

    // outputs
    push_output_balance_cell(
        &mut template,
        REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY - REVERSE_RECORD_COMMON_FEE,
        owner,
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_reverse_record_redeclare_multi_reverse_record_cell() {
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

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_reverse_record_redeclare_owner() {
    let (mut template, account, _) = before_each();

    // outputs
    push_output_reverse_record_cell(
        &mut template,
        REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY,
        // Simulate the ReverseRecordCell.lock is not the sender's lock.
        "0x050000000000000000000000000000000000002222",
        account,
    );

    challenge_tx(template.as_json(), Error::ReverseRecordCellLockError)
}

#[test]
fn challenge_reverse_record_redeclare_capacity() {
    let (mut template, account, owner) = before_each();

    // outputs
    push_output_reverse_record_cell(
        &mut template,
        // Simulate the ReverseRecordCell.capacity is not satisfied the basic requirement.
        REVERSE_RECORD_BASIC_CAPACITY + REVERSE_RECORD_PREPARED_FEE_CAPACITY - REVERSE_RECORD_COMMON_FEE - 1,
        owner,
        account,
    );

    challenge_tx(template.as_json(), Error::ReverseRecordCellCapacityError)
}
