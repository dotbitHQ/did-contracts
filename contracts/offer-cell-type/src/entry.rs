use alloc::{boxed::Box, format, string::String};
use ckb_std::{ckb_constants::Source, high_level};
use core::result::Result;
use das_core::{
    assert, assert_lock_equal,
    constants::*,
    data_parser, debug,
    eip712::{to_semantic_capacity, verify_eip712_hashes},
    error::Error,
    parse_account_cell_witness, parse_witness, util, verifiers, warn,
    witness_parser::WitnessesParser,
};
use das_map::{map::Map, util as map_util};
use das_types::{
    constants::{AccountStatus, DataType},
    mixer::AccountCellDataMixer,
    packed::*,
    prelude::*,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running offer-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(Error::ActionNotSupported),
    };
    let action = action_cp.as_slice();

    util::is_system_off(&mut parser)?;

    let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;

    debug!(
        "Route to {:?} action ...",
        String::from_utf8(action.to_vec()).map_err(|_| Error::ActionNotSupported)?
    );
    match action {
        b"make_offer" | b"edit_offer" => {
            parser.parse_config(&[DataType::ConfigCellMain, DataType::ConfigCellSecondaryMarket])?;
            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            let config_second_market = parser.configs.secondary_market()?;

            let sender_lock = high_level::load_cell_lock(0, Source::Input)?;
            let all_input_cells;
            if action == b"make_offer" {
                assert!(
                    input_cells.len() == 0 && output_cells.len() == 1,
                    Error::InvalidTransactionStructure,
                    "There should be only 1 OfferCell at outputs[0]."
                );
                assert!(
                    output_cells[0] == 0,
                    Error::InvalidTransactionStructure,
                    "There should be only 1 OfferCell at outputs[0]."
                );

                all_input_cells = util::find_balance_cells(config_main, sender_lock.as_reader(), Source::Input)?;
                verifiers::misc::verify_no_more_cells(&all_input_cells, Source::Input)?;
            } else {
                assert!(
                    input_cells.len() == 1 && output_cells.len() == 1,
                    Error::InvalidTransactionStructure,
                    "There should be at least 1 OfferCell in inputs and outputs."
                );
                assert!(
                    input_cells[0] == 0 && output_cells[0] == 0,
                    Error::InvalidTransactionStructure,
                    "There should be 1 OfferCell in inputs[0] and outputs[0]."
                );

                let balance_cells = util::find_balance_cells(config_main, sender_lock.as_reader(), Source::Input)?;
                all_input_cells = [input_cells.clone(), balance_cells].concat();
                verifiers::misc::verify_no_more_cells(&all_input_cells, Source::Input)?;
            }

            debug!("Verify if the change is transferred back to the sender properly.");

            let mut total_input_capacity = 0;
            for i in all_input_cells.iter() {
                total_input_capacity += high_level::load_cell_capacity(*i, Source::Input)?;
            }
            let offer_cell_capacity = high_level::load_cell_capacity(output_cells[0], Source::Output)?;
            let common_fee = u64::from(config_second_market.common_fee());
            if total_input_capacity > offer_cell_capacity + common_fee {
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
                Error::OfferCellLockError,
                "The OfferCell.lock should be the das-lock."
            );

            debug!("Verify if the OfferCell.lock is the same as the lock of inputs[0].");

            assert_lock_equal!(
                (all_input_cells[0], Source::Input),
                (output_cells[0], Source::Output),
                Error::OfferCellLockError,
                "The OfferCell.lock should be the same as the lock of inputs[0]."
            );

            let output_offer_cell_witness;
            let output_offer_cell_witness_reader;
            parse_witness!(
                output_offer_cell_witness,
                output_offer_cell_witness_reader,
                parser,
                output_cells[0],
                Source::Output,
                OfferCellData
            );

            debug!("Verify if the fields of the OfferCell is set correctly.");

            verify_price(
                config_second_market,
                output_offer_cell_witness_reader,
                output_cells[0],
                Source::Output,
            )?;
            verify_message_length(config_second_market, output_offer_cell_witness_reader)?;

            if action == b"make_offer" {
                verify_eip712_hashes(&parser, make_offer_to_semantic)?;

                let account = output_offer_cell_witness_reader.account().raw_data();
                let account_without_suffix = &account[0..account.len() - 4];
                verifiers::account_cell::verify_unavailable_accounts(&mut parser, account_without_suffix)?;
            } else {
                verify_eip712_hashes(&parser, edit_offer_to_semantic)?;

                let input_offer_cell_witness;
                let input_offer_cell_witness_reader;
                parse_witness!(
                    input_offer_cell_witness,
                    input_offer_cell_witness_reader,
                    parser,
                    input_cells[0],
                    Source::Input,
                    OfferCellData
                );

                assert!(
                    util::is_reader_eq(
                        input_offer_cell_witness_reader.account(),
                        output_offer_cell_witness_reader.account()
                    ),
                    Error::OfferCellAccountNotMatch,
                    "The OfferCell.account can not be changed."
                )
            }
        }
        b"cancel_offer" => {
            parser.parse_config(&[DataType::ConfigCellMain, DataType::ConfigCellSecondaryMarket])?;
            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            let config_second_market = parser.configs.secondary_market()?;

            verify_eip712_hashes(&parser, cancel_offer_to_semantic)?;

            assert!(
                input_cells.len() >= 1 && output_cells.len() == 0,
                Error::InvalidTransactionStructure,
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
                    Error::InvalidTransactionStructure,
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
        }
        b"accept_offer" => {
            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            parser.parse_config(&[
                DataType::ConfigCellMain,
                DataType::ConfigCellAccount,
                DataType::ConfigCellIncome,
                DataType::ConfigCellProfitRate,
                DataType::ConfigCellSecondaryMarket,
            ])?;
            parser.parse_cell()?;

            verify_eip712_hashes(&parser, accept_offer_to_semantic)?;

            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;
            let config_income = parser.configs.income()?;
            let config_profit_rate = parser.configs.profit_rate()?;
            let config_secondary_market = parser.configs.secondary_market()?;

            assert!(
                input_cells.len() == 1 && output_cells.len() == 0,
                Error::InvalidTransactionStructure,
                "There should be only 1 OfferCell in inputs."
            );
            assert!(
                input_cells[0] == 0,
                Error::InvalidTransactionStructure,
                "The first OfferCell should be started at inputs[0]."
            );

            let account_cell_type_id = config_main.type_id_table().account_cell();
            let (input_account_cells, output_account_cells) =
                util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, account_cell_type_id)?;

            assert!(
                input_account_cells.len() == 1 && output_account_cells.len() == 1,
                Error::InvalidTransactionStructure,
                "There should be 1 AccountCell in both inputs and outputs."
            );
            assert!(
                input_account_cells[0] == 1 && output_account_cells[0] == 0,
                Error::InvalidTransactionStructure,
                "The AccountCell should only appear in inputs[1] and outputs[0]."
            );

            let input_account_cell_witness: Box<dyn AccountCellDataMixer>;
            let input_account_cell_witness_reader;
            parse_account_cell_witness!(
                input_account_cell_witness,
                input_account_cell_witness_reader,
                parser,
                input_account_cells[0],
                Source::Input
            );

            let output_account_cell_witness: Box<dyn AccountCellDataMixer>;
            let output_account_cell_witness_reader;
            parse_account_cell_witness!(
                output_account_cell_witness,
                output_account_cell_witness_reader,
                parser,
                output_account_cells[0],
                Source::Output
            );

            let buyer_lock = high_level::load_cell_lock(input_cells[0], Source::Input)?;
            let seller_lock = high_level::load_cell_lock(input_account_cells[0], Source::Input)?;

            let cells = [input_cells.clone(), input_account_cells.clone()].concat();
            verifiers::misc::verify_no_more_cells_with_same_lock(buyer_lock.as_reader(), &cells, Source::Input)?;
            verifiers::misc::verify_no_more_cells_with_same_lock(seller_lock.as_reader(), &cells, Source::Input)?;

            debug!("Verify if the AccountCell is transferred properly.");

            verifiers::account_cell::verify_account_expiration(config_account, input_account_cells[0], timestamp)?;
            verifiers::account_cell::verify_account_cell_status(
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
                Error::OfferCellNewOwnerError,
                "The new owner of the AccountCell is not the buyer's lock.(expected: {}, current: {})",
                buyer_lock,
                new_owner_lock
            );

            debug!("Verify if the account is what the buyer want.");

            let account_cell_data = high_level::load_cell_data(input_account_cells[0], Source::Input)?;
            let current_account = data_parser::account_cell::get_account(&account_cell_data);

            let input_offer_cell_witness;
            let input_offer_cell_witness_reader;
            parse_witness!(
                input_offer_cell_witness,
                input_offer_cell_witness_reader,
                parser,
                input_cells[0],
                Source::Input,
                OfferCellData
            );

            let expected_account = input_offer_cell_witness_reader.account().raw_data();

            assert!(
                expected_account == current_account,
                Error::OfferCellAccountNotMatch,
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
                config_income,
                config_profit_rate,
                seller_lock.as_reader().into(),
                inviter_lock,
                channel_lock,
                price,
                common_fee,
                offer_cell_capacity,
            )?;
        }
        _ => return Err(Error::ActionNotSupported),
    }

    Ok(())
}

