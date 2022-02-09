use super::common::*;
use crate::util::{
    self, constants::*, error::Error, template_common_cell::*, template_generator::TemplateGenerator,
    template_parser::*,
};
use das_types_std::constants::*;
use serde_json::json;

fn push_input_proposal_cell_with_slices(template: &mut TemplateGenerator) {
    push_input_proposal_cell(
        template,
        json!({
            "capacity": "20_000_000_000",
            "witness": {
                "proposer_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": PROPOSER
                },
                "created_at_height": HEIGHT - 4,
                "slices": [
                    [
                        {
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00005.bit"
                        },
                        {
                            "account_id": "das00005.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00002.bit"
                        },
                    ],
                    [
                        {
                            "account_id": "das00004.bit",
                            "item_type": ProposalSliceItemType::Proposed as u8,
                            "next": "das00018.bit"
                        },
                        {
                            "account_id": "das00018.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00008.bit"
                        },
                        {
                            "account_id": "das00008.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00011.bit"
                        },
                    ]
                ]
            }
        }),
    );
}

fn push_input_slice_0(template: &mut TemplateGenerator) {
    let lock_scripts = gen_lock_scripts();

    push_input_account_cell_v1(
        template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x000000000000000000000000000000000000001111",
                "manager_lock_args": "0x000000000000000000000000000000000000001111"
            },
            "data": {
                "account": "das00012.bit",
                "next": "das00002.bit"
            },
            "witness": {
                "account": "das00012.bit",
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_input_pre_account_cell(
        template,
        json!({
            "capacity": util::gen_register_fee(8, true),
            "witness": {
                "account": "das00005.bit",
                "owner_lock_args": "0x05ffff00000000000000000000000000000000000505ffff000000000000000000000000000000000005",
                "inviter_lock": lock_scripts.inviter_1,
                "channel_lock": lock_scripts.channel_1,
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );
}

fn push_input_slice_1(template: &mut TemplateGenerator) {
    let lock_scripts = gen_lock_scripts();

    push_input_account_cell(
        template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x000000000000000000000000000000000000002222",
                "manager_lock_args": "0x000000000000000000000000000000000000002222"
            },
            "data": {
                "account": "das00004.bit",
                "next": "das00011.bit"
            },
            "witness": {
                "account": "das00004.bit",
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_pre_account_cell(
        template,
        json!({
            "capacity": util::gen_register_fee(8, true),
            "witness": {
                "account": "das00018.bit",
                "owner_lock_args": "0x05ffff00000000000000000000000000000000001805ffff000000000000000000000000000000000018",
                "inviter_lock": lock_scripts.inviter_2,
                "channel_lock": lock_scripts.channel_2,
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );
    push_input_pre_account_cell(
        template,
        json!({
            "capacity": util::gen_register_fee(8, true),
            "witness": {
                "account": "das00008.bit",
                "owner_lock_args": "0x05ffff00000000000000000000000000000000000805ffff000000000000000000000000000000000008",
                "inviter_lock": lock_scripts.inviter_2,
                "channel_lock": lock_scripts.channel_2,
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );
}

fn push_output_slice_0(template: &mut TemplateGenerator) {
    push_output_account_cell(
        template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x000000000000000000000000000000000000001111",
                "manager_lock_args": "0x000000000000000000000000000000000000001111"
            },
            "data": {
                "account": "das00012.bit",
                "next": "das00005.bit"
            },
            "witness": {
                "account": "das00012.bit",
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_account_cell(
        template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x05ffff000000000000000000000000000000000005",
                "manager_lock_args": "0x05ffff000000000000000000000000000000000005"
            },
            "data": {
                "account": "das00005.bit",
                "next": "das00002.bit",
                "expired_at": TIMESTAMP + YEAR_SEC
            },
            "witness": {
                "account": "das00005.bit",
                "status": (AccountStatus::Normal as u8),
                "registered_at": TIMESTAMP
            }
        }),
    );
}

fn push_output_slice_1(template: &mut TemplateGenerator) {
    push_output_account_cell(
        template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x000000000000000000000000000000000000002222",
                "manager_lock_args": "0x000000000000000000000000000000000000002222"
            },
            "data": {
                "account": "das00004.bit",
                "next": "das00018.bit"
            },
            "witness": {
                "account": "das00004.bit",
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_output_account_cell(
        template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x05ffff000000000000000000000000000000000018",
                "manager_lock_args": "0x05ffff000000000000000000000000000000000018"
            },
            "data": {
                "account": "das00018.bit",
                "next": "das00008.bit",
                "expired_at": TIMESTAMP + YEAR_SEC
            },
            "witness": {
                "account": "das00018.bit",
                "status": (AccountStatus::Normal as u8),
                "registered_at": TIMESTAMP
            }
        }),
    );
    push_output_account_cell(
        template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x05ffff000000000000000000000000000000000008",
                "manager_lock_args": "0x05ffff000000000000000000000000000000000008"
            },
            "data": {
                "account": "das00008.bit",
                "next": "das00011.bit",
                "expired_at": TIMESTAMP + YEAR_SEC
            },
            "witness": {
                "account": "das00008.bit",
                "status": (AccountStatus::Normal as u8),
                "registered_at": TIMESTAMP
            }
        }),
    );
}

fn push_output_income_cell_with_profit(template: &mut TemplateGenerator) {
    let lock_scripts = gen_lock_scripts();

    // Carry profits of all roles.
    push_output_income_cell(
        template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": lock_scripts.inviter_1,
                        "capacity": 38000000000u64
                    },
                    {
                        "belong_to": lock_scripts.inviter_2,
                        "capacity": 38000000000u64 * 2
                    },
                    {
                        "belong_to": lock_scripts.channel_1,
                        "capacity": 38000000000u64
                    },
                    {
                        "belong_to": lock_scripts.channel_2,
                        "capacity": 38000000000u64 * 2
                    },
                    {
                        "belong_to": lock_scripts.proposer,
                        "capacity": 19000000000u64 * 3
                    },
                    {
                        "belong_to": lock_scripts.das_wallet,
                        "capacity": 380000000000u64 * 3
                    }
                ]
            }
        }),
    );
}

