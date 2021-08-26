use super::data_parser;
use super::witness_parser::WitnessesParser;
use super::{debug, error::Error, util};
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{packed as ckb_packed, prelude::Unpack},
    error::SysError,
    high_level,
};
use das_types::{packed as das_packed, prelude::*};
use eip712::{hash_data, typed_data_v4, types::*};
use serde_json::Value;

pub fn calc_eip712_typed_data(
    parser: &WitnessesParser,
    action: das_packed::BytesReader,
    params: &[das_packed::BytesReader],
    digest: &str,
) -> Result<TypedDataV4, Error> {
    let type_id_table = parser.configs.main()?.type_id_table();

    let tx_action = extract_action(action, params)?;
    let (inputs_capacity, inputs) = extract_cells(parser, type_id_table, Source::Input)?;
    let (outputs_capacity, outputs) = extract_cells(parser, type_id_table, Source::Output)?;
    let inputs_capacity_str = shannon_to_ckb_str(inputs_capacity);
    let outputs_capacity_str = shannon_to_ckb_str(outputs_capacity);
    let fee_str = if outputs_capacity <= inputs_capacity {
        shannon_to_ckb_str(inputs_capacity - outputs_capacity)
    } else {
        format!(
            "-{}",
            shannon_to_ckb_str(outputs_capacity - inputs_capacity)
        )
    };

    let typed_data = typed_data_v4!({
        types: {
            EIP712Domain: [
                chainId: "uint256",
                name: "string",
                verifyingContract: "address",
                version: "string"
            ],
            Action: [
                action: "string",
                params: "string"
            ],
            Cell: [
              capacity: "string",
              lock: "string",
              type: "string",
              data: "string",
              extraData: "string"
            ],
            Transaction: [
              plainText: "string",
              inputsCapacity: "string",
              outputsCapacity: "string",
              fee: "string",
              action: "Action",
              inputs: "Cell[]",
              outputs: "Cell[]",
              digest: "bytes32"
            ]
        },
        primaryType: "Transaction",
        domain: {
            chainId: 1,
            name: "da.systems",
            verifyingContract: "0xb3dc32341ee4bae03c85cd663311de0b1b122955",
            version: "1"
        },
        message: {
            plainText: "Transfer account test.bit from A to B.",
            action: tx_action,
            inputsCapacity: inputs_capacity_str,
            outputsCapacity: outputs_capacity_str,
            fee: fee_str,
            inputs: inputs,
            outputs: outputs,
            digest: digest
        }
    });

    Ok(typed_data)
}

fn extract_action(
    action_in_bytes: das_packed::BytesReader,
    params_in_bytes: &[das_packed::BytesReader],
) -> Result<Value, Error> {
    let action = String::from_utf8(action_in_bytes.raw_data().to_vec())
        .map_err(|_| Error::EIP712SerializationError)?;

    let mut params = Vec::new();
    for param in params_in_bytes {
        if param.len() > 10 {
            params.push(format!("0x{}", util::hex_string(&param.raw_data()[..10])));
        } else {
            params.push(format!("0x{}", util::hex_string(param.raw_data())));
        }
    }

    Ok(Action::new(&action, &params.join(",")).into())
}

