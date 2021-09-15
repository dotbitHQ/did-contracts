use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use ckb_std::{ckb_constants::Source, debug, high_level};
use core::result::Result;
use core::slice::Iter;
use das_core::{assert, constants::*, error::Error, parse_witness, util, warn, witness_parser::WitnessesParser};
use das_types::{constants::DataType, packed::*, prelude::*};

pub fn main() -> Result<(), Error> {
    debug!("====== Running income-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    util::is_system_off(&mut parser)?;

    let action_data = parser.parse_action()?;
    let action = action_data.as_reader().action().raw_data();
    if action == b"create_income" {
        debug!("Route to create_income action ...");

        debug!("Find out IncomeCells ...");

        let this_type_script = high_level::load_script().map_err(|e| Error::from(e))?;
        let (input_cells, output_cells) =
            util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, this_type_script.as_reader())?;

        assert!(
            input_cells.len() == 0,
            Error::InvalidTransactionStructure,
            "Consuming IncomeCell is not allowed in create_income action."
        );
        assert!(
            output_cells.len() == 1,
            Error::InvalidTransactionStructure,
            "Only one IncomeCell can be created in create_income action."
        );

        util::is_cell_use_always_success_lock(output_cells[0], Source::Output)?;

        parser.parse_cell()?;
        parser.parse_config(&[DataType::ConfigCellIncome])?;

        let config_income = parser.configs.income()?;

        debug!("Read data of the IncomeCell ...");

        let income_cell_witness;
        let income_cell_witness_reader;
        parse_witness!(
            income_cell_witness,
            income_cell_witness_reader,
            parser,
            output_cells[0],
            Source::Output,
            IncomeCellData
        );

        assert!(
            income_cell_witness_reader.records().len() == 1,
            Error::InvalidTransactionStructure,
            "Only one record should exist in the IncomeCell."
        );

        let record = income_cell_witness_reader.records().get(0).unwrap();

        assert!(
            util::is_reader_eq(income_cell_witness_reader.creator(), record.belong_to()),
            Error::InvalidTransactionStructure,
            "The only one record should belong to the creator of the IncomeCell ."
        );
        assert!(
            util::is_reader_eq(record.capacity(), config_income.basic_capacity()),
            Error::InvalidTransactionStructure,
            "The only one record should has the same capacity with ConfigCellIncome.basic_capacity ."
        );

        let cell_capacity =
            high_level::load_cell_capacity(output_cells[0], Source::Output).map_err(|e| Error::from(e))?;
        let basic_capacity = u64::from(config_income.basic_capacity());
        assert!(
            cell_capacity == basic_capacity,
            Error::IncomeCellCapacityError,
            "The IncomeCell.capacity should equal to the basic capacity of IncomeCell. (expected: {}, current: {})",
            basic_capacity,
            cell_capacity
        );
    } else if action == b"consolidate_income" {
        debug!("Route to consolidate action ...");

        debug!("Find out IncomeCells ...");

        let this_type_script = high_level::load_script().map_err(|e| Error::from(e))?;
        let (input_cells, output_cells) =
            util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, this_type_script.as_reader())?;

        assert!(
            input_cells.len() >= 2,
            Error::IncomeCellConsolidateConditionNotSatisfied,
            "There should be at least 2 IncomeCell in this transaction."
        );
        assert!(
            input_cells.len() >= output_cells.len(),
            Error::IncomeCellConsolidateConditionNotSatisfied,
            "The number of IncomeCells in the outputs should be lesser than or equal to in the inputs."
        );

        parser.parse_config(&[
            DataType::ConfigCellMain,
            DataType::ConfigCellIncome,
            DataType::ConfigCellProfitRate,
        ])?;
        parser.parse_cell()?;

        let config_income = parser.configs.income()?;
        let income_cell_basic_capacity = u64::from(config_income.basic_capacity());
        let income_cell_max_records = u32::from(config_income.max_records()) as usize;
        let income_cell_min_transfer_capacity = u64::from(config_income.min_transfer_capacity());
        let income_consolidate_profit_rate = u32::from(parser.configs.profit_rate()?.income_consolidate()) as u64;

        debug!("Find all income records in inputs and merge them into unique script to capacity pair.");

        let mut creators = Vec::new();
        let mut input_records = Vec::new();
        for index in input_cells {
            let (_, _, entity) = parser.verify_and_get(index.to_owned(), Source::Input)?;
            let income_cell_witness = IncomeCellData::from_slice(entity.as_reader().raw_data())
                .map_err(|_| Error::WitnessEntityDecodingError)?;

            #[cfg(any(not(feature = "mainnet"), debug_assertions))]
            das_core::inspect::income_cell(Source::Input, index, None, Some(income_cell_witness.as_reader()));

            let creator = income_cell_witness.creator();
            let records = income_cell_witness.records();

            if records.len() == 1 {
                let first_record = records.get(0).unwrap();
                assert!(
                    !util::is_entity_eq(&first_record.belong_to(), &creator),
                    Error::IncomeCellConsolidateConditionNotSatisfied,
                    "Can not consolidate the IncomeCell which has only one record belong to the creator."
                );
            }

            for record in income_cell_witness.records().into_iter() {
                input_records = merge_record(input_records, record);
            }

            creators.push(creator);
        }

        // Always include DAS in the members which is free from consolidating fee.
        let das_wallet_lock = das_wallet_lock();
        creators.push(das_wallet_lock.into());

        debug!("Classify all income records in inputs for comparing them with outputs later.");

        let (records_should_transfer, mut records_should_keep, need_pad) = classify_income_records(
            income_consolidate_profit_rate,
            income_cell_max_records,
            income_cell_basic_capacity,
            income_cell_min_transfer_capacity,
            input_records,
        );

        #[cfg(any(not(feature = "mainnet"), debug_assertions))]
        inspect_records("Records should be kept:", &records_should_keep);
        #[cfg(any(not(feature = "mainnet"), debug_assertions))]
        inspect_records("Records should be transferred:", &records_should_transfer);

        debug!("Conclusion of need_pad: {}", need_pad);

        debug!("Classify all income records in outputs.");

        let mut output_records: Vec<(Script, u64)> = Vec::new();
        for (i, cell_index) in output_cells.iter().enumerate() {
            let (_, _, entity) = parser.verify_and_get(cell_index.to_owned(), Source::Output)?;
            let income_cell_witness = IncomeCellData::from_slice(entity.as_reader().raw_data())
                .map_err(|_| Error::WitnessEntityDecodingError)?;

            #[cfg(any(not(feature = "mainnet"), debug_assertions))]
            das_core::inspect::income_cell(
                Source::Output,
                cell_index.to_owned(),
                None,
                Some(income_cell_witness.as_reader()),
            );

            assert!(
                income_cell_witness.records().len() <= income_cell_max_records,
                Error::IncomeCellConsolidateError,
                "Output[{}] Each IncomeCell can not store more than {} records.",
                i,
                income_cell_max_records
            );

            let mut records_total_capacity = 0;
            for record in income_cell_witness.records().into_iter() {
                for exist_record in output_records.iter() {
                    assert!(
                        !util::is_entity_eq(&exist_record.0, &record.belong_to()),
                        Error::IncomeCellConsolidateError,
                        "Output[{}] There should be not duplicate income records in outputs.",
                        i
                    )
                }

                let capacity = u64::from(record.capacity());
                records_total_capacity += capacity;
                output_records.push((record.belong_to(), capacity));
            }

            let cell_capacity =
                high_level::load_cell_capacity(cell_index.to_owned(), Source::Output).map_err(|e| Error::from(e))?;
            assert!(
                records_total_capacity == cell_capacity,
                Error::IncomeCellConsolidateError,
                "Output[{}] The IncomeCell.capacity should be always equal to the total capacity of its records. (expected: {}, current: {})",
                i,
                records_total_capacity,
                cell_capacity
            );
            assert!(
                cell_capacity >= income_cell_basic_capacity,
                Error::IncomeCellConsolidateError,
                "Output[{}] The IncomeCell.capacity should be always greater than or equal to {} shannon.",
                i,
                income_cell_basic_capacity
            )
        }

        if records_should_keep.len() > 0 {
            assert!(
                output_records.len() > 0,
                Error::InvalidTransactionStructure,
                "There should be some IncomeCell in the outputs, because the count of records_should_keep is {}",
                records_should_keep.len()
            );
        }

        debug!("Check if transfer as expected.");

        let type_id_table = parser.configs.main()?.type_id_table();
        let das_lock = das_lock();
        let das_lock_reader = das_lock.as_reader();
        let mut records_used_for_pad = Vec::new();
        for (i, item) in records_should_transfer.into_iter().enumerate() {
            let lock_script = item.0.as_reader();
            let cells = util::find_cells_by_script(ScriptType::Lock, lock_script.into(), Source::Output)?;
            if cells.len() != 1 {
                if need_pad {
                    // If the IncomeCell needs capacity padding, and the records should be transferred are not transferred at all,
                    // we think it must be used for padding with all its capacity.
                    records_used_for_pad.push((item.0, item.1, false));
                    continue;
                } else {
                    // The length maybe 0, so do not use "Outputs[{}]" here.
                    warn!(
                        "There should be only one cell for the {}th record(records_should_transfer[{}]), but {} cells are found.",
                        i,
                        i,
                        cells.len()
                    );
                    return Err(Error::IncomeCellTransferError);
                }
            }

            let capacity_transferred =
                high_level::load_cell_capacity(cells[0], Source::Output).map_err(|e| Error::from(e))?;
            let mut capacity_should_be_transferred = item.1 / RATE_BASE * (RATE_BASE - income_consolidate_profit_rate);

            // If the record belongs to a IncomeCell creator, keeper should not take fee from it.
            for creator in creators.iter() {
                if util::is_entity_eq(&item.0, creator) {
                    capacity_should_be_transferred = item.1;
                }
            }

            debug!(
                "records_should_transfer[{}] {{ output_index: {}, args: {}, total: {}, capacity_transferred: {}, capacity_should_be_transferred: {} }}",
                i,
                cells[0],
                item.0.args(),
                item.1,
                capacity_transferred,
                capacity_should_be_transferred
            );

            if capacity_transferred < capacity_should_be_transferred {
                if need_pad {
                    let capacity_should_transferred_with_fee =
                        capacity_transferred / (RATE_BASE - income_consolidate_profit_rate) * RATE_BASE;
                    let capacity_should_remain_for_pad = item.1 - capacity_should_transferred_with_fee;

                    debug!("  Outputs[{}] recalculate fee from transferred part: {{ capacity_transferred: {}, final_fee: {} }}",
                        cells[0], capacity_transferred, capacity_should_transferred_with_fee - capacity_transferred
                    );

                    // If the IncomeCell needs capacity padding, and the records should be transferred are transferred parts of its capacity,
                    // we think the remain parts of capacity must be used for padding.
                    records_used_for_pad.push((item.0, capacity_should_remain_for_pad, false));
                } else {
                    warn!("Outputs[{}] The transferred capacity is less than expected. (capacity_in_record: {}, capacity_should_be_transferred: {}, capacity_transferred: {})",
                        cells[0], item.1, capacity_should_be_transferred, capacity_transferred
                    );
                    return Err(Error::IncomeCellTransferError);
                }
            // The capacity of transfer must be less than which in the records.
            } else if capacity_transferred > capacity_should_be_transferred {
                warn!(
                    "Outputs[{}] The transferred capacity is more than expected. (capacity_in_record: {}, expected: {}, current: {})",
                    cells[0], item.1, capacity_should_be_transferred, capacity_transferred
                );
                return Err(Error::IncomeCellTransferError);
            }

            verify_das_lock_and_balance_type(
                das_lock_reader.into(),
                type_id_table.balance_cell(),
                cells[0],
                Source::Output,
            )?;
        }

        #[cfg(any(not(feature = "mainnet"), debug_assertions))]
        inspect_records(
            "Records should be used to pad IncomeCell capacity:",
            &records_used_for_pad,
        );

        debug!("Check if consolidate as expected.");

        for record in output_records {
            let mut is_exist = false;
            // Check if record exists in the records_should_keep.
            for expected_record in records_should_keep.iter_mut() {
                if util::is_entity_eq(&record.0, &expected_record.0) {
                    assert!(
                        record.1 == expected_record.1,
                        Error::IncomeCellConsolidateError,
                        "The capacity of some records in the outputs is incorrect. (belong_to: {}, expected: {}, current: {})",
                        record.0,
                        expected_record.1,
                        record.1
                    );
                    expected_record.2 = true;
                    is_exist = true;
                }
            }

            if !is_exist {
                // Check if record exists in the records_used_for_pad.
                for expected_record in records_used_for_pad.iter_mut() {
                    if util::is_entity_eq(&record.0, &expected_record.0) {
                        assert!(
                            record.1 == expected_record.1,
                            Error::IncomeCellConsolidateError,
                            "The record should be transferred is not transferred completely, so we think parts of its capacity should be used for padding capacity, BUT the capacity used for padding is incorrect. (belong_to: {}, expected: {}, current: {})",
                            record.0,
                            expected_record.1,
                            record.1
                        );

                        expected_record.2 = true;
                        is_exist = true;
                    }
                }
            }

            assert!(
                is_exist,
                Error::IncomeCellConsolidateError,
                "Missing expected record in outputs. (expected: {:?})", record
            );
        }

        for record in records_should_keep.iter() {
            assert!(
                record.2,
                Error::IncomeCellConsolidateError,
                "The record should be kept is missing in outputs. (belong_to: {}, wasted: {})", record.0, record.1
            );
        }

        for record in records_used_for_pad.iter() {
            assert!(
                record.2,
                Error::IncomeCellConsolidateWaste,
                "The record should be transferred is not transferred completely, so we think parts of its capacity should be used for padding capacity, BUT the capacity is not used for padding. (belong_to: {}, wasted: {})",
                record.0,
                record.1
            );
        }
    } else if action == b"confirm_proposal" {
        debug!("Route to confirm_proposal action ...");
        util::require_type_script(
            &mut parser,
            TypeScript::ProposalCellType,
            Source::Input,
            Error::ProposalFoundInvalidTransaction,
        )?;
    } else if action == b"buy_account" {
        debug!("Route to buy_account action ...");
        util::require_type_script(
            &mut parser,
            TypeScript::AccountSaleCellType,
            Source::Input,
            Error::AccountCellFoundInvalidTransaction,
        )?;
    } else if action == b"renew_account" {
        debug!("Route to renew_account action ...");
        util::require_type_script(
            &mut parser,
            TypeScript::AccountCellType,
            Source::Input,
            Error::AccountCellFoundInvalidTransaction,
        )?;
    } else {
        warn!("The ActionData in witness has an undefine action.");
        return Err(Error::ActionNotSupported);
    }

    Ok(())
}

