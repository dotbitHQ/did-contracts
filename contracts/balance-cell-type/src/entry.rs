use alloc::vec::Vec;
use ckb_std::{ckb_constants::Source, high_level};
use das_core::{
    constants::{das_lock, DasLockType, ScriptType, TypeScript},
    data_parser, debug,
    error::Error,
    util, warn,
    witness_parser::WitnessesParser,
};
use das_types::packed as das_packed;

pub fn main() -> Result<(), Error> {
    debug!("====== Running balance-cell-type ======");

    let balance_cell_type = high_level::load_script().map_err(|e| Error::from(e))?;
    let balance_cell_type_reader = balance_cell_type.as_reader();
    let das_lock = das_lock();
    let das_lock_reader = das_lock.as_reader();

    let input_cells = util::find_cells_by_script(ScriptType::Type, balance_cell_type_reader, Source::Input)?;
    // We need to find all BalanceCells even it has no type script, so we use das-lock as the finding condition.
    let output_cells =
        util::find_cells_by_type_id(ScriptType::Lock, das_lock_reader.code_hash().into(), Source::Output)?;

    let mut parser = WitnessesParser::new()?;
    let mut is_unknown_action = false;

    if input_cells.len() > 0 {
        debug!("Check if cells with das-lock in inputs has correct typed data hash in its signature witness.");

        let action_cp = match parser.parse_action_with_params()? {
            Some((action, _)) => action.to_vec(),
            None => return Err(Error::ActionNotSupported),
        };
        let action = action_cp.as_slice();

        parser.parse_cell()?;

        // Because the semantic requirement of each action, some other type script is required to generate DAS_MESSAGE field in EIP712 properly.
        match action {
            b"transfer_account" | b"edit_manager" | b"edit_records" => {
                util::require_type_script(
                    &parser,
                    TypeScript::AccountCellType,
                    Source::Input,
                    Error::InvalidTransactionStructure,
                )?;
            }
            b"start_account_sale" => {
                util::require_type_script(
                    &parser,
                    TypeScript::AccountSaleCellType,
                    Source::Output,
                    Error::InvalidTransactionStructure,
                )?;
            }
            b"cancel_account_sale" | b"buy_account" | b"edit_account_sale" => {
                util::require_type_script(
                    &parser,
                    TypeScript::AccountSaleCellType,
                    Source::Input,
                    Error::InvalidTransactionStructure,
                )?;
            }
            b"declare_reverse_record" => {
                util::require_type_script(
                    &parser,
                    TypeScript::ReverseRecordCellType,
                    Source::Output,
                    Error::InvalidTransactionStructure,
                )?;
            }
            b"redeclare_reverse_record" | b"retract_reverse_record" => {
                util::require_type_script(
                    &parser,
                    TypeScript::ReverseRecordCellType,
                    Source::Input,
                    Error::InvalidTransactionStructure,
                )?;
            }
            b"make_offer" | b"edit_offer" => {
                util::require_type_script(
                    &parser,
                    TypeScript::OfferCellType,
                    Source::Output,
                    Error::InvalidTransactionStructure,
                )?;
            }
            b"cancel_offer" | b"accept_offer" => {
                util::require_type_script(
                    &parser,
                    TypeScript::OfferCellType,
                    Source::Input,
                    Error::InvalidTransactionStructure,
                )?;
            }
            b"enable_sub_account" | b"create_sub_account" | b"renew_sub_account" => {
                util::require_type_script(
                    &parser,
                    TypeScript::SubAccountCellType,
                    Source::Output,
                    Error::InvalidTransactionStructure,
                )?;
            }
            _ => {
                is_unknown_action = true;
            }
        }
    } else {
        debug!("Skip check typed data hashes, because no BalanceCell in inputs.")
    }

    if output_cells.len() > 0 {
        debug!("Check if any cells with das-lock in outputs lack of one of balance-cell-type, account-cell-type, account-sale-cell-type, account-auction-cell-type.");

        let mut available_type_scripts: Vec<das_packed::Script> = Vec::new();
        for index in output_cells {
            let lock = high_level::load_cell_lock(index, Source::Output)?;
            let lock_args = lock.as_reader().args().raw_data();
            let owner_type = data_parser::das_lock_args::get_owner_type(lock_args);
            let manager_type = data_parser::das_lock_args::get_owner_type(lock_args);

            // Check if cells with das-lock in outputs also has the type script named balance-cell-type, account-cell-type, account-sale-cell-type, account-auction-cell-type..
            if owner_type == DasLockType::ETHTypedData as u8 || manager_type == DasLockType::ETHTypedData as u8 {
                let type_opt = high_level::load_cell_type(index, Source::Output)?;
                match type_opt {
                    Some(type_) => {
                        let mut pass = false;
                        if util::is_reader_eq(balance_cell_type_reader, type_.as_reader()) {
                            pass = true;
                        } else {
                            if available_type_scripts.is_empty() {
                                debug!("Try to load type ID table from ConfigCellMain, because found some cells with das-lock not using balance-cell-type.");
                                let parser = WitnessesParser::new()?;

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

    if input_cells.len() > 0 && is_unknown_action {
        util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
    }

    Ok(())
}
