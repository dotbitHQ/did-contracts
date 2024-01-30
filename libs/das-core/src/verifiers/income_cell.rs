use alloc::boxed::Box;
use core::cmp::Ordering;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_map::map::Map;
use das_map::util as map_util;
use das_types::packed::*;
use das_types::prelude::*;

use crate::config::Config;
use crate::constants::ScriptType;
use crate::error::*;
use crate::{assert, code_to_error, debug, util, warn};

pub fn verify_newly_created(
    income_cell_witness_reader: IncomeCellDataReader,
    index: usize,
    source: Source,
) -> Result<(), Box<dyn ScriptError>> {
    // The IncomeCell should be a newly created cell with only one record which is belong to the creator, but we do not need to check everything here, so we only check the length.
    assert!(
        income_cell_witness_reader.records().len() == 1,
        ErrorCode::InvalidTransactionStructure,
        "{:?}[{}] The IncomeCell in inputs should be a newly created cell with only one record which is belong to the creator.",
        source,
        index
    );

    Ok(())
}

fn verify_records_limit(
    config_reader: ConfigCellIncomeReader,
    income_cell_witness_reader: IncomeCellDataReader,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("  Verify if the IncomeCell's records is out of limit.");

    let income_cell_max_records = u32::from(config_reader.max_records()) as usize;
    assert!(
        income_cell_witness_reader.records().len() <= income_cell_max_records,
        ErrorCode::InvalidTransactionStructure,
        "The IncomeCell can not store more than {} records.",
        income_cell_max_records
    );

    Ok(())
}

fn verify_cell_capacity_with_records_capacity(
    config_reader: ConfigCellIncomeReader,
    index: usize,
    source: Source,
    income_cell_witness_reader: IncomeCellDataReader,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("  Verify if the IncomeCell's capacity is equal to the sum of its records.");

    let basic_capacity = u64::from(config_reader.basic_capacity());
    let current_capacity = high_level::load_cell_capacity(index, source).map_err(Error::<ErrorCode>::from)?;

    let mut expected_capacity = 0;
    for record in income_cell_witness_reader.records().iter() {
        expected_capacity += u64::from(record.capacity());
    }

    assert!(
        current_capacity >= basic_capacity,
        ErrorCode::IncomeCellCapacityError,
        "{:?}[{}] The IncomeCell should have capacity bigger than or equal to the value in ConfigCellIncome.basic_capacity.",
        source,
        index
    );
    assert!(
        current_capacity == expected_capacity,
        ErrorCode::IncomeCellCapacityError,
        "{:?}[{}] The capacity of the IncomeCell should be {} shannon, but {} shannon found.",
        source,
        index,
        expected_capacity,
        current_capacity
    );

    Ok(())
}

pub fn verify_income_cells(profit_map: Map<Vec<u8>, u64>) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify the IncomeCells in inputs and outputs.");

    #[cfg(debug_assertions)]
    {
        debug!("  Profit map: {} total", profit_map.items.len());
        for (script_bytes, capacity) in profit_map.items.iter() {
            let script = Script::from_slice(&script_bytes.as_slice()).unwrap();
            debug!("    {{ script.args: {}, capacity: {} }}", script.args(), capacity);
        }
    }

    let total_profit = if profit_map.items.len() == 0 {
        0
    } else {
        profit_map.items.iter().map(|v| v.1).reduce(|acc, v| acc + v).unwrap()
    };
    let config_main = Config::get_instance().main()?;

    let (input_income_cells, output_income_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, config_main.type_id_table().income_cell())?;
    if profit_map.items.len() == 0 || total_profit == 0 {
        debug!("Since the profit is empty, there should be no IncomeCell in either the inputs or outputs.");

        super::common::verify_cell_number("IncomeCell", &input_income_cells, 0, &output_income_cells, 0)?;

        return Ok(());
    } else {
        debug!("Since the profit is not empty, there should be 1 IncomeCell in both the inputs and outputs.");

        super::common::verify_cell_number_range(
            "IncomeCell",
            &input_income_cells,
            (Ordering::Less, 2),
            &output_income_cells,
            (Ordering::Equal, 1),
        )?;
    }

    let config_income = Config::get_instance().income()?;

    // If an existing IncomeCell is used, collect all its records for later usage.
    let mut exist_records_opt = None;
    if input_income_cells.len() == 1 {
        let input_income_witness = util::parse_income_cell_witness(input_income_cells[0], Source::Input)?;
        let input_income_witness_reader = input_income_witness.as_reader();

        let mut tmp = Map::new();
        for item in input_income_witness_reader.records().iter() {
            let key = item.belong_to().as_slice().to_vec();
            let value = u64::from(item.capacity());

            map_util::add(&mut tmp, key, value);
        }
        exist_records_opt = Some(tmp);
    }

    let output_income_witness = util::parse_income_cell_witness(output_income_cells[0], Source::Output)?;
    let output_income_witness_reader = output_income_witness.as_reader();

    #[cfg(debug_assertions)]
    crate::inspect::income_cell(
        Source::Output,
        output_income_cells[0],
        None,
        Some(output_income_witness_reader),
    );

    super::misc::verify_always_success_lock(output_income_cells[0], Source::Output)?;
    verify_records_limit(config_income, output_income_witness_reader)?;
    verify_cell_capacity_with_records_capacity(
        config_income,
        output_income_cells[0],
        Source::Output,
        output_income_witness_reader,
    )?;

    // Combine records with the same belong_to.
    let mut output_records = Map::new();
    for item in output_income_witness_reader.records().iter() {
        let key = item.belong_to().as_slice().to_vec();
        let value = u64::from(item.capacity());

        map_util::add(&mut output_records, key, value);
    }

    if let Some(exist_records) = exist_records_opt.as_ref() {
        debug!("  Verify if the records in the IncomeCell in inputs is reserved correctly in outputs");

        for (key, exist_capacity) in exist_records.items.iter() {
            if let Some(current_capacity) = output_records.get(key) {
                assert!(
                    current_capacity >= exist_capacity,
                    ErrorCode::IncomeCellConsolidateConditionNotSatisfied,
                    "outputs[{}] There is some record in outputs has less capacity than itself in inputs which is not allowed. (belong_to: {})",
                    output_income_cells[0],
                    Script::from_slice(key.as_slice()).unwrap()
                );
            } else {
                warn!(
                    "outputs[{}] There is some records missing in outputs. (belong_to: {})",
                    output_income_cells[0],
                    Script::from_slice(key.as_slice()).unwrap()
                );
                return Err(code_to_error!(ErrorCode::IncomeCellConsolidateConditionNotSatisfied));
            }
        }
    }

    // Compare every records with profit_map to find out if every user get their profit properly.
    debug!("  Verify if the records in IncomeCell in outputs has carried profits of all users properly.");

    for (key, value) in output_records.items.iter() {
        let mut current_capacity = *value;

        if let Some(exist_records) = exist_records_opt.as_ref() {
            if let Some(&exist_capacity) = exist_records.get(key) {
                // In above verification we have assert current capacity must bigger than or equal to existing capacity.
                current_capacity -= exist_capacity;
            }
        }

        if let Some(&expected_capacity) = profit_map.get(key) {
            assert!(
                current_capacity >= expected_capacity,
                ErrorCode::IncomeCellProfitMismatch,
                "outputs[{}] The IncomeCell has a wrong record for some user.(belong_to: {}, expected: {}, current: {})",
                output_income_cells[0],
                Script::from_slice(key.as_slice()).unwrap(),
                expected_capacity,
                current_capacity
            );
        }
    }

    Ok(())
}
