use super::common::*;
use crate::util::{error::Error, template_common_cell::*, template_generator::*, template_parser::*};

fn before_each() -> (TemplateGenerator, &'static str) {
    let mut template = init("retract_reverse_record");
    let owner = "0x050000000000000000000000000000000000001111";

    // inputs
    push_input_reverse_record_cell(&mut template, 20_100_000_000, owner, "xxxxx.bit");
    push_input_reverse_record_cell(&mut template, 20_100_000_000, owner, "yyyyy.bit");
    push_input_reverse_record_cell(&mut template, 20_100_000_000, owner, "zzzzz.bit");

    (template, owner)
}

#[test]
fn test_reverse_record_retract() {
    let (mut template, owner) = before_each();

    // outputs
    push_output_balance_cell(&mut template, 20_100_000_000 * 3 - 10_000, owner);

    test_tx(template.as_json());
}

#[test]
fn challenge_reverse_record_retract_redundant_cells() {
    let (mut template, owner) = before_each();

    // inputs
    // Simulate containing redundant cells in inputs.
    push_input_balance_cell(&mut template, 10_000_000_000, owner);

    // outputs
    push_output_balance_cell(&mut template, 20_100_000_000 * 3 - 10_000, owner);

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_reverse_record_retract_reverse_record_cell_of_multi_lock() {
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

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_reverse_record_retract_change_owner() {
    let (mut template, _) = before_each();

    // outputs
    push_output_balance_cell(
        &mut template,
        20_100_000_000 * 3 - 10_000,
        // Simulate transfer change to another lock.
        "0x050000000000000000000000000000000000002222",
    );

    challenge_tx(template.as_json(), Error::ChangeError)
}

#[test]
fn challenge_reverse_record_retract_change_capacity() {
    let (mut template, owner) = before_each();

    // outputs
    push_output_balance_cell(
        &mut template,
        // Simulate transfer changes less than the user should get.
        20_100_000_000 * 3 - 10_000 - 1,
        owner,
    );

    challenge_tx(template.as_json(), Error::ChangeError)
}
