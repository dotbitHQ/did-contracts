use alloc::{boxed::Box, vec};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{packed as ckb_packed, prelude::*},
    high_level,
};
use das_core::{
    assert, assert_lock_equal, constants::*, data_parser, debug, error::Error, util, verifiers, warn,
    witness_parser::WitnessesParser,
};
use das_map::{map::Map, util as map_util};
use das_types::{constants::AccountStatus, mixer::*, packed::*};

pub fn main() -> Result<(), Error> {
    debug!("====== Running account-sale-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(Error::ActionNotSupported),
    };
    let action = action_cp.as_slice();

    util::is_system_off(&parser)?;

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.to_vec()).map_err(|_| Error::ActionNotSupported)?
    );
    match action {
        b"start_account_sale" | b"cancel_account_sale" | b"buy_account" => {
            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            parser.parse_cell()?;

            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;
            let config_secondary_market = parser.configs.secondary_market()?;

            let account_cell_type_id = config_main.type_id_table().account_cell();
            let (input_account_cells, output_account_cells) =
                util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, account_cell_type_id)?;
            let (input_sale_cells, output_sale_cells) = util::load_self_cells_in_inputs_and_outputs()?;

            assert!(
                input_account_cells.len() == 1 && output_account_cells.len() == 1,
                Error::InvalidTransactionStructure,
                "There should be 1 AccountCell in both inputs and outputs."
            );
            assert!(
                input_account_cells[0] == 0 && output_account_cells[0] == 0,
                Error::InvalidTransactionStructure,
                "The AccountCells should only appear in inputs[0] and outputs[0]."
            );

            let input_account_cell_witness =
                util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
            let input_account_cell_witness_reader = input_account_cell_witness.as_reader();
            let output_account_cell_witness =
                util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
            let output_account_cell_witness_reader = output_account_cell_witness.as_reader();

            match action {
                b"start_account_sale" => {
                    verifiers::account_cell::verify_unlock_role(action, &parser.params)?;

                    verifiers::common::verify_created_cell_in_correct_position(
                        "AccountSaleCell",
                        &input_sale_cells,
                        &output_sale_cells,
                        Some(1),
                    )?;

                    let sender_lock = high_level::load_cell_lock(0, Source::Input)?;
                    let sender_lock_reader = sender_lock.as_reader();
                    let input_balance_cells = util::find_balance_cells(config_main, sender_lock_reader, Source::Input)?;

                    debug!("Verify if there is no redundant cells in inputs.");

                    let all_cells = [input_account_cells.clone(), input_balance_cells.clone()].concat();
                    verifiers::misc::verify_no_more_cells(&all_cells, Source::Input)?;

                    debug!("Verify if sender get their change properly.");
                    verifiers::misc::verify_user_get_change_when_inputs_removed(
                        config_main,
                        sender_lock_reader,
                        &input_balance_cells,
                        &output_sale_cells,
                        u64::from(config_secondary_market.common_fee()),
                    )?;

                    debug!("Verify if the AccountCell is consistent in inputs and outputs.");

                    // The AccountCell should be consistent in inputs and outputs except the status field.
                    verify_account_cell_consistent_except_status(
                        config_account,
                        timestamp,
                        input_account_cells[0],
                        output_account_cells[0],
                        &input_account_cell_witness_reader,
                        &output_account_cell_witness_reader,
                    )?;

                    // If a user willing to sell owned account, the AccountCell should be in AccountStatus::Normal status.
                    verifiers::account_cell::verify_account_cell_status_update_correctly(
                        &input_account_cell_witness_reader,
                        &output_account_cell_witness_reader,
                        AccountStatus::Normal,
                        AccountStatus::Selling,
                    )?;

                    debug!("Verify if all fields of AccountSaleCell is properly set.");

                    let output_sale_cell_witness =
                        util::parse_account_sale_cell_witness(&parser, output_sale_cells[0], Source::Output)?;
                    let output_sale_cell_witness_reader = output_sale_cell_witness.as_reader();

                    verify_sale_cell_capacity(config_secondary_market, output_sale_cells[0])?;
                    verify_sale_cell_account_and_id(input_account_cells[0], &output_sale_cell_witness_reader)?;
                    verify_price(config_secondary_market, &output_sale_cell_witness_reader)?;
                    verify_description(config_secondary_market, &output_sale_cell_witness_reader)?;
                    verify_buyer_inviter_profit_rate(&output_sale_cell_witness_reader)?;
                    verify_started_at(timestamp, &output_sale_cell_witness_reader)?;

                    util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
                }
                b"cancel_account_sale" => {
                    verifiers::account_cell::verify_unlock_role(action, &parser.params)?;

                    verifiers::common::verify_removed_cell_in_correct_position(
                        "AccountSaleCell",
                        &input_sale_cells,
                        &output_sale_cells,
                        Some(1),
                    )?;

                    debug!("Verify if there is no redundant cells in inputs.");

                    let all_cells = [input_account_cells.clone(), input_sale_cells.clone()].concat();
                    verifiers::misc::verify_no_more_cells(&all_cells, Source::Input)?;

                    debug!("Verify if sender get their change properly.");

                    let sender_lock = high_level::load_cell_lock(0, Source::Input)?;
                    let sender_lock_reader = sender_lock.as_reader();
                    verifiers::misc::verify_user_get_change_when_inputs_removed(
                        config_main,
                        sender_lock_reader,
                        &input_sale_cells,
                        &(vec![]),
                        u64::from(config_secondary_market.common_fee()),
                    )?;

                    debug!(
                        "Verify if the AccountCell is consistent in inputs and outputs and its status is updated correctly."
                    );

                    verify_account_cell_consistent_except_status(
                        config_account,
                        timestamp,
                        input_account_cells[0],
                        output_account_cells[0],
                        &input_account_cell_witness_reader,
                        &output_account_cell_witness_reader,
                    )?;

                    // If a user want to cancel account sale, the AccountCell should be in AccountStatus::Selling status.
                    verifiers::account_cell::verify_account_cell_status_update_correctly(
                        &input_account_cell_witness_reader,
                        &output_account_cell_witness_reader,
                        AccountStatus::Selling,
                        AccountStatus::Normal,
                    )?;

                    debug!("Verify if the AccountSaleCell has the same account ID with the AccountCell inputs.");

                    let input_sale_cell_witness =
                        util::parse_account_sale_cell_witness(&parser, input_sale_cells[0], Source::Input)?;
                    let input_sale_cell_witness_reader = input_sale_cell_witness.as_reader();

                    verify_sale_cell_account_and_id(input_account_cells[0], &input_sale_cell_witness_reader)?;

                    util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
                }
                b"buy_account" => {
                    verifiers::common::verify_removed_cell_in_correct_position(
                        "AccountSaleCell",
                        &input_sale_cells,
                        &output_sale_cells,
                        Some(1),
                    )?;

                    let buyer_lock = high_level::load_cell_lock(2, Source::Input)?;
                    let buyer_lock_reader = buyer_lock.as_reader();
                    let input_balance_cells = util::find_balance_cells(config_main, buyer_lock_reader, Source::Input)?;

                    debug!("Verify if there is no redundant buyer's cells in inputs.");

                    verifiers::misc::verify_no_more_cells_with_same_lock(
                        buyer_lock_reader,
                        &input_balance_cells,
                        Source::Input,
                    )?;

                    debug!("Verify if the AccountCell is consistent in inputs and outputs.");

                    verifiers::account_cell::verify_account_expiration(
                        config_account,
                        input_account_cells[0],
                        timestamp,
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
                        vec!["status", "records"],
                    )?;

                    // If a user willing to buy the account, the AccountCell should be in AccountStatus::Selling status.
                    verifiers::account_cell::verify_account_cell_status_update_correctly(
                        &input_account_cell_witness_reader,
                        &output_account_cell_witness_reader,
                        AccountStatus::Selling,
                        AccountStatus::Normal,
                    )?;

                    verifiers::account_cell::verify_account_witness_record_empty(
                        &output_account_cell_witness_reader,
                        output_account_cells[0],
                        Source::Output,
                    )?;

                    debug!("Verify if the AccountSaleCell is belong to the AccountCell.");

                    let input_sale_cell_witness =
                        util::parse_account_sale_cell_witness(&parser, input_sale_cells[0], Source::Input)?;
                    let input_sale_cell_witness_reader = input_sale_cell_witness.as_reader();

                    verify_sale_cell_account_and_id(input_account_cells[0], &input_sale_cell_witness_reader)?;
                    // The cell carry refund capacity should be combined with the cell carry profit capacity, so skip checking refund here.
                    // verify_refund_correctly(config_main, config_secondary_market, input_sale_cells[0])?;

                    debug!("Verify if the AccountCell.lock is changed to new owner's lock properly.");

                    let output_account_cell_lock = high_level::load_cell_lock(output_account_cells[0], Source::Output)?;

                    assert!(
                        util::is_entity_eq(&buyer_lock, &output_account_cell_lock),
                        Error::AccountSaleCellNewOwnerError,
                        "The new owner's lock of AccountCell is mismatch with the BalanceCell in inputs.(expected: {}, current: {})",
                        buyer_lock,
                        output_account_cell_lock
                    );

                    debug!("Verify if buyer get their change properly.");

                    let price = u64::from(input_sale_cell_witness_reader.price());

                    let total_input_capacity = util::load_cells_capacity(&input_balance_cells, Source::Input)?;
                    // Actually, this assertion is already covered by `verify_user_get_change_when_inputs_removed()`, we write it here explict for better understanding
                    assert!(
                        total_input_capacity >= price,
                        Error::InvalidTransactionStructure,
                        "The buyer not pay enough to buy the account.(expected: {}, current: {})",
                        price,
                        total_input_capacity
                    );

                    verifiers::misc::verify_user_get_change_when_inputs_removed(
                        config_main,
                        buyer_lock_reader,
                        &input_balance_cells,
                        &(vec![]),
                        price,
                    )?;

                    debug!("Verify if the profit is distribute correctly.");

                    let seller_lock = util::derive_owner_lock_from_cell(input_account_cells[0], Source::Input)?;
                    let (inviter_lock, channel_lock) = decode_scripts_from_params(&parser.params)?;
                    let account_sale_cell_capacity =
                        high_level::load_cell_capacity(input_sale_cells[0], Source::Input)?;
                    let common_fee = u64::from(config_secondary_market.common_fee());

                    verify_profit_distribution(
                        &parser,
                        config_main,
                        seller_lock.as_reader(),
                        inviter_lock.as_reader(),
                        channel_lock.as_reader(),
                        &input_sale_cell_witness_reader,
                        account_sale_cell_capacity,
                        common_fee,
                    )?;

                    util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
                }
                _ => unreachable!(),
            }
        }
        b"edit_account_sale" => {
            verifiers::account_cell::verify_unlock_role(action, &parser.params)?;

            parser.parse_cell()?;

            let config_secondary_market_reader = parser.configs.secondary_market()?;

            let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            assert!(
                input_cells.len() == 1 && output_cells.len() == 1,
                Error::InvalidTransactionStructure,
                "There should be one AccountSaleCell in outputs and one AccountSaleCell in inputs."
            );
            assert!(
                input_cells[0] == 0 && output_cells[0] == 0,
                Error::InvalidTransactionStructure,
                "The AccountSaleCells should only appear at inputs[0] and outputs[0]."
            );

            debug!("Verify if there is no redundant cells in inputs.");

            verifiers::misc::verify_no_more_cells(&input_cells, Source::Input)?;

            let input_cell_witness = util::parse_account_sale_cell_witness(&parser, input_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();
            let output_cell_witness = util::parse_account_sale_cell_witness(&parser, output_cells[0], Source::Output)?;
            let output_cell_witness_reader = output_cell_witness.as_reader();

            verify_account_sale_cell_consistent(
                input_cells[0],
                output_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
            )?;

            verifiers::common::verify_tx_fee_spent_correctly(
                "AccountSaleCell",
                input_cells[0],
                output_cells[0],
                u64::from(config_secondary_market_reader.common_fee()),
                u64::from(config_secondary_market_reader.sale_cell_basic_capacity()),
            )?;

            let mut changed = false;

            let input_sale_price = u64::from(input_cell_witness_reader.price());
            let output_sale_price = u64::from(output_cell_witness_reader.price());
            if input_sale_price != output_sale_price {
                debug!(
                    "Sale price has been changed, verify if it higher than ConfigCellSecondaryMarket.sale_min_price."
                );
                verify_price(config_secondary_market_reader, &output_cell_witness_reader)?;
                changed = true;
            }

            let input_description = input_cell_witness_reader.description();
            let output_description = output_cell_witness_reader.description();
            if !util::is_reader_eq(input_description, output_description) {
                debug!("Description has been changed, verify if its size is less than ConfigCellSecondaryMarket.sale_description_bytes_limit.");
                verify_description(config_secondary_market_reader, &output_cell_witness_reader)?;
                changed = true;
            }

            if input_cell_witness_reader.version() == 1 {
                assert!(
                    output_cell_witness_reader.version() == 2,
                    Error::InvalidTransactionStructure,
                    "The AccountSaleCell should be upgrade to the latest version."
                );

                debug!("The profit rate of inviter has been changed, verify if its size is less than ConfigCellSecondaryMarket.sale_description_bytes_limit.");
                verify_buyer_inviter_profit_rate(&output_cell_witness_reader)?;
                changed = true;
            } else {
                let input_buyer_inviter_profit_rate = input_cell_witness_reader
                    .try_into_latest()
                    .unwrap()
                    .buyer_inviter_profit_rate();
                let output_buyer_inviter_profit_rate = output_cell_witness_reader
                    .try_into_latest()
                    .unwrap()
                    .buyer_inviter_profit_rate();
                if !util::is_reader_eq(input_buyer_inviter_profit_rate, output_buyer_inviter_profit_rate) {
                    debug!("The profit rate of inviter has been changed, verify if its size is less than ConfigCellSecondaryMarket.sale_description_bytes_limit.");
                    verify_buyer_inviter_profit_rate(&output_cell_witness_reader)?;
                    changed = true;
                }
            }

            assert!(
                changed,
                Error::InvalidTransactionStructure,
                "Either price or description should be modified."
            );

            util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
        }
        b"force_recover_account_status" => {
            util::require_type_script(
                &parser,
                TypeScript::AccountCellType,
                Source::Input,
                Error::InvalidTransactionStructure,
            )?;
        }
        _ => return Err(Error::ActionNotSupported),
    }

    Ok(())
}

