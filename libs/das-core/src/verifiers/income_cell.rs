use crate::{assert, error::Error, parse_witness, witness_parser::WitnessesParser};
use ckb_std::ckb_constants::Source;
use das_types::{packed::IncomeCellData, prelude::*};

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