fn extract_cells(
    parser: &WitnessesParser,
    type_id_table: das_packed::TypeIdTableReader,
    source: Source,
) -> Result<(u64, Vec<Cell>), Error> {
    let mut i = 0;
    let mut cells: Vec<Cell> = Vec::new();
    let mut total_capacity = 0;
    debug!("source = {:?}", source);
    loop {
        let ret = high_level::load_cell(i, source);
        match ret {
            Ok(cell) => {
                debug!("i = {:?}", i);
                let type_opt = cell.type_().to_opt();
                let data_in_bytes = util::load_cell_data(i, source)?;
                let capacity_in_shannon = cell.capacity().unpack();

                total_capacity += capacity_in_shannon;

                // Skip normal cells which has no type script and data
                if type_opt.is_none() && data_in_bytes.len() <= 0 {
                    continue;
                }

                let capacity = shannon_to_ckb_str(capacity_in_shannon);
                let lock = script_to_str(cell.lock().as_reader());

                macro_rules! extract_and_push {
                    ($cell_data_to_str:ident, $cell_witness_to_str:ident, $type_:expr) => {
                        let data = $cell_data_to_str(&data_in_bytes)?;
                        let extra_data =
                            $cell_witness_to_str(parser, &data_in_bytes[..32], i, source)?;
                        cells.push(Cell::new(&capacity, &lock, &$type_, &data, &extra_data));
                    };
                }

                match type_opt {
                    Some(type_script) => {
                        let type_script_reader =
                            das_packed::ScriptReader::from(type_script.as_reader());
                        let type_ = script_to_str(type_script.as_reader());
                        match type_script_reader.code_hash() {
                            // Handle cells which with DAS type script.
                            x if util::is_reader_eq(x, type_id_table.account_cell()) => {
                                extract_and_push!(
                                    account_cell_data_to_str,
                                    account_cell_witness_to_str,
                                    type_
                                );
                            }
                            x if util::is_reader_eq(x, type_id_table.apply_register_cell()) => {
                                let data = apply_register_cell_data_to_str(&data_in_bytes)?;
                                cells.push(Cell::new(&capacity, &lock, &type_, &data, ""));
                            }
                            x if util::is_reader_eq(x, type_id_table.pre_account_cell()) => {}
                            x if util::is_reader_eq(x, type_id_table.income_cell()) => {}
                            // Handle cells which with unknown type script.
                            _ => {
                                let data = common_data_to_str(&data_in_bytes);
                                cells.push(Cell::new(&capacity, &lock, &type_, &data, ""));
                            }
                        }
                    }
                    // Handle cells which has no type script.
                    _ => {
                        let data = common_data_to_str(&data_in_bytes);
                        cells.push(Cell::new(&capacity, &lock, "", &data, ""));
                    }
                }
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

    Ok((total_capacity, cells))
}

fn shannon_to_ckb_str(capacity: u64) -> String {
    let capacity_str = capacity.to_string();
    let length = capacity_str.len();
    let mut ret = String::new();
    if length > 8 {
        let integer = &capacity_str[0..length - 8];
        let mut decimal = &capacity_str[length - 8..length];
        decimal = decimal.trim_end_matches("0");
        if decimal.is_empty() {
            ret = ret + integer + " CKB";
        } else {
            ret = ret + integer + "." + decimal + " CKB";
        }
    } else {
        let decimal = capacity_str.trim_end_matches("0");
        ret = ret + "0." + decimal + " CKB";
    }

    ret
}

fn script_to_str(script: ckb_packed::ScriptReader) -> String {
    let code_hash = util::hex_string(&script.code_hash().raw_data().as_ref()[0..20]);
    let hash_type = util::hex_string(script.hash_type().as_slice());
    let args_in_bytes = script.args().raw_data();
    let args = if args_in_bytes.len() > 20 {
        util::hex_string(&args_in_bytes[0..20])
    } else {
        util::hex_string(args_in_bytes.as_ref())
    };

    String::new() + "0x" + &code_hash + "...,0x" + &hash_type + ",0x" + &args
}

fn common_data_to_str(data_in_bytes: &[u8]) -> String {
    if data_in_bytes.len() > 20 {
        util::hex_string(&data_in_bytes[0..20])
    } else {
        util::hex_string(data_in_bytes)
    }
}

fn account_cell_data_to_str(data_in_bytes: &[u8]) -> Result<String, Error> {
    let account_in_bytes = data_parser::account_cell::get_account(data_in_bytes);
    let expired_at = data_parser::account_cell::get_expired_at(data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec())
        .map_err(|_| Error::EIP712SerializationError)?;
    Ok(format!(
        "{{ account: {}, expired_at: {} }}",
        account,
        &expired_at.to_string()
    ))
}

fn account_cell_witness_to_str(
    parser: &WitnessesParser,
    expected_hash: &[u8],
    index: usize,
    source: Source,
) -> Result<String, Error> {
    let (_, _, entity) = parser.verify_with_hash_and_get(expected_hash, index, source)?;
    let witness = das_packed::AccountCellData::from_slice(entity.as_reader().raw_data())
        .map_err(|_| Error::WitnessEntityDecodingError)?;
    let witness_reader = witness.as_reader();

    let status = u8::from(witness_reader.status());
    let records_hash = util::blake2b_256(witness_reader.records().as_slice());

    Ok(format!(
        "{{ status: {}, records_hash: {} }}",
        status,
        util::hex_string(&records_hash)
    ))
}

fn apply_register_cell_data_to_str(data_in_bytes: &[u8]) -> Result<String, Error> {
    let height = data_parser::apply_register_cell::get_height(data_in_bytes);
    let timestamp = data_parser::apply_register_cell::get_timestamp(data_in_bytes);
    Ok(format!(
        "{{ height: {}, timestamp: {} }}",
        height.to_string(),
        timestamp.to_string()
    ))
}

fn pre_account_cell_data_to_str(data_in_bytes: &[u8]) -> Result<String, Error> {
    let id = data_parser::pre_account_cell::get_id(data_in_bytes);
    Ok(format!("{{ id: {} }}", util::hex_string(id)))
}

fn pre_account_cell_witness_to_str(
    parser: &WitnessesParser,
    expected_hash: &[u8],
    index: usize,
    source: Source,
) -> Result<String, Error> {
    let (_, _, entity) = parser.verify_with_hash_and_get(expected_hash, index, source)?;
    let witness = das_packed::PreAccountCellData::from_slice(entity.as_reader().raw_data())
        .map_err(|_| Error::WitnessEntityDecodingError)?;
    let witness_reader = witness.as_reader();

    let account = String::from_utf8(witness_reader.account().as_readable())
        .map_err(|_| Error::EIP712SerializationError)?;
    let refund_lock = script_to_str(witness_reader.refund_lock().into());
    let owner_lock_args = util::hex_string(witness_reader.owner_lock_args().raw_data());

    Ok(format!(
        "{{ account: {}, owner_lock_args: 0x{}, refund_lock: {} }}",
        account, owner_lock_args, refund_lock
    ))
}