fn decode_scripts_from_params(params: &[Bytes]) -> Result<(ckb_packed::Script, ckb_packed::Script), Error> {
    macro_rules! decode_script {
        ($param:expr, $name:expr) => {
            ckb_packed::Script::from_slice($param.raw_data()).map_err(|_| {
                warn!(
                    "Decoding {} in params failed.(bytes: 0x{})",
                    $name,
                    util::hex_string($param.raw_data())
                );
                Error::ParamsDecodingError
            })?
        };
    }

    let inviter_lock = decode_script!(params[0].as_reader(), "inviter_lock");
    let channel_lock = decode_script!(params[1].as_reader(), "channel_lock");

    Ok((inviter_lock, channel_lock))
}

fn verify_account_cell_consistent_except_status<'a>(
    config_account: ConfigCellAccountReader,
    timestamp: u64,
    input_account_cell: usize,
    output_account_cell: usize,
    input_account_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    output_account_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
) -> Result<(), Error> {
    verifiers::account_cell::verify_account_expiration(config_account, input_account_cell, timestamp)?;
    verifiers::account_cell::verify_account_capacity_not_decrease(input_account_cell, output_account_cell)?;
    verifiers::account_cell::verify_account_cell_consistent_with_exception(
        input_account_cell,
        output_account_cell,
        &input_account_cell_witness_reader,
        &output_account_cell_witness_reader,
        None,
        vec![],
        vec!["status"],
    )?;

    Ok(())
}

