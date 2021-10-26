use crate::{assert, error::Error, inspect, parse_witness, witness_parser::WitnessesParser};
use ckb_std::{ckb_constants::Source, high_level};
use das_map::map::Map;
use das_types::{packed::*, prelude::*};

pub fn verify_newly_created(parser: &WitnessesParser, input_income_cell: usize) -> Result<IncomeCellData, Error> {
    let input_income_cell_witness;
    let input_income_cell_witness_reader;
    parse_witness!(
        input_income_cell_witness,
        input_income_cell_witness_reader,
        parser,
        input_income_cell,
        Source::Input,
        IncomeCellData
    );

    // The IncomeCell should be a newly created cell with only one record which is belong to the creator, but we do not need to check everything here, so we only check the length.
    assert!(
        input_income_cell_witness_reader.records().len() == 1,
        Error::ProposalFoundInvalidTransaction,
        "The IncomeCell in inputs should be a newly created cell with only one record which is belong to the creator."
    );

    Ok(input_income_cell_witness)
}

pub fn verify_records_match_with_creating(
    config_income: ConfigCellIncomeReader,
    index: usize,
    source: Source,
    income_cell_witness_reader: IncomeCellDataReader,
    total_profit: u64,
    mut profit_map: Map<Vec<u8>, u64>,
) -> Result<(), Error> {
    #[cfg(any(not(feature = "mainnet"), debug_assertions))]
    inspect::income_cell(source, index, None, Some(income_cell_witness_reader));

    let income_cell_basic_capacity = u64::from(config_income.basic_capacity());

    // Verify if the IncomeCell.capacity is equal to the sum of all records.

    let skip = if total_profit > income_cell_basic_capacity {
        false
    } else {
        // If the profit is sufficient for IncomeCell's basic capacity skip the first record, because it is a convention that the first
        // always belong to the IncomeCell creator in this transaction.
        true
    };
    for (i, record) in income_cell_witness_reader.records().iter().enumerate() {
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
            Error::IncomeCellProfitMismatch,
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
        Error::IncomeCellProfitMismatch,
        "The IncomeCell in outputs should contains everyone's profit. (missing: {})",
        profit_map.len()
    );

    // Verify if the IncomeCell.capacity is equal to the sum of all records.

    let mut expected_income_cell_capacity = 0;
    for record in income_cell_witness_reader.records().iter() {
        expected_income_cell_capacity += u64::from(record.capacity());
    }

    let current_capacity = high_level::load_cell_capacity(index, source).map_err(Error::from)?;
    assert!(
        current_capacity >= income_cell_basic_capacity,
        Error::InvalidTransactionStructure,
        "The IncomeCell should have capacity bigger than or equal to the value in ConfigCellIncome.basic_capacity."
    );
    assert!(
        current_capacity == expected_income_cell_capacity,
        Error::IncomeCellProfitMismatch,
        "The capacity of the IncomeCell should be {} shannon, but {} shannon found.",
        expected_income_cell_capacity,
        current_capacity
    );

    Ok(())
}
