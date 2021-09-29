use super::{constants::*, error};
use crate::Loader;
use chrono::{DateTime, NaiveDateTime, Utc};
use ckb_testtool::context::Context;
use ckb_tool::{
    ckb_chain_spec::consensus::TYPE_ID_CODE_HASH,
    ckb_hash::{blake2b_256, new_blake2b},
    ckb_jsonrpc_types as rpc_types,
    ckb_types::{
        bytes,
        core::{ScriptHashType, TransactionView},
        h256,
        packed::*,
        prelude::*,
        H160, H256,
    },
};
use das_types::{packed as das_packed, util as das_types_util};
use lazy_static::lazy_static;
use std::{
    collections::HashSet,
    env,
    error::Error,
    fs::File,
    io,
    io::{BufRead, BufReader, Lines},
    path::PathBuf,
    str::FromStr,
};

lazy_static! {
    pub static ref SECP256K1: secp256k1::Secp256k1<secp256k1::All> = secp256k1::Secp256k1::new();
}

pub use das_types_util::hex_string;

pub fn contains_error(message: &str, err_code: error::Error) -> bool {
    let err_str = format!("ValidationFailure({})", (err_code as i8).to_string());
    message.contains(&err_str)
}

pub fn hex_to_bytes(input: &str) -> Vec<u8> {
    let hex = input.trim_start_matches("0x");
    if hex == "" {
        Vec::new()
    } else {
        hex::decode(hex).expect("Expect input to valid hex")
    }
}

pub fn hex_to_byte32(input: &str) -> Result<Byte32, Box<dyn Error>> {
    let hex = input.trim_start_matches("0x");
    let data = hex::decode(hex)?.into_iter().map(Byte::new).collect::<Vec<_>>();
    let mut inner = [Byte::new(0); 32];
    inner.copy_from_slice(&data);

    Ok(Byte32::new_builder().set(inner).build())
}

pub fn hex_to_hash(input: &str) -> Result<das_packed::Hash, Box<dyn Error>> {
    let hex = input.trim_start_matches("0x");
    let data = hex::decode(hex)?.into_iter().map(Byte::new).collect::<Vec<_>>();
    let mut inner = [Byte::new(0); 32];
    inner.copy_from_slice(&data);

    Ok(das_packed::Hash::new_builder().set(inner).build())
}

pub fn hex_to_u64(input: &str) -> Result<u64, Box<dyn Error>> {
    let hex = input.trim_start_matches("0x");
    if hex == "" {
        Ok(0u64)
    } else {
        Ok(u64::from_str_radix(hex, 16)?)
    }
}

pub fn get_type_id_bytes(name: &str) -> Vec<u8> {
    hex_to_bytes(
        TYPE_ID_TABLE
            .get(name)
            .expect(&format!("Can not find type ID for {}", name)),
    )
}

pub fn account_to_id(account: &str) -> Vec<u8> {
    let hash = blake2b_256(account);
    hash.get(..ACCOUNT_ID_LENGTH).unwrap().to_vec()
}

pub fn account_to_id_bytes(account: &str) -> Vec<u8> {
    account_to_id(account)
}

pub fn account_to_id_hex(account: &str) -> String {
    format!("0x{}", hex_string(account_to_id(account).as_slice()))
}

pub fn calc_account_cell_capacity(length: u64) -> u64 {
    (length * 100_000_000) + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY
}

pub fn deploy_dev_contract(
    context: &mut Context,
    binary_name: &str,
    index_opt: Option<usize>,
) -> (Byte32, OutPoint, CellDep) {
    let contract_bin: bytes::Bytes = Loader::default().load_binary(binary_name);

    deploy_contract(context, binary_name, contract_bin, index_opt)
}

pub fn deploy_builtin_contract(
    context: &mut Context,
    binary_name: &str,
    index_opt: Option<usize>,
) -> (Byte32, OutPoint, CellDep) {
    let contract_bin: bytes::Bytes = Loader::with_deployed_scripts().load_binary(binary_name);

    deploy_contract(context, binary_name, contract_bin, index_opt)
}

fn deploy_contract(
    context: &mut Context,
    binary_name: &str,
    contract_bin: bytes::Bytes,
    index_opt: Option<usize>,
) -> (Byte32, OutPoint, CellDep) {
    let args = binary_name
        .as_bytes()
        .to_vec()
        .into_iter()
        .map(Byte::new)
        .collect::<Vec<_>>();
    let type_ = Script::new_builder()
        .code_hash(Byte32::new_unchecked(bytes::Bytes::from(TYPE_ID_CODE_HASH.as_bytes())))
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::new_builder().set(args).build())
        .build();
    let type_id = type_.calc_script_hash();
    // Uncomment the line below can print type ID of each script in unit tests.
    // println!("script: {}, type_id: {}", binary_name, type_id);

    let out_point = mock_out_point(index_opt.unwrap_or(rand::random::<usize>()));
    mock_cell_with_outpoint(
        context,
        out_point.clone(),
        contract_bin.len() as u64,
        Script::default(),
        Some(type_),
        Some(contract_bin.to_vec()),
    );

    let cell_dep = CellDep::new_builder().out_point(out_point.clone()).build();

    (type_id, out_point, cell_dep)
}