fn verify_sale_cell_capacity(
    config_reader: ConfigCellSecondaryMarketReader,
    output_sale_cell_index: usize,
) -> Result<(), Error> {
    let account_sale_cell_capacity = high_level::load_cell_capacity(output_sale_cell_index, Source::Output)?;
    let expected = u64::from(config_reader.sale_cell_basic_capacity())
        + u64::from(config_reader.sale_cell_prepared_fee_capacity());

    assert!(
        account_sale_cell_capacity == expected,
        Error::AccountSaleCellCapacityError,
        "The AccountSaleCell.capacity should be equal to {} .",
        expected
    );

    Ok(())
}

fn verify_sale_cell_account_and_id<'a>(
    input_account_cell: usize,
    witness_reader: &Box<dyn AccountSaleCellDataReaderMixer + 'a>,
) -> Result<(), Error> {
    let input_account_cell_data = util::load_cell_data(input_account_cell, Source::Input)?;
    let account_cell_account = data_parser::account_cell::get_account(&input_account_cell_data);
    let account_cell_account_id = data_parser::account_cell::get_id(&input_account_cell_data);

    // read account_id from AccountSaleCell's witness
    let account_sale_cell_account_id = witness_reader.account_id().raw_data();
    // ensure the AccountSaleCell's args equal to accountCell's id
    assert!(
        account_cell_account_id == account_sale_cell_account_id,
        Error::AccountSaleCellAccountIdInvalid,
        "The AccountSaleCell.witness.account_id should be equal to the AccountCell.data.account_id ."
    );

    // read account from AccountSaleCell's witness
    let account_sale_cell_account = witness_reader.account().raw_data();
    // ensure the AccountSaleCell's args equal to accountCell's id
    assert!(
        account_cell_account == account_sale_cell_account,
        Error::AccountSaleCellAccountIdInvalid,
        "The AccountSaleCell.witness.account should be equal to the AccountCell.data.account ."
    );

    Ok(())
}

