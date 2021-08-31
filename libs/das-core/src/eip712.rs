use super::{
    constants::{config_cell_type, das_lock, ScriptType},
    data_parser, debug,
    error::Error,
    util,
    witness_parser::WitnessesParser,
};
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use bech32::{self, ToBase32, Variant};
use ckb_std::{ckb_constants::Source, ckb_types::prelude::Unpack, error::SysError, high_level};
use core::ops::Range;
use das_map::{map::Map, util::add};
use das_types::{packed as das_packed, prelude::*};
use eip712::{hash_data, typed_data_v4, types::*};
use serde_json::Value;
use std::prelude::v1::*;

pub fn gen_eip712_hash_from_tx(
    parser: &WitnessesParser,
    action: das_packed::BytesReader,
    params: &[das_packed::BytesReader],
) -> Result<Vec<u8>, Error> {
    let digest = "0x4eb68a6707ae16ce24fde8e5964f9f04c5a4abf9884f67b9425a5e1e65968119";
    let typed_data = tx_to_eip712_typed_data(&parser, action, &params, digest)?;

    let data = hash_data(typed_data).unwrap();
    debug!("data = {}", util::hex_string(&data));

    Ok(data)
}

pub fn tx_to_eip712_typed_data(
    parser: &WitnessesParser,
    action: das_packed::BytesReader,
    params: &[das_packed::BytesReader],
    digest: &str,
) -> Result<TypedDataV4, Error> {
    let type_id_table = parser.configs.main()?.type_id_table();

    let plain_text = tx_to_plaintext(parser, action, params)?;
    let tx_action = to_typed_action(action, params)?;
    let (inputs_capacity, inputs) = to_typed_cells(parser, type_id_table, Source::Input)?;
    let (outputs_capacity, outputs) = to_typed_cells(parser, type_id_table, Source::Output)?;
    let inputs_capacity_str = to_semantic_capacity(inputs_capacity);
    let outputs_capacity_str = to_semantic_capacity(outputs_capacity);
    let fee_str = if outputs_capacity <= inputs_capacity {
        to_semantic_capacity(inputs_capacity - outputs_capacity)
    } else {
        format!("-{}", to_semantic_capacity(outputs_capacity - inputs_capacity))
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
            plainText: plain_text,
            action: tx_action,
            inputsCapacity: inputs_capacity_str,
            outputsCapacity: outputs_capacity_str,
            fee: fee_str,
            inputs: inputs,
            outputs: outputs,
            digest: digest
        }
    });

    debug!("Extracted typed data: {}", typed_data);
    debug!("Attention! Because of compiling problem with the serde_json's preserve_order feature, the fields of the JSON needs to be resort manually when debugging.");

    Ok(typed_data)
}

fn tx_to_plaintext(
    _parser: &WitnessesParser,
    action_in_bytes: das_packed::BytesReader,
    _params_in_bytes: &[das_packed::BytesReader],
) -> Result<String, Error> {
    let ret;
    match action_in_bytes.raw_data() {
        b"transfer_account" => ret = transfer_account_to_semantic()?,
        b"transfer" | b"withdraw_from_wallet" => ret = transfer_to_semantic()?,
        _ => return Err(Error::ActionNotSupported),
    }

    Ok(ret)
}

fn transfer_account_to_semantic() -> Result<String, Error> {
    let this_type_script = high_level::load_script().map_err(|e| Error::from(e))?;
    let (input_cells, output_cells) =
        util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, this_type_script.as_reader())?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(input_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| Error::EIP712SerializationError)?;

    // Parse from address from the AccountCell's lock script in inputs.
    let from_lock = high_level::load_cell_lock(input_cells[0], Source::Input).map_err(|e| Error::from(e))?;
    let from_address = to_semantic_address(from_lock.as_reader().into(), 1..21)?;
    // Parse to address from the AccountCell's lock script in outputs.
    let to_lock = high_level::load_cell_lock(output_cells[0], Source::Output).map_err(|e| Error::from(e))?;
    let to_address = to_semantic_address(to_lock.as_reader().into(), 1..21)?;

    Ok(format!(
        "Transfer the account {} from {} to {}.",
        account, from_address, to_address
    ))
}

