use alloc::{boxed::Box, vec, vec::Vec};
use ckb_std::high_level::{load_cell_capacity, load_cell_type};
use ckb_std::{ckb_constants::Source, ckb_types::prelude::*, debug, high_level};
use core::mem::{size_of, MaybeUninit};
use das_core::{
    assert, constants::*, data_parser, error::Error, parse_account_cell_witness, parse_witness, util, warn,
    witness_parser::WitnessesParser,
};
use das_map::{map::Map, util as map_util};
use das_types::{
    constants::{AccountStatus, DataType, LockRole},
    mixer::*,
    packed::*,
    prelude::*,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running account-sale-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    util::is_system_off(&mut parser)?;

    let action_data = parser.parse_action()?;
    let action = action_data.as_reader().action().raw_data();
    let params = action_data.as_reader().params().raw_data();
    if action == b"sell_account"
        || action == b"cancel_account_sale"
        || action == b"buy_account"
        || action == b"edit_account_sale"
    {
        let timestamp = util::load_oracle_data(OracleCellType::Time)?;
        parser.parse_cell()?;
        let config_main = parser.configs.main()?;
        let secondary_market = parser.configs.secondary_market()?;
        let (input_sale_cells, output_sale_cells) = load_on_sale_cells()?;

        if action == b"sell_account" {
            // todo 将 accountCell 的校验，放这里
            debug!("Route to sell_account action ...");
            assert!(
                input_sale_cells.len() == 0 && output_sale_cells.len() == 1,
                Error::InvalidTransactionStructure,
                "There should be zero AccountSaleCell int input and one AccountSaleCell in output."
            );

            let output_cell_witness;
            let output_cell_witness_reader;
            parse_witness!(
                output_cell_witness,
                output_cell_witness_reader,
                parser,
                output_sale_cells[0],
                Source::Output,
                AccountSaleCellData
            );
            verify_sale_cell_account_id(config_main, output_cell_witness_reader)?;

            // todo 验证 das-lock 一致

            let sale_started_at = output_cell_witness_reader.started_at() as u64;
            // beginning time need to equal to timeCell's value
            assert!(
                sale_started_at == timestamp,
                Error::AccountSaleCellStartedAtInvalid,
                "The AccountSaleCell's started_at should be the same as the timestamp in the TimeCell.(expected: {}, current: {})",
                timestamp,
                sale_started_at
            );
            // todo desc bytes limit
            // check price
            let sale_price = output_cell_witness_reader.price() as u64;
            let min_price = secondary_market.min_sale_price() as u64;
            assert!(
                sale_price >= min_price,
                Error::AccountSaleCellPriceTooSmall,
                "The AccountSaleCell's price too small.(expected: >= {}, current: {})",
                min_price,
                sale_price
            );
        } else if action == b"cancel_account_sale" {
            debug!("Route to cancel_account_sale action ...");
            assert!(
                input_sale_cells.len() == 1 && output_sale_cells.len() == 0,
                Error::InvalidTransactionStructure,
                "There should be zero AccountSaleCell in output and one AccountSaleCell in input."
            );
            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            let input_cell_witness;
            let input_cell_witness_reader;
            parse_witness!(
                input_cell_witness,
                input_cell_witness_reader,
                parser,
                input_sale_cells[0],
                Source::Input,
                AccountSaleCellData
            );
            verify_sale_cell_account_id(config_main, input_cell_witness_reader)?;
        } else if action == b"buy_account" {
            debug!("Route to buy_account action ...");

            assert!(
                input_sale_cells.len() == 1 && output_sale_cells.len() == 0,
                Error::InvalidTransactionStructure,
                "There should be zero AccountSaleCell in output and one AccountSaleCell in input."
            );
            let config_main = parser.configs.main()?;
            let config_profit_rate = parser.configs.profit_rate()?;
            let input_cell_witness;
            let input_cell_witness_reader;
            parse_witness!(
                input_cell_witness,
                input_cell_witness_reader,
                parser,
                input_sale_cells[0],
                Source::Input,
                AccountSaleCellData
            );
            verify_sale_cell_account_id(config_main, input_cell_witness_reader)?;

            // verify price capacity todo remove?
            let buy_spent_total_capacity = verify_buy_account_price_capacity(input_cell_witness_reader)?;

            // verify income_cell profit
            let mut seller_should_except_profit = verify_buy_account_income_cell_profit(
                params,
                buy_spent_total_capacity,
                &mut parser,
                config_main,
                config_profit_rate,
            )?;

            /**
            verify seller's income, including two part:
            1. sale account income; (seller_should_except_profit)
            2. account_cell's refund
            */
            let input_account_sale_cell_capacity =
                load_cell_capacity(input_sale_cells[0], Source::Input).map_err(|e| Error::from(e))?;
            seller_should_except_profit = seller_should_except_profit + input_account_sale_cell_capacity;
            verify_buy_account_seller_income(seller_should_except_profit, config_main)?;
        } else if action == b"edit_account_sale" {
            debug!("Route to edit_account_sale action ...");
            assert!(
                input_sale_cells.len() == 1 && output_sale_cells.len() == 1,
                Error::AccountSaleCellNumberInvalid,
                "There should be one AccountSaleCell in output and one AccountSaleCell in input."
            );
            let input_cell_witness;
            let input_cell_witness_reader;
            parse_witness!(
                input_cell_witness,
                input_cell_witness_reader,
                parser,
                input_sale_cells[0],
                Source::Input,
                AccountSaleCellData
            );

            let output_cell_witness;
            let output_cell_witness_reader;
            parse_witness!(
                output_cell_witness,
                output_cell_witness_reader,
                parser,
                output_sale_cells[0],
                Source::Output,
                AccountSaleCellData
            );
            verify_input_output_sale_cell_account_id(
                config_main,
                input_cell_witness_reader,
                output_cell_witness_reader,
            )?;

            let old_sale_price = input_cell_witness_reader.price() as u64;
            let new_sale_price = output_cell_witness_reader.price() as u64;
            let min_price = secondary_market.min_sale_price() as u64;
            assert!(
                min_price != 0 && old_sale_price != new_sale_price && new_sale_price >= min_price,
                Error::AccountSaleCellPriceTooSmall,
                "The AccountSaleCell's price too small, or equal.(min: {}, old: {}, new: {})",
                min_price,
                old_sale_price,
                new_sale_price
            );
        }
    } else {
        return Err(Error::ActionNotSupported);
    }
    Ok(())
}