fn verify_price<'a>(
    config_reader: ConfigCellSecondaryMarketReader,
    witness_reader: &Box<dyn AccountSaleCellDataReaderMixer + 'a>,
) -> Result<(), Error> {
    let price = u64::from(witness_reader.price());
    let sale_min_price = u64::from(config_reader.sale_min_price());
    assert!(
        price >= sale_min_price,
        Error::AccountSaleCellPriceTooSmall,
        "The price of account should be higher than ConfigCellSecondaryMarket.sale_min_price.(expected: >= {}, current: {})",
        sale_min_price,
        price
    );

    Ok(())
}

fn verify_description<'a>(
    config_reader: ConfigCellSecondaryMarketReader,
    witness_reader: &Box<dyn AccountSaleCellDataReaderMixer + 'a>,
) -> Result<(), Error> {
    let description = witness_reader.description();
    let bytes_limit = u32::from(config_reader.sale_description_bytes_limit());
    assert!(
        description.len() <= bytes_limit as usize,
        Error::AccountSaleCellDescriptionTooLarge,
        "The size of description in bytes should be less than ConfigCellSecondaryMarket.sale_description_bytes_limit.(expected: <= {}, current: {})",
        bytes_limit,
        description.len()
    );

    Ok(())
}

fn verify_buyer_inviter_profit_rate<'a>(
    witness_reader: &Box<dyn AccountSaleCellDataReaderMixer + 'a>,
) -> Result<(), Error> {
    assert!(
        witness_reader.version() == 2,
        Error::InvalidTransactionStructure,
        "Only AccountSaleCell in version 2 can be created from now on."
    );

    let witness_reader = witness_reader.try_into_latest().unwrap();
    let profit_rate = u32::from(witness_reader.buyer_inviter_profit_rate()) as u64;

    assert!(
        profit_rate <= RATE_BASE,
        Error::AccountSaleCellProfitRateError,
        "The AccountSaleCell.witness.buyer_inviter_profit_rate should be less than or equal to {}.",
        RATE_BASE
    );

    Ok(())
}