pub fn deploy_shared_lib(
    context: &mut Context,
    binary_name: &str,
    index_opt: Option<usize>,
) -> (Byte32, OutPoint, CellDep) {
    let file: bytes::Bytes = Loader::default().load_binary(binary_name);

    let hash = blake2b_256(file.clone());
    let mut inner = [Byte::new(0); 32];
    for (i, item) in hash.iter().enumerate() {
        inner[i] = Byte::new(*item);
    }
    let code_hash = Byte32::new_builder().set(inner).build();

    let out_point = mock_out_point(index_opt.unwrap_or(rand::random::<usize>()));
    mock_cell_with_outpoint(
        context,
        out_point.clone(),
        file.len() as u64,
        Script::default(),
        None,
        Some(file.to_vec()),
    );

    let cell_dep = CellDep::new_builder().out_point(out_point.clone()).build();

    (code_hash, out_point, cell_dep)
}

pub fn mock_script(context: &mut Context, out_point: OutPoint, args: bytes::Bytes) -> Script {
    context
        .build_script(&out_point, args)
        .expect("Build script failed, can not find cell of script.")
}

pub fn mock_header_deps(context: &mut Context, header_hash: Byte32, number: u64, timestamp: u64) {
    let raw_header = RawHeader::new_builder()
        .number(number.pack())
        .timestamp(timestamp.pack())
        .build();
    let header = Header::new_builder().raw(raw_header).build().into_view();

    // Set header with manually specified hash will make writing tests much easier.
    context.headers.insert(header_hash, header);
}

pub fn mock_cell(
    context: &mut Context,
    capacity: u64,
    lock_script: Script,
    type_script: Option<Script>,
    data_opt: Option<Vec<u8>>,
) -> OutPoint {
    let data = data_opt.unwrap_or_default();
    let cell = CellOutput::new_builder()
        .capacity(capacity.pack())
        .lock(lock_script)
        .type_(ScriptOpt::new_builder().set(type_script).build())
        .build();

    // println!(
    //     "cell: {}",
    //     serde_json::to_string_pretty(&rpc_types::CellOutput::from(cell.clone())).unwrap()
    // );

    context.create_cell(cell, bytes::Bytes::from(data))
}

pub fn mock_out_point(index: usize) -> OutPoint {
    let index_bytes = (index as u64).to_be_bytes().to_vec();
    let tx_hash_bytes = [
        vec![0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        index_bytes,
    ]
    .concat();
    let tx_hash = Byte32::from_slice(&tx_hash_bytes).expect("The input of Byte32::from_slice is invalid.");

    OutPoint::new_builder().index(0u32.pack()).tx_hash(tx_hash).build()
}

pub fn mock_cell_with_outpoint(
    context: &mut Context,
    out_point: OutPoint,
    capacity: u64,
    lock_script: Script,
    type_script: Option<Script>,
    data_opt: Option<Vec<u8>>,
) {
    let data = data_opt.unwrap_or_default();

    context.create_cell_with_out_point(
        out_point,
        CellOutput::new_builder()
            .capacity(capacity.pack())
            .lock(lock_script)
            .type_(ScriptOpt::new_builder().set(type_script).build())
            .build(),
        bytes::Bytes::from(data),
    );
}

pub fn mock_input(out_point: OutPoint, since: Option<u64>) -> CellInput {
    let mut builder = CellInput::new_builder().previous_output(out_point);

    if let Some(data) = since {
        builder = builder.since(data.pack());
    }

    builder.build()
}

pub fn mock_output(capacity: u64, lock_script: Script, type_script: Option<Script>) -> CellOutput {
    CellOutput::new_builder()
        .capacity(capacity.pack())
        .lock(lock_script)
        .type_(ScriptOpt::new_builder().set(type_script).build())
        .build()
}

pub fn serialize_signature(signature: &secp256k1::recovery::RecoverableSignature) -> [u8; 65] {
    let (recov_id, data) = signature.serialize_compact();
    let mut signature_bytes = [0u8; 65];
    signature_bytes[0..64].copy_from_slice(&data[0..64]);
    signature_bytes[64] = recov_id.to_i32() as u8;
    signature_bytes
}

pub type SignerFn = Box<dyn FnMut(&HashSet<H160>, &H256, &rpc_types::Transaction) -> Result<Option<[u8; 65]>, String>>;

pub fn get_privkey_signer(input: &str) -> SignerFn {
    let privkey = secp256k1::SecretKey::from_str(input.trim_start_matches("0x")).unwrap();
    let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &privkey);
    let lock_arg =
        H160::from_slice(&blake2b_256(&pubkey.serialize()[..])[0..20]).expect("Generate hash(H160) from pubkey failed");
    Box::new(
        move |lock_args: &HashSet<H160>, message: &H256, _tx: &rpc_types::Transaction| {
            if lock_args.contains(&lock_arg) {
                if message == &h256!("0x0") {
                    Ok(Some([0u8; 65]))
                } else {
                    let message = secp256k1::Message::from_slice(message.as_bytes())
                        .expect("Convert to secp256k1 message failed");
                    let signature = SECP256K1.sign_recoverable(&message, &privkey);
                    Ok(Some(serialize_signature(&signature)))
                }
            } else {
                Ok(None)
            }
        },
    )
}

