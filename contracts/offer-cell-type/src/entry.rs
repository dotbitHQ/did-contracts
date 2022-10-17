use alloc::boxed::Box;
use alloc::string::String;
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::constants::*;
use das_core::error::*;
use das_core::witness_parser::WitnessesParser;
use das_core::{assert, assert_lock_equal, code_to_error, data_parser, debug, util, verifiers};
use das_map::map::Map;
use das_map::util as map_util;
use das_types::constants::AccountStatus;
use das_types::packed::*;
use das_types::prelude::*;

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running offer-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    };
    let action = action_cp.as_slice();

    util::is_system_off(&parser)?;

    let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;

    debug!(
        "Route to {:?} action ...",
        String::from_utf8(action.to_vec()).map_err(|_| ErrorCode::ActionNotSupported)?
    );
    match action {
        b"make_offer" | b"edit_offer" => {
            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            let config_second_market = parser.configs.secondary_market()?;

            if action == b"make_offer" {
                verifiers::common::verify_cell_number_and_position(
                    "OfferCell",
                    &input_cells,
                    &[],
                    &output_cells,
                    &[0],
                )?;
            } else {
                verifiers::common::verify_cell_number_and_position(
                    "OfferCell",
                    &input_cells,
                    &[0],
                    &output_cells,
                    &[0],
                )?;
            }

            let sender_lock = high_level::load_cell_lock(0, Source::Input)?;
            let balance_cells = util::find_balance_cells(config_main, sender_lock.as_reader(), Source::Input)?;
            let all_input_cells = if action == b"make_offer" {
                balance_cells
            } else {
                [input_cells.clone(), balance_cells].concat()
            };
            verifiers::misc::verify_no_more_cells(&all_input_cells, Source::Input)?;

            debug!("Verify if the change is transferred back to the sender properly.");

            let total_input_capacity = util::load_cells_capacity(&all_input_cells, Source::Input)?;
            let offer_cell_capacity = high_level::load_cell_capacity(output_cells[0], Source::Output)?;
            let common_fee = u64::from(config_second_market.common_fee());
            if total_input_capacity > offer_cell_capacity + common_fee {
                debug!(
                    "The buyer should get a change of {} shannon.",
                    total_input_capacity - offer_cell_capacity - common_fee
                );

                verifiers::misc::verify_user_get_change(
                    config_main,
                    sender_lock.as_reader(),
                    total_input_capacity - offer_cell_capacity - common_fee,
                )?;
            }

            debug!("Verify if the OfferCell.lock is the das-lock.");

            let expected_lock = das_lock();
            let current_lock = high_level::load_cell_lock(output_cells[0], Source::Output)?;
            assert!(
                util::is_type_id_equal(expected_lock.as_reader(), current_lock.as_reader()),
                ErrorCode::OfferCellLockError,
                "The OfferCell.lock should be the das-lock."
            );

            debug!("Verify if the OfferCell.lock is the same as the lock of inputs[0].");

            assert_lock_equal!(
                (all_input_cells[0], Source::Input),
                (output_cells[0], Source::Output),
                ErrorCode::OfferCellLockError,
                "The OfferCell.lock should be the same as the lock of inputs[0]."
            );

            let output_offer_cell_witness = util::parse_offer_cell_witness(&parser, output_cells[0], Source::Output)?;
            let output_offer_cell_witness_reader = output_offer_cell_witness.as_reader();

            if action == b"make_offer" {
                debug!("Verify if the fields of the OfferCell is set correctly.");

                verify_price(
                    config_second_market,
                    output_offer_cell_witness_reader,
                    output_cells[0],
                    Source::Output,
                    None,
                )?;
                verify_message_length(config_second_market, output_offer_cell_witness_reader)?;
            } else {
                let input_offer_cell_witness = util::parse_offer_cell_witness(&parser, input_cells[0], Source::Input)?;
                let input_offer_cell_witness_reader = input_offer_cell_witness.as_reader();

                debug!("Verify if the fields of the OfferCell is modified propoerly.");

                assert!(
                    util::is_reader_eq(
                        input_offer_cell_witness_reader.account(),
                        output_offer_cell_witness_reader.account()
                    ),
                    ErrorCode::OfferCellFieldCanNotModified,
                    "The OfferCell.account can not be modified."
                );

                assert!(
                    util::is_reader_eq(
                        input_offer_cell_witness_reader.inviter_lock(),
                        output_offer_cell_witness_reader.inviter_lock()
                    ),
                    ErrorCode::OfferCellFieldCanNotModified,
                    "The OfferCell.inviter_lock can not be modified."
                );

                assert!(
                    util::is_reader_eq(
                        input_offer_cell_witness_reader.channel_lock(),
                        output_offer_cell_witness_reader.channel_lock()
                    ),
                    ErrorCode::OfferCellFieldCanNotModified,
                    "The OfferCell.channel_lock can not be modified."
                );

                debug!("Verify if the fields of the OfferCell has been changed correctly.");

                let input_offer_capacity = high_level::load_cell_capacity(input_cells[0], Source::Input)?;
                let old_price = u64::from(input_offer_cell_witness_reader.price());
                let old_fee = input_offer_capacity - old_price;

                let output_offer_capacity = high_level::load_cell_capacity(output_cells[0], Source::Output)?;
                let new_price = u64::from(output_offer_cell_witness_reader.price());
                let new_fee = output_offer_capacity - new_price;

                assert!(
                    old_fee - new_fee <= common_fee,
                    ErrorCode::OfferCellCapacityError,
                    "The fee paid by the OfferCell should be less than or equal to {} shannon.(expected: {} = {}(old_fee) - {}(new_fee))",
                    common_fee,
                    old_fee - new_fee,
                    old_fee,
                    new_fee
                );

                verify_price(
                    config_second_market,
                    output_offer_cell_witness_reader,
                    output_cells[0],
                    Source::Output,
                    Some(new_fee),
                )?;

                let mut changed = false;
                if !util::is_reader_eq(
                    input_offer_cell_witness_reader.price(),
                    output_offer_cell_witness_reader.price(),
                ) {
                    changed = true;
                }
                if !util::is_reader_eq(
                    input_offer_cell_witness_reader.message(),
                    output_offer_cell_witness_reader.message(),
                ) {
                    verify_message_length(config_second_market, output_offer_cell_witness_reader)?;
                    changed = true;
                }

                assert!(
                    changed,
                    ErrorCode::InvalidTransactionStructure,
                    "The OfferCell has not been changed."
                );
            }

            let account = output_offer_cell_witness_reader.account().raw_data();
            let account_without_suffix = &account[0..account.len() - 4];
            verifiers::account_cell::verify_unavailable_accounts(&parser, account_without_suffix)?;

            util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
        }
        b"cancel_offer" => {
            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            let config_second_market = parser.configs.secondary_market()?;

            assert!(
                input_cells.len() >= 1 && output_cells.len() == 0,
                ErrorCode::InvalidTransactionStructure,
                "There should be at least 1 OfferCell in inputs."
            );

            // Stop transaction builder to spend users other cells in this transaction.
            verifiers::misc::verify_no_more_cells(&input_cells, Source::Input)?;

            debug!("Verify if all OfferCells in inputs has the same lock script with the first OfferCell.");

            let expected_lock_hash = high_level::load_cell_lock_hash(input_cells[0], Source::Input)?;
            let mut total_input_capacity = 0;
            for i in input_cells.iter() {
                let lock_hash = high_level::load_cell_lock_hash(*i, Source::Input)?;
                assert!(
                    expected_lock_hash == lock_hash,
                    ErrorCode::InvalidTransactionStructure,
                    "Inputs[{}] The OfferCell should has the same lock script with others.",
                    i
                );

                total_input_capacity += high_level::load_cell_capacity(*i, Source::Input)?;
            }

            debug!("Verify if all capacity have been refund to user correctly.");

            let expected_lock = high_level::load_cell_lock(input_cells[0], Source::Input)?;
            let common_fee = u64::from(config_second_market.common_fee());
            verifiers::misc::verify_user_get_change(
                config_main,
                expected_lock.as_reader(),
                total_input_capacity - common_fee,
            )?;

            util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
        }
        b"accept_offer" => {
            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            parser.parse_cell()?;

            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;
            let config_secondary_market = parser.configs.secondary_market()?;

            verifiers::common::verify_cell_number("AccountSaleCell", &input_cells, 1, &output_cells, 0)?;

            let account_cell_type_id = config_main.type_id_table().account_cell();
            let (input_account_cells, output_account_cells) =
                util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, account_cell_type_id)?;
            verifiers::common::verify_cell_number_and_position(
                "AccountCell",
                &input_account_cells,
                &[1],
                &output_account_cells,
                &[0],
            )?;

            let input_account_cell_witness =
                util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
            let input_account_cell_witness_reader = input_account_cell_witness.as_reader();
            let output_account_cell_witness =
                util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
            let output_account_cell_witness_reader = output_account_cell_witness.as_reader();

            let buyer_lock = high_level::load_cell_lock(input_cells[0], Source::Input)?;
            let seller_lock = util::derive_owner_lock_from_cell(input_account_cells[0], Source::Input)?;

            let cells = [input_cells.clone(), input_account_cells.clone()].concat();
            verifiers::misc::verify_no_more_cells_with_same_lock(buyer_lock.as_reader(), &cells, Source::Input)?;
            verifiers::misc::verify_no_more_cells_with_same_lock(seller_lock.as_reader(), &cells, Source::Input)?;

            debug!("Verify if the AccountCell is transferred properly.");

            verifiers::account_cell::verify_account_expiration(
                config_account,
                input_account_cells[0],
                Source::Input,
                timestamp,
            )?;
            verifiers::account_cell::verify_status(
                &input_account_cell_witness_reader,
                AccountStatus::Normal,
                input_account_cells[0],
                Source::Input,
            )?;

            verifiers::account_cell::verify_account_capacity_not_decrease(
                input_account_cells[0],
                output_account_cells[0],
            )?;
            verifiers::account_cell::verify_account_data_consistent(
                input_account_cells[0],
                output_account_cells[0],
                vec![],
            )?;
            verifiers::account_cell::verify_account_witness_consistent(
                input_account_cells[0],
                output_account_cells[0],
                &input_account_cell_witness_reader,
                &output_account_cell_witness_reader,
                vec![],
            )?;

            let new_owner_lock = high_level::load_cell_lock(output_account_cells[0], Source::Output)?;
            assert!(
                util::is_entity_eq(&buyer_lock, &new_owner_lock),
                ErrorCode::OfferCellNewOwnerError,
                "The new owner of the AccountCell is not the buyer's lock.(expected: {}, current: {})",
                buyer_lock,
                new_owner_lock
            );

            debug!("Verify if the account is what the buyer want.");

            let account_cell_data = high_level::load_cell_data(input_account_cells[0], Source::Input)?;
            let current_account = data_parser::account_cell::get_account(&account_cell_data);

            let input_offer_cell_witness = util::parse_offer_cell_witness(&parser, input_cells[0], Source::Input)?;
            let input_offer_cell_witness_reader = input_offer_cell_witness.as_reader();

            let expected_account = input_offer_cell_witness_reader.account().raw_data();

            assert!(
                expected_account == current_account,
                ErrorCode::OfferCellAccountMismatch,
                "The account should be {}, but {} found.",
                String::from_utf8(expected_account.to_vec()).unwrap(),
                String::from_utf8(current_account.to_vec()).unwrap()
            );

            debug!("Verify if the profit is distribute correctly.");

            let inviter_lock = input_offer_cell_witness_reader.inviter_lock();
            let channel_lock = input_offer_cell_witness_reader.channel_lock();
            let price = u64::from(input_offer_cell_witness_reader.price());
            let offer_cell_capacity = high_level::load_cell_capacity(input_cells[0], Source::Input)?;
            let common_fee = u64::from(config_secondary_market.common_fee());

            verify_profit_distribution(
                &parser,
                config_main,
                seller_lock.as_reader().into(),
                inviter_lock,
                channel_lock,
                price,
                common_fee,
                offer_cell_capacity,
            )?;

            util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
        }
        _ => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    }

    Ok(())
}

