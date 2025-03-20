use das_types::constants::Source;
use das_types::prelude::*;
use did_cell_type::error::ErrorCode;
use serde_json::{json, Value};

use super::tmpl::{gen_did_cell, gen_did_cell_with_lock};
use crate::did_cell_type::tmpl;
use crate::util::constants::TIMESTAMP;
use crate::util::template_common_cell::push_output_normal_cell;
use crate::util::template_parser::{challenge_tx, test_tx};

#[test]
fn challenge_combined_actions() {
    let mut base = tmpl::init();

    // let one_year: u64 = 365 * 24 * 60 * 60;
    let three_month: u64 = 3 * 30 * 24 * 60 * 60;

    // edit records | transfer account
    let account = b"c.bit";
    tmpl::push_did_cell(&mut base, account, TIMESTAMP, 2, Source::Input);
    tmpl::push_did_cell(&mut base, account, TIMESTAMP, 4, Source::Output);

    // Simulate combining edit and destroy actions in one transaction
    // success destroy account
    let account = b"d.bit";
    tmpl::push_did_cell(&mut base, account, TIMESTAMP - three_month - 1, 4, Source::Input);

    challenge_tx(base.as_json(), ErrorCode::DifferentAction);
}

#[test]
fn challenge_edit_records() {
    let mut base = tmpl::init();

    // edit records | transfer account
    let account = b"c.bit";
    tmpl::push_did_cell(&mut base, account, TIMESTAMP, 2, Source::Input);

    let (cell, _, _, ent3) = gen_did_cell(account, TIMESTAMP, 3);
    let (_, _, _, ent4) = gen_did_cell(account, TIMESTAMP, 4);

    let ent = ent3.as_builder().data(ent4.data()).build();
    let witness = [b"DID".to_vec(), ent.as_bytes().to_vec()].concat();
    let witness_hex = format!("0x{}", hex::encode(witness));
    base.push_cell_json(cell, Source::Output, None);
    base.outer_witnesses.push(witness_hex);

    challenge_tx(base.as_json(), ErrorCode::InvalidDidCellWitnessHash);
}

#[test]
fn challenge_destroy_did_cell() {
    let mut base = tmpl::init();

    let account = b"d.bit";
    tmpl::push_did_cell(&mut base, account, TIMESTAMP, 4, Source::Input);

    challenge_tx(base.as_json(), ErrorCode::NotValidToDestroy);
}

#[test]
fn challenge_transfor_did_cell() {
    let mut base = tmpl::init();

    // transfer account
    let account = b"c.bit";
    tmpl::push_did_cell(&mut base, account, TIMESTAMP, 2, Source::Input);
    tmpl::push_did_cell(&mut base, account, TIMESTAMP + 10, 2, Source::Output);

    challenge_tx(base.as_json(), ErrorCode::ExpireAtNotEqual);
}

#[test]
fn test_validate_capacity_to_larger_lock_1() {
    let mut base = tmpl::init();
    let (mut cell, _, _, _) = gen_did_cell_with_lock(
        b"a.bit",
        json!({
            "code_hash": "{{always_success}}",
            "hash_type": "type",
            "args": ""
        }),
    );
    // Simulating the existing capacity is exactly the same as the required capacity of the new cell.
    cell["capacity"] = Value::from(13_400_000_000u64);
    base.push_cell_json(cell, Source::Input, None);

    let (mut cell, witness, _, _) = gen_did_cell_with_lock(
        b"a.bit",
        json!({
            "code_hash": "{{always_success}}",
            "hash_type": "type",
            "args": "0xFF"
        }),
    );
    cell["capacity"] = Value::from(13_400_000_000u64);
    base.outer_witnesses.push(witness);
    base.push_cell_json(cell, Source::Output, None);
    test_tx(base.as_json());
}

