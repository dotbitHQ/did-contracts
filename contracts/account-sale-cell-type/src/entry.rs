use alloc::{boxed::Box, vec, vec::Vec};
use ckb_std::high_level::load_cell_capacity;
use ckb_std::{ckb_constants::Source, ckb_types::prelude::*, debug, high_level};
use core::mem::MaybeUninit;
use das_core::{
    assert, constants::*, data_parser, error::Error, parse_account_cell_witness, parse_witness, util, warn,
    witness_parser::WitnessesParser,
};
use das_map::{map::Map, util as map_util};
use das_types::{
    constants::{AccountStatus, DataType, LockRole},
    mixer::*,
    packed::*,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running account-sale-cell-type ======");
    let mut parser = WitnessesParser::new()?;
    let action_data = parser.parse_action()?;
    let action = action_data.as_reader().action().raw_data();
    let params = action_data.as_reader().params().raw_data();
    if action == b"sell_account"
        || action == b"cancel_account_sale"
        || action == b"buy_account"
        || action == b"edit_account_sale"
    {
        util::is_system_off(&mut parser)?;
        parser.parse_cell()?;

        let timestamp = util::load_oracle_data(OracleCellType::Time)?;
        parser.parse_config(&[
            DataType::ConfigCellMain,
            DataType::ConfigCellAccount,
            DataType::ConfigCellProfitRate,
            DataType::ConfigCellSecondaryMarket,
        ])?;
        let config_main = parser.configs.main()?;
        let config_account = parser.configs.account()?;
        let secondary_market = parser.configs.secondary_market()?;

        let (input_sale_cells, output_sale_cells) = load_on_sale_cells()?;

        if action == b"sell_account" {
            debug!("Route to sell_account action ...");
            assert!(
                input_sale_cells.len() == 0 && output_sale_cells.len() == 1,
                Error::InvalidTransactionStructure,
                "There should be zero AccountSaleCell int input and one AccountSaleCell in output."
            );
            account_cell_common_verify(
                &parser,
                config_account,
                timestamp,
                params,
                true,
                true,
                false,
                None,
                vec!["status"],
                output_sale_cells[0],
            )?;

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
            verify_account_sale_cell_field(timestamp, output_cell_witness_reader, secondary_market)?;
        } else if action == b"cancel_account_sale" {
            debug!("Route to cancel_account_sale action ...");
            assert!(
                input_sale_cells.len() == 1 && output_sale_cells.len() == 0,
                Error::InvalidTransactionStructure,
                "There should be zero AccountSaleCell in output and one AccountSaleCell in input."
            );

            account_cell_common_verify(
                &parser,
                config_account,
                timestamp,
                params,
                true,
                false,
                true,
                None,
                vec!["status"],
                0,
            )?;

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

            let config_profit_rate = parser.configs.profit_rate()?;

            account_cell_common_verify(
                &parser,
                config_account,
                timestamp,
                params,
                false,
                false,
                true,
                Some("owner"),
                vec!["status"],
                0,
            )?;

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

            // verify income_cell profit
            let account_price = u64::from(input_cell_witness_reader.price());
            let mut seller_should_except_profit =
                verify_buy_account_income_cell_profit(params, account_price, &parser, config_main, config_profit_rate)?;

            // verify seller's income, including two part:
            // 1. sale account income; (seller_should_except_profit)
            // 2. account_cell's refund
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
            verify_input_output_sale_cell_account_id_and_lock_args(
                input_sale_cells[0],
                output_sale_cells[0],
                input_cell_witness_reader,
                output_cell_witness_reader,
            )?;

            let old_sale_price = u64::from(input_cell_witness_reader.price());
            let new_sale_price = u64::from(output_cell_witness_reader.price());
            assert!(
                old_sale_price != new_sale_price,
                Error::AccountSaleEditPriceEqualError,
                "The new AccountSaleCell's price equal to the old one.(old: {}, new: {})",
                old_sale_price,
                new_sale_price
            );
            verify_account_sale_cell_field(timestamp, output_cell_witness_reader, secondary_market)?;
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
    // let input_account_cell_args = input_account_cell_lock.as_reader().args().raw_data();
    // let input_account_cell_owner = data_parser::das_lock_args::get_owner_lock_args(input_account_cell_args);
    let script = das_lock().as_builder().args(input_account_cell_lock.args()).build();
    // find out all the normal das-lock cell
    let das_lock_output_cells = util::find_only_lock_cell_by_script(script.as_reader(), Source::Output)?;
    let mut i: usize = 0;
    let das_lock_output_cells_len = das_lock_output_cells.len();
    let mut seller_current_capacity: u64 = 0;
    // todo 强制一个 das-lock normal cell
    while i < das_lock_output_cells_len {
        // let das_lock = high_level::load_cell_lock(das_lock_output_cells[i], Source::Output).map_err(|e| Error::from(e))?;
        // let das_lock_args = das_lock.as_reader().args().raw_data();
        // let output_das_lock_owner = data_parser::das_lock_args::get_owner_lock_args(das_lock_args);
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
        seller_current_capacity
    );
    Ok(())
}

fn verify_buy_account_income_cell_profit(
    params: &[u8],
    buy_spent_total_capacity: u64,
    parser: &WitnessesParser,
    config_main: ConfigCellMainReader,
    config_profit_rate: ConfigCellProfitRateReader,
) -> Result<u64, Error> {
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
    // debug!(
    //     "inviter is {}, channel is {}",
    //     util::hex_string(inviter_script.args().as_slice()),
    //     util::hex_string(channel_script.args().as_slice())
    // );
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
        let channel_profit_rate = u32::from(config_profit_rate.sale_channel()) as u64;
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
    Ok(leftover)
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

fn verify_input_output_sale_cell_account_id_and_lock_args(
    input_sale_cell_index: usize,
    output_sale_cell_index: usize,
    account_sale_input_cell_witness: AccountSaleCellDataReader,
    account_sale_output_cell_witness: AccountSaleCellDataReader,
) -> Result<(), Error> {
    // read account_id from AccountSaleCell's witness
    let account_id_2 = account_sale_input_cell_witness.account_id().raw_data();
    let account_id_3 = account_sale_output_cell_witness.account_id().raw_data();

    // ensure the AccountSaleCell's args equal to accountCell's id
    assert!(
        account_id_2 == account_id_3,
        Error::AccountSaleCellAccountIdInvalid,
        "input_account_sale_cell's accountId should equal to the output_account_sale_cell's accountId,input: {},output: {}",
        util::hex_string(account_id_2),
        util::hex_string(account_id_3)
    );

    let input_sale_cell_lock =
        high_level::load_cell_lock(input_sale_cell_index, Source::Input).map_err(|e| Error::from(e))?;
    let input_sale_cell_args = input_sale_cell_lock.as_reader().args().raw_data();
    let input_sale_cell_owner = data_parser::das_lock_args::get_owner_lock_args(input_sale_cell_args);

    let output_sale_cell_lock =
        high_level::load_cell_lock(output_sale_cell_index, Source::Output).map_err(|e| Error::from(e))?;
    let output_sale_cell_args = output_sale_cell_lock.as_reader().args().raw_data();
    let output_sale_cell_owner = data_parser::das_lock_args::get_owner_lock_args(output_sale_cell_args);

    assert!(
        input_sale_cell_owner == output_sale_cell_owner,
        Error::InvalidTransactionStructure,
        "input_account_sale_cell's owner should equal output_account_sale_cell's owner. (input: {}, output: {})",
        util::hex_string(input_sale_cell_owner),
        util::hex_string(output_sale_cell_owner)
    );

    Ok(())
}

fn load_on_sale_cells() -> Result<(Vec<usize>, Vec<usize>), Error> {
    let this_type_script = high_level::load_script().map_err(|e| Error::from(e))?;
    let (input_on_sale_cells, output_on_sale_cells) =
        util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, this_type_script.as_reader())?;
    Ok((input_on_sale_cells, output_on_sale_cells))
}

fn verify_unlock_role(params: &[u8], lock: LockRole) -> Result<(), Error> {
    debug!("Check if transaction is unlocked by {:?}.", lock);
    assert!(
        params.len() > 0 && params[0] == lock as u8,
        Error::AccountCellPermissionDenied,
        "This transaction should be unlocked by the {:?}'s signature.",
        lock
    );
    Ok(())
}

fn verify_account_expiration(
    config: ConfigCellAccountReader,
    account_cell_index: usize,
    current: u64,
) -> Result<(), Error> {
    debug!("Check if AccountCell is expired.");
    let data = util::load_cell_data(account_cell_index, Source::Input)?;
    let expired_at = data_parser::account_cell::get_expired_at(data.as_slice());
    let expiration_grace_period = u32::from(config.expiration_grace_period()) as u64;
    if current > expired_at {
        if current - expired_at > expiration_grace_period {
            warn!("The AccountCell has been expired. Will be recycled soon.");
            return Err(Error::AccountCellHasExpired);
        } else {
            warn!("The AccountCell has been in expiration grace period. Need to be renew as soon as possible.");
            return Err(Error::AccountCellInExpirationGracePeriod);
        }
    }
    Ok(())
}

fn verify_account_lock_consistent(
    input_account_index: usize,
    output_account_index: usize,
    changed_lock: Option<&str>,
) -> Result<(), Error> {
    debug!("Check if lock consistent in the AccountCell.");
    if let Some(lock) = changed_lock {
        let input_lock = high_level::load_cell_lock(input_account_index, Source::Input).map_err(|e| Error::from(e))?;
        let input_args = input_lock.as_reader().args().raw_data();
        let output_lock =
            high_level::load_cell_lock(output_account_index, Source::Output).map_err(|e| Error::from(e))?;
        let output_args = output_lock.as_reader().args().raw_data();
        if lock == "owner" {
            assert!(
                data_parser::das_lock_args::get_owner_lock_args(input_args)
                    != data_parser::das_lock_args::get_owner_lock_args(output_args),
                Error::AccountCellOwnerLockShouldBeModified,
                "The owner lock args in AccountCell.lock should be different in input and output."
            );

            assert!(
                data_parser::das_lock_args::get_manager_lock_args(output_args)
                    == data_parser::das_lock_args::get_owner_lock_args(output_args),
                Error::AccountCellManagerLockShouldBeModified,
                "The manager lock args in AccountCell.lock should be different in input and output."
            );
        } else {
            assert!(
                data_parser::das_lock_args::get_owner_lock_args(input_args)
                    == data_parser::das_lock_args::get_owner_lock_args(output_args),
                Error::AccountCellOwnerLockShouldNotBeModified,
                "The owner lock args in AccountCell.lock should be consistent in input and output."
            );

            assert!(
                data_parser::das_lock_args::get_manager_lock_args(input_args)
                    != data_parser::das_lock_args::get_manager_lock_args(output_args),
                Error::AccountCellManagerLockShouldBeModified,
                "The manager lock args in AccountCell.lock should be different in input and output."
            );
        }
    } else {
        util::is_cell_lock_equal(
            (input_account_index, Source::Input),
            (output_account_index, Source::Output),
        )?;
    }

    Ok(())
}

fn verify_account_data_consistent(
    input_account_index: usize,
    output_account_index: usize,
    except: Vec<&str>,
) -> Result<(), Error> {
    debug!("Check if AccountCell.data is consistent in input and output.");

    let input_data = util::load_cell_data(input_account_index, Source::Input)?;
    let output_data = util::load_cell_data(output_account_index, Source::Output)?;

    assert!(
        data_parser::account_cell::get_id(&input_data) == data_parser::account_cell::get_id(&output_data),
        Error::AccountCellDataNotConsistent,
        "The data.id field of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    assert!(
        data_parser::account_cell::get_next(&input_data) == data_parser::account_cell::get_next(&output_data),
        Error::AccountCellDataNotConsistent,
        "The data.next field of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    assert!(
        data_parser::account_cell::get_account(&input_data) == data_parser::account_cell::get_account(&output_data),
        Error::AccountCellDataNotConsistent,
        "The data.account field of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    if !except.contains(&"expired_at") {
        assert!(
            data_parser::account_cell::get_expired_at(&input_data)
                == data_parser::account_cell::get_expired_at(&output_data),
            Error::AccountCellDataNotConsistent,
            "The data.expired_at field of inputs[{}] and outputs[{}] should be the same.",
            input_account_index,
            output_account_index
        );
    }

    Ok(())
}

fn verify_account_capacity_not_decrease(input_account_index: usize, output_account_index: usize) -> Result<(), Error> {
    debug!("Check if capacity consistent in the AccountCell.");
    let input = high_level::load_cell_capacity(input_account_index, Source::Input).map_err(|e| Error::from(e))?;
    let output = high_level::load_cell_capacity(output_account_index, Source::Output).map_err(|e| Error::from(e))?;
    // ⚠️ Equal is not allowed here because we want to avoid abuse cell.
    assert!(
        input <= output,
        Error::CellLockCanNotBeModified,
        "The capacity of the AccountCell should be consistent or increased.(input: {}, output: {})",
        input,
        output
    );
    Ok(())
}

fn verify_account_witness_consistent<'a>(
    input_index: usize,
    output_index: usize,
    input_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    output_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    except: Vec<&str>,
) -> Result<(), Error> {
    debug!("Check if AccountCell.witness is consistent in input and output.");

    macro_rules! assert_field_consistent {
        ($input_witness_reader:expr, $output_witness_reader:expr, $( ($field:ident, $field_name:expr) ),*) => {
            $(
                assert!(
                    util::is_reader_eq(
                        $input_witness_reader.$field(),
                        $output_witness_reader.$field()
                    ),
                    Error::AccountCellProtectFieldIsModified,
                    "The witness.{} field of inputs[{}] and outputs[{}] should be the same.",
                    $field_name,
                    input_index,
                    output_index
                );
            )*
        };
    }

    macro_rules! assert_field_consistent_if_not_except {
        ($input_witness_reader:expr, $output_witness_reader:expr, $( ($field:ident, $field_name:expr) ),*) => {
            $(
                if !except.contains(&$field_name) {
                    assert_field_consistent!(
                        $input_witness_reader,
                        $output_witness_reader,
                        ($field, $field_name)
                    );
                }
            )*
        };
    }

    let output_witness_reader = output_witness_reader
        .try_into_latest()
        .map_err(|_| Error::NarrowMixerTypeFailed)?;
    // Migration for AccountCellData v1
    if input_witness_reader.version() == 1 {
        let input_witness_reader = input_witness_reader
            .try_into_v1()
            .map_err(|_| Error::NarrowMixerTypeFailed)?;

        assert_field_consistent!(
            input_witness_reader,
            output_witness_reader,
            (id, "id"),
            (account, "account"),
            (registered_at, "registered_at"),
            (status, "status")
        );

        assert_field_consistent_if_not_except!(input_witness_reader, output_witness_reader, (records, "records"));
    } else {
        let input_witness_reader = input_witness_reader
            .try_into_latest()
            .map_err(|_| Error::NarrowMixerTypeFailed)?;

        assert_field_consistent!(
            input_witness_reader,
            output_witness_reader,
            (id, "id"),
            (account, "account"),
            (registered_at, "registered_at")
        );

        assert_field_consistent_if_not_except!(
            input_witness_reader,
            output_witness_reader,
            (status, "status"),
            (records, "records"),
            (last_transfer_account_at, "last_transfer_account_at"),
            (last_edit_manager_at, "last_edit_manager_at"),
            (last_edit_records_at, "last_edit_records_at")
        );
    }

    Ok(())
}

fn verify_account_sale_status_selling_to_normal<'a>(
    input_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    output_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
) -> Result<(), Error> {
    let input_witness_reader = input_witness_reader
        .try_into_latest()
        .map_err(|_| Error::NarrowMixerTypeFailed)?;
    let input_account_status = u8::from(input_witness_reader.status());
    if input_account_status != (AccountStatus::Selling as u8) {
        return Err(Error::AccountCellSaleStatusMustSellingStatus);
    }
    let output_witness_reader = output_witness_reader
        .try_into_latest()
        .map_err(|_| Error::NarrowMixerTypeFailed)?;
    let output_account_status = u8::from(output_witness_reader.status());
    if output_account_status != (AccountStatus::Normal as u8) {
        return Err(Error::AccountCellSaleStatusMustNormalStatus);
    }
    Ok(())
}