fn push_output_normal_cell_with_refund(template: &mut TemplateGenerator) {
    // A refund of ProposalCell's capacity to proposer.
    push_output_normal_cell(template, 20_000_000_000, PROPOSER);
}

fn before_each() -> TemplateGenerator {
    let mut template = init_with_confirm();

    // inputs
    push_input_proposal_cell_with_slices(&mut template);
    push_input_slice_0(&mut template);
    push_input_slice_1(&mut template);

    template
}

#[test]
fn test_proposal_confirm_not_create_income_cell() {
    let mut template = before_each();

    // outputs
    push_output_slice_0(&mut template);
    push_output_slice_1(&mut template);
    push_output_income_cell_with_profit(&mut template);
    push_output_normal_cell_with_refund(&mut template);

    test_tx(template.as_json());
}

#[test]
fn test_proposal_confirm_create_income_cell() {
    let mut template = before_each();
    let lock_scripts = gen_lock_scripts();

    // outputs
    push_output_slice_0(&mut template);
    push_output_slice_1(&mut template);

    // Carry profits of all roles.
    push_output_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": COMMON_INCOME_CREATOR_LOCK_ARGS
                        },
                        "capacity": "20_000_000_000"
                    },
                    {
                        "belong_to": lock_scripts.inviter_1,
                        "capacity": 38000000000u64
                    },
                    {
                        "belong_to": lock_scripts.inviter_2,
                        "capacity": 38000000000u64 * 2
                    },
                    {
                        "belong_to": lock_scripts.channel_1,
                        "capacity": 38000000000u64
                    },
                    {
                        "belong_to": lock_scripts.channel_2,
                        "capacity": 38000000000u64 * 2
                    },
                    {
                        "belong_to": lock_scripts.proposer,
                        "capacity": 19000000000u64 * 3
                    },
                    {
                        "belong_to": lock_scripts.das_wallet,
                        "capacity": 380000000000u64 * 3
                    }
                ]
            }
        }),
    );

    push_output_normal_cell_with_refund(&mut template);

    test_tx(template.as_json());
}

