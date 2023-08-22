use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};

use ckb_std::ckb_constants::Source;
use ckb_std::error::SysError;
use ckb_std::high_level;
use das_core::constants::*;
use das_core::error::*;
use das_core::witness_parser::WitnessesParser;
use das_core::{assert, code_to_error, data_parser, debug, util, warn};
use das_map::map::Map;
use das_map::util as map_util;
use das_types::constants::{DataType, LockRole};
use das_types::mixer::AccountCellDataMixer;
use das_types::packed::*;
use das_types::prelude::*;
use eip712::util::to_semantic_capacity;

use super::eip712::{to_semantic_address, verify_eip712_hashes_if_has_das_lock};

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== EIP712 Lib ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    };
    let action = action_cp.as_slice();

    parser.parse_cell()?;

    debug!(
        "The action of the transaction is: {:?}",
        String::from_utf8(action.to_vec())
    );

    let func = match action {
        b"transfer_account" => transfer_account_to_semantic,
        b"edit_manager" => edit_manager_to_semantic,
        b"edit_records" => edit_records_to_semantic,
        b"start_account_sale" => start_account_sale_to_semantic,
        b"cancel_account_sale" => cancel_account_sale_to_semantic,
        b"buy_account" => buy_account_to_semantic,
        b"edit_account_sale" => edit_account_sale_to_semantic,
        b"make_offer" => make_offer_to_semantic,
        b"edit_offer" => edit_offer_to_semantic,
        b"cancel_offer" => cancel_offer_to_semantic,
        b"accept_offer" => accept_offer_to_semantic,
        b"retract_reverse_record" => retract_reverse_record_to_semantic,
        b"lock_account_for_cross_chain" => lock_account_for_cross_chain_to_semantic,
        b"create_approval" => create_approval_to_semantic,
        b"delay_approval" => delay_approval_to_semantic,
        _ => transfer_to_semantic,
    };

    verify_eip712_hashes_if_has_das_lock(&parser, func)?;

    Ok(())
}

fn transfer_account_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let (input_cells, output_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id_table_reader.account_cell())?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(input_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| ErrorCode::EIP712SerializationError)?;

    // Parse from address from the AccountCell's lock script in inputs.
    // let from_lock = high_level::load_cell_lock(input_cells[0], Source::Input)?;
    // let from_address = to_semantic_address(from_lock.as_reader().into(), 1..21)?;
    // Parse to address from the AccountCell's lock script in outputs.
    let to_lock = high_level::load_cell_lock(output_cells[0], Source::Output)?;
    let to_address = to_semantic_address(parser, to_lock.as_reader().into(), LockRole::Owner)?;

    Ok(format!("TRANSFER THE ACCOUNT {} TO {}", account, to_address))
}

fn edit_manager_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let (input_cells, _output_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id_table_reader.account_cell())?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(input_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| ErrorCode::EIP712SerializationError)?;

    // TODO Improve semantic message of this transaction.
    Ok(format!("EDIT MANAGER OF ACCOUNT {}", account))
}

fn edit_records_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let (input_cells, _output_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id_table_reader.account_cell())?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(input_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| ErrorCode::EIP712SerializationError)?;

    // TODO Improve semantic message of this transaction.
    Ok(format!("EDIT RECORDS OF ACCOUNT {}", account))
}

fn start_account_sale_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let account_cells =
        util::find_cells_by_type_id(ScriptType::Type, type_id_table_reader.account_cell(), Source::Input)?;
    let account_sale_cells = util::find_cells_by_type_id(
        ScriptType::Type,
        type_id_table_reader.account_sale_cell(),
        Source::Output,
    )?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(account_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| ErrorCode::EIP712SerializationError)?;

    let (version, _, witness) =
        parser.verify_and_get(DataType::AccountSaleCellData, account_sale_cells[0], Source::Output)?;

    let price = if version == 1 {
        let entity = AccountSaleCellDataV1::from_slice(witness.as_reader().raw_data()).map_err(|_| {
            warn!("EIP712 decoding AccountSaleCellData failed");
            ErrorCode::WitnessEntityDecodingError
        })?;
        to_semantic_capacity(u64::from(entity.price()))
    } else {
        let entity = AccountSaleCellData::from_slice(witness.as_reader().raw_data()).map_err(|_| {
            warn!("EIP712 decoding AccountSaleCellData failed");
            ErrorCode::WitnessEntityDecodingError
        })?;
        to_semantic_capacity(u64::from(entity.price()))
    };

    Ok(format!("SELL {} FOR {}", account, price))
}

