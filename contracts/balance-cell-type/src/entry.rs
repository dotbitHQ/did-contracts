use alloc::boxed::Box;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::constants::ScriptType;
use das_core::error::*;
use das_core::witness_parser::WitnessesParser;
use das_core::{code_to_error, debug, util, verifiers};
use das_types::constants::{das_lock, TypeScript};

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running balance-cell-type ======");

    let balance_cell_type = high_level::load_script().map_err(|e| Error::<ErrorCode>::from(e))?;
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
            None => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
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
                    ErrorCode::InvalidTransactionStructure,
                )?;
            }
            b"start_account_sale" => {
                util::require_type_script(
                    &parser,
                    TypeScript::AccountSaleCellType,
                    Source::Output,
                    ErrorCode::InvalidTransactionStructure,
                )?;
            }
            b"cancel_account_sale" | b"buy_account" | b"edit_account_sale" => {
                util::require_type_script(
                    &parser,
                    TypeScript::AccountSaleCellType,
                    Source::Input,
                    ErrorCode::InvalidTransactionStructure,
                )?;
            }
            b"declare_reverse_record" => {
                util::require_type_script(
                    &parser,
                    TypeScript::ReverseRecordCellType,
                    Source::Output,
                    ErrorCode::InvalidTransactionStructure,
                )?;
            }
            b"redeclare_reverse_record" | b"retract_reverse_record" => {
                util::require_type_script(
                    &parser,
                    TypeScript::ReverseRecordCellType,
                    Source::Input,
                    ErrorCode::InvalidTransactionStructure,
                )?;
            }
            b"make_offer" | b"edit_offer" => {
                util::require_type_script(
                    &parser,
                    TypeScript::OfferCellType,
                    Source::Output,
                    ErrorCode::InvalidTransactionStructure,
                )?;
            }
            b"cancel_offer" | b"accept_offer" => {
                util::require_type_script(
                    &parser,
                    TypeScript::OfferCellType,
                    Source::Input,
                    ErrorCode::InvalidTransactionStructure,
                )?;
            }
            b"enable_sub_account" | b"update_sub_account" => {
                util::require_type_script(
                    &parser,
                    TypeScript::SubAccountCellType,
                    Source::Output,
                    ErrorCode::InvalidTransactionStructure,
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
        let config_main_reader = parser.configs.main()?;
        verifiers::balance_cell::verify_das_lock_always_with_type(config_main_reader)?;
    }

    if input_cells.len() > 0 && is_unknown_action {
        util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
    }

    Ok(())
}
