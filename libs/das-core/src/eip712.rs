use super::{
    assert,
    constants::{config_cell_type, das_lock, ScriptType, CKB_HASH_DIGEST, SECP_SIGNATURE_SIZE},
    data_parser, debug,
    error::Error,
    util, warn,
    witness_parser::WitnessesParser,
};
use crate::constants::EIP712_CHAINID_SIZE;
use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec::Vec,
};
use bech32::{self, ToBase32, Variant};
use bs58;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{
        packed as ckb_packed,
        prelude::{Pack, Unpack},
    },
    error::SysError,
    high_level,
};
use core::convert::TryInto;
use core::ops::Range;
use das_map::{map::Map, util::add};
use das_types::{constants::LockRole, packed as das_packed, prelude::*};
use eip712::{hash_data, typed_data_v4, types::*};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::prelude::v1::*;

#[cfg(feature = "mainnet")]
const HRP: &str = "ckb";
#[cfg(not(feature = "mainnet"))]
const HRP: &str = "ckt";

const TRX_ADDR_PREFIX: u8 = 0x41;

pub fn verify_eip712_hashes(
    parser: &WitnessesParser,
    action: das_packed::BytesReader,
    params: &[das_packed::BytesReader],
) -> Result<(), Error> {
    let required_role = util::get_action_required_role(action);
    let das_lock = das_lock();
    let das_lock_reader = das_lock.as_reader();

    let mut i = 0;
    let mut input_groups_idxs: BTreeMap<Vec<u8>, Vec<usize>> = BTreeMap::new();
    let this_type = high_level::load_script().map_err(|e| Error::from(e))?;
    let this_type_reader = this_type.as_reader();
    loop {
        let ret = high_level::load_cell_lock(i, Source::Input);
        match ret {
            Ok(lock) => {
                let lock_reader = lock.as_reader();
                // Only take care of inputs with das-lock
                if util::is_script_equal(das_lock_reader, lock_reader) {
                    let type_;
                    let args;
                    if required_role == LockRole::Manager {
                        type_ = data_parser::das_lock_args::get_manager_type(lock_reader.args().raw_data());
                        args = data_parser::das_lock_args::get_manager_lock_args(lock_reader.args().raw_data());
                    } else {
                        type_ = data_parser::das_lock_args::get_owner_type(lock_reader.args().raw_data());
                        args = data_parser::das_lock_args::get_owner_lock_args(lock_reader.args().raw_data());
                    }
                    if type_ != 5 {
                        debug!(
                            "Inputs[{}] Found deprecated address type, skip verification for hash.",
                            i
                        );
                    } else {
                        let type_opt = high_level::load_cell_type(i, Source::Input).map_err(|e| Error::from(e))?;
                        if let Some(type_script) = type_opt {
                            if util::is_script_equal(this_type_reader, type_script.as_reader()) {
                                input_groups_idxs.entry(args.to_vec()).or_default().push(i);
                            }
                        }
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

    // debug!("input_groups_idxs = {:?}", input_groups_idxs);
    if input_groups_idxs.is_empty() {
        debug!("There is no cell in inputs has das-lock with correct type byte, skip checking hashes in witnesses ...");
    } else {
        debug!("Check if hashes of typed data in witnesses is correct ...");

        // The variable `i` has added 1 at the end of loop above, so do not add 1 again here.
        let input_size = i;
        let (digest_and_hash, eip712_chain_id) = tx_to_digest(input_groups_idxs, input_size)?;
        let mut typed_data = tx_to_eip712_typed_data(&parser, action, &params, eip712_chain_id)?;
        for index in digest_and_hash.keys() {
            let item = digest_and_hash.get(index).unwrap();
            let digest = util::hex_string(&item.digest);
            typed_data.digest(&digest);
            let expected_hash = hash_data(&typed_data).unwrap();

            debug!(
                "Calculated hash of EIP712 typed data with digest.(digest: {}, hash: {})",
                digest,
                util::hex_string(&expected_hash)
            );

            assert!(
                &item.typed_data_hash == expected_hash.as_slice(),
                Error::EIP712SignatureError,
                "Inputs[{}] The hash of EIP712 typed data is mismatched.(current: {}, expected: {})",
                index,
                util::hex_string(&item.typed_data_hash),
                util::hex_string(&expected_hash)
            );
        }
    }

    Ok(())
}

struct DigestAndHash {
    digest: [u8; 32],
    typed_data_hash: [u8; 32],
}

fn tx_to_digest(
    input_groups_idxs: BTreeMap<Vec<u8>, Vec<usize>>,
    input_size: usize,
) -> Result<(BTreeMap<usize, DigestAndHash>, Vec<u8>), Error> {
    let mut ret: BTreeMap<usize, DigestAndHash> = BTreeMap::new();
    let mut eip712_chain_id = Vec::new();
    for (_key, input_group_idxs) in input_groups_idxs {
        let init_witness_idx = input_group_idxs[0];
        let witness_bytes = util::load_witnesses(init_witness_idx)?;
        // CAREFUL: This is only works for secp256k1_blake160_sighash_all, cause das-lock does not support secp256k1_blake160_multisig_all currently.
        let init_witness = ckb_packed::WitnessArgs::from_slice(&witness_bytes).map_err(|_| {
            warn!(
                "Inputs[{}] Witness can not be decoded as WitnessArgs.(data: {})",
                init_witness_idx,
                util::hex_string(&witness_bytes)
            );
            Error::EIP712DecodingWitnessArgsError
        })?;

        // Reset witness_args to empty status for calculation of digest.
        match init_witness.as_reader().lock().to_opt() {
            Some(lock_of_witness) => {
                // TODO Do not create empty_witness, this is an incorrect way.
                // The right way is loading it from witnesses array, and set the bytes in its lock to 0u8.
                let empty_signature = ckb_packed::BytesOpt::new_builder()
                    .set(Some(vec![0u8; SECP_SIGNATURE_SIZE].pack()))
                    .build();
                let empty_witness = ckb_packed::WitnessArgs::new_builder().lock(empty_signature).build();
                let tx_hash = high_level::load_tx_hash().map_err(|_| Error::ItemMissing)?;

                let mut blake2b = util::new_blake2b();
                blake2b.update(&tx_hash);
                blake2b.update(&(empty_witness.as_bytes().len() as u64).to_le_bytes());
                blake2b.update(&empty_witness.as_bytes());
                for idx in input_group_idxs.iter().skip(1).cloned() {
                    let other_witness_bytes = util::load_witnesses(idx)?;
                    blake2b.update(&(other_witness_bytes.len() as u64).to_le_bytes());
                    blake2b.update(&other_witness_bytes);
                }
                let mut i = input_size;
                loop {
                    let ret = util::load_witnesses(i);
                    match ret {
                        Ok(outter_witness_bytes) => {
                            blake2b.update(&(outter_witness_bytes.len() as u64).to_le_bytes());
                            blake2b.update(&outter_witness_bytes);
                        }
                        Err(Error::IndexOutOfBound) => {
                            break;
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    }

                    i += 1;
                }
                let mut message = [0u8; 32];
                blake2b.finalize(&mut message);

                debug!(
                    "Inputs[{}] Generate digest.(args: {}, result: {})",
                    init_witness_idx,
                    util::hex_string(&_key),
                    util::hex_string(&message)
                );

                assert!(
                    lock_of_witness.len() == SECP_SIGNATURE_SIZE + CKB_HASH_DIGEST + EIP712_CHAINID_SIZE,
                    Error::EIP712SignatureError,
                    "Inputs[{}] The length of signature is invalid.(current: {}, expected: {})",
                    init_witness_idx,
                    lock_of_witness.len(),
                    SECP_SIGNATURE_SIZE + CKB_HASH_DIGEST + EIP712_CHAINID_SIZE
                );

                if eip712_chain_id.is_empty() {
                    let from = SECP_SIGNATURE_SIZE + CKB_HASH_DIGEST;
                    let to = from + EIP712_CHAINID_SIZE;
                    eip712_chain_id = lock_of_witness.raw_data()[from..to].to_vec();
                }

                let typed_data_hash =
                    &lock_of_witness.raw_data()[SECP_SIGNATURE_SIZE..SECP_SIGNATURE_SIZE + CKB_HASH_DIGEST];
                ret.insert(
                    init_witness_idx,
                    DigestAndHash {
                        digest: message,
                        typed_data_hash: typed_data_hash.try_into().unwrap(),
                    },
                );
            }
            None => {
                return Err(Error::EIP712SignatureError);
            }
        }
    }

    Ok((ret, eip712_chain_id))
}

pub fn tx_to_eip712_typed_data(
    parser: &WitnessesParser,
    action: das_packed::BytesReader,
    params: &[das_packed::BytesReader],
    chain_id: Vec<u8>,
) -> Result<TypedDataV4, Error> {
    let type_id_table = parser.configs.main()?.type_id_table();
    let plain_text = tx_to_plaintext(parser, type_id_table, action, params)?;
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

    let chain_id_num = u64::from_be_bytes(chain_id.try_into().unwrap());
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
            chainId: chain_id_num,
            name: "da.systems",
            verifyingContract: "0x0000000000000000000000000000000020210722",
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
            digest: ""
        }
    });

    #[cfg(debug_assertions)]
    debug!("Extracted typed data: {}", typed_data);
    #[cfg(debug_assertions)]
    debug!("Attention! Because of compiling problem with the serde_json's preserve_order feature, the fields of the JSON needs to be resort manually when debugging.");

    Ok(typed_data)
}

fn tx_to_plaintext(
    _parser: &WitnessesParser,
    type_id_table_reader: das_packed::TypeIdTableReader,
    action_in_bytes: das_packed::BytesReader,
    _params_in_bytes: &[das_packed::BytesReader],
) -> Result<String, Error> {
    let ret;
    match action_in_bytes.raw_data() {
        // For account-cell-type only
        b"transfer_account" | b"edit_manager" | b"edit_records" => match action_in_bytes.raw_data() {
            b"transfer_account" => ret = transfer_account_to_semantic(type_id_table_reader)?,
            b"edit_manager" => ret = edit_manager_to_semantic(type_id_table_reader)?,
            b"edit_records" => ret = edit_records_to_semantic(type_id_table_reader)?,
            _ => return Err(Error::ActionNotSupported),
        },
        // For balance-cell-type only
        b"transfer" | b"withdraw_from_wallet" => ret = transfer_to_semantic()?,
        _ => return Err(Error::ActionNotSupported),
    }

    Ok(ret)
}

fn transfer_account_to_semantic(type_id_table_reader: das_packed::TypeIdTableReader) -> Result<String, Error> {
    let (input_cells, output_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id_table_reader.account_cell())?;

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

fn edit_manager_to_semantic(type_id_table_reader: das_packed::TypeIdTableReader) -> Result<String, Error> {
    let (input_cells, _output_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id_table_reader.account_cell())?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(input_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| Error::EIP712SerializationError)?;

    // TODO Improve semantic message of this transaction.
    Ok(format!("Edit manager of account {} .", account))
}

fn edit_records_to_semantic(type_id_table_reader: das_packed::TypeIdTableReader) -> Result<String, Error> {
    let (input_cells, _output_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id_table_reader.account_cell())?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(input_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| Error::EIP712SerializationError)?;

    // TODO Improve semantic message of this transaction.
    Ok(format!("Edit records of account {} .", account))
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
                    let address = to_semantic_address(lock.as_reader(), 1..21)?;
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
    let address;
    if util::is_script_equal(das_lock().as_reader(), lock_reader.into()) {
        // If this is a das-lock, convert it to address base on args.
        let args_in_bytes = lock_reader.args().raw_data();
        match args_in_bytes[0] {
            0 => {
                let pubkey_hash = args_in_bytes[range.clone()].to_vec();

                // The first byte is address type, 0x01 is for short address.
                // The second byte is CodeHashIndex, 0x00 is for SECP256K1 + blake160.
                let mut data = vec![1u8, 0];
                // This is the payload of address.
                data = [data, pubkey_hash].concat();

                let value = bech32::encode(&HRP.to_string(), data.to_base32(), Variant::Bech32)
                    .map_err(|_| Error::EIP712SematicError)?;
                address = format!("CKB:{}", value)
            }
            4 => {
                let mut raw = [0u8; 21];
                raw[0] = TRX_ADDR_PREFIX;
                raw[1..21].copy_from_slice(&args_in_bytes[range]);
                address = format!("TRX:{}", b58encode_check(&raw));
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

        let value = bech32::encode(&HRP.to_string(), data.to_base32(), Variant::Bech32)
            .map_err(|_| Error::EIP712SematicError)?;
        address = format!("CKB:{}", value)
    }

    Ok(address)
}

fn b58encode_check<T: AsRef<[u8]>>(raw: T) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_ref());
    let digest1 = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(&digest1);
    let digest = hasher.finalize();

    let mut input = raw.as_ref().to_owned();
    input.extend(&digest[..4]);
    let mut output = String::new();
    bs58::encode(&input).into(&mut output).unwrap();

    output
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
                    i += 1;
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
        if capacity_str == "0" {
            ret = String::from("0 CKB");
        } else {
            let padded_str = format!("{:0>8}", capacity_str);
            let decimal = padded_str.trim_end_matches("0");
            ret = ret + "0." + decimal + " CKB";
        }
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
        "{{ status: {}, records_hash: 0x{} }}",
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
                4, 150, 163, 186, 206, 90, 218, 207, 99, 126, 183, 204, 121, 213, 120, 127, 66, 71, 218, 75, 190, 4,
                150, 163, 186, 206, 90, 218, 207, 99, 126, 183, 204, 121, 213, 120, 127, 66, 71, 218, 75, 190,
            ]))
            .build();

        let expected = "TRX:TPhiVyQZ5xyvVK2KS2LTke8YvXJU5wxnbN";
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

    #[test]
    fn test_to_semantic_capacity() {
        let expected = "0 CKB";
        let result = to_semantic_capacity(0);
        assert_eq!(result, expected);

        let expected = "1 CKB";
        let result = to_semantic_capacity(100_000_000);
        assert_eq!(result, expected);

        let expected = "0.0001 CKB";
        let result = to_semantic_capacity(10_000);
        assert_eq!(result, expected);

        let expected = "1000.0001 CKB";
        let result = to_semantic_capacity(100_000_010_000);
        assert_eq!(result, expected);

        let expected = "1000 CKB";
        let result = to_semantic_capacity(100_000_000_000);
        assert_eq!(result, expected);
    }
}
