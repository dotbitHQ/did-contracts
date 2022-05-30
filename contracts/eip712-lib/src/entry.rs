use super::eip712::{to_semantic_address, to_semantic_capacity, verify_eip712_hashes_if_has_das_lock};
use alloc::{format, string::String};
use ckb_std::{ckb_constants::Source, error::SysError, high_level};
use das_core::{assert, constants::*, data_parser, debug, error::Error, util, warn, witness_parser::WitnessesParser};
use das_map::{map::Map, util as map_util};
use das_types::{
    constants::{DataType, LockRole},
    packed::*,
    prelude::*,
};

pub fn main() -> Result<(), Error> {
    debug!("====== EIP712 Lib ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(Error::ActionNotSupported),
    };
    let action = action_cp.as_slice();

    parser.parse_cell()?;

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
        b"declare_reverse_record" => declare_reverse_record_to_semantic,
        b"redeclare_reverse_record" => redeclare_reverse_record_to_semantic,
        b"retract_reverse_record" => retract_reverse_record_to_semantic,
        b"lock_account_for_cross_chain" => lock_account_for_cross_chain_to_semantic,
        _ => transfer_to_semantic,
    };

    verify_eip712_hashes_if_has_das_lock(&parser, func)?;

    Ok(())
}

fn transfer_account_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let (input_cells, output_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id_table_reader.account_cell())?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(input_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| Error::EIP712SerializationError)?;

    // Parse from address from the AccountCell's lock script in inputs.
    // let from_lock = high_level::load_cell_lock(input_cells[0], Source::Input)?;
    // let from_address = to_semantic_address(from_lock.as_reader().into(), 1..21)?;
    // Parse to address from the AccountCell's lock script in outputs.
    let to_lock = high_level::load_cell_lock(output_cells[0], Source::Output)?;
    let to_address = to_semantic_address(parser, to_lock.as_reader().into(), LockRole::Owner)?;

    Ok(format!("TRANSFER THE ACCOUNT {} TO {}", account, to_address))
}

fn edit_manager_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let (input_cells, _output_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id_table_reader.account_cell())?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(input_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| Error::EIP712SerializationError)?;

    // TODO Improve semantic message of this transaction.
    Ok(format!("EDIT MANAGER OF ACCOUNT {}", account))
}

fn edit_records_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let (input_cells, _output_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id_table_reader.account_cell())?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(input_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| Error::EIP712SerializationError)?;

    // TODO Improve semantic message of this transaction.
    Ok(format!("EDIT RECORDS OF ACCOUNT {}", account))
}

fn start_account_sale_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
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
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| Error::EIP712SerializationError)?;

    let (version, _, witness) =
        parser.verify_and_get(DataType::AccountSaleCellData, account_sale_cells[0], Source::Output)?;

    let price = if version == 1 {
        let entity = AccountSaleCellDataV1::from_slice(witness.as_reader().raw_data()).map_err(|_| {
            warn!("EIP712 decoding AccountSaleCellData failed");
            Error::WitnessEntityDecodingError
        })?;
        to_semantic_capacity(u64::from(entity.price()))
    } else {
        let entity = AccountSaleCellData::from_slice(witness.as_reader().raw_data()).map_err(|_| {
            warn!("EIP712 decoding AccountSaleCellData failed");
            Error::WitnessEntityDecodingError
        })?;
        to_semantic_capacity(u64::from(entity.price()))
    };

    Ok(format!("SELL {} FOR {}", account, price))
}

fn edit_account_sale_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
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
            Error::WitnessEntityDecodingError
        })?;
        to_semantic_capacity(u64::from(entity.price()))
    } else {
        let entity = AccountSaleCellData::from_slice(witness.as_reader().raw_data()).map_err(|_| {
            warn!("EIP712 decoding AccountSaleCellData failed");
            Error::WitnessEntityDecodingError
        })?;
        to_semantic_capacity(u64::from(entity.price()))
    };

    Ok(format!("EDIT SALE INFO, CURRENT PRICE IS {}", price))
}

fn cancel_account_sale_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let account_cells =
        util::find_cells_by_type_id(ScriptType::Type, type_id_table_reader.account_cell(), Source::Input)?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(account_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| Error::EIP712SerializationError)?;

    Ok(format!("CANCEL SALE OF {}", account))
}

fn buy_account_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
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
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| Error::EIP712SerializationError)?;

    let (version, _, witness) =
        parser.verify_and_get(DataType::AccountSaleCellData, account_sale_cells[0], Source::Input)?;

    let price = if version == 1 {
        let entity = AccountSaleCellDataV1::from_slice(witness.as_reader().raw_data()).map_err(|_| {
            warn!("EIP712 decoding AccountSaleCellData failed");
            Error::WitnessEntityDecodingError
        })?;
        to_semantic_capacity(u64::from(entity.price()))
    } else {
        let entity = AccountSaleCellData::from_slice(witness.as_reader().raw_data()).map_err(|_| {
            warn!("EIP712 decoding AccountSaleCellData failed");
            Error::WitnessEntityDecodingError
        })?;
        to_semantic_capacity(u64::from(entity.price()))
    };

    Ok(format!("BUY {} WITH {}", account, price))
}

