use das_types_std::constants::*;
use serde_json::{json, Value};

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;
use crate::util::{self};

fn before_each() -> TemplateGenerator {
    let mut template = init_update();

    // cell_deps
    push_simple_dep_account_cell(&mut template);

    // inputs
    template.restore_sub_account_v2(vec![
        json!({
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
            "status": AccountStatus::ApprovedTransfer as u8,
            "approval": {
                "action": "transfer",
                "params": {
                    "platform_lock": {
                        "owner_lock_args": CHANNEL,
                        "manager_lock_args": CHANNEL
                    },
                    "protected_until": TIMESTAMP + DAY_SEC,
                    "sealed_until": TIMESTAMP + DAY_SEC * 2,
                    "delay_count_remain": 1,
                    "to_lock": {
                        "owner_lock_args": OWNER_2,
                        "manager_lock_args": OWNER_2
                    }
                }
            }
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        }),
    ]);
    push_simple_input_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    template
}

fn push_simple_sub_account_witness(template: &mut TemplateGenerator, sub_account_partial: Value) {
    let mut sub_account = json!({
        "action": SubAccountAction::DelayApproval.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
        "old_sub_account_version": 2,
        "new_sub_account_version": 2,
        "sub_account": {
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
            "status": AccountStatus::ApprovedTransfer as u8,
            "approval": {
                "action": "transfer",
                "params": {
                    "platform_lock": {
                        "owner_lock_args": CHANNEL,
                        "manager_lock_args": CHANNEL
                    },
                    "protected_until": TIMESTAMP + DAY_SEC,
                    "sealed_until": TIMESTAMP + DAY_SEC * 2,
                    "delay_count_remain": 1,
                    "to_lock": {
                        "owner_lock_args": OWNER_2,
                        "manager_lock_args": OWNER_2
                    }
                }
            }
        },
        "edit_key": "approval",
        "edit_value": {
            "action": "transfer",
            "params": {
                "platform_lock": {
                    "owner_lock_args": CHANNEL,
                    "manager_lock_args": CHANNEL
                },
                "protected_until": TIMESTAMP + DAY_SEC,
                "sealed_until": TIMESTAMP + DAY_SEC * (3 + 1),
                "delay_count_remain": 0,
                "to_lock": {
                    "owner_lock_args": OWNER_2,
                    "manager_lock_args": OWNER_2
                }
            }
        }
    });
    util::merge_json(&mut sub_account, sub_account_partial);

    // Simulate upgrate the SubAccount version in this transaction.
    template.push_sub_account_witness_v2(sub_account);
}

#[test]
fn test_sub_account_approval_delay() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_approval_delay_action_modified() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_value": {
                // Simulate providing a invali approval action.
                "action": "xxxxxx",
                "params": {
                    "platform_lock": {
                        "owner_lock_args": CHANNEL,
                        "manager_lock_args": CHANNEL
                    },
                    "protected_until": TIMESTAMP + DAY_SEC,
                    "sealed_until": TIMESTAMP + DAY_SEC * (3 + 1),
                    "delay_count_remain": 0,
                    "to_lock": {
                        "owner_lock_args": OWNER_2,
                        "manager_lock_args": OWNER_2
                    }
                }
            }
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ApprovalParamsCanNotBeChanged)
}

#[test]
fn challenge_sub_account_approval_delay_platform_lock_modified() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_value": {
                "action": "transfer",
                "params": {
                    // Simulate modifying the platform_lock of the approval.
                    "platform_lock": {
                        "owner_lock_args": INVITER,
                        "manager_lock_args": CHANNEL
                    },
                    "protected_until": TIMESTAMP + DAY_SEC,
                    "sealed_until": TIMESTAMP + DAY_SEC * (3 + 1),
                    "delay_count_remain": 0,
                    "to_lock": {
                        "owner_lock_args": OWNER_2,
                        "manager_lock_args": OWNER_2
                    }
                }
            }
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ApprovalParamsCanNotBeChanged)
}

#[test]
fn challenge_sub_account_approval_delay_protected_until_modified() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_value": {
                "action": "transfer",
                "params": {
                    "platform_lock": {
                        "owner_lock_args": CHANNEL,
                        "manager_lock_args": CHANNEL
                    },
                    // Simulate modifying the action of the approval.
                    "protected_until": TIMESTAMP + DAY_SEC + 1,
                    "sealed_until": TIMESTAMP + DAY_SEC * 2,
                    "delay_count_remain": 0,
                    "to_lock": {
                        "owner_lock_args": OWNER_2,
                        "manager_lock_args": OWNER_2
                    }
                }
            }
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ApprovalParamsCanNotBeChanged)
}

