use alloc::{boxed::Box, string::String, vec, vec::Vec};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{packed as ckb_packed, prelude::*},
    high_level,
};
use das_core::{
    assert, constants::*, data_parser, debug, eip712::verify_eip712_hashes, error::Error, inspect,
    parse_account_cell_witness, parse_witness, util, util::find_cells_by_script, verifiers::account_cell, warn,
    witness_parser::WitnessesParser,
};
use das_map::{map::Map, util as map_util};
use das_types::{
    constants::{AccountStatus, DataType},
    mixer::*,
    packed::*,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running account-sale-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_opt = parser.parse_action_with_params()?;
    if action_opt.is_none() {
        return Err(Error::ActionNotSupported);
    }

    let (action_raw, params_raw) = action_opt.unwrap();
    let action = action_raw.as_reader().raw_data();
    let params = params_raw.iter().map(|param| param.as_reader()).collect::<Vec<_>>();

    util::is_system_off(&mut parser)?;
    account_cell::verify_unlock_role(action_raw.as_reader(), &params)?;

    debug!(
        "Route to {:?} action ...",
        String::from_utf8(action.to_vec()).map_err(|_| Error::ActionNotSupported)?
    );
    if action == b"start_account_sale" || action == b"cancel_account_sale" || action == b"buy_account" {
        let timestamp = util::load_oracle_data(OracleCellType::Time)?;

        parser.parse_cell()?;
        parser.parse_config(&[
            DataType::ConfigCellMain,
            DataType::ConfigCellAccount,
            DataType::ConfigCellSecondaryMarket,
        ])?;
        if action == b"buy_account" {
            parser.parse_config(&[DataType::ConfigCellProfitRate, DataType::ConfigCellIncome])?;
        }

        let config_main = parser.configs.main()?;
        let config_account = parser.configs.account()?;
        let config_secondary_market = parser.configs.secondary_market()?;

        let (input_account_cells, output_account_cells) = load_account_cells(config_main)?;
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

        if action == b"start_account_sale" {
            assert!(
                input_sale_cells.len() == 0 && output_sale_cells.len() == 1,
                Error::InvalidTransactionStructure,
                "There should be 0 AccountSaleCell in inputs and 1 AccountSaleCell in outputs."
            );
            assert!(
                output_sale_cells[0] == 1,
                Error::InvalidTransactionStructure,
                "The AccountSaleCell should only appear in outputs[1]."
            );

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
            account_cell::verify_account_cell_status_update_correctly(
                &input_account_cell_witness_reader,
                &output_account_cell_witness_reader,
                AccountStatus::Normal,
                AccountStatus::Selling,
            )?;

            debug!("Verify if all fields of AccountSaleCell is properly set.");

            let output_sale_cell_witness;
            let output_sale_cell_witness_reader;
            parse_witness!(
                output_sale_cell_witness,
                output_sale_cell_witness_reader,
                parser,
                output_sale_cells[0],
                Source::Output,
                AccountSaleCellData
            );

            verify_sale_cell_capacity(config_secondary_market, output_sale_cells[0])?;
            verify_sale_cell_account_and_id(input_account_cells[0], output_sale_cell_witness_reader)?;
            verify_price(config_secondary_market, output_sale_cell_witness_reader)?;
            verify_description(config_secondary_market, output_sale_cell_witness_reader)?;
            verify_started_at(timestamp, output_sale_cell_witness_reader)?;
        } else if action == b"cancel_account_sale" {
            assert!(
                input_sale_cells.len() == 1 && output_sale_cells.len() == 0,
                Error::InvalidTransactionStructure,
                "There should be 0 AccountSaleCell in outputs and 1 AccountSaleCell in inputs."
            );
            assert!(
                input_sale_cells[0] == 1,
                Error::InvalidTransactionStructure,
                "The AccountSaleCell should only appear in inputs[1]."
            );

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
            account_cell::verify_account_cell_status_update_correctly(
                &input_account_cell_witness_reader,
                &output_account_cell_witness_reader,
                AccountStatus::Selling,
                AccountStatus::Normal,
            )?;

            debug!("Verify if the AccountSaleCell has the same account ID with the AccountCell inputs.");

            let input_sale_cell_witness;
            let input_sale_cell_witness_reader;
            parse_witness!(
                input_sale_cell_witness,
                input_sale_cell_witness_reader,
                parser,
                input_sale_cells[0],
                Source::Input,
                AccountSaleCellData
            );

            verify_sale_cell_account_and_id(input_account_cells[0], input_sale_cell_witness_reader)?;
            verify_refund_correctly(config_main, config_secondary_market, input_sale_cells[0])?;
        } else if action == b"buy_account" {
            assert!(
                input_sale_cells.len() == 1 && output_sale_cells.len() == 0,
                Error::InvalidTransactionStructure,
                "There should be 0 AccountSaleCell in outputs and 1 AccountSaleCell in inputs."
            );
            assert!(
                input_sale_cells[0] == 1,
                Error::InvalidTransactionStructure,
                "The AccountSaleCell should only appear in inputs[0]."
            );

            let config_profit_rate = parser.configs.profit_rate()?;

            debug!("Verify if the AccountCell is consistent in inputs and outputs.");

            account_cell::verify_account_expiration(config_account, input_account_cells[0], timestamp)?;
            account_cell::verify_account_data_consistent(input_account_cells[0], output_account_cells[0], vec![])?;
            account_cell::verify_account_capacity_not_decrease(input_account_cells[0], output_account_cells[0])?;
            account_cell::verify_account_witness_consistent(
                input_account_cells[0],
                output_account_cells[0],
                &input_account_cell_witness_reader,
                &output_account_cell_witness_reader,
                vec!["status"],
            )?;

            // If a user willing to buy the account, the AccountCell should be in AccountStatus::Selling status.
            account_cell::verify_account_cell_status_update_correctly(
                &input_account_cell_witness_reader,
                &output_account_cell_witness_reader,
                AccountStatus::Selling,
                AccountStatus::Normal,
            )?;

            debug!("Verify if the AccountSaleCell is belong to the AccountCell.");

            let input_sale_cell_witness;
            let input_sale_cell_witness_reader;
            parse_witness!(
                input_sale_cell_witness,
                input_sale_cell_witness_reader,
                parser,
                input_sale_cells[0],
                Source::Input,
                AccountSaleCellData
            );

            verify_sale_cell_account_and_id(input_account_cells[0], input_sale_cell_witness_reader)?;
            // The cell carry refund capacity should be combined with the cell carry profit capacity, so skip checking refund here.
            // verify_refund_correctly(config_main, config_secondary_market, input_sale_cells[0])?;

            debug!("Verify if the AccountCell.lock is changed to new owner's lock properly.");

            let balance_cell_type_id = config_main.type_id_table().balance_cell();
            let balance_cells = util::find_cells_by_type_id(ScriptType::Type, balance_cell_type_id, Source::Input)?;

            assert!(
                balance_cells.len() > 0,
                Error::InvalidTransactionStructure,
                "There should be some BalanceCell in inputs to pay for the deal, but none found."
            );

            let new_owner_lock = high_level::load_cell_lock(balance_cells[0], Source::Input).map_err(Error::from)?;
            let output_account_cell_lock =
                high_level::load_cell_lock(output_account_cells[0], Source::Output).map_err(Error::from)?;

            assert!(
                util::is_entity_eq(&new_owner_lock, &output_account_cell_lock),
                Error::AccountSaleCellNewOwnerError,
                "The new owner's lock of AccountCell is mismatch with the BalanceCell in inputs.(expected: {}, current: {})",
                new_owner_lock,
                output_account_cell_lock
            );

            debug!("Verify if the changes for buyer is correctly");

            let mut paied_capacity = 0;
            for i in balance_cells {
                let lock = high_level::load_cell_lock(i, Source::Input)?;
                if util::is_entity_eq(&lock, &new_owner_lock) {
                    paied_capacity += high_level::load_cell_capacity(i, Source::Input).map_err(Error::from)?;
                }
            }

            let change_cells = util::find_cells_by_type_id_and_filter(
                ScriptType::Type,
                balance_cell_type_id,
                Source::Output,
                |i, source| {
                    let lock = high_level::load_cell_lock(i, source)?;
                    Ok(util::is_entity_eq(&lock, &new_owner_lock))
                },
            )?;
            let mut change_capacity = 0;
            for i in change_cells {
                change_capacity += high_level::load_cell_capacity(i, Source::Output).map_err(Error::from)?;
            }

            let price = u64::from(input_sale_cell_witness_reader.price());
            assert!(
                paied_capacity >= price,
                Error::AccountSaleCellNotPayEnough,
                "The buyer not pay enough to buy the account.(expected: {}, current: {})",
                price,
                paied_capacity
            );

            let expected_change_capacity = paied_capacity - price;
            assert!(
                change_capacity == expected_change_capacity,
                Error::AccountSaleCellChangeError,
                "The buyer({}) has paied {} shannon and the price is {} shannon, so the change should be {} shannon.(expected: {}, current: {})",
                new_owner_lock.as_reader().args(),
                paied_capacity,
                price,
                expected_change_capacity,
                expected_change_capacity,
                change_capacity
            );

            debug!("Verify if the profit is distribute correctly.");

            let seller_lock = util::derive_owner_lock_from_cell(input_account_cells[0], Source::Input)?;
            let (inviter_lock, channel_lock) = decode_scripts_from_params(params)?;
            let account_sale_cell_capacity =
                high_level::load_cell_capacity(input_sale_cells[0], Source::Input).map_err(Error::from)?;
            let common_fee = u64::from(config_secondary_market.common_fee());

            verify_profit_distribution(
                &parser,
                config_main,
                config_profit_rate,
                seller_lock.as_reader(),
                inviter_lock.as_reader(),
                channel_lock.as_reader(),
                price,
                account_sale_cell_capacity,
                common_fee,
            )?;
        }
    } else if action == b"edit_account_sale" {
        parser.parse_config(&[DataType::ConfigCellSecondaryMarket])?;
        parser.parse_cell()?;

        verify_eip712_hashes(&parser, action_raw.as_reader(), &params)?;

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

        let input_cell_witness;
        let input_cell_witness_reader;
        parse_witness!(
            input_cell_witness,
            input_cell_witness_reader,
            parser,
            input_cells[0],
            Source::Input,
            AccountSaleCellData
        );

        let output_cell_witness;
        let output_cell_witness_reader;
        parse_witness!(
            output_cell_witness,
            output_cell_witness_reader,
            parser,
            output_cells[0],
            Source::Output,
            AccountSaleCellData
        );

        verify_account_sale_cell_consistent(
            input_cells[0],
            output_cells[0],
            input_cell_witness_reader,
            output_cell_witness_reader,
        )?;

        verify_tx_fee_spent_correctly(config_secondary_market_reader, input_cells[0], output_cells[0])?;

        let mut changed = false;

        let input_sale_price = u64::from(input_cell_witness_reader.price());
        let output_sale_price = u64::from(output_cell_witness_reader.price());
        if input_sale_price != output_sale_price {
            debug!("Sale price has been changed, verify if it higher than ConfigCellSecondaryMarket.sale_min_price.");
            verify_price(config_secondary_market_reader, output_cell_witness_reader)?;
            changed = true;
        }

        let input_description = input_cell_witness_reader.description();
        let output_description = output_cell_witness_reader.description();
        if !util::is_reader_eq(input_description, output_description) {
            debug!("Description has been changed, verify if its size is less than ConfigCellSecondaryMarket.sale_description_bytes_limit.");
            verify_description(config_secondary_market_reader, output_cell_witness_reader)?;
            changed = true;
        }

        assert!(
            changed,
            Error::InvalidTransactionStructure,
            "Either price or description should be modified."
        );
    } else if action == b"force_recover_account_status" {
        util::require_type_script(
            &mut parser,
            TypeScript::AccountCellType,
            Source::Input,
            Error::InvalidTransactionStructure,
        )?;
    } else {
        return Err(Error::ActionNotSupported);
    }
    Ok(())
}