fn edit_account_sale_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let account_sale_cells = util::find_cells_by_type_id(
        ScriptType::Type,
        type_id_table_reader.account_sale_cell(),
        Source::Output,
    )?;

    let (version, _, witness) =
        parser.verify_and_get(DataType::AccountSaleCellData, account_sale_cells[0], Source::Output)?;

    let price = if version == 1 {
        let entity = AccountSaleCellDataV1::from_slice(witness.as_reader().raw_data()).map_err(|_| {
            warn!("EIP712 decoding AccountSaleCellData failed");
            ErrorCode::WitnessEntityDecodingError
        })?;
        to_semantic_capacity(u64::from(entity.price()))
    } else {
        let entity = AccountSaleCellData::from_slice(witness.as_reader().raw_data()).map_err(|_| {
            warn!("EIP712 decoding AccountSaleCellData failed");
            ErrorCode::WitnessEntityDecodingError
        })?;
        to_semantic_capacity(u64::from(entity.price()))
    };

    Ok(format!("EDIT SALE INFO, CURRENT PRICE IS {}", price))
}

fn cancel_account_sale_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let account_cells =
        util::find_cells_by_type_id(ScriptType::Type, type_id_table_reader.account_cell(), Source::Input)?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(account_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| ErrorCode::EIP712SerializationError)?;

    Ok(format!("CANCEL SALE OF {}", account))
}

fn buy_account_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let account_cells =
        util::find_cells_by_type_id(ScriptType::Type, type_id_table_reader.account_cell(), Source::Input)?;
    let account_sale_cells = util::find_cells_by_type_id(
        ScriptType::Type,
        type_id_table_reader.account_sale_cell(),
        Source::Input,
    )?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(account_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| ErrorCode::EIP712SerializationError)?;

    let (version, _, witness) =
        parser.verify_and_get(DataType::AccountSaleCellData, account_sale_cells[0], Source::Input)?;

    let price = if version == 1 {
        let entity = AccountSaleCellDataV1::from_slice(witness.as_reader().raw_data()).map_err(|_| {
            warn!("EIP712 decoding AccountSaleCellData failed");
            ErrorCode::WitnessEntityDecodingError
        })?;
        to_semantic_capacity(u64::from(entity.price()))
    } else {
        let entity = AccountSaleCellData::from_slice(witness.as_reader().raw_data()).map_err(|_| {
            warn!("EIP712 decoding AccountSaleCellData failed");
            ErrorCode::WitnessEntityDecodingError
        })?;
        to_semantic_capacity(u64::from(entity.price()))
    };

    Ok(format!("BUY {} WITH {}", account, price))
}

fn offer_to_semantic(parser: &WitnessesParser, source: Source) -> Result<(String, String), Box<dyn ScriptError>> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let offer_cells = util::find_cells_by_type_id(ScriptType::Type, type_id_table_reader.offer_cell(), source)?;

    assert!(
        offer_cells.len() > 0,
        ErrorCode::InvalidTransactionStructure,
        "There should be at least 1 OfferCell in transaction."
    );

    let witness = util::parse_offer_cell_witness(parser, offer_cells[0], source)?;
    let witness_reader = witness.as_reader();

    let account = String::from_utf8(witness_reader.account().raw_data().to_vec()).map_err(|_| {
        warn!("EIP712 decoding OfferCellData failed");
        ErrorCode::WitnessEntityDecodingError
    })?;
    let amount = to_semantic_capacity(u64::from(witness_reader.price()));

    Ok((account, amount))
}

fn make_offer_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    let (account, amount) = offer_to_semantic(parser, Source::Output)?;
    Ok(format!("MAKE AN OFFER ON {} WITH {}", account, amount))
}

fn edit_offer_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    let (_, old_amount) = offer_to_semantic(parser, Source::Input)?;
    let (account, new_amount) = offer_to_semantic(parser, Source::Output)?;
    Ok(format!(
        "CHANGE THE OFFER ON {} FROM {} TO {}",
        account, old_amount, new_amount
    ))
}

fn cancel_offer_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let offer_cells = util::find_cells_by_type_id(ScriptType::Type, type_id_table_reader.offer_cell(), Source::Input)?;

    Ok(format!("CANCEL {} OFFER(S)", offer_cells.len()))
}

fn accept_offer_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    let (account, amount) = offer_to_semantic(parser, Source::Input)?;
    Ok(format!("ACCEPT THE OFFER ON {} WITH {}", account, amount))
}

fn retract_reverse_record_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let source = Source::Input;
    let reverse_record_cells =
        util::find_cells_by_type_id(ScriptType::Type, type_id_table_reader.reverse_record_cell(), source)?;
    let lock =
        Script::from(high_level::load_cell_lock(reverse_record_cells[0], source).map_err(Error::<ErrorCode>::from)?);
    let address = to_semantic_address(parser, lock.as_reader(), LockRole::Owner)?;

    Ok(format!("RETRACT REVERSE RECORDS ON {}", address))
}

fn lock_account_for_cross_chain_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let account_cells =
        util::find_cells_by_type_id(ScriptType::Type, type_id_table_reader.account_cell(), Source::Input)?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(account_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| ErrorCode::EIP712SerializationError)?;

    Ok(format!("LOCK {} FOR CROSS CHAIN", account))
}

