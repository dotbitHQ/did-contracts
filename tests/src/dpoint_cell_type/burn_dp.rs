use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init(json!({ "action": "burn_dp" }));

    // inputs
    push_input_dpoint_cell( &mut template, 100, OWNER);

    template
}

#[test]
fn test_dpoint_burn_dp() {
    let mut template = before_each();

    // outputs
    push_output_dpoint_cell(&mut template, 50, OWNER);
    // TODO change SUPER_LOCK_ARGS to a server lock mock
    push_output_normal_cell(&mut template, BASIC_CAPACITY, SUPER_LOCK_ARGS);

    test_tx(template.as_json());
}
