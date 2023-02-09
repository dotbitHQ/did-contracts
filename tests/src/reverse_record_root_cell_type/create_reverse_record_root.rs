use super::common::*;
use crate::util::{template_generator::*, template_common_cell::*, template_parser::*, accounts::SUPER_LOCK_ARGS};

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