fn verify_message_length(
    config_second_market: ConfigCellSecondaryMarketReader,
    offer_cell_witness: OfferCellDataReader,
) -> Result<(), Error> {
    let max_length = u32::from(config_second_market.offer_message_bytes_limit()) as usize;
    let message_length = offer_cell_witness.message().len();

    assert!(
        max_length >= message_length,
        Error::OfferCellMessageTooLong,
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
) -> Result<(), Error> {
    let min_price = u64::from(config_second_market.offer_min_price());
    let basic_capacity = u64::from(config_second_market.offer_cell_basic_capacity());
    let prepared_fee_capacity = u64::from(config_second_market.offer_cell_prepared_fee_capacity());

    let current_price = u64::from(offer_cell_witness.price());
    let current_capacity = high_level::load_cell_capacity(index, source)?;

    assert!(
        current_capacity >= basic_capacity + prepared_fee_capacity,
        Error::OfferCellCapacityError,
        "The OfferCell.capacity should be at least {}.(basic_capacity: {}, prepared_fee_capacity: {})",
        basic_capacity + prepared_fee_capacity,
        basic_capacity,
        prepared_fee_capacity
    );

    assert!(
        current_price >= min_price,
        Error::OfferCellCapacityError,
        "The OfferCell.witness.price is too low.(min_price: {})",
        min_price
    );

    assert!(
        current_capacity == current_price + prepared_fee_capacity,
        Error::OfferCellCapacityError,
        "The OfferCell.capacity should contain its price and prepared fee.(price: {}, current_capacity: {})",
        current_price,
        current_capacity
    );

    Ok(())
}

fn verify_profit_distribution(
    parser: &WitnessesParser,
    config_main: ConfigCellMainReader,
    config_income: ConfigCellIncomeReader,
    config_profit_rate: ConfigCellProfitRateReader,
    seller_lock_reader: ScriptReader,
    inviter_lock_reader: ScriptReader,
    channel_lock_reader: ScriptReader,
    price: u64,
    common_fee: u64,
    offer_cell_capacity: u64,
) -> Result<(), Error> {
    let default_script = Script::default();
    let default_script_reader = default_script.as_reader();

    let income_cell_type_id = config_main.type_id_table().income_cell();
    let (input_income_cells, output_income_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, income_cell_type_id)?;

    // Because we do not verify the consistency of the creator, so there must be no IncomeCell in inputs.
    assert!(
        input_income_cells.len() == 0,
        Error::InvalidTransactionStructure,
        "There should be no IncomeCell in inputs."
    );
    assert!(
        output_income_cells.len() == 1 && output_income_cells[0] == 1,
        Error::InvalidTransactionStructure,
        "There should be 1 IncomeCell at outputs[1]."
    );

    verifiers::misc::verify_always_success_lock(output_income_cells[0], Source::Output)?;

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

    debug!("Check if other roles get their profit properly.");

    let output_income_cell_witness;
    let output_income_cell_witness_reader;
    parse_witness!(
        output_income_cell_witness,
        output_income_cell_witness_reader,
        parser,
        output_income_cells[0],
        Source::Output,
        IncomeCellData
    );

    verifiers::income_cell::verify_records_match_with_creating(
        parser.configs.income()?,
        output_income_cells[0],
        Source::Output,
        output_income_cell_witness_reader,
        profit_map,
    )?;

    let income_cell_max_records = u32::from(config_income.max_records()) as usize;
    assert!(
        output_income_cell_witness_reader.records().len() <= income_cell_max_records,
        Error::IncomeCellConsolidateError,
        "The IncomeCell can not store more than {} records.",
        income_cell_max_records
    );

    Ok(())
}

fn offer_to_semantic(parser: &WitnessesParser, source: Source) -> Result<(String, String), Error> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let offer_cells = util::find_cells_by_type_id(ScriptType::Type, type_id_table_reader.offer_cell(), source)?;
    let witness;
    let witness_reader;

    assert!(
        offer_cells.len() > 0,
        Error::InvalidTransactionStructure,
        "There should be at least 1 OfferCell in transaction."
    );

    parse_witness!(witness, witness_reader, parser, offer_cells[0], source, OfferCellData);

    let account = String::from_utf8(witness_reader.account().raw_data().to_vec()).map_err(|_| {
        warn!("EIP712 decoding OfferCellData failed");
        Error::WitnessEntityDecodingError
    })?;
    let amount = to_semantic_capacity(u64::from(witness_reader.price()));

    Ok((account, amount))
}

fn make_offer_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let (account, amount) = offer_to_semantic(parser, Source::Output)?;
    Ok(format!("MAKE AN OFFER ON {} WITH {}", account, amount))
}

fn edit_offer_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let (_, old_amount) = offer_to_semantic(parser, Source::Output)?;
    let (account, new_amount) = offer_to_semantic(parser, Source::Output)?;
    Ok(format!(
        "CHANGE THE OFFER ON {} FROM {} TO {}",
        account, old_amount, new_amount
    ))
}

fn cancel_offer_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let offer_cells = util::find_cells_by_type_id(ScriptType::Type, type_id_table_reader.offer_cell(), Source::Input)?;

    Ok(format!("CANCEL {} OFFERS", offer_cells.len()))
}

fn accept_offer_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let (account, amount) = offer_to_semantic(parser, Source::Input)?;
    Ok(format!("ACCEPT THE OFFER ON {} WITH {}", account, amount))
}