fn verify_started_at<'a>(
    current_timestamp: u64,
    witness_reader: &Box<dyn AccountSaleCellDataReaderMixer + 'a>,
) -> Result<(), Error> {
    let started_at = u64::from(witness_reader.started_at());

    assert!(
        current_timestamp == started_at,
        Error::AccountSaleCellStartedAtInvalid,
        "The AccountSaleCell.witness.started_at should be equal to the timestamp in TimeCell.(expected: {}, current: {})",
        current_timestamp,
        started_at
    );

    Ok(())
}

fn verify_account_sale_cell_consistent<'a>(
    input_cell: usize,
    output_cell: usize,
    input_cell_witness_reader: &Box<dyn AccountSaleCellDataReaderMixer + 'a>,
    output_cell_witness_reader: &Box<dyn AccountSaleCellDataReaderMixer + 'a>,
) -> Result<(), Error> {
    debug!("Verify if AccountSaleCell consistent in inputs and outputs.");

    assert_lock_equal!(
        (input_cell, Source::Input),
        (output_cell, Source::Output),
        Error::InvalidTransactionStructure,
        "The AccountSaleCell.lock should be consistent in inputs and outputs."
    );

    let input_account_id = input_cell_witness_reader.account_id();
    let output_account_id = output_cell_witness_reader.account_id();
    assert!(
        util::is_reader_eq(input_account_id, output_account_id),
        Error::AccountSaleCellAccountIdInvalid,
        "The AccountSaleCell.witness.account_id should be consistent in inputs and outputs.(input: {}, output: {})",
        util::hex_string(input_account_id.raw_data()),
        util::hex_string(output_account_id.raw_data())
    );

    let input_account = input_cell_witness_reader.account();
    let output_account = output_cell_witness_reader.account();
    assert!(
        util::is_reader_eq(input_account, output_account),
        Error::AccountSaleCellAccountIdInvalid,
        "The AccountSaleCell.witness.account should be consistent in inputs and outputs.(input: {}, output: {})",
        util::hex_string(input_account.raw_data()),
        util::hex_string(output_account.raw_data())
    );

    let input_started_at = input_cell_witness_reader.started_at();
    let output_started_at = output_cell_witness_reader.started_at();
    assert!(
        util::is_reader_eq(input_started_at, output_started_at),
        Error::AccountSaleCellStartedAtInvalid,
        "The AccountSaleCell.witness.started_at should be consistent in inputs and outputs.(input: {}, output: {})",
        util::hex_string(input_started_at.raw_data()),
        util::hex_string(output_started_at.raw_data())
    );

    Ok(())
}