pub fn build_signature<S: FnMut(&H256, &rpc_types::Transaction) -> Result<[u8; SECP_SIGNATURE_SIZE], String>>(
    tx: &TransactionView,
    input_size: usize,
    input_group_idxs: &[usize],
    witnesses: &[Bytes],
    mut signer: S,
) -> Result<bytes::Bytes, String> {
    let init_witness_idx = input_group_idxs[0];
    let init_witness = if witnesses[init_witness_idx].raw_data().is_empty() {
        WitnessArgs::default()
    } else {
        WitnessArgs::from_slice(witnesses[init_witness_idx].raw_data().as_ref()).map_err(|err| err.to_string())?
    };

    let init_witness = init_witness
        .as_builder()
        .lock(Some(bytes::Bytes::from(vec![0u8; SECP_SIGNATURE_SIZE])).pack())
        .build();

    let mut blake2b = new_blake2b();
    blake2b.update(tx.hash().as_slice());
    blake2b.update(&(init_witness.as_bytes().len() as u64).to_le_bytes());
    blake2b.update(&init_witness.as_bytes());
    for idx in input_group_idxs.iter().skip(1).cloned() {
        let other_witness: &Bytes = &witnesses[idx];
        blake2b.update(&(other_witness.len() as u64).to_le_bytes());
        blake2b.update(&other_witness.raw_data());
    }
    for outter_witness in &witnesses[input_size..witnesses.len()] {
        blake2b.update(&(outter_witness.len() as u64).to_le_bytes());
        blake2b.update(&outter_witness.raw_data());
    }
    let mut message = [0u8; 32];
    blake2b.finalize(&mut message);
    let message = H256::from(message);
    signer(&message, &tx.data().into()).map(|data| bytes::Bytes::from(data.to_vec()))
}

pub fn prepend_molecule_like_length(raw: Vec<u8>) -> Vec<u8> {
    // Prepend length of bytes to raw data, include the bytes of length itself.
    let mut entity = (raw.len() as u32 + 4).to_le_bytes().to_vec();
    entity.extend(raw);

    entity
}

pub fn read_lines(file_name: &str) -> io::Result<Lines<BufReader<File>>> {
    let dir = env::current_dir().unwrap();
    let mut file_path = PathBuf::new();
    file_path.push(dir);
    file_path.push("data");
    file_path.push(file_name);

    // Read record keys from file, then sort them.
    let file = File::open(file_path)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn gen_timestamp(datetime: &str) -> u64 {
    let navie_datetime =
        NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S").expect("Invalid datetime format.");
    let datetime = DateTime::<Utc>::from_utc(navie_datetime, Utc);
    datetime.timestamp() as u64
}

pub fn gen_register_fee(account_length: usize, has_inviter: bool) -> u64 {
    let price_in_usd = match account_length {
        1 => ACCOUNT_PRICE_1_CHAR,
        2 => ACCOUNT_PRICE_2_CHAR,
        3 => ACCOUNT_PRICE_3_CHAR,
        4 => ACCOUNT_PRICE_4_CHAR,
        _ => ACCOUNT_PRICE_5_CHAR,
    };

    let price_in_ckb = price_in_usd / CKB_QUOTE * 100_000_000;

    if has_inviter {
        price_in_ckb * (RATE_BASE - INVITED_DISCOUNT) / RATE_BASE
            + ACCOUNT_BASIC_CAPACITY
            + ACCOUNT_PREPARED_FEE_CAPACITY
            + (account_length as u64 + 4) * 100_000_000
    } else {
        price_in_ckb
            + ACCOUNT_BASIC_CAPACITY
            + ACCOUNT_PREPARED_FEE_CAPACITY
            + (account_length as u64 + 4) * 100_000_000
    }
}
