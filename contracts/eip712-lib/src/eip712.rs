use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::convert::{TryFrom, TryInto};

use bs58;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed as ckb_packed;
use ckb_std::ckb_types::prelude::{Entity, Pack, Unpack};
use ckb_std::error::SysError;
use ckb_std::high_level;
use das_core::constants::*;
use das_core::error::*;
use das_core::witness_parser::WitnessesParser;
use das_core::{assert as das_assert, code_to_error, data_parser, debug, util, warn};
use das_types::constants::{DasLockType, DataType, LockRole};
use das_types::mixer::AccountCellDataMixer;
use das_types::packed as das_packed;
use das_types::prelude::*;
use eip712::eip712::*;
use eip712::util::*;
use eip712::{hash_data, typed_data_v4};
use sha2::{Digest, Sha256};

const TRX_ADDR_PREFIX: u8 = 0x41;
const DATA_OMIT_SIZE: usize = 20;
const PARAM_OMIT_SIZE: usize = 10;

pub fn verify_eip712_hashes(
    parser: &WitnessesParser,
    tx_to_das_message: fn(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>>,
) -> Result<(), Box<dyn ScriptError>> {
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
                            "Inputs[{}] is not the address type supporting EIP712, skip verification for hash.",
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
                return Err(Error::<ErrorCode>::from(err).into());
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
                das_assert!(
                    &item.typed_data_hash == expected_hash.as_slice(),
                    ErrorCode::EIP712SignatureError,
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
    tx_to_das_message: fn(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>>,
) -> Result<(), Box<dyn ScriptError>> {
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
) -> Result<(BTreeMap<usize, DigestAndHash>, Vec<u8>), Box<dyn ScriptError>> {
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
            ErrorCode::EIP712DecodingWitnessArgsError
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
                let tx_hash = high_level::load_tx_hash().map_err(|_| ErrorCode::ItemMissing)?;

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
                        Err(err) => {
                            if err.as_i8() == ErrorCode::IndexOutOfBound as i8 {
                                break;
                            } else {
                                return Err(err);
                            }
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

                das_assert!(
                    lock_of_witness.len() == SECP_SIGNATURE_SIZE + CKB_HASH_DIGEST + EIP712_CHAINID_SIZE,
                    ErrorCode::EIP712SignatureError,
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
                return Err(code_to_error!(ErrorCode::EIP712SignatureError));
            }
        }
    }

    Ok((ret, eip712_chain_id))
}

pub fn tx_to_eip712_typed_data(
    parser: &WitnessesParser,
    chain_id: Vec<u8>,
    tx_to_das_message: fn(parser: &WitnessesParser) -> Result<String, Box<dyn ScriptError>>,
) -> Result<TypedDataV4, Box<dyn ScriptError>> {
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
            inputsCapacity: inputs_capacity_str,
            outputsCapacity: outputs_capacity_str,
            fee: fee_str,
            action: tx_action,
            inputs: inputs,
            outputs: outputs,
            digest: ""
        }
    });

    #[cfg(debug_assertions)]
    debug!("Extracted typed data: {}", typed_data);

    Ok(typed_data)
}

pub fn to_semantic_address(
    parser: &WitnessesParser,
    lock_reader: das_packed::ScriptReader,
    role: LockRole,
) -> Result<String, Box<dyn ScriptError>> {
    let address;
    let hash_type: Vec<u8> = lock_reader.hash_type().as_slice().to_vec();
    let code_hash = lock_reader.code_hash().raw_data().to_vec();
    let args = lock_reader.args().raw_data().to_vec();

    match parser.get_lock_script_type(lock_reader) {
        Some(LockScript::DasLock) => {
            // If this is a das-lock, convert it to address base on args.
            let args_in_bytes = lock_reader.args().raw_data();
            let das_lock_type =
                DasLockType::try_from(args_in_bytes[0]).map_err(|_| ErrorCode::EIP712SerializationError)?;
            match das_lock_type {
                DasLockType::CKBSingle => {
                    let pubkey_hash = if role == LockRole::Owner {
                        data_parser::das_lock_args::get_owner_lock_args(args_in_bytes).to_vec()
                    } else {
                        data_parser::das_lock_args::get_manager_lock_args(args_in_bytes).to_vec()
                    };

                    address = format!(
                        "{}",
                        to_short_address(vec![0], pubkey_hash)
                            .map_err(|_| Error::new(ErrorCode::EIP712SematicError, String::new()))?
                    )
                }
                DasLockType::TRON => {
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
                _ => return Err(code_to_error!(ErrorCode::EIP712SematicError)),
            }
        }
        Some(LockScript::Secp256k1Blake160SignhashLock) => {
            // If this is a secp256k1_blake160_signhash_all lock, convert it to short address.
            let args = lock_reader.args().raw_data().to_vec();
            address = format!(
                "{}",
                to_short_address(vec![0], args)
                    .map_err(|_| Error::new(ErrorCode::EIP712SematicError, String::new()))?
            )
        }
        _ => {
            // If this is a other lock, convert it to full address.
            address = format!(
                "{}",
                to_full_address(code_hash, hash_type, args)
                    .map_err(|_| Error::new(ErrorCode::EIP712SematicError, String::new()))?
            );
        }
    }

    // debug!("lock: {} => address: {}", lock_reader, address);
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

fn to_typed_action(parser: &WitnessesParser) -> Result<Value, Box<dyn ScriptError>> {
    let action = String::from_utf8(parser.action.clone()).map_err(|_| ErrorCode::EIP712SerializationError)?;
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
) -> Result<(u64, Value), Box<dyn ScriptError>> {
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
                return Err(Error::<ErrorCode>::from(err).into());
            }
        }

        i += 1;
    }

    Ok((total_capacity, Value::Array(cells)))
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
            Some(TypeScript::OfferCellType) => String::from("offer-cell-type"),
            Some(TypeScript::PreAccountCellType) => String::from("pre-account-cell-type"),
            Some(TypeScript::ProposalCellType) => String::from("proposal-cell-type"),
            Some(TypeScript::ReverseRecordCellType) => String::from("reverse-record-cell-type"),
            Some(TypeScript::SubAccountCellType) => String::from("sub-account-cell-type"),
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

fn to_semantic_account_cell_data(data_in_bytes: &[u8]) -> Result<String, Box<dyn ScriptError>> {
    let account_in_bytes = data_parser::account_cell::get_account(data_in_bytes);
    let expired_at = data_parser::account_cell::get_expired_at(data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| ErrorCode::EIP712SerializationError)?;
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
) -> Result<String, Box<dyn ScriptError>> {
    let (version, _, entity) = parser.verify_with_hash_and_get(expected_hash, data_type, index, source)?;
    let witness: Box<dyn AccountCellDataMixer> = if version == 2 {
        Box::new(
            das_packed::AccountCellDataV2::from_slice(entity.as_reader().raw_data()).map_err(|_| {
                warn!("EIP712 decoding AccountCellDataV2 failed");
                ErrorCode::WitnessEntityDecodingError
            })?,
        )
    } else {
        Box::new(
            das_packed::AccountCellData::from_slice(entity.as_reader().raw_data()).map_err(|_| {
                warn!("EIP712 decoding AccountCellData failed");
                ErrorCode::WitnessEntityDecodingError
            })?,
        )
    };
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
}
