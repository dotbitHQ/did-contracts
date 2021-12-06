use super::{assert, constants::*, data_parser, debug, error::Error, util, warn, witness_parser::WitnessesParser};
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
use core::convert::{TryFrom, TryInto};
use das_types::constants::DataType;
use das_types::{constants::LockRole, packed as das_packed, prelude::*};
use eip712::{eip712::*, hash_data, typed_data_v4};
use sha2::{Digest, Sha256};
use std::prelude::v1::*;

#[cfg(feature = "mainnet")]
const HRP: &str = "ckb";
#[cfg(not(feature = "mainnet"))]
const HRP: &str = "ckt";

const TRX_ADDR_PREFIX: u8 = 0x41;
const DATA_OMIT_SIZE: usize = 20;
const PARAM_OMIT_SIZE: usize = 10;

pub fn verify_eip712_hashes(
    parser: &WitnessesParser,
    tx_to_das_message: fn(parser: &WitnessesParser) -> Result<String, Error>,
) -> Result<(), Error> {
    let required_role_opt = util::get_action_required_role(&parser.action);
    let das_lock = das_lock();
    let das_lock_reader = das_lock.as_reader();

    let mut i = match parser.action.as_slice() {
        // In buy_account transaction, the inputs[0] and inputs[1] is belong to sellers, because buyers have paid enough, so we do not need
        // their signature here.
        b"buy_account" => 2,
        // In accept_offer transaction, the inputs[0] is belong to buyer, because it is seller to send this transaction for accepting offer,
        // so we do not need the buyer's signature here.
        b"accept_offer" => 1,
        _ => 0,
    };
    let mut input_groups_idxs: BTreeMap<Vec<u8>, Vec<usize>> = BTreeMap::new();
    loop {
        let ret = high_level::load_cell_lock(i, Source::Input);
        match ret {
            Ok(lock) => {
                let lock_reader = lock.as_reader();
                // Only take care of inputs with das-lock
                if util::is_type_id_equal(das_lock_reader, lock_reader) {
                    let args = lock_reader.args().raw_data().to_vec();
                    let type_of_args = if required_role_opt.is_some() && required_role_opt == Some(LockRole::Manager) {
                        data_parser::das_lock_args::get_manager_type(lock_reader.args().raw_data())
                    } else {
                        data_parser::das_lock_args::get_owner_type(lock_reader.args().raw_data())
                    };
                    if type_of_args != DasLockType::ETHTypedData as u8 {
                        debug!(
                            "Inputs[{}] Found deprecated address type, skip verification for hash.",
                            i
                        );
                    } else {
                        input_groups_idxs.entry(args.to_vec()).or_default().push(i);
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
        let mut typed_data = tx_to_eip712_typed_data(&parser, eip712_chain_id, tx_to_das_message)?;
        for index in digest_and_hash.keys() {
            let item = digest_and_hash.get(index).unwrap();
            let digest = util::hex_string(&item.digest);
            typed_data.digest(digest.clone());
            let expected_hash = hash_data(&typed_data).unwrap();

            debug!(
                "Calculated hash of EIP712 typed data with digest.(digest: 0x{}, hash: 0x{})",
                digest,
                util::hex_string(&expected_hash)
            );

            // CAREFUL We need to skip the final verification here because transactions are often change when developing, that will break all tests contains EIP712 verification.
            if cfg!(not(feature = "dev")) {
                assert!(
                    &item.typed_data_hash == expected_hash.as_slice(),
                    Error::EIP712SignatureError,
                    "Inputs[{}] The hash of EIP712 typed data is mismatched.(current: 0x{}, expected: 0x{})",
                    index,
                    util::hex_string(&item.typed_data_hash),
                    util::hex_string(&expected_hash)
                );
            }
        }
    }

    Ok(())
}

pub fn verify_eip712_hashes_if_has_das_lock(
    parser: &WitnessesParser,
    tx_to_das_message: fn(parser: &WitnessesParser) -> Result<String, Error>,
) -> Result<(), Error> {
    let das_lock = das_lock();
    let input_cells =
        util::find_cells_by_type_id(ScriptType::Lock, das_lock.as_reader().code_hash().into(), Source::Input)?;
    if input_cells.len() > 0 {
        verify_eip712_hashes(parser, tx_to_das_message)
    } else {
        Ok(())
    }
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
                "Inputs[{}] Witness can not be decoded as WitnessArgs.(data: 0x{})",
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
                    "Inputs[{}] Generate digest.(args: 0x{}, result: 0x{})",
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
    chain_id: Vec<u8>,
    tx_to_das_message: fn(parser: &WitnessesParser) -> Result<String, Error>,
) -> Result<TypedDataV4, Error> {
    let type_id_table = parser.configs.main()?.type_id_table();

    let plain_text = tx_to_das_message(parser)?;
    let tx_action = to_typed_action(parser)?;
    let (inputs_capacity, inputs) = to_typed_cells(parser, type_id_table, Source::Input)?;
    let (outputs_capacity, outputs) = to_typed_cells(parser, type_id_table, Source::Output)?;
    let inputs_capacity_str = to_semantic_capacity(inputs_capacity);
    let outputs_capacity_str = to_semantic_capacity(outputs_capacity);

    let fee_str = if outputs_capacity <= inputs_capacity {
        to_semantic_capacity(inputs_capacity - outputs_capacity)
    } else {
        format!("-{}", to_semantic_capacity(outputs_capacity - inputs_capacity))
    };

    let chain_id_num = u64::from_be_bytes(chain_id.try_into().unwrap()).to_string();
    let typed_data = typed_data_v4!({
        types: {
            EIP712Domain: {
                chainId: "uint256",
                name: "string",
                verifyingContract: "address",
                version: "string"
            },
            Action: {
                action: "string",
                params: "string"
            },
            Cell: {
                capacity: "string",
                lock: "string",
                type: "string",
                data: "string",
                extraData: "string"
            },
            Transaction: {
                DAS_MESSAGE: "string",
                inputsCapacity: "string",
                outputsCapacity: "string",
                fee: "string",
                action: "Action",
                inputs: "Cell[]",
                outputs: "Cell[]",
                digest: "bytes32"
            }
        },
        primaryType: "Transaction",
        domain: {
            chainId: chain_id_num,
            name: "da.systems",
            verifyingContract: "0x0000000000000000000000000000000020210722",
            version: "1"
        },
        message: {
            DAS_MESSAGE: plain_text,
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
    debug!("Extracted typed data: {:?}", typed_data);

    Ok(typed_data)
}

pub fn to_semantic_address(
    parser: &WitnessesParser,
    lock_reader: das_packed::ScriptReader,
    role: LockRole,
) -> Result<String, Error> {
    let address;

    match parser.get_lock_script_type(lock_reader) {
        Some(LockScript::DasLock) => {
            // If this is a das-lock, convert it to address base on args.
            let args_in_bytes = lock_reader.args().raw_data();
            let das_lock_type = DasLockType::try_from(args_in_bytes[0]).map_err(|_| Error::EIP712SerializationError)?;
            match das_lock_type {
                DasLockType::CKBSingle => {
                    let pubkey_hash = if role == LockRole::Owner {
                        data_parser::das_lock_args::get_owner_lock_args(args_in_bytes).to_vec()
                    } else {
                        data_parser::das_lock_args::get_manager_lock_args(args_in_bytes).to_vec()
                    };

                    // The first byte is address type, 0x01 is for short address.
                    // The second byte is CodeHashIndex, 0x00 is for SECP256K1 + blake160.
                    let mut data = vec![1u8, 0];
                    // This is the payload of address.
                    data = [data, pubkey_hash].concat();

                    let value = bech32::encode(&HRP.to_string(), data.to_base32(), Variant::Bech32)
                        .map_err(|_| Error::EIP712SematicError)?;
                    address = format!("{}", value)
                }
                DasLockType::TRX => {
                    let mut raw = [0u8; 21];
                    raw[0] = TRX_ADDR_PREFIX;
                    if role == LockRole::Owner {
                        raw[1..21].copy_from_slice(data_parser::das_lock_args::get_owner_lock_args(args_in_bytes));
                    } else {
                        raw[1..21].copy_from_slice(data_parser::das_lock_args::get_manager_lock_args(args_in_bytes));
                    }
                    address = format!("{}", b58encode_check(&raw));
                }
                DasLockType::ETH | DasLockType::ETHTypedData => {
                    let pubkey_hash = if role == LockRole::Owner {
                        data_parser::das_lock_args::get_owner_lock_args(args_in_bytes).to_vec()
                    } else {
                        data_parser::das_lock_args::get_manager_lock_args(args_in_bytes).to_vec()
                    };
                    address = format!("0x{}", util::hex_string(&pubkey_hash));
                }
                _ => return Err(Error::EIP712SematicError),
            }
        }
        Some(LockScript::Secp256k1Blake160SignhashLock) => {
            // If this is a secp256k1_blake160_signhash_all lock, convert it to short address.
            let hash_type: Vec<u8> = vec![1];
            let code_index = vec![0];
            let args = lock_reader.args().raw_data().to_vec();

            address = format!("{}", script_to_address(code_index, hash_type, args)?)
        }
        Some(LockScript::Secp256k1Blake160MultisigLock) => {
            // If this is a secp256k1_blake160_multisig_all lock, convert it to short address.
            let hash_type: Vec<u8> = vec![1];
            let code_index = vec![1];
            let args = lock_reader.args().raw_data().to_vec();

            address = format!("{}", script_to_address(code_index, hash_type, args)?)
        }
        _ => {
            // If this is a unknown lock, convert it to full address.
            let hash_type: Vec<u8> = if lock_reader.hash_type().as_slice()[0] == 0 {
                vec![2]
            } else {
                vec![4]
            };
            let code_hash = lock_reader.code_hash().raw_data().to_vec();
            let args = lock_reader.args().raw_data().to_vec();

            address = format!("{}", script_to_address(code_hash, hash_type, args)?)
        }
    }

    Ok(address)
}

fn script_to_address(code_hash: Vec<u8>, hash_type: Vec<u8>, args: Vec<u8>) -> Result<String, Error> {
    // This is the payload of address.
    let data = [hash_type, code_hash, args].concat();

    bech32::encode(&HRP.to_string(), data.to_base32(), Variant::Bech32).map_err(|_| Error::EIP712SematicError)
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

fn to_typed_action(parser: &WitnessesParser) -> Result<Value, Error> {
    let action = String::from_utf8(parser.action.clone()).map_err(|_| Error::EIP712SerializationError)?;
    let mut params = Vec::new();
    for param in parser.params.iter() {
        if param.len() > 10 {
            params.push(format!(
                "0x{}...",
                util::hex_string(&param.raw_data()[..PARAM_OMIT_SIZE])
            ));
        } else {
            params.push(format!("0x{}", util::hex_string(&param.raw_data())));
        }
    }

    Ok(typed_data_v4!(@object {
        action: action,
        params: params.join(",")
    }))
}

fn to_typed_cells(
    parser: &WitnessesParser,
    type_id_table_reader: das_packed::TypeIdTableReader,
    source: Source,
) -> Result<(u64, Value), Error> {
    let mut i = 0;
    let mut cells: Vec<Value> = Vec::new();
    let mut total_capacity = 0;
    loop {
        let ret = high_level::load_cell(i, source);
        match ret {
            Ok(cell) => {
                let type_opt = cell.type_().to_opt();
                let data_in_bytes = util::load_cell_data(i, source)?;
                let capacity_in_shannon = cell.capacity().unpack();

                total_capacity += capacity_in_shannon;

                // Skip NormalCells which has no type script.
                if type_opt.is_none() {
                    i += 1;
                    continue;
                }

                let capacity = to_semantic_capacity(capacity_in_shannon);
                let lock = to_typed_script(
                    parser,
                    ScriptType::Lock,
                    das_packed::ScriptReader::from(cell.lock().as_reader()),
                );

                macro_rules! extract_and_push {
                    ($cell_data_to_str:ident, $cell_witness_to_str:ident, $data_type:expr, $type_:expr) => {
                        let data = $cell_data_to_str(&data_in_bytes)?;
                        let extra_data = $cell_witness_to_str(parser, &data_in_bytes[..32], $data_type, i, source)?;
                        cells.push(
                            typed_data_v4!(@object {
                                capacity: capacity,
                                lock: lock,
                                type: $type_,
                                data: data,
                                extraData: extra_data
                            })
                        )
                    };
                }

                match type_opt {
                    Some(type_script) => {
                        let type_script_reader = das_packed::ScriptReader::from(type_script.as_reader());
                        // Skip BalanceCells which has the type script named balance-cell-type.
                        if util::is_reader_eq(type_script_reader.code_hash(), type_id_table_reader.balance_cell()) {
                            i += 1;
                            continue;
                        }

                        let type_ = to_typed_script(
                            parser,
                            ScriptType::Type,
                            das_packed::ScriptReader::from(type_script.as_reader()),
                        );
                        match type_script_reader.code_hash() {
                            // Handle cells which with DAS type script.
                            x if util::is_reader_eq(x, type_id_table_reader.account_cell()) => {
                                extract_and_push!(
                                    to_semantic_account_cell_data,
                                    to_semantic_account_witness,
                                    DataType::AccountCellData,
                                    type_
                                );
                            }
                            // Handle cells which with unknown type script.
                            _ => {
                                let data = to_typed_common_data(&data_in_bytes);
                                cells.push(typed_data_v4!(@object {
                                    capacity: capacity,
                                    lock: lock,
                                    type: type_,
                                    data: data,
                                    extraData: ""
                                }));
                            }
                        }
                    }
                    // Handle cells which has no type script.
                    _ => {
                        let data = to_typed_common_data(&data_in_bytes);
                        cells.push(typed_data_v4!(@object {
                            capacity: capacity,
                            lock: lock,
                            type: "",
                            data: data,
                            extraData: ""
                        }));
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

    Ok((total_capacity, Value::Array(cells)))
}

pub fn to_semantic_capacity(capacity: u64) -> String {
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

fn to_typed_script(parser: &WitnessesParser, script_type: ScriptType, script: das_packed::ScriptReader) -> String {
    let code_hash = if script_type == ScriptType::Lock {
        match parser.get_lock_script_type(script) {
            Some(LockScript::AlwaysSuccessLock) => String::from("always-success"),
            Some(LockScript::DasLock) => String::from("das-lock"),
            Some(LockScript::Secp256k1Blake160SignhashLock) => String::from("account-cell-type"),
            Some(LockScript::Secp256k1Blake160MultisigLock) => String::from("account-sale-cell-type"),
            _ => format!(
                "0x{}...",
                util::hex_string(&script.code_hash().raw_data().as_ref()[0..DATA_OMIT_SIZE])
            ),
        }
    } else {
        match parser.get_type_script_type(script) {
            Some(TypeScript::ApplyRegisterCellType) => String::from("apply-register-cell-type"),
            Some(TypeScript::AccountCellType) => String::from("account-cell-type"),
            Some(TypeScript::AccountSaleCellType) => String::from("account-sale-cell-type"),
            Some(TypeScript::AccountAuctionCellType) => String::from("account-auction-cell-type"),
            Some(TypeScript::BalanceCellType) => String::from("balance-cell-type"),
            Some(TypeScript::ConfigCellType) => String::from("config-cell-type"),
            Some(TypeScript::IncomeCellType) => String::from("income-cell-type"),
            Some(TypeScript::PreAccountCellType) => String::from("pre-account-cell-type"),
            Some(TypeScript::ProposalCellType) => String::from("proposal-cell-type"),
            Some(TypeScript::ReverseRecordCellType) => String::from("reverse-record-cell-type"),
            _ => format!(
                "0x{}...",
                util::hex_string(&script.code_hash().raw_data().as_ref()[0..DATA_OMIT_SIZE])
            ),
        }
    };

    let hash_type = util::hex_string(script.hash_type().as_slice());
    let args_in_bytes = script.args().raw_data();
    let args = if args_in_bytes.len() > DATA_OMIT_SIZE {
        util::hex_string(&args_in_bytes[0..DATA_OMIT_SIZE]) + "..."
    } else {
        util::hex_string(args_in_bytes.as_ref())
    };

    String::new() + &code_hash + ",0x" + &hash_type + ",0x" + &args
}

fn to_typed_common_data(data_in_bytes: &[u8]) -> String {
    if data_in_bytes.len() > DATA_OMIT_SIZE {
        format!("0x{}", util::hex_string(&data_in_bytes[0..DATA_OMIT_SIZE]) + "...")
    } else if !data_in_bytes.is_empty() {
        format!("0x{}", util::hex_string(data_in_bytes))
    } else {
        String::new()
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
    data_type: DataType,
    index: usize,
    source: Source,
) -> Result<String, Error> {
    let (_, _, entity) = parser.verify_with_hash_and_get(expected_hash, data_type, index, source)?;
    let witness = das_packed::AccountCellData::from_slice(entity.as_reader().raw_data()).map_err(|_| {
        warn!("EIP712 decoding AccountCellData failed");
        Error::WitnessEntityDecodingError
    })?;
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
    use das_types::packed as das_packed;

    #[test]
    fn test_eip712_to_semantic_address() {
        // TODO Refactor with a static function
        let script_table = ScriptTable {
            das_lock: das_lock(),
            signall_lock: signall_lock(),
            multisign_lock: multisign_lock(),
        };

        // 0x00/cde11acafefadb5cb437eb33ab8bbca958ad2a86/00/cde11acafefadb5cb437eb33ab8bbca958ad2a86
        let mut lock: das_packed::Script = das_lock().into();
        lock = lock
            .as_builder()
            .args(das_packed::Bytes::from(vec![
                0, 205, 225, 26, 202, 254, 250, 219, 92, 180, 55, 235, 51, 171, 139, 188, 169, 88, 173, 42, 134, 0,
                205, 225, 26, 202, 254, 250, 219, 92, 180, 55, 235, 51, 171, 139, 188, 169, 88, 173, 42, 134,
            ]))
            .build();

        let expected = "ckt1qyqvmcg6etl04k6uksm7kvat3w72jk9d92rq5tn6px";
        let address = to_semantic_address(&script_table, lock.as_reader(), LockRole::Owner).unwrap();
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

        let expected = "0x94770827e8897417a2bbaf072c71c12f5a003278";
        let address = to_semantic_address(&script_table, lock.as_reader(), LockRole::Owner).unwrap();
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

        let expected = "TPhiVyQZ5xyvVK2KS2LTke8YvXJU5wxnbN";
        let address = to_semantic_address(&script_table, lock.as_reader(), LockRole::Owner).unwrap();
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

        let expected = "0x94770827e8897417a2bbaf072c71c12f5a003278";
        let address = to_semantic_address(&script_table, lock.as_reader(), LockRole::Owner).unwrap();
        assert_eq!(&address, expected);
    }

    // #[test]
    // fn test_eip712_to_typed_script() {
    //     let account_cell_type_id = das_packed::Hash::from([1u8; 32]);
    //     let table_id_table = das_packed::TypeIdTable::new_builder()
    //         .account_cell(account_cell_type_id.clone())
    //         .build();
    //     let das_lock = das_packed::Script::from(das_lock());
    //     let always_success_lock = das_packed::Script::from(always_success_lock());
    //     let config_cell_type = das_packed::Script::from(config_cell_type());
    //
    //     let account_type_script = das_packed::Script::new_builder()
    //         .code_hash(account_cell_type_id)
    //         .hash_type(das_packed::Byte::new(1))
    //         .args(das_packed::Bytes::default())
    //         .build();
    //
    //     let expected = "account-cell-type,0x01,0x";
    //     let result = to_typed_script(
    //         table_id_table.as_reader(),
    //         config_cell_type.as_reader().code_hash(),
    //         das_lock.as_reader().code_hash(),
    //         always_success_lock.as_reader().code_hash(),
    //         account_type_script.as_reader(),
    //     );
    //     assert_eq!(result, expected);
    //
    //     let other_type_script = das_packed::Script::new_builder()
    //         .code_hash(das_packed::Hash::from([9u8; 32]))
    //         .hash_type(das_packed::Byte::new(1))
    //         .args(das_packed::Bytes::from(vec![10u8; 21]))
    //         .build();
    //
    //     let expected =
    //         "0x0909090909090909090909090909090909090909...,0x01,0x0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a...";
    //     let result = to_typed_script(
    //         table_id_table.as_reader(),
    //         config_cell_type.as_reader().code_hash(),
    //         das_lock.as_reader().code_hash(),
    //         always_success_lock.as_reader().code_hash(),
    //         other_type_script.as_reader(),
    //     );
    //     assert_eq!(result, expected);
    //
    //     let other_type_script = das_packed::Script::new_builder()
    //         .code_hash(das_packed::Hash::from([9u8; 32]))
    //         .hash_type(das_packed::Byte::new(1))
    //         .args(das_packed::Bytes::from(vec![10u8; 20]))
    //         .build();
    //
    //     let expected = "0x0909090909090909090909090909090909090909...,0x01,0x0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a";
    //     let result = to_typed_script(
    //         table_id_table.as_reader(),
    //         ScriptType::Type,
    //         other_type_script.as_reader(),
    //     );
    //     assert_eq!(result, expected);
    // }

    #[test]
    fn test_eip712_to_semantic_capacity() {
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