fn parse_approval_tx_info(
    parser: &WitnessesParser,
) -> Result<(usize, usize, String, Box<dyn AccountCellDataMixer>), Box<dyn ScriptError>> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let (input_cells, output_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id_table_reader.account_cell())?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(input_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| ErrorCode::EIP712SerializationError)?;

    let witness = util::parse_account_cell_witness(parser, output_cells[0], Source::Output)?;

    Ok((input_cells[0], output_cells[0], account, witness))
}

fn create_approval_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    let (_, output_index, account, witness) = parse_approval_tx_info(parser)?;
    let witness_reader = witness.as_reader();
    let witness_reader = match witness_reader.try_into_latest() {
        Ok(reader) => reader,
        Err(_) => {
            warn!(
                "{:?}[{}] The AccountCell should be upgraded to the latest version.",
                Source::Output,
                output_index
            );
            return Err(code_to_error!(AccountCellErrorCode::WitnessParsingError));
        }
    };

    let approval_reader = witness_reader.approval();
    match approval_reader.action().raw_data() {
        b"transfer" => {
            let approval_params = AccountApprovalTransfer::from_compatible_slice(approval_reader.params().raw_data())
                .map_err(|e| {
                warn!(
                    "{:?}[{}] Decoding approval.params failed: {}",
                    Source::Output,
                    output_index,
                    e.to_string()
                );
                return code_to_error!(AccountCellErrorCode::WitnessParsingError);
            })?;

            let to_lock = approval_params.to_lock();
            let to_address = to_semantic_address(parser, to_lock.as_reader().into(), LockRole::Owner)?;
            let sealed_until = u64::from(approval_params.sealed_until());

            Ok(format!(
                "APPROVE TRANSFER {} TO {} AFTER {}",
                account, to_address, sealed_until
            ))
        }
        _ => {
            warn!(
                "{:?}[{}] Found unsupported approval action: {:?}",
                Source::Output,
                output_index,
                String::from_utf8(approval_reader.action().raw_data().to_vec())
            );
            return Err(code_to_error!(AccountCellErrorCode::ApprovalActionUndefined));
        }
    }
}

fn delay_approval_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    let (_, output_index, account, witness) = parse_approval_tx_info(parser)?;
    let witness_reader = witness.as_reader();
    let witness_reader = match witness_reader.try_into_latest() {
        Ok(reader) => reader,
        Err(_) => {
            warn!(
                "{:?}[{}] The AccountCell should be upgraded to the latest version.",
                Source::Output,
                output_index
            );
            return Err(code_to_error!(AccountCellErrorCode::WitnessParsingError));
        }
    };

    let approval_reader = witness_reader.approval();
    match approval_reader.action().raw_data() {
        b"transfer" => {
            let approval_params = AccountApprovalTransfer::from_compatible_slice(approval_reader.params().raw_data())
                .map_err(|e| {
                warn!(
                    "{:?}[{}] Decoding approval.params failed: {}",
                    Source::Output,
                    output_index,
                    e.to_string()
                );
                return code_to_error!(AccountCellErrorCode::WitnessParsingError);
            })?;

            let sealed_until = u64::from(approval_params.sealed_until());

            Ok(format!(
                "DELAY THE TRANSFER APPROVAL OF {} TO {}",
                account, sealed_until
            ))
        }
        _ => {
            warn!(
                "{:?}[{}] Found unsupported approval action: {:?}",
                Source::Output,
                output_index,
                String::from_utf8(approval_reader.action().raw_data().to_vec())
            );
            return Err(code_to_error!(AccountCellErrorCode::ApprovalActionUndefined));
        }
    }
}

fn transfer_to_semantic(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>> {
    fn sum_cells(parser: &WitnessesParser, source: Source) -> Result<String, Box<dyn ScriptError>> {
        let mut i = 0;
        let mut capacity_map = Map::new();
        loop {
            let ret = high_level::load_cell_capacity(i, source);
            match ret {
                Ok(capacity) => {
                    let lock =
                        Script::from(high_level::load_cell_lock(i, source).map_err(|e| Error::<ErrorCode>::from(e))?);
                    let address = to_semantic_address(parser, lock.as_reader(), LockRole::Owner)?;
                    map_util::add(&mut capacity_map, address, capacity);
                }
                Err(SysError::IndexOutOfBound) => {
                    break;
                }
                Err(err) => {
                    return Err(Error::<ErrorCode>::from(err).into());
                }
            }

            i += 1;
        }

        let mut comma = "";
        let mut ret = String::new();
        for (address, capacity) in capacity_map.items {
            ret += format!("{}{}({})", comma, address, to_semantic_capacity(capacity)).as_str();
            comma = ", ";
        }

        Ok(ret)
    }

    let inputs = sum_cells(parser, Source::Input)?;
    let outputs = sum_cells(parser, Source::Output)?;

    Ok(format!("TRANSFER FROM {} TO {}", inputs, outputs))
}