fn transfer_to_semantic() -> Result<String, Error> {
    fn sum_cells(source: Source) -> Result<String, Error> {
        let mut i = 0;
        let mut capacity_map = Map::new();
        loop {
            let ret = high_level::load_cell_capacity(i, source);
            match ret {
                Ok(capacity) => {
                    let lock =
                        das_packed::Script::from(high_level::load_cell_lock(i, source).map_err(|e| Error::from(e))?);
                    let address = to_semantic_address(lock.as_reader(), 0..1)?;
                    add(&mut capacity_map, address, capacity);
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

    let inputs = sum_cells(Source::Input)?;
    let outputs = sum_cells(Source::Output)?;

    Ok(format!("Transfer from {} to {}.", inputs, outputs))
}

fn to_semantic_address(lock_reader: das_packed::ScriptReader, range: Range<usize>) -> Result<String, Error> {
    let das_lock = das_packed::Script::from(das_lock());
    let das_lock_reader = das_lock.as_reader();

    #[cfg(feature = "mainnet")]
    let hrp = "ckb";
    #[cfg(not(feature = "mainnet"))]
    let hrp = "ckt";

    let address;
    if util::is_reader_eq(das_lock_reader.code_hash(), lock_reader.code_hash()) {
        // If this is a das-lock, convert it to address base on args.
        let args_in_bytes = lock_reader.args().raw_data();
        match args_in_bytes[0] {
            0 => {
                let pubkey_hash = args_in_bytes[range].to_vec();

                // The first byte is address type, 0x01 is for short address.
                // The second byte is CodeHashIndex, 0x00 is for SECP256K1 + blake160.
                let mut data = vec![1u8, 0];
                // This is the payload of address.
                data = [data, pubkey_hash].concat();

                let value = bech32::encode(&hrp.to_string(), data.to_base32(), Variant::Bech32)
                    .map_err(|_| Error::EIP712SematicError)?;
                address = format!("CKB:{}", value)
            }
            4 => {
                address = format!("TRX:0x{}", util::hex_string(&args_in_bytes[range]));
            }
            3 | 5 => {
                address = format!("ETH:0x{}", util::hex_string(&args_in_bytes[range]));
            }
            _ => return Err(Error::EIP712SematicError),
        }
    } else {
        // If this is a unknown lock, convert it to full address.
        let hash_type: Vec<u8> = if lock_reader.hash_type().as_slice()[0] == 0 {
            vec![2]
        } else {
            vec![4]
        };
        let code_hash = lock_reader.code_hash().raw_data().to_vec();
        let args = lock_reader.args().raw_data().to_vec();

        // This is the payload of address.
        let data = [hash_type, code_hash, args].concat();

        let value = bech32::encode(&hrp.to_string(), data.to_base32(), Variant::Bech32)
            .map_err(|_| Error::EIP712SematicError)?;
        address = format!("CKB:{}", value)
    }

    Ok(address)
}

fn to_typed_action(
    action_in_bytes: das_packed::BytesReader,
    params_in_bytes: &[das_packed::BytesReader],
) -> Result<Value, Error> {
    let action = String::from_utf8(action_in_bytes.raw_data().to_vec()).map_err(|_| Error::EIP712SerializationError)?;

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

fn to_typed_cells(
    parser: &WitnessesParser,
    type_id_table_reader: das_packed::TypeIdTableReader,
    source: Source,
) -> Result<(u64, Vec<Cell>), Error> {
    let mut i = 0;
    let mut cells: Vec<Cell> = Vec::new();
    let mut total_capacity = 0;
    let das_lock = das_packed::Script::from(das_lock());
    let config_cell_type = das_packed::Script::from(config_cell_type());
    loop {
        let ret = high_level::load_cell(i, source);
        match ret {
            Ok(cell) => {
                let type_opt = cell.type_().to_opt();
                let data_in_bytes = util::load_cell_data(i, source)?;
                let capacity_in_shannon = cell.capacity().unpack();

                total_capacity += capacity_in_shannon;

                // Skip normal cells which has no type script and data
                if type_opt.is_none() && data_in_bytes.len() <= 0 {
                    continue;
                }

                let capacity = to_semantic_capacity(capacity_in_shannon);
                let lock = to_typed_script(
                    type_id_table_reader,
                    config_cell_type.as_reader().code_hash(),
                    das_lock.as_reader().code_hash(),
                    das_packed::ScriptReader::from(cell.lock().as_reader()),
                );

                macro_rules! extract_and_push {
                    ($cell_data_to_str:ident, $cell_witness_to_str:ident, $type_:expr) => {
                        let data = $cell_data_to_str(&data_in_bytes)?;
                        let extra_data = $cell_witness_to_str(parser, &data_in_bytes[..32], i, source)?;
                        cells.push(Cell::new(&capacity, &lock, &$type_, &data, &extra_data));
                    };
                }

                match type_opt {
                    Some(type_script) => {
                        let type_script_reader = das_packed::ScriptReader::from(type_script.as_reader());
                        let type_ = to_typed_script(
                            type_id_table_reader,
                            config_cell_type.as_reader().code_hash(),
                            das_lock.as_reader().code_hash(),
                            das_packed::ScriptReader::from(type_script.as_reader()),
                        );
                        match type_script_reader.code_hash() {
                            // Handle cells which with DAS type script.
                            x if util::is_reader_eq(x, type_id_table_reader.account_cell()) => {
                                extract_and_push!(to_semantic_account_cell_data, to_semantic_account_witness, type_);
                            }
                            // Handle cells which with unknown type script.
                            _ => {
                                let data = to_typed_common_data(&data_in_bytes);
                                cells.push(Cell::new(&capacity, &lock, &type_, &data, ""));
                            }
                        }
                    }
                    // Handle cells which has no type script.
                    _ => {
                        let data = to_typed_common_data(&data_in_bytes);
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

fn to_semantic_capacity(capacity: u64) -> String {
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

fn to_typed_script(
    type_id_table_reader: das_packed::TypeIdTableReader,
    config_cell_type: das_packed::HashReader,
    das_lock: das_packed::HashReader,
    script: das_packed::ScriptReader,
) -> String {
    let code_hash = match script.code_hash() {
        x if util::is_reader_eq(x, type_id_table_reader.pre_account_cell()) => String::from("pre-account-cell-type"),
        x if util::is_reader_eq(x, type_id_table_reader.apply_register_cell()) => {
            String::from("apply-register-cell-type")
        }
        x if util::is_reader_eq(x, type_id_table_reader.account_cell()) => String::from("account-cell-type"),
        x if util::is_reader_eq(x, type_id_table_reader.proposal_cell()) => String::from("proposal-cell-type"),
        x if util::is_reader_eq(x, type_id_table_reader.income_cell()) => String::from("income-cell-type"),
        x if util::is_reader_eq(x, config_cell_type) => String::from("config-cell-type"),
        x if util::is_reader_eq(x, das_lock) => String::from("das-lock"),
        _ => format!(
            "0x{}...",
            util::hex_string(&script.code_hash().raw_data().as_ref()[0..20])
        ),
    };

    let hash_type = util::hex_string(script.hash_type().as_slice());
    let args_in_bytes = script.args().raw_data();
    let args = if args_in_bytes.len() > 20 {
        util::hex_string(&args_in_bytes[0..20]) + "..."
    } else {
        util::hex_string(args_in_bytes.as_ref())
    };

    String::new() + &code_hash + ",0x" + &hash_type + ",0x" + &args
}

fn to_typed_common_data(data_in_bytes: &[u8]) -> String {
    if data_in_bytes.len() > 20 {
        util::hex_string(&data_in_bytes[0..20]) + "..."
    } else {
        util::hex_string(data_in_bytes)
    }
}

fn to_semantic_account_cell_data(data_in_bytes: &[u8]) -> Result<String, Error> {
    let account_in_bytes = data_parser::account_cell::get_account(data_in_bytes);
    let expired_at = data_parser::account_cell::get_expired_at(data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| Error::EIP712SerializationError)?;
    Ok(format!(
        "{{ account: {}, expired_at: {} }}",
        account,
        &expired_at.to_string()
    ))
}

fn to_semantic_account_witness(
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{constants::*, util};
    use das_types::{packed as das_packed, prelude::*};

    #[test]
    fn test_to_semantic_address() {
        // 0x00/cde11acafefadb5cb437eb33ab8bbca958ad2a86/00/cde11acafefadb5cb437eb33ab8bbca958ad2a86
        let mut lock: das_packed::Script = das_lock().into();
        lock = lock
            .as_builder()
            .args(das_packed::Bytes::from(vec![
                0, 205, 225, 26, 202, 254, 250, 219, 92, 180, 55, 235, 51, 171, 139, 188, 169, 88, 173, 42, 134, 0,
                205, 225, 26, 202, 254, 250, 219, 92, 180, 55, 235, 51, 171, 139, 188, 169, 88, 173, 42, 134,
            ]))
            .build();

        let expected = "CKB:ckt1qyqvmcg6etl04k6uksm7kvat3w72jk9d92rq5tn6px";
        let address = to_semantic_address(lock.as_reader(), 1..21).unwrap();
        assert_eq!(&address, expected);

        // 0x03/94770827e8897417a2bbaf072c71c12f5a003278/03/94770827e8897417a2bbaf072c71c12f5a003278
        let mut lock: das_packed::Script = das_lock().into();
        lock = lock
            .as_builder()
            .args(das_packed::Bytes::from(vec![
                3, 148, 119, 8, 39, 232, 137, 116, 23, 162, 187, 175, 7, 44, 113, 193, 47, 90, 0, 50, 120, 3, 148, 119,
                8, 39, 232, 137, 116, 23, 162, 187, 175, 7, 44, 113, 193, 47, 90, 0, 50, 120,
            ]))
            .build();

        let expected = "ETH:0x94770827e8897417a2bbaf072c71c12f5a003278";
        let address = to_semantic_address(lock.as_reader(), 1..21).unwrap();
        assert_eq!(&address, expected);

        // 0x04/e4d75a3e74a3bc8199b48ff76d984b3a5bb1e218/04/e4d75a3e74a3bc8199b48ff76d984b3a5bb1e218
        let mut lock: das_packed::Script = das_lock().into();
        lock = lock
            .as_builder()
            .args(das_packed::Bytes::from(vec![
                4, 228, 215, 90, 62, 116, 163, 188, 129, 153, 180, 143, 247, 109, 152, 75, 58, 91, 177, 226, 24, 4,
                228, 215, 90, 62, 116, 163, 188, 129, 153, 180, 143, 247, 109, 152, 75, 58, 91, 177, 226, 24,
            ]))
            .build();

        let expected = "TRX:0xe4d75a3e74a3bc8199b48ff76d984b3a5bb1e218";
        let address = to_semantic_address(lock.as_reader(), 1..21).unwrap();
        assert_eq!(&address, expected);

        // 0x05/94770827e8897417a2bbaf072c71c12f5a003278/05/94770827e8897417a2bbaf072c71c12f5a003278
        let mut lock: das_packed::Script = das_lock().into();
        lock = lock
            .as_builder()
            .args(das_packed::Bytes::from(vec![
                5, 148, 119, 8, 39, 232, 137, 116, 23, 162, 187, 175, 7, 44, 113, 193, 47, 90, 0, 50, 120, 5, 148, 119,
                8, 39, 232, 137, 116, 23, 162, 187, 175, 7, 44, 113, 193, 47, 90, 0, 50, 120,
            ]))
            .build();

        let expected = "ETH:0x94770827e8897417a2bbaf072c71c12f5a003278";
        let address = to_semantic_address(lock.as_reader(), 1..21).unwrap();
        assert_eq!(&address, expected);
    }

    #[test]
    fn test_to_typed_script() {
        let account_cell_type_id = das_packed::Hash::from([1u8; 32]);
        let table_id_table = das_packed::TypeIdTable::new_builder()
            .account_cell(account_cell_type_id.clone())
            .build();
        let das_lock = das_packed::Script::from(das_lock());
        let config_cell_type = das_packed::Script::from(config_cell_type());

        let account_type_script = das_packed::Script::new_builder()
            .code_hash(account_cell_type_id)
            .hash_type(das_packed::Byte::new(1))
            .args(das_packed::Bytes::default())
            .build();

        let expected = "account-cell-type,0x01,0x";
        let result = to_typed_script(
            table_id_table.as_reader(),
            config_cell_type.as_reader().code_hash(),
            das_lock.as_reader().code_hash(),
            account_type_script.as_reader(),
        );
        assert_eq!(result, expected);

        let other_type_script = das_packed::Script::new_builder()
            .code_hash(das_packed::Hash::from([9u8; 32]))
            .hash_type(das_packed::Byte::new(1))
            .args(das_packed::Bytes::from(vec![10u8; 21]))
            .build();

        let expected =
            "0x0909090909090909090909090909090909090909...,0x01,0x0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a...";
        let result = to_typed_script(
            table_id_table.as_reader(),
            config_cell_type.as_reader().code_hash(),
            das_lock.as_reader().code_hash(),
            other_type_script.as_reader(),
        );
        assert_eq!(result, expected);

        let other_type_script = das_packed::Script::new_builder()
            .code_hash(das_packed::Hash::from([9u8; 32]))
            .hash_type(das_packed::Byte::new(1))
            .args(das_packed::Bytes::from(vec![10u8; 20]))
            .build();

        let expected = "0x0909090909090909090909090909090909090909...,0x01,0x0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a";
        let result = to_typed_script(
            table_id_table.as_reader(),
            config_cell_type.as_reader().code_hash(),
            das_lock.as_reader().code_hash(),
            other_type_script.as_reader(),
        );
        assert_eq!(result, expected);
    }
}
