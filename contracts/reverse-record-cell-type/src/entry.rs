use ckb_std::{ckb_constants::Source, high_level};
use core::cmp::Ordering;
use core::result::Result;
use das_core::{
    assert, assert_lock_equal,
    constants::{das_lock, TypeScript},
    debug,
    error::Error,
    util, verifiers,
    witness_parser::WitnessesParser,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running reverse-record-cell-type ======");

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

    let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
    match action {
        b"declare_reverse_record" => {
            let config_main = parser.configs.main()?;
            let config_reverse_resolution = parser.configs.reverse_resolution()?;

            verifiers::common::verify_cell_number_and_position(
                "ReverseRecordCell",
                &input_cells,
                &[],
                &output_cells,
                &[0],
            )?;

            let sender_lock = high_level::load_cell_lock(0, Source::Input)?;
            let reverse_record_cell_capacity = u64::from(config_reverse_resolution.record_basic_capacity())
                + u64::from(config_reverse_resolution.record_prepared_fee_capacity());
            let common_fee = u64::from(config_reverse_resolution.common_fee());

            let balance_cells = util::find_balance_cells(config_main, sender_lock.as_reader(), Source::Input)?;
            verifiers::misc::verify_no_more_cells(&balance_cells, Source::Input)?;

            debug!("Verify if the ReverseRecordCell.capacity is correct.");

            let current_capacity = high_level::load_cell_capacity(output_cells[0], Source::Output)?;
            assert!(
                // Because the ReverseRecordCell will store account in data, it's capacity is dynamic.
                current_capacity >= reverse_record_cell_capacity,
                Error::ReverseRecordCellCapacityError,
                "The ReverseRecordCell.capacity should be at least {} shannon.(current: {})",
                reverse_record_cell_capacity,
                current_capacity
            );

            debug!("Verify if the change is transferred back to the sender properly.");
            let total_input_capacity = util::load_cells_capacity(&balance_cells, Source::Input)?;
            // Allow the transaction builder to pay for the user, or something like that.
            if total_input_capacity > current_capacity + common_fee {
                verifiers::misc::verify_user_get_change(
                    config_main,
                    sender_lock.as_reader(),
                    total_input_capacity - current_capacity - common_fee,
                )?;
            }

            debug!("Verify if the ReverseRecordCell.lock is the same as the lock of inputs[0].");

            assert_lock_equal!(
                (balance_cells[0], Source::Input),
                (output_cells[0], Source::Output),
                Error::ReverseRecordCellLockError,
                "The ReverseRecordCell.lock should be the same as the lock of inputs[0]."
            );

            debug!("Verify if the ReverseRecordCell.lock is the das-lock.");

            let expected_lock = das_lock();
            let current_lock = high_level::load_cell_lock(output_cells[0], Source::Output)?;
            assert!(
                util::is_type_id_equal(expected_lock.as_reader(), current_lock.as_reader()),
                Error::ReverseRecordCellLockError,
                "The ReverseRecordCell.lock should be the das-lock."
            );

            util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
        }
        b"redeclare_reverse_record" => {
            let config_reverse_resolution = parser.configs.reverse_resolution()?;
            verifiers::common::verify_cell_number_and_position(
                "ReverseRecordCell",
                &input_cells,
                &[0],
                &output_cells,
                &[0],
            )?;

            // Stop transaction builder to spend users other cells in this transaction.
            // TODO Support extra cells to pay for transaction fees.
            verifiers::misc::verify_no_more_cells(&input_cells, Source::Input)?;

            debug!("Verify if the fee paied by ReverseRecordCell.capacity is not out of limitation.");

            let expected_fee = u64::from(config_reverse_resolution.common_fee());
            let input_capacity = high_level::load_cell_capacity(0, Source::Input)?;
            let output_capacity = high_level::load_cell_capacity(0, Source::Output)?;
            assert!(
                output_capacity >= input_capacity - expected_fee,
                Error::ReverseRecordCellCapacityError,
                "The ReverseRecordCell.capacity should remain equal to or more than {} shannon.(available_fee: {})",
                input_capacity - expected_fee,
                expected_fee
            );

            debug!("Verify if the ReverseRecordCell.lock is consistent.");

            assert_lock_equal!(
                (input_cells[0], Source::Input),
                (output_cells[0], Source::Output),
                Error::ReverseRecordCellLockError,
                "The ReverseRecordCell.lock should be consistent in inputs and outputs."
            );

            debug!("Verify if the ReverseRecordCell.data.account has been modified.");

            let input_account = high_level::load_cell_data(input_cells[0], Source::Input)?;
            let output_account = high_level::load_cell_data(output_cells[0], Source::Output)?;
            assert!(
                input_account != output_account,
                Error::InvalidTransactionStructure,
                "The ReverseRecordCell.data.account should be modified."
            );

            util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
        }
        b"retract_reverse_record" => {
            let config_main = parser.configs.main()?;
            let config_reverse_resolution = parser.configs.reverse_resolution()?;

            verifiers::common::verify_cell_number_range(
                "ReverseRecordCell",
                &input_cells,
                (Ordering::Greater, 0),
                &output_cells,
                (Ordering::Equal, 0),
            )?;

            verifiers::misc::verify_no_more_cells(&input_cells, Source::Input)?;

            debug!(
                "Verify if all ReverseRecordCells in inputs has the same lock script with the first ReverseRecordCell."
            );

            let expected_lock_hash = high_level::load_cell_lock_hash(input_cells[0], Source::Input)?;
            let mut total_input_capacity = 0;
            for i in input_cells.iter() {
                let lock_hash = high_level::load_cell_lock_hash(*i, Source::Input)?;
                assert!(
                    expected_lock_hash == lock_hash,
                    Error::InvalidTransactionStructure,
                    "Inputs[{}] The ReverseRecordCell should has the same lock script with others.",
                    i
                );

                // CAREFUL, ensure that the total input capacity is calculated from real cells in inputs, because the ReverseRecordCells' capacity is dynamic.
                total_input_capacity += high_level::load_cell_capacity(*i, Source::Input)?;
            }

            debug!("Verify if all capacity have been refund to user correctly.");

            let expected_lock = high_level::load_cell_lock(input_cells[0], Source::Input)?;
            let common_fee = u64::from(config_reverse_resolution.common_fee());
            verifiers::misc::verify_user_get_change(
                config_main,
                expected_lock.as_reader(),
                total_input_capacity - common_fee,
            )?;

            util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
        }
        _ => return Err(Error::ActionNotSupported),
    }

    Ok(())
}
