use alloc::{boxed::Box, vec, vec::Vec};
use ckb_std::{ckb_constants::Source, ckb_types::prelude::*, debug, high_level};
use das_core::{
    assert,
    constants::{
        das_lock, das_wallet_lock, OracleCellType, ScriptType, TypeScript, CUSTOM_KEYS_NAMESPACE,
    },
    data_parser,
    error::Error,
    parse_account_cell_witness, parse_witness, util, warn,
    witness_parser::WitnessesParser,
};
use das_types::{
    constants::{DataType, LockRole, AccountStatus},
    mixer::*,
    packed::*,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running on-sale-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    util::is_system_off(&mut parser)?;

    let action_data = parser.parse_action()?;
    let action = action_data.as_reader().action().raw_data();
    // let params = action_data.as_reader().params().raw_data();
    if action == b"start_account_sale" || action == b"cancel_account_sale" {
        let timestamp = util::load_oracle_data(OracleCellType::Time)?;
        parser.parse_cell()?;
        let config_main = parser.configs.main()?;
        let (input_on_sale_cells, output_on_sale_cells) = load_on_sale_cells()?;

        assert!(
            output_cell_witness_reader.version() == 2,
            Error::DataTypeUpgradeRequired,
            "The witness of the OnSaleCell in outputs should be upgrade to version 2."
        );
        if action == b"start_account_sale" {
            debug!("Route to start_account_sale action ...");
            assert!(
                input_on_sale_cells.len() == 0 && output_on_sale_cells.len() == 1,
                Error::OnSaleCellNumberInvalid,
                "There should be zero OnSaleCell int input and one OnSaleCell in output.");

            verify_on_sale_cell_args_equal_account_id(config_main,output_on_sale_cells,Source::Output)?;

            let output_cell_witness;
            let output_cell_witness_reader;
            parse_witness!(
                output_cell_witness,
                output_cell_witness_reader,
                parser,
                output_on_sale_cell_index,
                Source::Output,
                OnSaleCellData
            );

            let sale_started_at = output_cell_witness_reader.started_at() as u64;
            // beginning time need to equal to timeCell's value
            assert!(
                sale_started_at == timestamp,
                Error::OnSaleCellStartedAtInvalid,
                "The OnSaleCell's started_at should be the same as the timestamp in the TimeCell.(expected: {}, current: {})",
                timestamp,
                sale_started_at
            );
            // price logic
            let sale_price = output_cell_witness_reader.price() as u64;
            let _min_price:u64 = 100 * 100_000_000; // example: 100 ckb
            assert!(
                sale_price >= _min_price,
                Error::OnSaleCellPriceTooSmall,
                "The OnSaleCell's price too small.(expected: >= {}, current: {})",
                _min_price,
                sale_price
            );
        } else if action == b"cancel_account_sale" {
            debug!("Route to cancel_account_sale action ...");
            assert!(
                input_on_sale_cells.len() == 1 && output_on_sale_cells.len() == 0,
                Error::OnSaleCellNumberInvalid,
                "There should be zero OnSaleCell in output and zero OnSaleCell in input.");
            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            verify_on_sale_cell_args_equal_account_id(config_main,input_on_sale_cells,Source::Input)?;
        }
    } else {
        return Err(Error::ActionNotSupported)
    }
    Ok(())
}

fn verify_on_sale_cell_args_equal_account_id(
    config_main: ConfigCellMainReader,
    on_sale_cells: Vec<usize>,
    source: Source
) -> Result<(), Error> {
    let account_cell_type_id = config_main.type_id_table().account_cell();
    let output_account_cells = util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, Source::Output)?;

    assert!(output_account_cells.len() == 1, Error::OnSaleCellAccountCellMustOne, "Output must include one account_cell");

    // read account_id from output accountCell
    let output_account_cell_index = output_account_cells[0];
    let output_data = util::load_cell_data(output_account_cell_index, Source::Output)?;
    let account_id = data_parser::account_cell::get_id(&output_data);

    // read args from onSaleCell's type_script
    let on_sale_cell_index = on_sale_cells[0];
    let output_type = high_level::load_cell_type(on_sale_cell_index, source)
        .map_err(|e| Error::from(e))?;
    let type_args = output_type.as_reader().args().raw_data();

    // ensure the onSaleCell's args equal to accountCell's id
    assert!(account_id == type_args, Error::OnSaleCellArgsInvalid, "OnSaleCell's args should equal to the accountCell");

    Ok(())
}

fn load_on_sale_cells() -> Result<(Vec<usize>, Vec<usize>), Error> {
    let this_type_script = high_level::load_script().map_err(|e| Error::from(e))?;
    let (input_on_sale_cells, output_on_sale_cells) =
        util::find_cells_by_script_in_inputs_and_outputs(
            ScriptType::Type,
            this_type_script.as_reader(),
        )?;
    Ok((input_on_sale_cells, output_on_sale_cells))
}