use super::common::*;
use crate::util::{template_generator::*, template_common_cell::*, template_parser::*};

fn before_each() -> TemplateGenerator {
    let mut template = init("create_reverse_record_root");

    template
}

#[test]
fn test_reverse_record_retract() {
    let mut template = before_each();

    // inputs

    // outputs
    push_output_balance_cell(&mut template, 20_100_000_000 * 3 - 10_000, owner);

    test_tx(template.as_json());
}