#[test]
fn test_validate_capacity_to_larger_lock_2() {
    let mut base = tmpl::init();

    let (mut cell, _, _, _) = gen_did_cell_with_lock(
        b"a.bit",
        json!({
            "code_hash": "{{always_success}}",
            "hash_type": "type",
            "args": ""
        }),
    );
    // Simulating the existing capacity is a lot more than the required capacity of the new cell.
    cell["capacity"] = Value::from(19_500_000_000u64);
    base.push_cell_json(cell, Source::Input, None);

    let (mut cell, witness, _, _) = gen_did_cell_with_lock(
        b"a.bit",
        json!({
            "code_hash": "{{always_success}}",
            "hash_type": "type",
            "args": "0xFF"
        }),
    );
    cell["capacity"] = Value::from(13_400_000_000u64);
    base.outer_witnesses.push(witness);
    base.push_cell_json(cell, Source::Output, None);

    // The withdrawn capacity
    push_output_normal_cell(&mut base, 6_100_000_000, "0x0000000000000000000000000000000000000000");

    test_tx(base.as_json());
}

#[test]
fn test_validate_capacity_to_smaller_lock_1() {
    let mut base = tmpl::init();

    let (mut cell, _, _, _) = gen_did_cell_with_lock(
        b"a.bit",
        json!({
            "code_hash": "{{always_success}}",
            "hash_type": "type",
            "args": "0xFF"
        }),
    );
    // Simulating has only 1 redundant CKB that cannot be withdrawn.
    cell["capacity"] = Value::from(13_400_000_000u64);
    base.push_cell_json(cell, Source::Input, None);

    let (mut cell, witness, _, _) = gen_did_cell_with_lock(
        b"a.bit",
        json!({
            "code_hash": "{{always_success}}",
            "hash_type": "type",
            "args": ""
        }),
    );
    cell["capacity"] = Value::from(13_400_000_000u64 - 100_000);
    base.outer_witnesses.push(witness);
    base.push_cell_json(cell, Source::Output, None);

    test_tx(base.as_json());
}

#[test]
fn challenge_validate_capacity_to_smaller_lock_1() {
    let mut base = tmpl::init();

    let (mut cell, _, _, _) = gen_did_cell_with_lock(
        b"a.bit",
        json!({
            "code_hash": "{{always_success}}",
            "hash_type": "type",
            "args": "0xFF"
        }),
    );
    cell["capacity"] = Value::from(13_400_000_000u64);
    base.push_cell_json(cell, Source::Input, None);

    let (mut cell, witness, _, _) = gen_did_cell_with_lock(
        b"a.bit",
        json!({
            "code_hash": "{{always_success}}",
            "hash_type": "type",
            "args": ""
        }),
    );
    // Simulating spend more CKB than the limit.
    // Occupied capacity: 13_300_000_000
    // Expected remaining capacity: 13_400_000_000 - 100_000
    cell["capacity"] = Value::from(13_400_000_000u64 - 100_000 - 1);
    base.outer_witnesses.push(witness);
    base.push_cell_json(cell, Source::Output, None);

    challenge_tx(base.as_json(), ErrorCode::WrongCapacity);
}

#[test]
fn test_validate_capacity_to_smaller_lock_2() {
    let mut base = tmpl::init();

    let (mut cell, _, _, _) = gen_did_cell_with_lock(
        b"a.bit",
        json!({
            "code_hash": "{{always_success}}",
            "hash_type": "type",
            "args": "0xFF"
        }),
    );
    // Simulating has more than 1 redundant CKB and it can be withdrawn.
    cell["capacity"] = Value::from(19_500_000_000u64);
    base.push_cell_json(cell, Source::Input, None);

    let (mut cell, witness, _, _) = gen_did_cell_with_lock(
        b"a.bit",
        json!({
            "code_hash": "{{always_success}}",
            "hash_type": "type",
            "args": ""
        }),
    );
    cell["capacity"] = Value::from(13_400_000_000u64);
    base.outer_witnesses.push(witness);
    base.push_cell_json(cell, Source::Output, None);

    // The withdrawn capacity
    push_output_normal_cell(&mut base, 6_100_000_000, "0x0000000000000000000000000000000000000000");

    test_tx(base.as_json());
}