pub fn unpack_number(slice: &[u8]) -> u32 {
    // the size of slice should be checked before call this function
    #[allow(clippy::uninit_assumed_init)]
    let mut b: [u8; 4] = unsafe { MaybeUninit::uninit().assume_init() };
    b.copy_from_slice(&slice[..4]);
    u32::from_le_bytes(b)
}

fn verify_buy_account_seller_income(
    seller_should_except_profit: u64,
    config_main: ConfigCellMainReader,
) -> Result<(), Error> {
    let account_cell_type_id = config_main.type_id_table().account_cell();
    let input_account_cells = util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, Source::Input)?;
    assert!(
        input_account_cells.len() == 1,
        Error::AccountSaleInputAccountCellNumberInvalid,
        "There should be one AccountCell in input."
    );
    let input_account_cell_lock =
        high_level::load_cell_lock(input_account_cells[0], Source::Input).map_err(|e| Error::from(e))?;
    let input_account_cell_args = input_account_cell_lock.as_reader().args().raw_data();
    let input_account_cell_owner = data_parser::das_lock_args::get_owner_lock_args(input_account_cell_args);
    let mut script = das_lock();
    script = script.as_builder().args(input_account_cell_lock.args()).build();
    // find out all the normal das-lock cell
    let das_lock_output_cells = util::find_only_lock_cell_by_script(script.as_reader(), Source::Output)?;
    let mut i: usize = 0;
    let das_lock_output_cells_len = das_lock_output_cells.len();
    let mut seller_current_capacity: u64 = 0;
    // todo 强制一个 das-lock normal cell
    while i < das_lock_output_cells_len {
        let das_lock =
            high_level::load_cell_lock(das_lock_output_cells[i], Source::Output).map_err(|e| Error::from(e))?;
        let das_lock_args = das_lock.as_reader().args().raw_data();
        let output_das_lock_owner = data_parser::das_lock_args::get_owner_lock_args(das_lock_args);
        //  calculate the seller's 'normal das-lock cell' total capacity
        let das_lock_output_cell_capacity =
            load_cell_capacity(das_lock_output_cells[i], Source::Output).map_err(|e| Error::from(e))?;
        seller_current_capacity = seller_current_capacity + das_lock_output_cell_capacity;
        i += 1;
    }
    assert!(
        seller_should_except_profit != 0 && seller_should_except_profit == seller_current_capacity,
        Error::AccountSaleOutputDasLockCellInvalid,
        "The seller account_cell's total capacity not enough.(expect: >= {}, current: {})",
        seller_should_except_profit,
        seller_expect_capacity,
    );
    Ok(())
}

