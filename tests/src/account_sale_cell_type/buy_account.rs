use super::common::*;
use crate::util::{
    self, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::TemplateParser,
};
use ckb_testtool::context::Context;
use das_types::{constants::*, packed::*, prelude::*};
use serde_json::json;

fn push_input_account_cell(template: &mut TemplateGenerator, owner: &str, account: &str, timestamp: u64) {
    template.push_input(
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": account,
                "next": "yyyyy.bit",
                "expired_at": (timestamp + YEAR_SEC),
            },
            "witness": {
                "account": account,
                "registered_at": (timestamp - MONTH_SEC),
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Selling as u8)
            }
        }),
        Some(2),
    );
    template.push_empty_witness();
}

fn push_input_account_sale_cell(
    template: &mut TemplateGenerator,
    owner: &str,
    account: &str,
    price: u64,
    timestamp: u64,
) {
    template.push_input(
        json!({
            "capacity": "20_100_000_000",
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                "account": account,
                "price": price.to_string(),
                "description": "This is some account description.",
                "started_at": timestamp
            }
        }),
        None,
    );
    template.push_empty_witness();
}

fn push_output_account_cell(template: &mut TemplateGenerator, owner: &str, account: &str, timestamp: u64) {
    template.push_output(
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": account,
                "next": "yyyyy.bit",
                "expired_at": (timestamp + YEAR_SEC),
            },
            "witness": {
                "account": account,
                "registered_at": (timestamp - MONTH_SEC),
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(2),
    );
}

fn push_output_income_cell(template: &mut TemplateGenerator) {
    template.push_output(
        json!({
            "lock": {
                "code_hash": "{{always_success}}"
            },
            "type": {
                "code_hash": "{{income-cell-type}}"
            },
            "witness": {
                "records": [
                    // It is a conversion in this transaction that the first record always belong to the creator of the IncomeCell.
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": "0x0000000000000000000000000000000000000000"
                        },
                        "capacity": "20_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args("0x050000000000000000000000000000000000008888", None)
                        },
                        "capacity": "200_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args("0x050000000000000000000000000000000000009999", None)
                        },
                        "capacity": "200_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": "200_000_000"
                    }
                ]
            }
        }),
        None,
    );
}

fn gen_inviter_and_channel_locks(inviter_args: &str, channel_args: &str) -> (Script, Script) {
    let inviter_lock = gen_fake_das_lock(&gen_das_lock_args(inviter_args, None));
    let channel_lock = gen_fake_das_lock(&gen_das_lock_args(channel_args, None));
    (inviter_lock, channel_lock)
}

fn gen_params(inviter_args: &str, channel_args: &str) -> String {
    let (inviter_lock, channel_lock) = gen_inviter_and_channel_locks(inviter_args, channel_args);

    format!(
        "0x{}{}00",
        util::bytes_to_hex(inviter_lock.as_slice()),
        util::bytes_to_hex(channel_lock.as_slice())
    )
}

fn before_each(price: u64, paied: u64) -> (TemplateGenerator, u64, &'static str, &'static str, &'static str) {
    let params = gen_params(
        "0x050000000000000000000000000000000000008888",
        "0x050000000000000000000000000000000000009999",
    );
    let (mut template, timestamp) = init_with_profit_rate("buy_account", Some(&params));
    let seller = "0x050000000000000000000000000000000000001111";
    let buyer = "0x050000000000000000000000000000000000002222";
    let account = "xxxxx.bit";

    // inputs
    push_input_account_cell(&mut template, seller, account, timestamp);
    push_input_account_sale_cell(&mut template, seller, account, price, timestamp);
    push_input_balance_cell(&mut template, paied, buyer);

    (template, timestamp, seller, buyer, account)
}

test_with_generator!(test_account_sale_buy_create_income_cell, || {
    let (mut template, timestamp, seller, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

    // outputs
    push_output_account_cell(&mut template, buyer, account, timestamp);
    push_output_income_cell(&mut template);
    // 194 CKB(price) + 20_099_990_000(refund of AccountSaleCell)
    push_output_balance_cell(&mut template, 39_499_990_000, seller);
    push_output_balance_cell(&mut template, 20_000_000_000, buyer);

    template.as_json()
});

test_with_generator!(test_account_sale_buy_not_create_income_cell, || {
    let (mut template, timestamp, seller, buyer, account) = before_each(1_000_000_000_000, 2_000_000_000_000);

    // outputs
    push_output_account_cell(&mut template, buyer, account, timestamp);

    template.push_output(
        json!({
            "lock": {
                "code_hash": "{{always_success}}"
            },
            "type": {
                "code_hash": "{{income-cell-type}}"
            },
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args("0x050000000000000000000000000000000000008888", None)
                        },
                        "capacity": "10_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args("0x050000000000000000000000000000000000009999", None)
                        },
                        "capacity": "10_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": "10_000_000_000"
                    }
                ]
            }
        }),
        None,
    );

    push_output_balance_cell(&mut template, 990099990000, seller);
    push_output_balance_cell(&mut template, 1_000_000_000_000, buyer);

    template.as_json()
});