#[test]
fn challenge_validate_capacity_to_smaller_lock_2() {
    let mut base = tmpl::init();

    let (mut cell, _, _, _) = gen_did_cell_with_lock(
        b"a.bit",
        json!({
            "code_hash": "{{always_success}}",
            "hash_type": "type",
            "args": "0xFF"
        }),
    );
    cell["capacity"] = Value::from(19_500_000_000u64);
    base.push_cell_json(cell, Source::Input, None);

    let (mut cell, witness, _, _) = gen_did_cell_with_lock(
        b"a.bit",
        json!({
            "code_hash": "{{always_success}}",
            "hash_type": "type",
            "args": ""
        }),
    );
    // Simulating left with less than 1 CKB after withdrawing.
    // Occupied capacity: 13_300_000_000
    // Expected remaining capacity: 13_400_000_000
    cell["capacity"] = Value::from(13_400_000_000u64 - 1);
    base.outer_witnesses.push(witness);
    base.push_cell_json(cell, Source::Output, None);

    // The withdrawn capacity
    push_output_normal_cell(&mut base, 6_100_000_000, "0x0000000000000000000000000000000000000000");

    challenge_tx(base.as_json(), ErrorCode::WrongCapacity);
}

#[test]
fn challenge_validate_action() {
    let mut base = tmpl::init();
    let (cell, _, _, _) = gen_did_cell(b"a.bit", TIMESTAMP, 2);
    base.push_cell_json(cell, Source::Input, None);

    let (cell, witness, _, _) = gen_did_cell(b"a.bit", TIMESTAMP, 3);
    base.outer_witnesses.push(witness);
    base.push_cell_json(cell, Source::Output, None);

    let (cell, _, _, _) = gen_did_cell(b"b.bit", TIMESTAMP, 2);
    base.push_cell_json(cell, Source::Input, None);
    challenge_tx(base.as_json(), ErrorCode::DifferentAction);
}

#[test]
fn challenge_validate_witness() {
    let mut base = tmpl::init();
    let (cell, _, _, _) = gen_did_cell(b"a.bit", TIMESTAMP, 2);
    base.push_cell_json(cell, Source::Input, None);

    let (cell, _, _, _) = gen_did_cell(b"a.bit", TIMESTAMP, 3);
    base.push_cell_json(cell, Source::Output, None);
    challenge_tx(base.as_json(), ErrorCode::InvalidDidCellWitnessHash);
}

#[test]
fn test_validate_vitness() {
    let mut base = tmpl::init();
    let (cell, _, _, _) = gen_did_cell(b"a.bit", TIMESTAMP, 2);
    base.push_cell_json(cell, Source::Input, None);

    let (cell, witness, _, _) = gen_did_cell(b"a.bit", TIMESTAMP, 3);
    base.outer_witnesses.push(witness);
    base.push_cell_json(cell, Source::Output, None);
    test_tx(base.as_json());

    let (cell, _, _, _) = gen_did_cell(b"b.bit", TIMESTAMP, 2);
    base.push_cell_json(cell, Source::Input, None);

    let (cell, _, _, _) = gen_did_cell(b"b.bit", TIMESTAMP, 2);
    base.push_cell_json(cell, Source::Output, None);
    test_tx(base.as_json());

    let (cell, _, _, _) = gen_did_cell(b"c.bit", TIMESTAMP, 2);
    base.push_cell_json(cell, Source::Input, None);

    let (cell, witness, _, _) = gen_did_cell(b"c.bit", TIMESTAMP, 2);
    base.outer_witnesses.push(witness);
    base.push_cell_json(cell, Source::Output, None);
    test_tx(base.as_json());
}