fn merge_record(mut input_records: Vec<(Script, u64)>, record: IncomeRecord) -> Vec<(Script, u64)> {
    for exist_record in input_records.iter_mut() {
        if util::is_entity_eq(&exist_record.0, &record.belong_to()) {
            exist_record.1 += u64::from(record.capacity());
            return input_records;
        }
    }

    input_records.push((record.belong_to(), u64::from(record.capacity())));
    input_records
}

fn calc_total_records_capacity(records: Iter<(Script, u64, bool)>) -> u64 {
    // There is no reduce method here, so we use for...in instead.
    let mut total = 0;
    for record in records {
        total += record.1;
    }

    total
}

fn classify_income_records(
    income_consolidate_profit_rate: u64,
    income_cell_max_records: usize,
    income_cell_basic_capacity: u64,
    income_cell_min_transfer_capacity: u64,
    input_records: Vec<(Script, u64)>,
) -> (Vec<(Script, u64, bool)>, Vec<(Script, u64, bool)>, bool) {
    let mut records_should_transfer = Vec::new();
    let mut records_should_keep = Vec::new();

    for record in input_records.into_iter() {
        let capacity_after_fee_paid = record.1 / RATE_BASE * (RATE_BASE - income_consolidate_profit_rate);

        debug!(
            "  {{ args: {}, capacity_after_fee_paid: {} = {}(record.capacity) - {}(record.capacity) * {}(income_consolidate_profit_rate) / {}(RATE_BASE) }}",
            record.0.args(),
            capacity_after_fee_paid,
            record.1,
            record.1,
            income_consolidate_profit_rate,
            RATE_BASE
        );

        if capacity_after_fee_paid >= income_cell_min_transfer_capacity {
            records_should_transfer.push((record.0, record.1, false));
        } else {
            records_should_keep.push((record.0, record.1, false));
        }
    }

    let remain_capacity = calc_total_records_capacity(records_should_keep.iter());

    // Calculate how many IncomeCell is required in outputs.
    let mut output_income_cell_count: u64;
    if records_should_keep.len() <= 0 {
        output_income_cell_count = 0;
    } else {
        output_income_cell_count = (records_should_keep.len() / income_cell_max_records) as u64;
        if records_should_keep.len() % income_cell_max_records != 0 {
            output_income_cell_count += 1
        }
    }

    (
        records_should_transfer,
        records_should_keep,
        // If the total capacity remains in IncomeCell is not enough, that means the IncomeCell needs padding.
        // If the total capacity remains 0, that means no IncomeCell is needed is outputs.
        remain_capacity != 0 && remain_capacity < (income_cell_basic_capacity * output_income_cell_count),
    )
}