fn decode_scripts_from_params(params: Vec<BytesReader>) -> Result<(ckb_packed::Script, ckb_packed::Script), Error> {
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

    let inviter_lock = decode_script!(params[0], "inviter_lock");
    let channel_lock = decode_script!(params[1], "channel_lock");

    Ok((inviter_lock, channel_lock))
}

fn load_account_cells(config_main: ConfigCellMainReader) -> Result<(Vec<usize>, Vec<usize>), Error> {
    let account_cell_type_id = config_main.type_id_table().account_cell();
    let (input_account_cells, output_account_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, account_cell_type_id)?;
    Ok((input_account_cells, output_account_cells))
}

fn verify_account_cell_consistent_except_status<'a>(
    config_account: ConfigCellAccountReader,
    timestamp: u64,
    input_account_cell: usize,
    output_account_cell: usize,
    input_account_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    output_account_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
) -> Result<(), Error> {
    account_cell::verify_account_expiration(config_account, input_account_cell, timestamp)?;
    account_cell::verify_account_lock_consistent(input_account_cell, output_account_cell, None)?;
    account_cell::verify_account_data_consistent(input_account_cell, output_account_cell, vec![])?;
    account_cell::verify_account_capacity_not_decrease(input_account_cell, output_account_cell)?;
    account_cell::verify_account_witness_consistent(
        input_account_cell,
        output_account_cell,
        &input_account_cell_witness_reader,
        &output_account_cell_witness_reader,
        vec!["status"],
    )?;

    Ok(())
}