fn verify_profit_distribution<'a>(
    parser: &WitnessesParser,
    config_main: ConfigCellMainReader,
    seller_lock_reader: ckb_packed::ScriptReader,
    inviter_lock_reader: ckb_packed::ScriptReader,
    channel_lock_reader: ckb_packed::ScriptReader,
    input_sale_cell_witness_reader: &Box<dyn AccountSaleCellDataReaderMixer + 'a>,
    account_sale_cell_capacity: u64,
    common_fee: u64,
) -> Result<(), Error> {
    let config_profit_rate = parser.configs.profit_rate()?;
    let price = u64::from(input_sale_cell_witness_reader.price());

    let default_script = ckb_packed::Script::default();
    let default_script_reader = default_script.as_reader();

    let mut profit_map = Map::new();

    debug!("Calculate profit distribution for all roles.");

    let mut profit_of_seller = price;
    let mut profit_rate_of_das = u32::from(config_profit_rate.sale_das()) as u64;

    if !util::is_reader_eq(default_script_reader, inviter_lock_reader) {
        let profit_rate = if input_sale_cell_witness_reader.version() == 2 {
            let witness_reader = input_sale_cell_witness_reader.try_into_latest().unwrap();
            u32::from(witness_reader.buyer_inviter_profit_rate()) as u64
        } else {
            u32::from(config_profit_rate.sale_buyer_inviter()) as u64
        };
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

    let expected_capacity = profit_of_seller + account_sale_cell_capacity - common_fee;
    verifiers::misc::verify_user_get_change(config_main, seller_lock_reader, expected_capacity)?;

    verifiers::income_cell::verify_income_cells(parser, profit_map)?;

    Ok(())
}
