use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init(json!({ "action": "transfer_dp" }));

    // inputs
    push_input_dpoint_cell( &mut template, 100, OWNER);

    template
}

#[test]
fn test_dpoint_transfer_dp() {
    let mut template = before_each();

    // outputs
    push_output_dpoint_cell(&mut template, 100, OWNER);

    test_tx(template.as_json());
}
