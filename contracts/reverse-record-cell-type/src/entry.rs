use alloc::string::String;
use ckb_std::{ckb_constants::Source, high_level};
use core::result::Result;
use das_core::{
    assert, constants::das_lock, constants::ScriptType, data_parser, debug, error::Error, util, verifiers,
    witness_parser::WitnessesParser,
};
use das_types::constants::DataType;
use das_types::packed::ConfigCellMainReader;

pub fn main() -> Result<(), Error> {
    debug!("====== Running reverse-record-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_opt = parser.parse_action_with_params()?;
    if action_opt.is_none() {
        return Err(Error::ActionNotSupported);
    }

    let (action_raw, _) = action_opt.unwrap();
    let action = action_raw.as_reader().raw_data();

    util::is_system_off(&mut parser)?;

    debug!(
        "Route to {:?} action ...",
        String::from_utf8(action.to_vec()).map_err(|_| Error::ActionNotSupported)?
    );

    let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
    match action {
        b"declare_reverse_record" => {
            parser.parse_config(&[DataType::ConfigCellMain, DataType::ConfigCellReverseResolution])?;
            let config_main = parser.configs.main()?;
            let config_reverse_resolution = parser.configs.reverse_resolution()?;

            assert!(
                input_cells.len() == 0 && output_cells.len() == 1,
                Error::InvalidTransactionStructure,
                "There should be only 1 ReverseRecordCell at outputs[0]."
            );
            assert!(
                output_cells[0] == 0,
                Error::InvalidTransactionStructure,
                "There should be only 1 ReverseRecordCell at outputs[0]."
            );

            let sender_lock = high_level::load_cell_lock(0, Source::Input).map_err(Error::from)?;

            let balance_cells = util::find_balance_cells(config_main, sender_lock.as_reader())?;
            verifiers::misc::verify_no_more_cells(&balance_cells, Source::Input)?;

            debug!("Verify if the change is transferred back to the sender properly.");

            let mut total_input_capacity = 0;
            for i in balance_cells {
                total_input_capacity += high_level::load_cell_capacity(i, Source::Input).map_err(Error::from)?;
            }
            let reverse_record_cell_capacity = u64::from(config_reverse_resolution.record_basic_capacity())
                + u64::from(config_reverse_resolution.record_prepared_fee_capacity());
            let common_fee = u64::from(config_reverse_resolution.common_fee());
            verifiers::misc::verify_user_get_change(
                config_main,
                sender_lock.as_reader(),
                total_input_capacity - reverse_record_cell_capacity - common_fee,
            )?;

            debug!("Verify if the ReverseRecordCell.capacity is correct.");

            let current_capacity =
                high_level::load_cell_capacity(output_cells[0], Source::Output).map_err(Error::from)?;
            assert!(
                reverse_record_cell_capacity == current_capacity,
                Error::ReverseRecordCellCapacityError,
                "The ReverseRecordCell.capacity should be {} shannon.(current: {})",
                reverse_record_cell_capacity,
                current_capacity
            );

            debug!("Verify if the ReverseRecordCell.lock is the same as the lock of inputs[0].");

            let expected_lock_hash =
                high_level::load_cell_lock_hash(output_cells[0], Source::Output).map_err(Error::from)?;
            let current_lock_hash =
                high_level::load_cell_lock_hash(output_cells[0], Source::Output).map_err(Error::from)?;
            assert!(
                expected_lock_hash == current_lock_hash,
                Error::ReverseRecordCellLockError,
                "The ReverseRecordCell.lock should be the same as the lock of inputs[0]."
            );

            debug!("Verify if the ReverseRecordCell.lock is the das-lock.");

            let expected_lock = das_lock();
            let current_lock = high_level::load_cell_lock(output_cells[0], Source::Output).map_err(Error::from)?;
            assert!(
                util::is_type_id_equal(expected_lock.as_reader(), current_lock.as_reader()),
                Error::ReverseRecordCellLockError,
                "The ReverseRecordCell.lock should be the das-lock."
            );

            verify_account_exist(config_main, output_cells[0])?;
        }
        b"redeclare_reverse_record" => {
            parser.parse_config(&[DataType::ConfigCellMain, DataType::ConfigCellReverseResolution])?;
            let config_main = parser.configs.main()?;
            let config_reverse_resolution = parser.configs.reverse_resolution()?;

            assert!(
                input_cells.len() == 1 && output_cells.len() == 1,
                Error::InvalidTransactionStructure,
                "There should be 1 ReverseRecordCell in both inputs and outputs."
            );
            assert!(
                input_cells[0] == 0 && output_cells[0] == 0,
                Error::InvalidTransactionStructure,
                "The ReverseRecordCells should only exist at inputs[0] and outputs[0]."
            );

            // Stop transaction builder to spend users other cells in this transaction.
            // TODO Support extra cells to pay for transaction fees.
            verifiers::misc::verify_no_more_cells(&input_cells, Source::Input)?;

            debug!("Verify if the fee paied by ReverseRecordCell.capacity is not out of limitation.");

            let expected_fee = u64::from(config_reverse_resolution.common_fee());
            let input_capacity = high_level::load_cell_capacity(0, Source::Input).map_err(Error::from)?;
            let output_capacity = high_level::load_cell_capacity(0, Source::Output).map_err(Error::from)?;
            assert!(
                output_capacity >= input_capacity - expected_fee,
                Error::ReverseRecordCellCapacityError,
                "The ReverseRecordCell.capacity should remain equal to or more than {} shannon.(available_fee: {})",
                input_capacity - expected_fee,
                expected_fee
            );

            debug!("Verify if the ReverseRecordCell.lock is consistent.");

            let expected_lock_hash =
                high_level::load_cell_lock_hash(input_cells[0], Source::Input).map_err(Error::from)?;
            let current_lock_hash =
                high_level::load_cell_lock_hash(output_cells[0], Source::Output).map_err(Error::from)?;
            assert!(
                expected_lock_hash == current_lock_hash,
                Error::ReverseRecordCellLockError,
                "The ReverseRecordCell.lock should be consistent in inputs and outputs."
            );

            debug!("Verify if the ReverseRecordCell.data.account has been modified.");

            let input_account = high_level::load_cell_data(input_cells[0], Source::Input).map_err(Error::from)?;
            let output_account = high_level::load_cell_data(output_cells[0], Source::Output).map_err(Error::from)?;
            assert!(
                input_account != output_account,
                Error::InvalidTransactionStructure,
                "The ReverseRecordCell.data.account should be modified."
            );

            verify_account_exist(config_main, output_cells[0])?;
        }
        b"retract_reverse_record" => {
            parser.parse_config(&[DataType::ConfigCellMain, DataType::ConfigCellReverseResolution])?;
            let config_main = parser.configs.main()?;
            let config_reverse_resolution = parser.configs.reverse_resolution()?;

            assert!(
                input_cells.len() >= 1 && output_cells.len() == 0,
                Error::InvalidTransactionStructure,
                "There should be at least 1 ReverseRecordCell in inputs."
            );
            assert!(
                input_cells[0] == 0,
                Error::InvalidTransactionStructure,
                "The first ReverseRecordCell should be started at inputs[0]."
            );

            verifiers::misc::verify_no_more_cells(&input_cells, Source::Input)?;

            debug!(
                "Verify if all ReverseRecordCells in inputs has the same lock script with the first ReverseRecordCell."
            );

            let expected_lock_hash =
                high_level::load_cell_lock_hash(input_cells[0], Source::Input).map_err(Error::from)?;
            let mut total_input_capacity = 0;
            for i in input_cells.iter() {
                let lock_hash = high_level::load_cell_lock_hash(*i, Source::Input).map_err(Error::from)?;
                assert!(
                    expected_lock_hash == lock_hash,
                    Error::InvalidTransactionStructure,
                    "Inputs[{}] The ReverseRecordCell should has the same lock script with others.",
                    i
                );

                total_input_capacity += high_level::load_cell_capacity(*i, Source::Input).map_err(Error::from)?;
            }

            debug!("Verify if all capacity have been refund to user correctly.");

            let expected_lock = high_level::load_cell_lock(input_cells[0], Source::Input).map_err(Error::from)?;
            let common_fee = u64::from(config_reverse_resolution.common_fee());
            verifiers::misc::verify_user_get_change(
                config_main,
                expected_lock.as_reader(),
                total_input_capacity - common_fee,
            )?;
        }
        _ => return Err(Error::ActionNotSupported),
    }

    Ok(())
}

fn verify_account_exist(config_main: ConfigCellMainReader, output_cell: usize) -> Result<(), Error> {
    debug!("Verify if the ReverseRecordCell.data.account is really exist.");

    let account_cell_type_id = config_main.type_id_table().account_cell();
    let account_cells = util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, Source::CellDep)?;
    assert!(
        account_cells.len() == 1,
        Error::InvalidTransactionStructure,
        "There should be only 1 AccountCell in cell_deps."
    );

    let account_cell_data = high_level::load_cell_data(account_cells[0], Source::CellDep).map_err(Error::from)?;
    let expected_account = data_parser::account_cell::get_account(&account_cell_data);
    let current_account = high_level::load_cell_data(output_cell, Source::Output).map_err(Error::from)?;
    assert!(
        expected_account == current_account.as_slice(),
        Error::ReverseRecordCellAccountError,
        "The ReverseRecordCell.data.account should be the same as the account of the AccountCell."
    );

    Ok(())
}