fn verify_buy_account_income_cell_profit(
    params: &[u8],
    buy_spent_total_capacity: u64,
    parser: &mut WitnessesParser,
    config_main: ConfigCellMainReader,
    config_profit_rate: ConfigCellProfitRateReader,
) -> Result<(u64), Error> {
    let min_param_bytes_len = 106; // min bytes len, two default scripts
    assert!(
        params.len() >= min_param_bytes_len,
        Error::AccountSaleCellBuyParamInvalid,
        "The params invalid. (len expected: >= {}, current: {})",
        min_param_bytes_len,
        params.len()
    );
    let inviter_script_bytes_total_len = unpack_number(&params[..4]) as usize;
    let inviter_script =
        Script::from_slice(&params[..inviter_script_bytes_total_len]).map_err(|_| Error::WitnessActionDecodingError)?;
    let channel_script =
        Script::from_slice(&params[inviter_script_bytes_total_len..]).map_err(|_| Error::WitnessActionDecodingError)?;
    let mut profit_map = Map::new();
    let mut leftover: u64 = 0;
    if !inviter_script.args().is_empty() {
        let inviter_profit_rate = u32::from(config_profit_rate.sale_inviter()) as u64;
        let inviter_profit = inviter_profit_rate * (buy_spent_total_capacity / RATE_BASE);
        if inviter_profit != 0 {
            map_util::add(&mut profit_map, inviter_script.as_slice().to_vec(), inviter_profit);
        }
    }
    if !channel_script.args().is_empty() {
        let channel_profit_rate = u32::from(config_profit_rate.channel()) as u64;
        let channel_profit = channel_profit_rate * (buy_spent_total_capacity / RATE_BASE);
        if channel_profit != 0 {
            map_util::add(&mut profit_map, channel_script.as_slice().to_vec(), channel_profit);
        }
    }
    if profit_map.is_empty() {
        leftover = buy_spent_total_capacity;
    } else {
        let income_cell_type_id = config_main.type_id_table().income_cell();
        // todo support input income_cell
        let output_income_cells = util::find_cells_by_type_id(ScriptType::Type, income_cell_type_id, Source::Output)?;
        assert!(
            output_income_cells.len() == 1,
            Error::AccountSaleCellIncomeCellMissing,
            "The number of IncomeCells in outputs should be exactly 1 . (expected: == 1, current: {})",
            output_income_cells.len()
        );
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
        let mut income_cell_expected_capacity = 0;
        for (i, record) in output_income_cell_witness_reader.records().iter().enumerate() {
            let key = record.belong_to().as_slice().to_vec();
            let recorded_profit = u64::from(record.capacity());
            let result = profit_map.get(&key);
            assert!(
                result.is_some(),
                Error::AccountSaleCellIncomeCellRecordsInvalid,
                "  IncomeCell.records[{}] is invalid, inviter or channel is missing. (belong_to: {})",
                i,
                record.belong_to()
            );
            let expected_profit = result.unwrap();
            assert!(
                &recorded_profit == expected_profit,
                Error::AccountSaleCellIncomeCellRecordsProfitErr,
                "  IncomeCell.records[{}] The capacity of a profit record is incorrect. (expected: {}, current: {}, belong_to: {})",
                i,
                expected_profit,
                recorded_profit,
                record.belong_to()
            );
            profit_map.remove(&key);
            income_cell_expected_capacity += recorded_profit;
        }
        assert!(
            profit_map.is_empty(),
            Error::InvalidTransactionStructure,
            "profit_map is not empty"
        );
        let income_cell_current_capacity =
            load_cell_capacity(output_income_cells[0], Source::Output).map_err(|e| Error::from(e))?;
        assert!(
            income_cell_expected_capacity != 0 && income_cell_expected_capacity == income_cell_current_capacity,
            Error::AccountSaleCellIncomeCellTotalCapacityErr,
            "The capacity of the IncomeCell should be {}, but {} found.",
            income_cell_expected_capacity,
            income_cell_current_capacity
        );
        leftover = buy_spent_total_capacity - income_cell_expected_capacity;
    }
    Ok((leftover))
}