#[test]
fn challenge_sub_account_approval_delay_sealed_until_not_increased() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_value": {
                "action": "transfer",
                "params": {
                    "platform_lock": {
                        "owner_lock_args": CHANNEL,
                        "manager_lock_args": CHANNEL
                    },
                    "protected_until": TIMESTAMP + DAY_SEC,
                    // Simulate not modifying the sealed_until of the approval.
                    "sealed_until": TIMESTAMP + DAY_SEC * 2,
                    "delay_count_remain": 0,
                    "to_lock": {
                        "owner_lock_args": OWNER_2,
                        "manager_lock_args": OWNER_2
                    }
                }
            }
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ApprovalParamsSealedUntilIncrementError)
}

#[test]
fn challenge_sub_account_approval_delay_sealed_until_increased_too_long() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_value": {
                "action": "transfer",
                "params": {
                    "platform_lock": {
                        "owner_lock_args": CHANNEL,
                        "manager_lock_args": CHANNEL
                    },
                    "protected_until": TIMESTAMP + DAY_SEC,
                    // Simulate increasing the sealed_until to too long..
                    "sealed_until": TIMESTAMP + DAY_SEC * (2 + 10) + 1,
                    "delay_count_remain": 0,
                    "to_lock": {
                        "owner_lock_args": OWNER_2,
                        "manager_lock_args": OWNER_2
                    }
                }
            }
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ApprovalParamsSealedUntilIncrementError)
}

#[test]
fn challenge_sub_account_approval_delay_count_remain_is_empty() {
    let mut template = init_update();

    // cell_deps
    push_simple_dep_account_cell(&mut template);

    // inputs
    template.restore_sub_account_v2(vec![
        json!({
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
            "status": AccountStatus::ApprovedTransfer as u8,
            "approval": {
                "action": "transfer",
                "params": {
                    "platform_lock": {
                        "owner_lock_args": CHANNEL,
                        "manager_lock_args": CHANNEL
                    },
                    "protected_until": TIMESTAMP + DAY_SEC,
                    // Simulate try to delay even the delay_count_remain is 0.
                    "sealed_until": TIMESTAMP + DAY_SEC * 2,
                    "delay_count_remain": 0,
                    "to_lock": {
                        "owner_lock_args": OWNER_2,
                        "manager_lock_args": OWNER_2
                    }
                }
            }
        }),
    ]);
    push_simple_input_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "approval": {
                    "params": {
                        "delay_count_remain": 0,
                    }
                }
            },
            "edit_value": {
                "action": "transfer",
                "params": {
                    "platform_lock": {
                        "owner_lock_args": CHANNEL,
                        "manager_lock_args": CHANNEL
                    },
                    "protected_until": TIMESTAMP + DAY_SEC,
                    "sealed_until": TIMESTAMP + DAY_SEC * 3,
                    "delay_count_remain": 0,
                    "to_lock": {
                        "owner_lock_args": OWNER_2,
                        "manager_lock_args": OWNER_2
                    }
                }
            }
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ApprovalParamsDelayCountNotEnough)
}

#[test]
fn challenge_sub_account_approval_delay_count_remain_not_decreased() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_value": {
                "action": "transfer",
                "params": {
                    "platform_lock": {
                        "owner_lock_args": CHANNEL,
                        "manager_lock_args": CHANNEL
                    },
                    "protected_until": TIMESTAMP + DAY_SEC,
                    // Simulate not decreasing the delay_remain_count.
                    "delay_count_remain": 1,
                    "to_lock": {
                        "owner_lock_args": OWNER_2,
                        "manager_lock_args": OWNER_2
                    }
                }
            }
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ApprovalParamsDelayCountDecrementError)
}

#[test]
fn challenge_sub_account_approval_delay_to_lock_modified() {
    let mut template = before_each();

    // outputs
    let sub_account = json!({
        "action": SubAccountAction::DelayApproval.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
        "old_sub_account_version": 2,
        "new_sub_account_version": 2,
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
            "status": AccountStatus::ApprovedTransfer as u8,
            "approval": {
                "action": "transfer",
                "params": {
                    "platform_lock": {
                        "owner_lock_args": CHANNEL,
                        "manager_lock_args": CHANNEL
                    },
                    "protected_until": TIMESTAMP + DAY_SEC,
                    "sealed_until": TIMESTAMP + DAY_SEC * 2,
                    "delay_count_remain": 1,
                    "to_lock": {
                        "owner_lock_args": OWNER_2,
                        "manager_lock_args": OWNER_2
                    }
                }
            }
        },
        "edit_key": "approval",
        "edit_value": {
            "action": "transfer",
            "params": {
                "platform_lock": {
                    "owner_lock_args": CHANNEL,
                    "manager_lock_args": CHANNEL
                },
                "protected_until": TIMESTAMP + DAY_SEC,
                "sealed_until": TIMESTAMP + DAY_SEC * 2,
                "delay_count_remain": 0,
                // The to_lock can not be overided by field name, so the whole cell is copy-pasted here.
                // Simulate modifying the to lock.
                "to_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": DUMMY_LOCK_ARGS
                }
            }
        }
    });
    template.push_sub_account_witness_v2(sub_account);

    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ApprovalParamsCanNotBeChanged)
}