fn verify_das_lock_and_balance_type(
    das_lock_reader: ScriptReader,
    balance_cell_type_id: HashReader,
    index: usize,
    source: Source,
) -> Result<(), Error> {
    let lock = high_level::load_cell_lock(index, source).map_err(|e| Error::from(e))?;
    let lock_reader = lock.as_reader();

    if util::is_script_equal(das_lock_reader.into(), lock_reader) {
        let type_of_lock = lock_reader.args().raw_data()[0];
        if type_of_lock == DasLockType::ETHTypedData as u8 {
            let type_opt = high_level::load_cell_type(index, source).map_err(|e| Error::from(e))?;
            assert!(
                type_opt.is_some(),
                Error::InvalidTransactionStructure,
                "Outputs[{}] The NormalCells in outputs with das-lock type 5 should have balance-cell-type in their type field.",
                index
            );

            let type_ = type_opt.unwrap();
            let type_reader = type_.as_reader();
            let hash_type = type_reader.hash_type().as_slice()[0];
            assert!(
                util::is_reader_eq(type_reader.code_hash().into(), balance_cell_type_id)
                    && hash_type == ScriptHashType::Type as u8,
                Error::InvalidTransactionStructure,
                "Outputs[{}] The NormalCells in outputs with das-lock type 5 should have balance-cell-type in their type field.",
                index
            )
        } else {
            let type_opt = high_level::load_cell_type(index, source).map_err(|e| Error::from(e))?;
            assert!(
                type_opt.is_none(),
                Error::InvalidTransactionStructure,
                "Outputs[{}] The NormalCells in outputs with das-lock which type other than 5 should not have balance-cell-type in their type field.",
                index
            );
        }
    }

    Ok(())
}

#[cfg(any(not(feature = "mainnet"), debug_assertions))]
fn inspect_records(title: &str, records: &Vec<(Script, u64, bool)>) {
    debug!("{} {} total", title, records.len());

    for (i, record) in records.iter().enumerate() {
        debug!(
            "  {{ index: {}, belong_to.args: {}, capacity: {} }}",
            i,
            record.0.args(),
            record.1
        );
    }
}