fn verify_message_length(
    config_second_market: ConfigCellSecondaryMarketReader,
    offer_cell_witness: OfferCellDataReader,
) -> Result<(), Box<dyn ScriptError>> {
    let max_length = u32::from(config_second_market.offer_message_bytes_limit()) as usize;
    let message_length = offer_cell_witness.message().len();

    assert!(
        max_length >= message_length,
        ErrorCode::OfferCellMessageTooLong,
        "The OfferCell.witness.message is too long.(max_length_in_bytes: {})",
        max_length
    );

    Ok(())
}

fn verify_price(
    config_second_market: ConfigCellSecondaryMarketReader,
    offer_cell_witness: OfferCellDataReader,
    index: usize,
    source: Source,
    exist_fee: Option<u64>,
) -> Result<(), Box<dyn ScriptError>> {
    let basic_capacity = u64::from(config_second_market.offer_cell_basic_capacity());
    let fee = if let Some(exist_fee) = exist_fee {
        exist_fee
    } else {
        u64::from(config_second_market.offer_cell_prepared_fee_capacity())
    };

    let current_price = u64::from(offer_cell_witness.price());
    let current_capacity = high_level::load_cell_capacity(index, source)?;

    assert!(
        current_price >= basic_capacity,
        ErrorCode::OfferCellCapacityError,
        "The OfferCell.price should be more than or equal to the basic capacity.(current_price: {}, basic_capacity: {})",
        current_price,
        basic_capacity
    );
    assert!(
        current_capacity == current_price + fee,
        ErrorCode::OfferCellCapacityError,
        "The OfferCell.capacity should contain its price and prepared fee.(price: {}, current_capacity: {})",
        current_price,
        current_capacity
    );

    Ok(())
}