fn verify_buy_account_price_capacity(input_cell_witness_reader: AccountSaleCellDataReader) -> Result<(u64), Error> {
    let account_price = input_cell_witness_reader.price() as u64;
    let fee_das_lock_cells = util::find_only_lock_cell_by_script(das_lock().as_reader(), Source::Input)?;
    let fee_das_lock_cells_len = fee_das_lock_cells.len();
    let mut i: usize = 0;
    let mut buy_spent_total_capacity: u64 = 0;
    while i < fee_das_lock_cells_len {
        let capacity = load_cell_capacity(fee_das_lock_cells[i], Source::Input).map_err(|e| Error::from(e))?;
        buy_spent_total_capacity = buy_spent_total_capacity + capacity;
        i += 1;
    }
    assert!(
        account_price != 0 && account_price == total_capacity,
        Error::AccountSalePriceNotEqual,
        "Buy account price not equal. (expected: >= {}, current: {})",
        account_price,
        total_capacity
    );
    Ok(buy_spent_total_capacity)
}

fn verify_sale_cell_account_id(
    config_main: ConfigCellMainReader,
    account_sale_cell_witness_reader: AccountSaleCellDataReader,
) -> Result<(), Error> {
    let account_cell_type_id = config_main.type_id_table().account_cell();
    let output_account_cells = util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, Source::Output)?;

    assert!(
        output_account_cells.len() == 1,
        Error::InvalidTransactionStructure,
        "Output must include one account_cell"
    );

    // read account_id from output accountCell
    let output_account_cell_index = output_account_cells[0];
    let output_data = util::load_cell_data(output_account_cell_index, Source::Output)?;
    let account_cell_account_id = data_parser::account_cell::get_id(&output_data);

    // read account_id from AccountSaleCell's witness
    let account_sale_cell_account_id = account_sale_cell_witness_reader.account_id().raw_data();

    // ensure the AccountSaleCell's args equal to accountCell's id
    assert!(
        account_cell_account_id == account_sale_cell_account_id,
        Error::AccountSaleCellAccountIdInvalid,
        "AccountSaleCell's accountId should equal to the accountCell"
    );
    Ok(())
}

fn verify_input_output_sale_cell_account_id(
    config_main: ConfigCellMainReader,
    account_sale_input_cell_witness: AccountSaleCellDataReader,
    account_sale_output_cell_witness: AccountSaleCellDataReader,
) -> Result<(), Error> {
    let account_cell_type_id = config_main.type_id_table().account_cell();
    let output_account_cells = util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, Source::Output)?;

    assert!(
        output_account_cells.len() == 1,
        Error::AccountSaleCellAccountCellMustOne,
        "Output must include one account_cell"
    );

    // read account_id from output accountCell
    let output_account_cell_index = output_account_cells[0];
    let output_data = util::load_cell_data(output_account_cell_index, Source::Output)?;
    let account_id = data_parser::account_cell::get_id(&output_data);

    // read account_id from AccountSaleCell's witness
    let account_id_2 = account_sale_input_cell_witness.account_id().raw_data();
    let account_id_3 = account_sale_output_cell_witness.account_id().raw_data();

    // ensure the AccountSaleCell's args equal to accountCell's id
    assert!(
        account_id == account_id_2 && account_id == account_id_3,
        Error::AccountSaleCellAccountIdInvalid,
        "AccountSaleCell's accountId should equal to the accountCell.accountId: {},input: {},output: {}",
        account_id,
        account_id_2,
        account_id_3
    );
    Ok(())
}

fn load_on_sale_cells() -> Result<(Vec<usize>, Vec<usize>), Error> {
    let this_type_script = high_level::load_script().map_err(|e| Error::from(e))?;
    let (input_on_sale_cells, output_on_sale_cells) =
        util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, this_type_script.as_reader())?;
    Ok((input_on_sale_cells, output_on_sale_cells))
}