fn offer_to_semantic(parser: &WitnessesParser, source: Source) -> Result<(String, String), Error> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let offer_cells = util::find_cells_by_type_id(ScriptType::Type, type_id_table_reader.offer_cell(), source)?;

    assert!(
        offer_cells.len() > 0,
        Error::InvalidTransactionStructure,
        "There should be at least 1 OfferCell in transaction."
    );

    let witness = util::parse_offer_cell_witness(parser, offer_cells[0], source)?;
    let witness_reader = witness.as_reader();

    let account = String::from_utf8(witness_reader.account().raw_data().to_vec()).map_err(|_| {
        warn!("EIP712 decoding OfferCellData failed");
        Error::WitnessEntityDecodingError
    })?;
    let amount = to_semantic_capacity(u64::from(witness_reader.price()));

    Ok((account, amount))
}

fn make_offer_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let (account, amount) = offer_to_semantic(parser, Source::Output)?;
    Ok(format!("MAKE AN OFFER ON {} WITH {}", account, amount))
}

fn edit_offer_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let (_, old_amount) = offer_to_semantic(parser, Source::Input)?;
    let (account, new_amount) = offer_to_semantic(parser, Source::Output)?;
    Ok(format!(
        "CHANGE THE OFFER ON {} FROM {} TO {}",
        account, old_amount, new_amount
    ))
}

fn cancel_offer_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let offer_cells = util::find_cells_by_type_id(ScriptType::Type, type_id_table_reader.offer_cell(), Source::Input)?;

    Ok(format!("CANCEL {} OFFER(S)", offer_cells.len()))
}

fn accept_offer_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let (account, amount) = offer_to_semantic(parser, Source::Input)?;
    Ok(format!("ACCEPT THE OFFER ON {} WITH {}", account, amount))
}

fn reverse_record_to_semantic(parser: &WitnessesParser, source: Source) -> Result<(String, String), Error> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let reverse_record_cells =
        util::find_cells_by_type_id(ScriptType::Type, type_id_table_reader.reverse_record_cell(), source)?;
    debug!(
        "type_id_table_reader.reverse_record_cell() = {:?}",
        type_id_table_reader.reverse_record_cell()
    );
    assert!(
        reverse_record_cells.len() == 1,
        Error::InvalidTransactionStructure,
        "There should be 1 ReverseRecordCell in transaction."
    );

    let data = high_level::load_cell_data(reverse_record_cells[0], source).map_err(Error::from)?;
    let account = String::from_utf8(data).map_err(|_| Error::EIP712SerializationError)?;
    let lock = Script::from(high_level::load_cell_lock(reverse_record_cells[0], source).map_err(Error::from)?);
    let address = to_semantic_address(parser, lock.as_reader(), LockRole::Owner)?;

    Ok((address, account))
}

fn declare_reverse_record_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let (address, account) = reverse_record_to_semantic(parser, Source::Output)?;
    Ok(format!("DECLARE A REVERSE RECORD FROM {} TO {}", address, account))
}

fn redeclare_reverse_record_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let (address, account) = reverse_record_to_semantic(parser, Source::Output)?;
    Ok(format!("REDECLARE A REVERSE RECORD FROM {} TO {}", address, account))
}

fn retract_reverse_record_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let source = Source::Input;
    let reverse_record_cells =
        util::find_cells_by_type_id(ScriptType::Type, type_id_table_reader.reverse_record_cell(), source)?;
    let lock = Script::from(high_level::load_cell_lock(reverse_record_cells[0], source).map_err(Error::from)?);
    let address = to_semantic_address(parser, lock.as_reader(), LockRole::Owner)?;

    Ok(format!("RETRACT REVERSE RECORDS ON {}", address))
}

fn lock_account_for_cross_chain_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let account_cells =
        util::find_cells_by_type_id(ScriptType::Type, type_id_table_reader.account_cell(), Source::Input)?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(account_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| Error::EIP712SerializationError)?;

    Ok(format!("LOCK {} FOR CROSS CHAIN", account))
}

fn transfer_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    fn sum_cells(parser: &WitnessesParser, source: Source) -> Result<String, Error> {
        let mut i = 0;
        let mut capacity_map = Map::new();
        loop {
            let ret = high_level::load_cell_capacity(i, source);
            match ret {
                Ok(capacity) => {
                    let lock = Script::from(high_level::load_cell_lock(i, source).map_err(|e| Error::from(e))?);
                    let address = to_semantic_address(parser, lock.as_reader(), LockRole::Owner)?;
                    map_util::add(&mut capacity_map, address, capacity);
                }
                Err(SysError::IndexOutOfBound) => {
                    break;
                }
                Err(err) => {
                    return Err(Error::from(err));
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