fn verify_profit_distribution(
    parser: &WitnessesParser,
    config_main: ConfigCellMainReader,
    seller_lock_reader: ScriptReader,
    inviter_lock_reader: ScriptReader,
    channel_lock_reader: ScriptReader,
    price: u64,
    common_fee: u64,
    offer_cell_capacity: u64,
) -> Result<(), Box<dyn ScriptError>> {
    let config_profit_rate = parser.configs.profit_rate()?;
    let default_script = Script::default();
    let default_script_reader = default_script.as_reader();

    let mut profit_map = Map::new();

    debug!("Calculate profit distribution for all roles.");

    let mut profit_of_seller = price;
    let mut profit_rate_of_das = u32::from(config_profit_rate.sale_das()) as u64;

    if !util::is_reader_eq(default_script_reader, inviter_lock_reader) {
        let profit_rate = u32::from(config_profit_rate.sale_buyer_inviter()) as u64;
        let profit = price / RATE_BASE * profit_rate;

        map_util::add(&mut profit_map, inviter_lock_reader.as_slice().to_vec(), profit);
        profit_of_seller -= profit;
        debug!("  The profit of the invitor: {}", profit);
    } else {
        profit_rate_of_das += u32::from(config_profit_rate.sale_buyer_inviter()) as u64;
    }

    if !util::is_reader_eq(default_script_reader, channel_lock_reader) {
        let profit_rate = u32::from(config_profit_rate.sale_buyer_channel()) as u64;
        let profit = price / RATE_BASE * profit_rate;

        map_util::add(&mut profit_map, channel_lock_reader.as_slice().to_vec(), profit);
        profit_of_seller -= profit;
        debug!("  The profit of the channel: {}", profit);
    } else {
        profit_rate_of_das += u32::from(config_profit_rate.sale_buyer_channel()) as u64;
    }

    let profit = price / RATE_BASE * profit_rate_of_das;
    let das_wallet_lock = das_wallet_lock();

    map_util::add(&mut profit_map, das_wallet_lock.as_slice().to_vec(), profit);
    profit_of_seller -= profit;
    debug!("  The profit of DAS: {}", profit);

    debug!("Check if seller get their profit properly.");

    let expected_capacity = if offer_cell_capacity > price + common_fee {
        // If the OfferCell takes some fee with it, the seller should get exactly their profit.
        profit_of_seller
    } else {
        // If the OfferCell does not contain any fee, the seller should get their profit with a bit of fee has been took.
        profit_of_seller - common_fee
    };
    verifiers::misc::verify_user_get_change(config_main, seller_lock_reader.into(), expected_capacity)?;

    verifiers::income_cell::verify_income_cells(parser, profit_map)?;

    Ok(())
}
