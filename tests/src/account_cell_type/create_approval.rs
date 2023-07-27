// use das_types_std::constants::AccountStatus;
use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
// use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::TemplateGenerator;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init("create_approval", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            }
        }),
    );

    template
}

#[test]
fn xxxx_account_create_approval() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                "approval": {
                    "action": "transfer",
                    "params": {
                        "platform_lock": {
                            "owner_lock_args": CHANNEL,
                            "manager_lock_args": CHANNEL
                        },
                        "protected_until": TIMESTAMP + DAY_SEC,
                        "sealed_until": TIMESTAMP + DAY_SEC * 3,
                        "delay_count_remain": 1,
                        "to_lock": {
                            "owner_lock_args": OWNER_2,
                            "manager_lock_args": OWNER_2
                        }
                    }
                }
            }
        }),
    );

    test_tx(template.as_json())
}