fn verify_sale_cell_capacity(config_reader: ConfigCellSecondaryMarketReader, output_cell: usize) -> Result<(), Error> {
    let capacity = high_level::load_cell_capacity(output_cell, Source::Output).map_err(Error::from)?;
    let expected = u64::from(config_reader.sale_cell_basic_capacity())
        + u64::from(config_reader.sale_cell_prepared_fee_capacity());

    assert!(
        capacity == expected,
        Error::AccountSaleCellCapacityError,
        "The AccountSaleCell.capacity should be equal to {} .",
        expected
    );

    Ok(())
}

fn verify_sale_cell_account_and_id(
    input_account_cell: usize,
    account_sale_cell_witness_reader: AccountSaleCellDataReader,
) -> Result<(), Error> {
    let input_account_cell_data = util::load_cell_data(input_account_cell, Source::Input)?;
    let account_cell_account = data_parser::account_cell::get_account(&input_account_cell_data);
    let account_cell_account_id = data_parser::account_cell::get_id(&input_account_cell_data);

    // read account_id from AccountSaleCell's witness
    let account_sale_cell_account_id = account_sale_cell_witness_reader.account_id().raw_data();
    // ensure the AccountSaleCell's args equal to accountCell's id
    assert!(
        account_cell_account_id == account_sale_cell_account_id,
        Error::AccountSaleCellAccountIdInvalid,
        "The AccountSaleCell.witness.account_id should be equal to the AccountCell.data.account_id ."
    );

    // read account from AccountSaleCell's witness
    let account_sale_cell_account = account_sale_cell_witness_reader.account().raw_data();
    // ensure the AccountSaleCell's args equal to accountCell's id
    assert!(
        account_cell_account == account_sale_cell_account,
        Error::AccountSaleCellAccountIdInvalid,
        "The AccountSaleCell.witness.account should be equal to the AccountCell.data.account ."
    );

    Ok(())
}

