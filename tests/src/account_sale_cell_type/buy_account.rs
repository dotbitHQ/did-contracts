use super::common::*;
use crate::util::{
    self, accounts::*, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*,
};
use das_types_std::{constants::*, packed::*, prelude::*};
use serde_json::json;

fn push_simple_output_income_cell(template: &mut TemplateGenerator) {
    push_output_income_cell(
        template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": "20_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args(INVITER, None)
                        },
                        "capacity": 2_000_000_000.to_string()
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args(CHANNEL, None)
                        },
                        "capacity": 2_000_000_000.to_string()
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": 2_000_000_000.to_string()
                    }
                ]
            }
        }),
    );
}

fn push_common_outputs(template: &mut TemplateGenerator) {
    push_output_account_cell(
        template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    push_simple_output_income_cell(template);
    push_output_balance_cell(
        template,
        PRICE - 6_000_000_000 + ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY
            - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );
}

fn gen_params(inviter_args: &str, channel_args: &str) -> String {
    let inviter_lock = if !inviter_args.is_empty() {
        gen_fake_das_lock(&gen_das_lock_args(inviter_args, None))
    } else {
        Script::default()
    };
    let channel_lock = if !channel_args.is_empty() {
        gen_fake_das_lock(&gen_das_lock_args(channel_args, None))
    } else {
        Script::default()
    };

    format!(
        "0x{}{}00",
        hex::encode(inviter_lock.as_slice()),
        hex::encode(channel_lock.as_slice())
    )
}

fn before_each(paid: u64) -> TemplateGenerator {
    let params = gen_params(INVITER, CHANNEL);
    let mut template = init_with_profit_rate("buy_account", Some(&params));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE.to_string(),
            }
        }),
    );
    push_input_balance_cell(&mut template, paid, BUYER);

    template
}

#[test]
fn test_account_sale_buy_create_income_cell() {
    let mut template = before_each(PRICE);

    // outputs
    push_common_outputs(&mut template);

    test_tx(template.as_json());
}

#[test]
fn test_account_sale_buy_not_create_income_cell() {
    let price = 1_000_000_000_000u64;
    let paid = 1_100_000_000_000u64;
    let params = gen_params(INVITER, CHANNEL);
    let mut template = init_with_profit_rate("buy_account", Some(&params));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": price.to_string(),
            }
        }),
    );
    push_input_balance_cell(&mut template, paid, BUYER);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    push_output_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args(INVITER, None)
                        },
                        "capacity": "10_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args(CHANNEL, None)
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
    );

    push_output_balance_cell(
        &mut template,
        1_000_000_000_000 - 30_000_000_000 + ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY
            - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );
    push_output_balance_cell(&mut template, paid - price, BUYER);

    test_tx(template.as_json());
}

#[test]
fn test_account_sale_buy_no_inviter_and_channel() {
    let price = 1_000_000_000_000u64;
    let paid = 2_000_000_000_000u64;
    let params = gen_params("", "");
    let mut template = init_with_profit_rate("buy_account", Some(&params));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": price.to_string(),
            }
        }),
    );
    push_input_balance_cell(&mut template, paid, BUYER);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    push_output_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": "30_000_000_000"
                    }
                ]
            }
        }),
    );

    push_output_balance_cell(
        &mut template,
        1_000_000_000_000 - 30_000_000_000 + ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY
            - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );
    push_output_balance_cell(&mut template, paid - price, BUYER);

    test_tx(template.as_json());
}

#[test]
fn test_account_sale_buy_create_with_custom_buyer_inviter_profit_rate() {
    let paid = PRICE;
    let params = gen_params(INVITER, CHANNEL);
    let mut template = init_with_profit_rate("buy_account", Some(&params));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE.to_string(),
                // Simulate custom the profit rate of the buyer's inviter to 20% .
                "buyer_inviter_profit_rate": SALE_BUYER_INVITER_PROFIT_RATE * 20
            }
        }),
    );
    push_input_balance_cell(&mut template, paid, BUYER);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": "20_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args(INVITER, None)
                        },
                        // Simulate custom the profit rate of the buyer's inviter to 20% .
                        "capacity": 40_000_000_000u64.to_string()
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args(CHANNEL, None)
                        },
                        "capacity": 2_000_000_000.to_string()
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": 2_000_000_000.to_string()
                    }
                ]
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        PRICE - 6_000_000_000 + ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY
            - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    test_tx(template.as_json());
}

#[test]
fn test_account_sale_buy_old_version() {
    let paid = PRICE;
    let params = gen_params(INVITER, CHANNEL);
    let mut template = init_with_profit_rate("buy_account", Some(&params));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell_v1(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE.to_string(),
            }
        }),
    );
    push_input_balance_cell(&mut template, paid, BUYER);

    // outputs
    push_common_outputs(&mut template);

    test_tx(template.as_json());
}

#[test]
fn challenge_account_sale_buy_account_expired() {
    let params = gen_params(INVITER, CHANNEL);
    let mut template = init_with_profit_rate("buy_account", Some(&params));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT,
                // Simulate the situation AccountCell has expired.
                "expired_at": (TIMESTAMP - 1),
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE.to_string(),
            }
        }),
    );
    push_input_balance_cell(&mut template, PRICE, BUYER);

    // outputs
    push_common_outputs(&mut template);

    challenge_tx(template.as_json(), Error::AccountCellInExpirationGracePeriod)
}