#[test]
fn challenge_proposal_confirm_height() {
    let mut template = init_with_confirm();

    // inputs
    push_input_proposal_cell(
        &mut template,
        json!({
            "capacity": "20_000_000_000",
            "witness": {
                "proposer_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": PROPOSER
                },
                "created_at_height": HEIGHT,
                "slices": [
                    [
                        {
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00005.bit"
                        },
                        {
                            "account_id": "das00005.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00002.bit"
                        },
                    ],
                    [
                        {
                            "account_id": "das00004.bit",
                            "item_type": ProposalSliceItemType::Proposed as u8,
                            "next": "das00018.bit"
                        },
                        {
                            "account_id": "das00018.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00008.bit"
                        },
                        {
                            "account_id": "das00008.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00011.bit"
                        },
                    ]
                ]
            }
        }),
    );

    push_input_slice_0(&mut template);
    push_input_slice_1(&mut template);

    // outputs
    push_output_slice_0(&mut template);
    push_output_slice_1(&mut template);
    push_output_income_cell_with_profit(&mut template);
    push_output_normal_cell_with_refund(&mut template);

    challenge_tx(template.as_json(), Error::ProposalConfirmNeedWaitLonger);
}

#[test]
fn challenge_proposal_confirm_account_cell_modified_1() {
    let mut template = before_each();

    // outputs
    // slices[0]
    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x000000000000000000000000000000000000001111",
                "manager_lock_args": "0x000000000000000000000000000000000000001111"
            },
            "data": {
                "account": "das00012.bit",
                "next": "das00005.bit"
            },
            "witness": {
                "account": "das00012.bit",
                // Simulate AccountCell.status is modified.
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x05ffff000000000000000000000000000000000005",
                "manager_lock_args": "0x05ffff000000000000000000000000000000000005"
            },
            "data": {
                "account": "das00005.bit",
                "next": "das00009.bit",
                "expired_at": TIMESTAMP + YEAR_SEC
            },
            "witness": {
                "account": "das00005.bit",
                "status": (AccountStatus::Normal as u8),
                "registered_at": TIMESTAMP
            }
        }),
    );

    push_output_slice_1(&mut template);
    push_output_income_cell_with_profit(&mut template);
    push_output_normal_cell_with_refund(&mut template);

    challenge_tx(template.as_json(), Error::AccountCellProtectFieldIsModified);
}

#[test]
fn challenge_proposal_confirm_account_cell_modified_2() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            // Simulate the AccountCell.capacity is modified.
            "capacity": util::gen_account_cell_capacity(8) - 1,
            "lock": {
                "owner_lock_args": "0x000000000000000000000000000000000000001111",
                "manager_lock_args": "0x000000000000000000000000000000000000001111"
            },
            "data": {
                "account": "das00012.bit",
                "next": "das00005.bit"
            },
            "witness": {
                "account": "das00012.bit",
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x05ffff000000000000000000000000000000000005",
                "manager_lock_args": "0x05ffff000000000000000000000000000000000005"
            },
            "data": {
                "account": "das00005.bit",
                "next": "das00002.bit",
                "expired_at": TIMESTAMP + YEAR_SEC
            },
            "witness": {
                "account": "das00005.bit",
                "status": (AccountStatus::Normal as u8),
                "registered_at": TIMESTAMP
            }
        }),
    );

    push_output_slice_1(&mut template);
    push_output_income_cell_with_profit(&mut template);
    push_output_normal_cell_with_refund(&mut template);

    challenge_tx(template.as_json(), Error::CellCapacityMustConsistent);
}

#[test]
fn challenge_proposal_confirm_account_cell_next_mismatch() {
    let mut template = init_with_confirm();
    let lock_scripts = gen_lock_scripts();

    // inputs
    push_input_proposal_cell_with_slices(&mut template);

    push_input_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x000000000000000000000000000000000000001111",
                "manager_lock_args": "0x000000000000000000000000000000000000001111"
            },
            "data": {
                "account": "das00012.bit",
                // CAREFUL! The key point of this test is that the AccountCell has been updated by another PreAccountCell with the same account as current one.
                // But the next in ProposalCell.slices is still old one. When this happens, the transaction shall be rejected.

                // Simulate the AccountCell das00009.bit has been registered before the proposal confirmed.
                "next": "das00009.bit"
            },
            "witness": {
                "account": "das00012.bit",
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee(8, true),
            "witness": {
                "account": "das00005.bit",
                "owner_lock_args": "0x05ffff00000000000000000000000000000000000505ffff000000000000000000000000000000000005",
                "inviter_lock": lock_scripts.inviter_1,
                "channel_lock": lock_scripts.channel_1,
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );

    push_input_slice_1(&mut template);

    // outputs
    push_output_slice_0(&mut template);
    push_output_slice_1(&mut template);
    push_output_income_cell_with_profit(&mut template);
    push_output_normal_cell_with_refund(&mut template);

    challenge_tx(template.as_json(), Error::ProposalCellNextError);
}

