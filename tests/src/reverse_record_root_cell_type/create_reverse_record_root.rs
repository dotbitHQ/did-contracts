use super::common::*;
use crate::util::accounts::SUPER_LOCK_ARGS;
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