#[test]
fn challenge_account_sale_buy_account_capacity() {
    let mut template = before_each(PRICE);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            // Simulate the AccountCell.capacity has been modified accidentally.
            "capacity": util::gen_account_cell_capacity(5) - 1,
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "account": ACCOUNT,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(
        &mut template,
        PRICE - 6_000_000_000 + ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY
            - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), Error::AccountCellChangeCapacityError)
}

#[test]
fn challenge_account_sale_buy_input_account_status() {
    let params = gen_params(INVITER, CHANNEL);
    let mut template = init_with_profit_rate("buy_account", Some(&params));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                // Simulate the AccountCell.status is wrong in inputs.
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE.to_string(),
            }
        }),
    );
    push_input_balance_cell(&mut template, 20_000_000_000, BUYER);

    // outputs
    push_common_outputs(&mut template);

    challenge_tx(template.as_json(), Error::AccountCellStatusLocked)
}

#[test]
fn challenge_account_sale_buy_output_account_status() {
    let mut template = before_each(PRICE);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "account": ACCOUNT,
                // Simulate the AccountCell.status is wrong in outputs.
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(
        &mut template,
        PRICE - 6_000_000_000 + ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY
            - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), Error::AccountCellStatusLocked)
}

#[test]
fn challenge_account_sale_buy_sale_account() {
    let params = gen_params(INVITER, CHANNEL);
    let mut template = init_with_profit_rate("buy_account", Some(&params));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "account": ACCOUNT,
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                // Simulate the AccountSaleCell.account is wrong in inputs.
                "account": "zzzzz.bit",
                "price": 20_000_000_000u64.to_string(),
            }
        }),
    );
    push_input_balance_cell(&mut template, 20_000_000_000, BUYER);

    // outputs
    push_common_outputs(&mut template);

    challenge_tx(template.as_json(), Error::AccountSaleCellAccountIdInvalid)
}

#[test]
fn challenge_account_sale_buy_wrong_owner() {
    let mut template = before_each(PRICE);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                // Simulate transferring AccountCell to unknown owner.
                "owner_lock_args": "0x050000000000000000000000000000000000003333",
                "manager_lock_args": "0x050000000000000000000000000000000000003333"
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "account": ACCOUNT,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(
        &mut template,
        PRICE - 6_000_000_000 + ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY
            - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellNewOwnerError)
}

#[test]
fn challenge_account_sale_buy_change_owner() {
    let paid = PRICE * 2;
    let mut template = before_each(paid);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "account": ACCOUNT,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(
        &mut template,
        PRICE - 6_000_000_000 + ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY
            - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );
    // Simulate transfer changes to another lock.
    push_output_balance_cell(
        &mut template,
        paid - PRICE,
        "0x052222000000000000000000000000000000002222",
    );

    challenge_tx(template.as_json(), Error::ChangeError)
}

#[test]
fn challenge_account_sale_buy_change_capacity() {
    let paid = PRICE * 2;
    let mut template = before_each(paid);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "account": ACCOUNT,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(
        &mut template,
        PRICE - 6_000_000_000 + ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY
            - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );
    // Simulate transfer changes less than the user should get.
    push_output_balance_cell(&mut template, paid - PRICE - 1, BUYER);

    challenge_tx(template.as_json(), Error::ChangeError)
}

#[test]
fn challenge_account_sale_buy_seller_profit_owner() {
    let mut template = before_each(PRICE);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "account": ACCOUNT,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    // Simulate transfer profit to another lock.
    push_output_balance_cell(
        &mut template,
        PRICE - 6_000_000_000 + ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY
            - SECONDARY_MARKET_COMMON_FEE,
        "0x051111000000000000000000000000000000001111",
    );

    challenge_tx(template.as_json(), Error::ChangeError)
}

#[test]
fn challenge_account_sale_buy_seller_profit_capacity() {
    let mut template = before_each(PRICE);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "account": ACCOUNT,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    // Simulate transfer profit less than the SELLER should get.
    push_output_balance_cell(
        &mut template,
        PRICE - 6_000_000_000 + ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY
            - SECONDARY_MARKET_COMMON_FEE
            - 1,
        SELLER,
    );

    challenge_tx(template.as_json(), Error::ChangeError)
}

#[test]
fn challenge_account_sale_buy_not_clear_records() {
    let params = gen_params(INVITER, CHANNEL);
    let mut template = init_with_profit_rate("buy_account", Some(&params));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Selling as u8),
                "records": json!([
                    {
                        "type": "address",
                        "key": "eth",
                        "label": "Personal",
                        "value": "0x0000000000000000000000000000000000000000",
                    },
                    {
                        "type": "address",
                        "key": "eth",
                        "label": "Company",
                        "value": "0x0000000000000000000000000000000000001111",
                    }
                ])
            }
        }),
    );
    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE.to_string(),
            }
        }),
    );
    push_input_balance_cell(&mut template, PRICE, BUYER);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Normal as u8),
                // Simulate not clearing all records when transferring.
                "records": json!([
                    {
                        "type": "address",
                        "key": "eth",
                        "label": "Personal",
                        "value": "0x0000000000000000000000000000000000000000",
                    }
                ])
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(
        &mut template,
        PRICE - 6_000_000_000 + ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY
            - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), Error::AccountCellRecordNotEmpty)
}
