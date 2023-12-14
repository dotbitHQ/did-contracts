use alloc::boxed::Box;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::config::Config;
use das_core::constants::ScriptType;
use das_core::error::*;
use das_core::{code_to_error, debug, util, verifiers, warn};
use das_types::constants::{das_lock, Action, TypeScript};
use witness_parser::WitnessesParserV1;

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running balance-cell-type ======");

    let parser = WitnessesParserV1::get_instance();
    parser.init().map_err(|_err| {
        warn!("{:?}", _err);
        code_to_error!(ErrorCode::WitnessDataDecodingError)
    })?;

    let balance_cell_type = high_level::load_script().map_err(|e| Error::<ErrorCode>::from(e))?;
    let balance_cell_type_reader = balance_cell_type.as_reader();
    let das_lock = das_lock();
    let das_lock_reader = das_lock.as_reader();

    let input_cells = util::find_cells_by_script(ScriptType::Type, balance_cell_type_reader, Source::Input)?;
    // We need to find all BalanceCells even it has no type script, so we use das-lock as the finding condition.
    let output_cells =
        util::find_cells_by_type_id(ScriptType::Lock, das_lock_reader.code_hash().into(), Source::Output)?;

    let mut is_unknown_action = false;

    if input_cells.len() > 0 {
        debug!("Check if cells with das-lock in inputs has correct typed data hash in its signature witness.");

        let parser = WitnessesParserV1::get_instance();
        parser
            .init()
            .map_err(|_err| code_to_error!(ErrorCode::WitnessDataDecodingError))?;

        // Because the semantic requirement of each action, some other type script is required to generate DAS_MESSAGE field in EIP712 properly.
        match parser.action {
            Action::TransferAccount | Action::EditManager | Action::EditRecords => {
                util::require_type_script(
                    TypeScript::AccountCellType,
                    Source::Input,
                    ErrorCode::InvalidTransactionStructure,
                )?;
            }
            Action::StartAccountSale => {
                util::require_type_script(
                    TypeScript::AccountSaleCellType,
                    Source::Output,
                    ErrorCode::InvalidTransactionStructure,
                )?;
            }
            Action::CancelAccountSale | Action::BuyAccount | Action::EditAccountSale => {
                util::require_type_script(
                    TypeScript::AccountSaleCellType,
                    Source::Input,
                    ErrorCode::InvalidTransactionStructure,
                )?;
            }
            Action::RetractReverseRecord => {
                util::require_type_script(
                    TypeScript::ReverseRecordCellType,
                    Source::Input,
                    ErrorCode::InvalidTransactionStructure,
                )?;
            }
            Action::MakeOffer | Action::EditOffer => {
                util::require_type_script(
                    TypeScript::OfferCellType,
                    Source::Output,
                    ErrorCode::InvalidTransactionStructure,
                )?;
            }
            Action::CancelOffer | Action::AcceptOffer => {
                util::require_type_script(
                    TypeScript::OfferCellType,
                    Source::Input,
                    ErrorCode::InvalidTransactionStructure,
                )?;
            }
            Action::EnableSubAccount | Action::UpdateSubAccount => {
                util::require_type_script(
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
        let config_main_reader = Config::get_instance().main()?;
        verifiers::balance_cell::verify_das_lock_always_with_type(config_main_reader)?;
    }

    if input_cells.len() > 0 && is_unknown_action {
        util::exec_by_type_id(TypeScript::EIP712Lib, &[])?;
    }

    Ok(())
}