fn verify_price(
    config_reader: ConfigCellSecondaryMarketReader,
    witness_reader: AccountSaleCellDataReader,
) -> Result<(), Error> {
    let sale_min_price = u64::from(config_reader.sale_min_price());
    let price = u64::from(witness_reader.price());
    assert!(
        price >= sale_min_price,
        Error::AccountSaleCellPriceTooSmall,
        "The price of account should be higher than ConfigCellSecondaryMarket.sale_min_price.(expected: >= {}, current: {})",
        sale_min_price,
        price
    );

    Ok(())
}

fn verify_description(
    config_reader: ConfigCellSecondaryMarketReader,
    witness_reader: AccountSaleCellDataReader,
) -> Result<(), Error> {
    let bytes_limit = u32::from(config_reader.sale_description_bytes_limit());
    let description = witness_reader.description();
    assert!(
        description.len() <= bytes_limit as usize,
        Error::AccountSaleCellDescriptionTooLarge,
        "The size of description in bytes should be less than ConfigCellSecondaryMarket.sale_description_bytes_limit.(expected: <= {}, current: {})",
        bytes_limit,
        description.len()
    );

    Ok(())
}

fn verify_started_at(current_timestamp: u64, witness_reader: AccountSaleCellDataReader) -> Result<(), Error> {
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

fn verify_account_sale_cell_consistent(
    input_cell: usize,
    output_cell: usize,
    input_cell_witness_reader: AccountSaleCellDataReader,
    output_cell_witness_reader: AccountSaleCellDataReader,
) -> Result<(), Error> {
    debug!("Verify if AccountSaleCell consistent in inputs and outputs.");

    let input_lock_hash = high_level::load_cell_lock_hash(input_cell, Source::Input).map_err(Error::from)?;
    let output_lock_hash = high_level::load_cell_lock_hash(output_cell, Source::Output).map_err(Error::from)?;
    assert!(
        input_lock_hash == output_lock_hash,
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

fn verify_tx_fee_spent_correctly(
    config_reader: ConfigCellSecondaryMarketReader,
    input_cell: usize,
    output_cell: usize,
) -> Result<(), Error> {
    debug!("Verify if AccountSaleCell paid fee correctly.");

    let basic_capacity = u64::from(config_reader.sale_cell_basic_capacity());
    let input_capacity = high_level::load_cell_capacity(input_cell, Source::Input).map_err(Error::from)?;
    let output_capacity = high_level::load_cell_capacity(output_cell, Source::Output).map_err(Error::from)?;

    if input_capacity > output_capacity {
        assert!(
            output_capacity >= basic_capacity,
            Error::AccountSaleCellFeeError,
            "The AccountSaleCell has no more capacity as fee for this transaction.(current_capacity: {}, min_capacity: {})",
            input_capacity,
            basic_capacity
        );

        let expected_fee = u64::from(config_reader.common_fee());
        assert!(
            output_capacity >= input_capacity - expected_fee,
            Error::AccountSaleCellFeeError,
            "The fee should be equal to or less than ConfigCellSecondaryMarket.common_fee .(expected: {}, result: {})",
            expected_fee,
            input_capacity - output_capacity
        );
    }

    Ok(())
}

fn verify_refund_correctly(
    config_main: ConfigCellMainReader,
    config_secondary_market: ConfigCellSecondaryMarketReader,
    input_sale_cell: usize,
) -> Result<(), Error> {
    debug!("Verify if the AccountSaleCell has been refund correctly.");

    let balance_cell_type_id = config_main.type_id_table().balance_cell();
    let refund_cells = util::find_cells_by_type_id(ScriptType::Type, balance_cell_type_id, Source::Output)?;
    assert!(
        refund_cells.len() == 1,
        Error::AccountSaleCellRefundError,
        "There should only 1 cell used to refund, but {} found.",
        refund_cells.len()
    );

    let refund_lock = high_level::load_cell_lock(refund_cells[0], Source::Output).map_err(Error::from)?;
    // Build expected refund lock.
    let expected_refund_lock = util::derive_owner_lock_from_cell(input_sale_cell, Source::Input)?;
    assert!(
        util::is_entity_eq(&refund_lock, &expected_refund_lock),
        Error::AccountSaleCellRefundError,
        "The NormalCell for refunding should have the owner's lock script.(expected: {}, current: {})",
        expected_refund_lock,
        refund_lock
    );

    let input_capacity = high_level::load_cell_capacity(input_sale_cell, Source::Input).map_err(Error::from)?;
    let refund_capacity = high_level::load_cell_capacity(refund_cells[0], Source::Output).map_err(Error::from)?;
    let expected_fee = u64::from(config_secondary_market.common_fee());
    assert!(
        refund_capacity >= input_capacity - expected_fee,
        Error::AccountSaleCellRefundError,
        "The refund should be equal to or more than {} shannon, but {} shannon found.",
        refund_capacity,
        input_capacity
    );

    Ok(())
}

fn verify_profit_distribution(
    parser: &WitnessesParser,
    config_main: ConfigCellMainReader,
    config_profit_rate: ConfigCellProfitRateReader,
    seller_lock_reader: ckb_packed::ScriptReader,
    inviter_lock_reader: ckb_packed::ScriptReader,
    channel_lock_reader: ckb_packed::ScriptReader,
    price: u64,
    account_sale_cell_capacity: u64,
    common_fee: u64,
) -> Result<(), Error> {
    let income_cell_basic_capacity = u64::from(parser.configs.income()?.basic_capacity());

    let default_script = ckb_packed::Script::default();
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

    util::is_cell_use_always_success_lock(output_income_cells[0], Source::Output)?;

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

    let balance_cell_type_id = config_main.type_id_table().balance_cell();
    let balance_cell_type = util::type_id_to_script(balance_cell_type_id);
    let das_lock_cells = find_cells_by_script(ScriptType::Lock, seller_lock_reader, Source::Output)?;
    let mut seller_balance_cells = Vec::new();
    for i in das_lock_cells {
        let type_script_opt = high_level::load_cell_type(i, Source::Output).map_err(Error::from)?;
        if let Some(type_script) = type_script_opt {
            if util::is_type_id_equal(balance_cell_type.as_reader().into(), type_script.as_reader()) {
                seller_balance_cells.push(i);
            }
        }
    }
    assert!(
        seller_balance_cells.len() == 1,
        Error::AccountSaleCellProfitError,
        "There should only 1 cell used to carry profit and refund, but {} found.",
        seller_balance_cells.len()
    );

    let seller_balance_cell_capacity =
        high_level::load_cell_capacity(seller_balance_cells[0], Source::Output).map_err(Error::from)?;
    let expected_capacity = profit_of_seller + account_sale_cell_capacity - common_fee;
    assert!(
        seller_balance_cell_capacity >= expected_capacity,
        Error::AccountSaleCellProfitError,
        "The capacity of seller's NormalCell should be equal to or more than {} shannon, but {} shannon found.(profit: {}, refund: {}, common_fee: {})",
        expected_capacity,
        seller_balance_cell_capacity,
        profit_of_seller,
        account_sale_cell_capacity,
        common_fee
    );

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

    #[cfg(any(not(feature = "mainnet"), debug_assertions))]
    inspect::income_cell(
        Source::Output,
        output_income_cells[0],
        None,
        Some(output_income_cell_witness_reader),
    );

    let total_profit_in_income = price - profit_of_seller;
    let skip = if total_profit_in_income > income_cell_basic_capacity {
        false
    } else {
        // If the profit is sufficient for IncomeCell's basic capacity skip the first record, because it is a convention that the first
        // always belong to the IncomeCell creator in this transaction.
        true
    };
    for (i, record) in output_income_cell_witness_reader.records().iter().enumerate() {
        if skip && i == 0 {
            continue;
        }

        let key = record.belong_to().as_slice().to_vec();
        let recorded_capacity = u64::from(record.capacity());
        let result = profit_map.get(&key);

        // This will allow creating IncomeCell will NormalCells in inputs.
        if result.is_none() {
            continue;
        }

        let expected_capacity = result.unwrap();
        assert!(
            &recorded_capacity == expected_capacity,
            Error::AccountSaleCellProfitError,
            "IncomeCell.records[{}] The capacity of a profit record is incorrect. (expected: {}, current: {}, belong_to: {})",
            i,
            expected_capacity,
            recorded_capacity,
            record.belong_to()
        );

        profit_map.remove(&key);
    }

    assert!(
        profit_map.is_empty(),
        Error::AccountSaleCellProfitError,
        "The IncomeCell in outputs should contains everyone's profit. (missing: {})",
        profit_map.len()
    );

    let mut expected_income_cell_capacity = 0;
    for record in output_income_cell_witness_reader.records().iter() {
        expected_income_cell_capacity += u64::from(record.capacity());
    }

    let current_capacity =
        high_level::load_cell_capacity(output_income_cells[0], Source::Output).map_err(Error::from)?;
    assert!(
        current_capacity >= income_cell_basic_capacity,
        Error::InvalidTransactionStructure,
        "The IncomeCell should have capacity bigger than or equal to the value in ConfigCellIncome.basic_capacity."
    );
    assert!(
        current_capacity == expected_income_cell_capacity,
        Error::AccountSaleCellProfitError,
        "The capacity of the IncomeCell should be {} shannon, but {} shannon found.",
        expected_income_cell_capacity,
        current_capacity
    );

    Ok(())
}