fn load_account_cells(config_main: ConfigCellMainReader) -> Result<(Vec<usize>, Vec<usize>), Error> {
    let account_cell_type_id = config_main.type_id_table().account_cell();
    let input_account_cells = util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, Source::Input)?;
    let output_account_cells = util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, Source::Output)?;
    Ok((input_account_cells, output_account_cells))
}

fn account_cell_common_verify(
    parser: &WitnessesParser,
    config_account: ConfigCellAccountReader,
    timestamp: u64,
    params: &[u8],
    check_owner_role: bool,
    check_das_lock_owner: bool,
    check_output_account_cell_selling_2_normal: bool,
    changed_lock: Option<&str>,
    witness_except: Vec<&str>,
    output_account_sale_cell_index: usize,
) -> Result<(), Error> {
    if check_owner_role {
        verify_unlock_role(params, LockRole::Owner)?;
    }
    let (input_account_cells, output_account_cells) = load_account_cells(parser.configs.main()?)?;

    assert!(
        input_account_cells.len() == 1 && output_account_cells.len() == 1,
        Error::InvalidTransactionStructure,
        "There should be only one account_cell in input and output"
    );

    let input_cell_witness: Box<dyn AccountCellDataMixer>;
    let input_cell_witness_reader;
    parse_account_cell_witness!(
        input_cell_witness,
        input_cell_witness_reader,
        parser,
        input_account_cells[0],
        Source::Input
    );

    let output_cell_witness: Box<dyn AccountCellDataMixer>;
    let output_cell_witness_reader;
    parse_account_cell_witness!(
        output_cell_witness,
        output_cell_witness_reader,
        parser,
        output_account_cells[0],
        Source::Output
    );

    if check_output_account_cell_selling_2_normal {
        verify_account_sale_status_selling_to_normal(&input_cell_witness_reader, &output_cell_witness_reader)?;
    } else {
        let input_witness_reader = input_cell_witness_reader
            .try_into_latest()
            .map_err(|_| Error::NarrowMixerTypeFailed)?;
        let input_account_status = u8::from(input_witness_reader.status());
        if input_account_status != (AccountStatus::Normal as u8) {
            return Err(Error::AccountCellSaleStatusMustNormalStatus);
        }

        let output_witness_reader = output_cell_witness_reader
            .try_into_latest()
            .map_err(|_| Error::NarrowMixerTypeFailed)?;
        let output_account_status = u8::from(output_witness_reader.status());
        if output_account_status != (AccountStatus::Selling as u8) {
            return Err(Error::AccountCellSaleStatusMustSellingStatus);
        }
    }

    verify_account_expiration(config_account, input_account_cells[0], timestamp)?;
    verify_account_lock_consistent(input_account_cells[0], output_account_cells[0], changed_lock)?;
    verify_account_data_consistent(input_account_cells[0], output_account_cells[0], vec![])?;
    verify_account_capacity_not_decrease(input_account_cells[0], output_account_cells[0])?;
    verify_account_witness_consistent(
        input_account_cells[0],
        output_account_cells[0],
        &input_cell_witness_reader,
        &output_cell_witness_reader,
        witness_except,
    )?;

    if check_das_lock_owner {
        // verify das-lock args
        let input_account_cell_lock =
            high_level::load_cell_lock(input_account_cells[0], Source::Input).map_err(|e| Error::from(e))?;
        let input_account_cell_args = input_account_cell_lock.as_reader().args().raw_data();
        let input_account_cell_owner = data_parser::das_lock_args::get_owner_lock_args(input_account_cell_args);

        let output_account_sale_cell_lock =
            high_level::load_cell_lock(output_account_sale_cell_index, Source::Output).map_err(|e| Error::from(e))?;
        let output_account_sale_cell_args = output_account_sale_cell_lock.as_reader().args().raw_data();
        let output_account_sale_cell_owner =
            data_parser::das_lock_args::get_owner_lock_args(output_account_sale_cell_args);

        assert!(
            input_account_cell_owner == output_account_sale_cell_owner,
            Error::InvalidTransactionStructure,
            "account_cell's owner should equal account_sale_cell. (account_cell: {}, sale_cell: {})",
            util::hex_string(input_account_cell_owner),
            util::hex_string(output_account_sale_cell_owner)
        );
    }

    Ok(())
}