challenge_with_generator!(
    challenge_account_sale_buy_account_expired,
    Error::AccountCellInExpirationGracePeriod,
    || {
        let params = gen_params(
            "0x050000000000000000000000000000000000008888",
            "0x050000000000000000000000000000000000009999",
        );
        let (mut template, timestamp) = init_with_profit_rate("buy_account", Some(&params));
        let seller = "0x050000000000000000000000000000000000001111";
        let buyer = "0x050000000000000000000000000000000000002222";
        let account = "xxxxx.bit";

        // inputs
        template.push_input(
            json!({
                "capacity": util::gen_account_cell_capacity(5),
                "lock": {
                    "owner_lock_args": seller,
                    "manager_lock_args": seller
                },
                "type": {
                    "code_hash": "{{account-cell-type}}"
                },
                "data": {
                    "account": account,
                    "next": "yyyyy.bit",
                    // Simulate the situation AccountCell has expired.
                    "expired_at": (timestamp - 1),
                },
                "witness": {
                    "account": account,
                    "registered_at": (timestamp - YEAR_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    "status": (AccountStatus::Selling as u8)
                }
            }),
            Some(2),
        );
        template.push_empty_witness();

        push_input_account_sale_cell(&mut template, seller, account, 20_000_000_000, timestamp);
        push_input_balance_cell(&mut template, 20_000_000_000, buyer);

        // outputs
        template.push_output(
            json!({
                "capacity": util::gen_account_cell_capacity(5),
                "lock": {
                    "owner_lock_args": buyer,
                    "manager_lock_args": buyer
                },
                "type": {
                    "code_hash": "{{account-cell-type}}"
                },
                "data": {
                    "account": account,
                    "next": "yyyyy.bit",
                    "expired_at": (timestamp - 1),
                },
                "witness": {
                    "account": account,
                    "registered_at": (timestamp - YEAR_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    "status": (AccountStatus::Normal as u8)
                }
            }),
            Some(2),
        );

        push_output_income_cell(&mut template);
        push_output_balance_cell(&mut template, 39_499_990_000, seller);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_account_capacity,
    Error::AccountCellChangeCapacityError,
    || {
        let (mut template, timestamp, seller, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

        // outputs
        template.push_output(
            json!({
                // Simulate the AccountCell.capacity has been modified accidentally.
                "capacity": util::gen_account_cell_capacity(5) - 1,
                "lock": {
                    "owner_lock_args": buyer,
                    "manager_lock_args": buyer
                },
                "type": {
                    "code_hash": "{{account-cell-type}}"
                },
                "data": {
                    "account": account,
                    "next": "yyyyy.bit",
                    "expired_at": (timestamp + YEAR_SEC),
                },
                "witness": {
                    "account": account,
                    "registered_at": (timestamp - MONTH_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    "status": (AccountStatus::Normal as u8)
                }
            }),
            Some(2),
        );

        push_output_income_cell(&mut template);
        push_output_balance_cell(&mut template, 39_499_990_000, seller);
        push_output_balance_cell(&mut template, 20_000_000_000, buyer);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_input_account_status,
    Error::AccountCellStatusLocked,
    || {
        let params = gen_params(
            "0x050000000000000000000000000000000000008888",
            "0x050000000000000000000000000000000000009999",
        );
        let (mut template, timestamp) = init_with_profit_rate("buy_account", Some(&params));
        let seller = "0x050000000000000000000000000000000000001111";
        let buyer = "0x050000000000000000000000000000000000002222";
        let account = "xxxxx.bit";

        // inputs
        template.push_input(
            json!({
                "capacity": util::gen_account_cell_capacity(5),
                "lock": {
                    "owner_lock_args": seller,
                    "manager_lock_args": seller
                },
                "type": {
                    "code_hash": "{{account-cell-type}}"
                },
                "data": {
                    "account": account,
                    "next": "yyyyy.bit",
                    "expired_at": (timestamp + YEAR_SEC),
                },
                "witness": {
                    "account": account,
                    "registered_at": (timestamp - MONTH_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    // Simulate the AccountCell.status is wrong in inputs.
                    "status": (AccountStatus::Normal as u8)
                }
            }),
            Some(2),
        );
        template.push_empty_witness();

        push_input_account_sale_cell(&mut template, seller, account, 20_000_000_000, timestamp);
        push_input_balance_cell(&mut template, 20_000_000_000, buyer);

        // outputs
        push_output_account_cell(&mut template, buyer, account, timestamp);
        push_output_income_cell(&mut template);
        push_output_balance_cell(&mut template, 39_499_990_000, seller);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_output_account_status,
    Error::AccountCellStatusLocked,
    || {
        let (mut template, timestamp, seller, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

        // outputs
        template.push_output(
            json!({
                "capacity": util::gen_account_cell_capacity(5),
                "lock": {
                    "owner_lock_args": buyer,
                    "manager_lock_args": buyer
                },
                "type": {
                    "code_hash": "{{account-cell-type}}"
                },
                "data": {
                    "account": account,
                    "next": "yyyyy.bit",
                    "expired_at": (timestamp + YEAR_SEC),
                },
                "witness": {
                    "account": account,
                    "registered_at": (timestamp - MONTH_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    // Simulate the AccountCell.status is wrong in outputs.
                    "status": (AccountStatus::Selling as u8)
                }
            }),
            Some(2),
        );

        push_output_income_cell(&mut template);
        push_output_balance_cell(&mut template, 39_499_990_000, seller);
        push_output_balance_cell(&mut template, 20_000_000_000, buyer);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_sale_account,
    Error::AccountSaleCellAccountIdInvalid,
    || {
        let params = gen_params(
            "0x050000000000000000000000000000000000008888",
            "0x050000000000000000000000000000000000009999",
        );
        let (mut template, timestamp) = init_with_profit_rate("buy_account", Some(&params));
        let seller = "0x050000000000000000000000000000000000001111";
        let buyer = "0x050000000000000000000000000000000000002222";
        let account = "xxxxx.bit";

        // inputs
        push_input_account_cell(&mut template, seller, account, timestamp);

        template.push_input(
            json!({
                "capacity": "20_100_000_000",
                "lock": {
                    "owner_lock_args": seller,
                    "manager_lock_args": seller
                },
                "type": {
                    "code_hash": "{{account-sale-cell-type}}"
                },
                "witness": {
                    // Simulate the AccountSaleCell.account is wrong in inputs.
                    "account": "zzzzz.bit",
                    "price": "20_000_000_000",
                    "description": "This is some account description.",
                    "started_at": timestamp
                }
            }),
            None,
        );
        template.push_empty_witness();

        push_input_balance_cell(&mut template, 20_000_000_000, buyer);

        // outputs
        push_output_account_cell(&mut template, buyer, account, timestamp);
        push_output_income_cell(&mut template);
        push_output_balance_cell(&mut template, 39_499_990_000, seller);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_wrong_owner,
    Error::AccountSaleCellNewOwnerError,
    || {
        let (mut template, timestamp, seller, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

        // outputs
        template.push_output(
            json!({
                "capacity": util::gen_account_cell_capacity(5),
                "lock": {
                    // Simulate transferring AccountCell to unknown owner.
                    "owner_lock_args": "0x050000000000000000000000000000000000003333",
                    "manager_lock_args": "0x050000000000000000000000000000000000003333"
                },
                "type": {
                    "code_hash": "{{account-cell-type}}"
                },
                "data": {
                    "account": account,
                    "next": "yyyyy.bit",
                    "expired_at": (timestamp + YEAR_SEC),
                },
                "witness": {
                    "account": account,
                    "registered_at": (timestamp - MONTH_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    "status": (AccountStatus::Normal as u8)
                }
            }),
            Some(2),
        );

        push_output_income_cell(&mut template);
        push_output_balance_cell(&mut template, 39_499_990_000, seller);
        push_output_balance_cell(&mut template, 20_000_000_000, buyer);

        template.as_json()
    }
);

challenge_with_generator!(challenge_account_sale_buy_change_owner, Error::ChangeError, || {
    let (mut template, timestamp, seller, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

    // outputs
    push_output_account_cell(&mut template, buyer, account, timestamp);
    push_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 39_499_990_000, seller);
    // Simulate transfer changes to another lock.
    push_output_balance_cell(
        &mut template,
        20_000_000_000,
        "0x050000000000000000000000000000000000003333",
    );

    template.as_json()
});

challenge_with_generator!(challenge_account_sale_buy_change_capacity, Error::ChangeError, || {
    let (mut template, timestamp, seller, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

    // outputs
    push_output_account_cell(&mut template, buyer, account, timestamp);
    push_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 39_499_990_000, seller);
    // Simulate transfer changes less than the user should get.
    push_output_balance_cell(&mut template, 20_000_000_000 - 1, buyer);

    template.as_json()
});

challenge_with_generator!(
    challenge_account_sale_buy_seller_profit_owner,
    Error::ChangeError,
    || {
        let (mut template, timestamp, seller, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

        // outputs
        push_output_account_cell(&mut template, buyer, account, timestamp);
        push_output_income_cell(&mut template);
        // Simulate transfer profit to another lock.
        push_output_balance_cell(
            &mut template,
            39_499_990_000,
            "0x050000000000000000000000000000000000003333",
        );
        push_output_balance_cell(&mut template, 20_000_000_000, buyer);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_seller_profit_capacity,
    Error::ChangeError,
    || {
        let (mut template, timestamp, seller, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

        // outputs
        push_output_account_cell(&mut template, buyer, account, timestamp);
        push_output_income_cell(&mut template);
        // Simulate transfer profit less than the seller should get.
        push_output_balance_cell(&mut template, 39_499_980_000, seller);
        push_output_balance_cell(&mut template, 20_000_000_000, buyer);

        template.as_json()
    }
);