#[test]
fn challenge_proposal_confirm_refund() {
    let mut template = before_each();

    // outputs
    push_output_slice_0(&mut template);
    push_output_slice_1(&mut template);
    push_output_income_cell_with_profit(&mut template);

    // Simulate refund capacity is less than the ProposalCell.capacity .
    push_output_normal_cell(&mut template, 20_000_000_000 - 1, PROPOSER);

    challenge_tx(template.as_json(), Error::ProposalConfirmRefundError);
}

#[test]
fn challenge_proposal_confirm_income_records_capacity() {
    let mut template = before_each();
    let lock_scripts = gen_lock_scripts();

    // outputs
    push_output_slice_0(&mut template);
    push_output_slice_1(&mut template);

    // Carry profits of all roles.
    push_output_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": lock_scripts.inviter_1,
                        // Simulate creating some records with invalid capacity.
                        "capacity": 38000000000u64 - 1
                    },
                    {
                        "belong_to": lock_scripts.inviter_2,
                        "capacity": 38000000000u64 * 2
                    },
                    {
                        "belong_to": lock_scripts.channel_1,
                        "capacity": 38000000000u64
                    },
                    {
                        "belong_to": lock_scripts.channel_2,
                        "capacity": 38000000000u64 * 2
                    },
                    {
                        "belong_to": lock_scripts.proposer,
                        "capacity": 19000000000u64 * 3
                    },
                    {
                        "belong_to": lock_scripts.das_wallet,
                        "capacity": 380000000000u64 * 3
                    }
                ]
            }
        }),
    );

    push_output_normal_cell_with_refund(&mut template);

    challenge_tx(template.as_json(), Error::IncomeCellProfitMismatch);
}

#[test]
fn challenge_proposal_confirm_income_cell_capacity() {
    let mut template = before_each();
    let lock_scripts = gen_lock_scripts();

    // outputs
    push_output_slice_0(&mut template);
    push_output_slice_1(&mut template);

    // Carry profits of all roles.
    push_output_income_cell(
        &mut template,
        json!({
            // Simulate inconsistent capacity with the summary of records.
            "capacity": 20_000_000_000u64,
            "witness": {
                "records": [
                    {
                        "belong_to": lock_scripts.inviter_1,
                        "capacity": 38_000_000_000u64
                    },
                    {
                        "belong_to": lock_scripts.inviter_2,
                        "capacity": 38_000_000_000u64 * 2
                    },
                    {
                        "belong_to": lock_scripts.channel_1,
                        "capacity": 38_000_000_000u64
                    },
                    {
                        "belong_to": lock_scripts.channel_2,
                        "capacity": 38_000_000_000u64 * 2
                    },
                    {
                        "belong_to": lock_scripts.proposer,
                        "capacity": 19_000_000_000u64 * 3
                    },
                    {
                        "belong_to": lock_scripts.das_wallet,
                        "capacity": 380_000_000_000u64 * 3
                    }
                ]
            }
        }),
    );

    push_output_normal_cell_with_refund(&mut template);

    challenge_tx(template.as_json(), Error::IncomeCellCapacityError);
}

#[test]
fn challenge_proposal_confirm_new_account_cell_capacity() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x000000000000000000000000000000000000001111",
                "manager_lock_args": "0x000000000000000000000000000000000000001111"
            },
            "data": {
                "account": "das00012.bit",
                "next": "das00005.bit"
            },
            "witness": {
                "account": "das00012.bit",
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_account_cell(
        &mut template,
        json!({
            // Simulate the capacity of new AccountCell is invalid.
            "capacity": util::gen_register_fee(8, true) - 1,
            "lock": {
                "owner_lock_args": "0x05ffff000000000000000000000000000000000005",
                "manager_lock_args": "0x05ffff000000000000000000000000000000000005"
            },
            "data": {
                "account": "das00005.bit",
                "next": "das00002.bit",
                "expired_at": TIMESTAMP + YEAR_SEC
            },
            "witness": {
                "account": "das00005.bit",
                "status": (AccountStatus::Normal as u8),
                "registered_at": TIMESTAMP
            }
        }),
    );

    push_output_slice_1(&mut template);
    push_output_income_cell_with_profit(&mut template);
    push_output_normal_cell_with_refund(&mut template);

    challenge_tx(template.as_json(), Error::ProposalConfirmNewAccountCellCapacityError);
}
