use serde_json::Value;

use super::common::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before() -> TemplateGenerator {
    init("apply_register")
}

#[test]
fn test_apply_register_simple() {
    let mut template = before();

    push_output_apply_register_cell(&mut template, Value::Null);
    test_tx(template.as_json())
}

#[test]
fn challenge_apply_register_consuming_cell() {
    let mut template = before();

    // Simulate consuming ApplyRegisterCell.
    push_input_apply_register_cell(&mut template, Value::Null, None);

    push_output_apply_register_cell(&mut template, Value::Null);

    challenge_tx(template.as_json(), ErrorCode::InvalidTransactionStructure)
}