fn verify_account_sale_cell_field(
    timestamp: u64,
    output_cell_witness_reader: AccountSaleCellDataReader,
    secondary_market: ConfigCellSecondaryMarketReader,
) -> Result<(), Error> {
    let sale_started_at = u64::from(output_cell_witness_reader.started_at());
    // beginning time need to equal to timeCell's value
    assert!(
        sale_started_at == timestamp,
        Error::AccountSaleCellStartedAtInvalid,
        "The AccountSaleCell's started_at should be the same as the timestamp in the TimeCell.(expected: {}, current: {})",
        timestamp,
        sale_started_at
    );
    // description bytes limit
    let description_bytes_len = output_cell_witness_reader.description().len() as u32;
    let description_bytes_len_limit = u32::from(secondary_market.sale_description_bytes_limit());
    assert!(
        description_bytes_len_limit >= description_bytes_len,
        Error::AccountSaleCellDescriptionTooLarge,
        "The AccountSaleCell's description bytes too large.(expected: >= {}, current: {})",
        description_bytes_len_limit,
        description_bytes_len
    );

    // check price
    let sale_price = u64::from(output_cell_witness_reader.price());
    let min_price = u64::from(secondary_market.min_sale_price());
    assert!(
        sale_price >= min_price,
        Error::AccountSaleCellPriceTooSmall,
        "The AccountSaleCell's price too small.(expected: >= {}, current: {})",
        min_price,
        sale_price
    );
    Ok(())
}
