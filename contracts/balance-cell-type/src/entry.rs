use alloc::vec::Vec;
use ckb_std::{ckb_constants::Source, high_level};
use das_core::{
    constants::{das_lock, DasLockType, ScriptType, TypeScript},
    data_parser, debug,
    eip712::verify_eip712_hashes,
    error::Error,
    util, warn,
    witness_parser::WitnessesParser,
};
use das_types::{constants::DataType, packed as das_packed};

pub fn main() -> Result<(), Error> {
    debug!("====== Running balance-cell-type ======");

    let das_lock = das_lock();
    let das_lock_reader = das_lock.as_reader();

    let (input_cells, output_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Lock, das_lock_reader.code_hash().into())?;

    if input_cells.len() > 0 {
        debug!("Check if cells with das-lock in inputs has correct typed data hash in its signature witness.");

        let mut parser = WitnessesParser::new()?;
        let action_opt = parser.parse_action_with_params()?;
        if action_opt.is_none() {
            return Err(Error::ActionNotSupported);
        }

        let (action_raw, params_raw) = action_opt.unwrap();
        let params = params_raw.iter().map(|param| param.as_reader()).collect::<Vec<_>>();

        parser.parse_config(&[DataType::ConfigCellMain])?;
        parser.parse_cell()?;

        if action_raw.as_reader().raw_data() == b"buy_account" {
            util::require_type_script(
                &mut parser,
                TypeScript::AccountSaleCellType,
                Source::Input,
                Error::InvalidTransactionStructure,
            )?;
        }

        verify_eip712_hashes(&parser, action_raw.as_reader(), &params)?;
    } else {
        debug!("Skip check typed data hashes, because no BalanceCell in inputs.")
    }

    if output_cells.len() > 0 {
        debug!("Check if any cells with das-lock in outputs lack of one of balance-cell-type, account-cell-type, account-sale-cell-type, account-auction-cell-type.");

        let this_script = high_level::load_script().map_err(|e| Error::from(e))?;
        let this_script_reader = this_script.as_reader();

        let mut available_type_scripts: Vec<das_packed::Script> = Vec::new();
        for index in output_cells {
            let lock = high_level::load_cell_lock(index, Source::Output).map_err(Error::from)?;
            let lock_args = lock.as_reader().args().raw_data();
            let owner_type = data_parser::das_lock_args::get_owner_type(lock_args);
            let manager_type = data_parser::das_lock_args::get_owner_type(lock_args);

            // Check if cells with das-lock in outputs also has the type script named balance-cell-type, account-cell-type, account-sale-cell-type, account-auction-cell-type..
            if owner_type == DasLockType::ETHTypedData as u8 || manager_type == DasLockType::ETHTypedData as u8 {
                let type_opt = high_level::load_cell_type(index, Source::Output).map_err(Error::from)?;
                match type_opt {
                    Some(type_) => {
                        let mut pass = false;
                        if util::is_reader_eq(this_script_reader, type_.as_reader()) {
                            pass = true;
                        } else {
                            if available_type_scripts.is_empty() {
                                debug!("Try to load type ID table from ConfigCellMain, because found some cells with das-lock not using balance-cell-type.");
                                let mut parser = WitnessesParser::new()?;
                                parser.parse_config(&[DataType::ConfigCellMain])?;

                                macro_rules! push_type_script {
                                    ($type_id_name:ident) => {
                                        let type_id = parser.configs.main()?.type_id_table().$type_id_name();
                                        let type_script = util::type_id_to_script(type_id);
                                        available_type_scripts.push(type_script);
                                    };
                                }

                                push_type_script!(account_cell);
                                push_type_script!(account_sale_cell);
                                push_type_script!(account_auction_cell);
                                push_type_script!(offer_cell);
                                push_type_script!(reverse_record_cell);
                            }

                            for script in available_type_scripts.iter() {
                                if util::is_type_id_equal(script.as_reader().into(), type_.as_reader()) {
                                    pass = true;
                                }
                            }
                        }

                        if !pass {
                            warn!("Outputs[{}] This cell has das-lock, so it should also has one of the specific type scripts.", index);
                            return Err(Error::BalanceCellFoundSomeOutputsLackOfType);
                        }
                    }
                    _ => {
                        warn!("Outputs[{}] This cell has das-lock, so it should also has one of the specific type scripts.", index);
                        return Err(Error::BalanceCellFoundSomeOutputsLackOfType);
                    }
                }
            }
        }
    }

    Ok(())
}
