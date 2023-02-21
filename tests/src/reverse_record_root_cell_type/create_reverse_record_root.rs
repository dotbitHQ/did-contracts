use super::common::*;
use crate::util::accounts::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let template = init("create_reverse_record_root");

    template
}

#[test]
fn test_reverse_record_root_create() {
    let mut template = before_each();

    // inputs
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    // outputs
    push_output_reverse_record_root_cell(&mut template);

    test_tx(template.as_json());
}

#[test]
fn challenge_reverse_record_root_create_without_super_lock() {
    let mut template = before_each();

    // inputs
    push_input_normal_cell(&mut template, 0, OWNER_1_WITHOUT_TYPE);

    // outputs
    push_output_reverse_record_root_cell(&mut template);

    challenge_tx(template.as_json(), ErrorCode::SuperLockIsRequired);
}
