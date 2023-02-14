use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::{format, vec};
use core::convert::TryInto;
use core::fmt::Debug;

use blake2b_ref::{Blake2b, Blake2bBuilder};
use ckb_std::ckb_constants::{CellField, Source};
use ckb_std::ckb_types::bytes;
use ckb_std::ckb_types::core::ScriptHashType;
use ckb_std::ckb_types::packed::*;
use ckb_std::ckb_types::prelude::*;
use ckb_std::cstr_core::CStr;
use ckb_std::error::SysError;
use ckb_std::{high_level, syscalls};
use das_types::constants::{DasLockType, DataType, LockRole, WITNESS_HEADER};
use das_types::mixer::*;
use das_types::packed::{self as das_packed};
pub use das_types::util::{hex_string, is_entity_eq, is_reader_eq};
#[cfg(test)]
use hex::FromHexError;

use super::constants::*;
use super::data_parser;
use super::error::*;
use super::types::ScriptLiteral;
use super::witness_parser::WitnessesParser;

#[cfg(test)]
pub fn hex_to_unpacked_bytes(input: &str) -> Result<bytes::Bytes, FromHexError> {
    let trimed_input = input.trim_start_matches("0x");
    if trimed_input == "" {
        Ok(bytes::Bytes::default())
    } else {
        Ok(bytes::Bytes::from(hex::decode(trimed_input)?))
    }
}

#[cfg(test)]
pub fn hex_to_bytes(input: &str) -> Result<Bytes, FromHexError> {
    let trimed_input = input.trim_start_matches("0x");
    if trimed_input == "" {
        Ok(Bytes::default())
    } else {
        Ok(bytes::Bytes::from(hex::decode(trimed_input)?).pack())
    }
}

#[cfg(test)]
pub fn hex_to_byte32(input: &str) -> Result<Byte32, FromHexError> {
    let hex = input.trim_start_matches("0x");
    let data = hex::decode(hex)?.into_iter().map(Byte::new).collect::<Vec<_>>();
    let mut inner = [Byte::new(0); 32];
    inner.copy_from_slice(&data);

    Ok(Byte32::new_builder().set(inner).build())
}

pub fn first_n_bytes_to_hex(bytes: &[u8], n: usize) -> String {
    bytes
        .get(..n)
        .map(|v| format!("0x{}...", hex_string(v)))
        .or(Some(String::from("0x")))
        .unwrap()
}

pub fn script_literal_to_script(script: ScriptLiteral) -> Script {
    Script::new_builder()
        .code_hash(script.code_hash.pack())
        .hash_type(Byte::new(script.hash_type as u8))
        .args(bytes::Bytes::from(script.args).pack())
        .build()
}

pub fn type_id_to_script(type_id: das_packed::HashReader) -> das_packed::Script {
    das_packed::Script::new_builder()
        .code_hash(type_id.to_entity())
        .hash_type(das_packed::Byte::new(ScriptType::Type as u8))
        .build()
}

pub fn is_unpacked_bytes_eq(a: &bytes::Bytes, b: &bytes::Bytes) -> bool {
    **a == **b
}

pub fn is_type_id_equal(script_a: ScriptReader, script_b: ScriptReader) -> bool {
    // CAREFUL: It is critical that must ensure both code_hash and hash_type are consistent to identify the same script.
    is_reader_eq(script_a.code_hash(), script_b.code_hash()) && script_a.hash_type() == script_b.hash_type()
}

pub fn find_cells_by_type_id(
    script_type: ScriptType,
    type_id: das_packed::HashReader,
    source: Source,
) -> Result<Vec<usize>, Box<dyn ScriptError>> {
    let mut i = 0;
    let mut cell_indexes = Vec::new();
    loop {
        let offset = 16;
        // Here we use 33 bytes to store code_hash and hash_type together.
        let mut code_hash = [0u8; 33];
        let ret = match script_type {
            ScriptType::Lock => syscalls::load_cell_by_field(&mut code_hash, offset, i, source, CellField::Lock),
            ScriptType::Type => syscalls::load_cell_by_field(&mut code_hash, offset, i, source, CellField::Type),
        };

        match ret {
            Ok(_) => {
                // Since script.as_slice().len() must larger than the length of code_hash.
                unreachable!()
            }
            Err(SysError::LengthNotEnough(_)) => {
                // Build an array with specific code_hash and hash_type
                let mut type_id_with_hash_type = [0u8; 33];
                let (left, _) = type_id_with_hash_type.split_at_mut(32);
                left.copy_from_slice(type_id.raw_data());
                type_id_with_hash_type[32] = ScriptType::Type as u8;

                if code_hash == type_id_with_hash_type {
                    cell_indexes.push(i);
                }
            }
            Err(SysError::ItemMissing) if script_type == ScriptType::Type => {}
            Err(SysError::IndexOutOfBound) => {
                break;
            }
            Err(err) => {
                return Err(err.into());
            }
        }

        i += 1;
    }

    Ok(cell_indexes)
}

pub fn find_cells_by_type_id_in_inputs_and_outputs(
    script_type: ScriptType,
    type_id: das_packed::HashReader,
) -> Result<(Vec<usize>, Vec<usize>), Box<dyn ScriptError>> {
    let input_cells = find_cells_by_type_id(script_type, type_id, Source::Input)?;
    let output_cells = find_cells_by_type_id(script_type, type_id, Source::Output)?;

    Ok((input_cells, output_cells))
}

pub fn find_cells_by_type_id_and_filter<F: Fn(usize, Source) -> Result<bool, Box<dyn ScriptError>>>(
    script_type: ScriptType,
    type_id: das_packed::HashReader,
    source: Source,
    filter: F,
) -> Result<Vec<usize>, Box<dyn ScriptError>> {
    let cell_indexes = find_cells_by_type_id(script_type, type_id, source)?;
    let mut ret = Vec::new();
    for i in cell_indexes {
        if filter(i, source)? {
            ret.push(i);
        }
    }

    Ok(ret)
}

pub fn find_only_cell_by_type_id(
    script_type: ScriptType,
    type_id: das_packed::HashReader,
    source: Source,
) -> Result<usize, Box<dyn ScriptError>> {
    let cells = find_cells_by_type_id(script_type, type_id, source)?;

    das_assert!(
        cells.len() == 1,
        ErrorCode::InvalidTransactionStructure,
        "Only one cell expected existing in this transaction, but found {:?} in {:?}.",
        cells.len(),
        source
    );

    Ok(cells[0])
}

pub fn find_cells_by_script(
    script_type: ScriptType,
    script: ScriptReader,
    source: Source,
) -> Result<Vec<usize>, Box<dyn ScriptError>> {
    let mut i = 0;
    let mut cell_indexes = Vec::new();
    let expected_hash = blake2b_256(script.as_slice());
    loop {
        let ret = match script_type {
            ScriptType::Lock => high_level::load_cell_lock_hash(i, source).map(Some),
            _ => high_level::load_cell_type_hash(i, source),
        };

        match ret {
            Ok(Some(hash)) if hash == expected_hash => {
                cell_indexes.push(i);
            }
            Ok(_) => {}
            Err(SysError::IndexOutOfBound) => {
                break;
            }
            Err(err) => {
                return Err(err.into());
            }
        }

        i += 1;
    }

    Ok(cell_indexes)
}

pub fn find_cells_by_script_in_inputs_and_outputs(
    script_type: ScriptType,
    script: ScriptReader,
) -> Result<(Vec<usize>, Vec<usize>), Box<dyn ScriptError>> {
    let input_cells = find_cells_by_script(script_type, script, Source::Input)?;
    let output_cells = find_cells_by_script(script_type, script, Source::Output)?;

    Ok((input_cells, output_cells))
}

pub fn find_cells_by_script_and_filter<F: Fn(usize, Source) -> Result<bool, Box<dyn ScriptError>>>(
    script_type: ScriptType,
    script: ScriptReader,
    source: Source,
    filter: F,
) -> Result<Vec<usize>, Box<dyn ScriptError>> {
    let cell_indexes = find_cells_by_script(script_type, script, source)?;
    let mut ret = Vec::new();
    for i in cell_indexes {
        if filter(i, source)? {
            ret.push(i);
        }
    }

    Ok(ret)
}

pub fn find_balance_cells(
    config_main: das_packed::ConfigCellMainReader,
    user_lock_reader: ScriptReader,
    source: Source,
) -> Result<Vec<usize>, Box<dyn ScriptError>> {
    let das_lock = das_lock();
    if is_type_id_equal(das_lock.as_reader(), user_lock_reader) {
        let args = user_lock_reader.args().raw_data();
        let lock_type = data_parser::das_lock_args::get_owner_type(args);
        let lock_args = data_parser::das_lock_args::get_owner_lock_args(args);

        let all_cells;
        if lock_type == DasLockType::ETH as u8 || lock_type == DasLockType::ETHTypedData as u8 {
            // If the lock type is DasLockType::ETH or DasLockType::ETHTypedData treat the different types as the same lock.
            let cells = find_cells_by_script(ScriptType::Lock, user_lock_reader, source)?;
            let compatible_args;
            if lock_type == DasLockType::ETH as u8 {
                compatible_args = [
                    vec![DasLockType::ETHTypedData as u8],
                    lock_args.to_vec(),
                    vec![DasLockType::ETHTypedData as u8],
                    lock_args.to_vec(),
                ]
                .concat();
            } else {
                compatible_args = [
                    vec![DasLockType::ETH as u8],
                    lock_args.to_vec(),
                    vec![DasLockType::ETH as u8],
                    lock_args.to_vec(),
                ]
                .concat();
            }
            let compatible_lock = user_lock_reader
                .to_entity()
                .as_builder()
                .args(das_packed::Bytes::from(compatible_args).into())
                .build();
            let cells_with_compatible_lock =
                find_cells_by_script(ScriptType::Lock, compatible_lock.as_reader(), source)?;

            all_cells = [cells, cells_with_compatible_lock].concat();
        } else {
            all_cells = find_cells_by_script(ScriptType::Lock, user_lock_reader, source)?;
        }

        let balance_cell_type_script = type_id_to_script(config_main.type_id_table().balance_cell());
        let mut cells = Vec::new();
        for i in all_cells {
            let type_script_opt = high_level::load_cell_type(i, source)?;
            if let Some(type_script) = type_script_opt {
                if is_type_id_equal(type_script.as_reader(), balance_cell_type_script.as_reader().into()) {
                    cells.push(i);
                }
            } else {
                cells.push(i);
            }
        }

        Ok(cells)
    } else {
        // Currently only BalanceCells with das-lock is supported.
        unreachable!();
    }
}

pub fn find_all_balance_cells(
    config_main: das_packed::ConfigCellMainReader,
    source: Source,
) -> Result<Vec<usize>, Box<dyn ScriptError>> {
    let das_lock = das_lock();
    let all_cells = find_cells_by_type_id(
        ScriptType::Lock,
        das_packed::HashReader::from(das_lock.code_hash().as_reader()),
        source,
    )?;

    let balance_cell_type_script = type_id_to_script(config_main.type_id_table().balance_cell());
    let mut cells = Vec::new();
    for i in all_cells {
        let type_script_opt = high_level::load_cell_type(i, source)?;
        if let Some(type_script) = type_script_opt {
            if is_type_id_equal(type_script.as_reader(), balance_cell_type_script.as_reader().into()) {
                cells.push(i);
            }
        } else {
            cells.push(i);
        }
    }

    Ok(cells)
}

pub fn load_data<F: Fn(&mut [u8], usize) -> Result<usize, SysError>>(syscall: F) -> Result<Vec<u8>, SysError> {
    // The buffer length should be a little bigger than the size of the biggest data.
    let mut buf = [0u8; 2000];
    let extend_buf = [0u8; 5000];
    match syscall(&mut buf, 0) {
        Ok(len) => {
            let data = buf[..len].to_vec();
            // debug!("{:?}", data);
            Ok(data)
        }
        Err(SysError::LengthNotEnough(actual_size)) => {
            debug!("Actual data size: {}", actual_size);
            // read 30000 bytes in 733165 cycles
            // read 2500 bytes in 471424 cycles
            let mut data = Vec::with_capacity(actual_size);
            loop {
                if data.len() >= actual_size {
                    break;
                }
                data.extend(extend_buf.iter());
            }
            let loaded_len = buf.len();
            data[..loaded_len].copy_from_slice(&buf);
            let len = syscall(&mut data[loaded_len..], loaded_len)?;
            debug_assert_eq!(len + loaded_len, actual_size);

            // read 30000 bytes in 22806311 cycles
            // read 2500 bytes in 2223269 cycles
            // let mut data = Vec::with_capacity(actual_size);
            // data.resize(actual_size, 0);
            // let loaded_len = buf.len();
            // data[..loaded_len].copy_from_slice(&buf);
            // let len = syscall(&mut data[loaded_len..], loaded_len)?;
            // debug_assert_eq!(len + loaded_len, actual_size);

            Ok(data)
        }
        Err(err) => Err(err),
    }
}

pub fn load_cell_data(index: usize, source: Source) -> Result<Vec<u8>, Box<dyn ScriptError>> {
    load_data(|buf, offset| syscalls::load_cell_data(buf, offset, index, source)).map_err(|err| err.into())
}

pub fn load_header(index: usize, source: Source) -> Result<Header, Box<dyn ScriptError>> {
    match high_level::load_header(index, source) {
        Ok(header) => Ok(header),
        Err(err) => {
            warn!(
                "{:?}[{}] Loading header failed, maybe the block_hash is not filled in the header_deps: {:?}",
                source, index, err
            );
            Err(err.into())
        }
    }
}

pub fn load_oracle_data(type_: OracleCellType) -> Result<u64, Box<dyn ScriptError>> {
    let type_script;
    match type_ {
        OracleCellType::Height => {
            debug!("Finding HeightCell in cell_deps ...");
            type_script = height_cell_type();
        }
        OracleCellType::Time => {
            debug!("Finding TimeCell in cell_deps ...");
            type_script = time_cell_type();
        }
        OracleCellType::Quote => {
            debug!("Finding QuoteCell in cell_deps ...");
            type_script = quote_cell_type();
        }
    }

    // There must be one OracleCell in the cell_deps, no more and no less.
    let ret = find_cells_by_script(ScriptType::Type, type_script.as_reader(), Source::CellDep)?;
    das_assert!(
        ret.len() == 1,
        ErrorCode::OracleCellIsRequired,
        "There should be one cell of {:?} in cell_deps, no more and no less, but {} found.",
        type_,
        ret.len()
    );

    debug!("cell_deps[{}] Parsing outputs_data of {:?}Cell ...", ret[0], type_);

    // Read the passed timestamp from outputs_data of TimeCell
    let data = load_cell_data(ret[0], Source::CellDep)?;
    let data_in_uint = match data.get(2..) {
        Some(bytes) => {
            das_assert!(
                bytes.len() == 8,
                ErrorCode::OracleCellDataDecodingError,
                "Decoding data from cell of {:?} failed, uint64 with big-endian expected.",
                type_
            );
            u64::from_be_bytes(bytes.try_into().unwrap())
        }
        _ => {
            warn!("Decoding data from cell of {:?} failed, data is missing.", type_);
            return Err(code_to_error!(ErrorCode::OracleCellDataDecodingError));
        }
    };

    Ok(data_in_uint as u64)
}

pub fn load_cells_capacity(cells: &[usize], source: Source) -> Result<u64, Box<dyn ScriptError>> {
    let mut total_input_capacity = 0;
    for i in cells.iter() {
        total_input_capacity += high_level::load_cell_capacity(*i, source)?;
    }

    Ok(total_input_capacity)
}

pub fn load_self_cells_in_inputs_and_outputs() -> Result<(Vec<usize>, Vec<usize>), Box<dyn ScriptError>> {
    let this_type_script = high_level::load_script()?;
    let this_type_script_reader = this_type_script.as_reader();

    let input_cells = find_cells_by_script(ScriptType::Type, this_type_script_reader, Source::Input)?;
    let output_cells = find_cells_by_script(ScriptType::Type, this_type_script_reader, Source::Output)?;

    Ok((input_cells, output_cells))
}

pub fn load_witnesses(index: usize) -> Result<Vec<u8>, Box<dyn ScriptError>> {
    let mut buf = [];
    let ret = syscalls::load_witness(&mut buf, 0, index, Source::Input);

    match ret {
        // Data which length is too short to be DAS witnesses, so ignore it.
        Ok(_) => Ok(buf.to_vec()),
        Err(SysError::LengthNotEnough(actual_size)) => {
            // debug!("Load witnesses[{}]: size: {} Bytes", index, actual_size);
            let mut buf = vec![0u8; actual_size];
            syscalls::load_witness(&mut buf, 0, index, Source::Input)?;
            Ok(buf)
        }
        Err(e) => Err(e.into()),
    }
}

pub fn load_das_witnesses(index: usize) -> Result<Vec<u8>, Box<dyn ScriptError>> {
    let mut buf = [0u8; 7];
    let ret = syscalls::load_witness(&mut buf, 0, index, Source::Input);

    match ret {
        // Data which length is too short to be DAS witnesses, so ignore it.
        Ok(_) => {
            warn!("The witnesses[{}] is too short to be DAS witness.", index);
            Err(code_to_error!(ErrorCode::WitnessReadingError))
        }
        Err(SysError::LengthNotEnough(actual_size)) => {
            if let Some(raw) = buf.get(..3) {
                das_assert!(
                    raw == &WITNESS_HEADER,
                    ErrorCode::WitnessReadingError,
                    "The witness should start with \"das\" 3 bytes."
                );
            }

            if actual_size > 32000 {
                warn!("The witnesses[{}] should be less than 32KB because the signall lock do not support more than that.", index);
                Err(SysError::LengthNotEnough(actual_size).into())
            } else {
                let mut buf = vec![0u8; actual_size];
                syscalls::load_witness(&mut buf, 0, index, Source::Input)?;
                Ok(buf)
            }
        }
        Err(e) => {
            warn!("Load witness[{}] failed: {:?}", index, e);
            Err(e.into())
        }
    }
}

pub fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(CKB_HASH_DIGEST)
        .personal(CKB_HASH_PERSONALIZATION)
        .build()
}

pub fn blake2b_256<T: AsRef<[u8]>>(s: T) -> [u8; 32] {
    let mut result = [0u8; CKB_HASH_DIGEST];
    let mut blake2b = Blake2bBuilder::new(CKB_HASH_DIGEST)
        .personal(CKB_HASH_PERSONALIZATION)
        .build();
    blake2b.update(s.as_ref());
    blake2b.finalize(&mut result);
    result
}

pub fn blake2b_das<T: AsRef<[u8]>>(s: T) -> [u8; 32] {
    let mut result = [0u8; CKB_HASH_DIGEST];
    let mut blake2b = Blake2bBuilder::new(CKB_HASH_DIGEST)
        .personal(b"2021-07-22 12:00")
        .build();
    blake2b.update(s.as_ref());
    blake2b.finalize(&mut result);
    result
}

pub fn blake2b_smt<T: AsRef<[u8]>>(s: T) -> [u8; 32] {
    let mut result = [0u8; 32];
    let mut blake2b = Blake2bBuilder::new(32).personal(b"sparsemerkletree").key(&[]).build();
    blake2b.update(s.as_ref());
    blake2b.finalize(&mut result);
    result
}

pub fn is_cell_lock_equal(cell_a: (usize, Source), cell_b: (usize, Source)) -> Result<(), Box<dyn ScriptError>> {
    let a_lock_hash = high_level::load_cell_lock_hash(cell_a.0, cell_a.1)?;
    let b_lock_hash = high_level::load_cell_lock_hash(cell_b.0, cell_b.1)?;

    das_assert!(
        a_lock_hash == b_lock_hash,
        ErrorCode::CellLockCanNotBeModified,
        "The lock script of {:?}[{}]({}) and {:?}[{}]({}) should be the same.",
        cell_a.1,
        cell_a.0,
        hex_string(&a_lock_hash),
        cell_b.1,
        cell_b.0,
        hex_string(&b_lock_hash)
    );

    Ok(())
}

pub fn is_cell_capacity_equal(cell_a: (usize, Source), cell_b: (usize, Source)) -> Result<(), Box<dyn ScriptError>> {
    let a_capacity = high_level::load_cell_capacity(cell_a.0, cell_a.1)?;
    let b_capacity = high_level::load_cell_capacity(cell_b.0, cell_b.1)?;

    das_assert!(
        a_capacity == b_capacity,
        ErrorCode::CellCapacityMustConsistent,
        "The capacity of {:?}[{}]({}) should be equal to {:?}[{}]({}).",
        cell_a.1,
        cell_a.0,
        a_capacity,
        cell_b.1,
        cell_b.0,
        b_capacity
    );

    Ok(())
}

pub fn is_system_off(parser: &WitnessesParser) -> Result<(), Box<dyn ScriptError>> {
    let config_main = parser.configs.main()?;
    let status = u8::from(config_main.status());
    if status == 0 {
        warn!("The DAS system is currently off.");
        return Err(code_to_error!(ErrorCode::SystemOff));
    }

    Ok(())
}

pub fn get_length_in_price(account_length: u64) -> u8 {
    if account_length > ACCOUNT_MAX_PRICED_LENGTH.into() {
        ACCOUNT_MAX_PRICED_LENGTH
    } else {
        account_length as u8
    }
}

pub fn is_init_day(current_timestamp: u64) -> Result<(), Box<dyn ScriptError>> {
    use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

    let current = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(current_timestamp as i64, 0), Utc);

    // On CKB main net, AKA Lina, some actions can be only executed at or before the initialization day of DAS.
    if cfg!(feature = "mainnet") {
        let init_day = Utc.ymd(2021, 7, 22).and_hms(12, 00, 00);
        // Otherwise, any account longer than two chars in length can be registered.
        das_assert!(
            current <= init_day,
            ErrorCode::InitDayHasPassed,
            "The day of DAS initialization has passed."
        );
    }

    Ok(())
}

pub fn is_account_id_in_collection(account_id: &[u8], collection: &[u8]) -> bool {
    let length = collection.len();
    if length <= 0 {
        return false;
    }

    let first = &collection[0..20];
    let last = &collection[length - 20..];

    return if account_id < first {
        debug!("The account is less than the first preserved account, skip.");
        false
    } else if account_id > last {
        debug!("The account is bigger than the last preserved account, skip.");
        false
    } else {
        let accounts_total = collection.len() / ACCOUNT_ID_LENGTH;
        let mut start_account_index = 0;
        let mut end_account_index = accounts_total - 1;

        loop {
            let mid_account_index = (start_account_index + end_account_index) / 2;
            // debug!("mid_account_index = {:?}", mid_account_index);
            let mid_account_start_byte_index = mid_account_index * ACCOUNT_ID_LENGTH;
            let mid_account_end_byte_index = mid_account_start_byte_index + ACCOUNT_ID_LENGTH;
            let mid_account_bytes = collection
                .get(mid_account_start_byte_index..mid_account_end_byte_index)
                .unwrap();

            if mid_account_bytes < account_id {
                start_account_index = mid_account_index + 1;
                // debug!("<");
            } else if mid_account_bytes > account_id {
                // debug!(">");
                end_account_index = if mid_account_index > 1 {
                    mid_account_index - 1
                } else {
                    0
                };
            } else {
                return true;
            }

            if start_account_index > end_account_index || end_account_index == 0 {
                break;
            }
        }

        false
    };
}

pub fn calc_account_storage_capacity(
    config_account: das_packed::ConfigCellAccountReader,
    account_name_storage: u64,
    owner_lock_args: das_packed::BytesReader,
) -> u64 {
    // TODO MIXIN Fix this with new data structure.
    let lock_type = data_parser::das_lock_args::get_owner_type(owner_lock_args.raw_data());
    let basic_capacity = if lock_type == DasLockType::MIXIN as u8 {
        23_000_000_000u64
    } else {
        u64::from(config_account.basic_capacity())
    };

    let basic_capacity = basic_capacity;
    let prepared_fee_capacity = u64::from(config_account.prepared_fee_capacity());
    basic_capacity + prepared_fee_capacity + (account_name_storage * 100_000_000)
}

pub fn calc_yearly_capacity(yearly_price: u64, quote: u64, discount: u32) -> u64 {
    let total;
    if yearly_price < quote {
        total = yearly_price * 100_000_000 / quote;
    } else {
        total = yearly_price / quote * 100_000_000;
    }

    total - (total * discount as u64 / 10000)
}

pub fn calc_duration_from_paid(paid: u64, yearly_price: u64, quote: u64, discount: u32) -> u64 {
    let yearly_capacity = calc_yearly_capacity(yearly_price, quote, discount);

    // Original formula: duration = (paid / yearly_capacity) * 365 * 86400
    // But CKB VM can only handle uint, so we put division to later for higher precision.
    paid * 365 / yearly_capacity * 86400
}

fn get_type_id(
    parser: &WitnessesParser,
    type_script: TypeScript,
) -> Result<das_packed::HashReader, Box<dyn ScriptError>> {
    let config = parser.configs.main()?;

    let type_id = match type_script {
        TypeScript::AccountCellType => config.type_id_table().account_cell(),
        TypeScript::AccountSaleCellType => config.type_id_table().account_sale_cell(),
        TypeScript::AccountAuctionCellType => config.type_id_table().account_auction_cell(),
        TypeScript::ApplyRegisterCellType => config.type_id_table().apply_register_cell(),
        TypeScript::BalanceCellType => config.type_id_table().balance_cell(),
        TypeScript::ConfigCellType => parser.config_cell_type_id.as_reader(),
        TypeScript::IncomeCellType => config.type_id_table().income_cell(),
        TypeScript::OfferCellType => config.type_id_table().offer_cell(),
        TypeScript::PreAccountCellType => config.type_id_table().pre_account_cell(),
        TypeScript::ProposalCellType => config.type_id_table().proposal_cell(),
        TypeScript::ReverseRecordCellType => config.type_id_table().reverse_record_cell(),
        TypeScript::SubAccountCellType => config.type_id_table().sub_account_cell(),
        TypeScript::EIP712Lib => config.type_id_table().eip712_lib(),
    };

    Ok(type_id)
}

pub fn require_type_script(
    parser: &WitnessesParser,
    type_script: TypeScript,
    source: Source,
    err: ErrorCode,
) -> Result<(), Box<dyn ScriptError>> {
    let type_id = get_type_id(parser, type_script.clone())?;

    debug!(
        "Require on: 0x{}({:?}) in {:?}",
        hex_string(type_id.raw_data()),
        type_script,
        source
    );

    // Find out required cell in current transaction.
    let required_cells = find_cells_by_type_id(ScriptType::Type, type_id, source)?;

    das_assert!(
        required_cells.len() > 0,
        err,
        "The cells in {:?} which has type script 0x{}({:?}) is required in this transaction.",
        source,
        hex_string(type_id.raw_data()),
        type_script
    );

    Ok(())
}

pub fn require_super_lock() -> Result<(), Box<dyn ScriptError>> {
    let super_lock = super_lock();
    let has_super_lock = find_cells_by_script(ScriptType::Lock, super_lock.as_reader(), Source::Input)?.len() > 0;

    das_assert!(
        has_super_lock,
        ErrorCode::SuperLockIsRequired,
        "Super lock is required."
    );

    Ok(())
}

/// Get the role required by each action
///
/// Only the actions require manager role is list here for simplified purpose.
pub fn get_action_required_role(action: &[u8]) -> Option<LockRole> {
    match action {
        // account-cell-type
        b"edit_records" => Some(LockRole::Manager),
        _ => Some(LockRole::Owner),
    }
}

pub fn derive_owner_lock_from_cell(input_cell: usize, source: Source) -> Result<Script, Box<dyn ScriptError>> {
    let lock = high_level::load_cell_lock(input_cell, source)?;
    let lock_bytes = lock.as_reader().args().raw_data();
    let owner_lock_type = data_parser::das_lock_args::get_owner_type(lock_bytes);
    let owner_lock_args = data_parser::das_lock_args::get_owner_lock_args(lock_bytes);

    // Build expected refund lock.
    let args = das_packed::Bytes::from(
        [
            vec![owner_lock_type],
            owner_lock_args.to_vec(),
            vec![owner_lock_type],
            owner_lock_args.to_vec(),
        ]
        .concat(),
    );
    let lock_of_balance_cell = lock.as_builder().args(args.into()).build();

    Ok(lock_of_balance_cell)
}

pub fn derive_manager_lock_from_cell(input_cell: usize, source: Source) -> Result<Script, Box<dyn ScriptError>> {
    let lock = high_level::load_cell_lock(input_cell, source)?;
    let lock_bytes = lock.as_reader().args().raw_data();
    let manager_lock_type = data_parser::das_lock_args::get_manager_type(lock_bytes);
    let manager_lock_args = data_parser::das_lock_args::get_manager_lock_args(lock_bytes);

    // Build expected refund lock.
    let args = das_packed::Bytes::from(
        [
            vec![manager_lock_type],
            manager_lock_args.to_vec(),
            vec![manager_lock_type],
            manager_lock_args.to_vec(),
        ]
        .concat(),
    );
    let lock_of_balance_cell = lock.as_builder().args(args.into()).build();

    Ok(lock_of_balance_cell)
}

pub fn diff_das_lock_args(lock_a: &[u8], lock_b: &[u8]) -> (bool, bool) {
    macro_rules! diff {
        ($role:expr, $fn_get_type:ident, $fn_get_args:ident) => {{
            let input_lock_type = data_parser::das_lock_args::$fn_get_type(lock_a);
            let input_pubkey_hash = data_parser::das_lock_args::$fn_get_args(lock_a);
            let output_lock_type = data_parser::das_lock_args::$fn_get_type(lock_b);
            let output_pubkey_hash = data_parser::das_lock_args::$fn_get_args(lock_b);

            let lock_type_consistent = if input_lock_type == DasLockType::ETH as u8 {
                output_lock_type == input_lock_type || output_lock_type == DasLockType::ETHTypedData as u8
            } else {
                output_lock_type == input_lock_type
            };

            !(lock_type_consistent && input_pubkey_hash == output_pubkey_hash)
        }};
    }

    let owner_changed = diff!("owner", get_owner_type, get_owner_lock_args);
    let manager_changed = diff!("manager", get_manager_type, get_manager_lock_args);

    (owner_changed, manager_changed)
}

pub fn is_das_lock_owner_manager_same(lock_args: &[u8]) -> bool {
    let owner_type = data_parser::das_lock_args::get_owner_type(lock_args);
    let owner_pubkey_hash = data_parser::das_lock_args::get_owner_lock_args(lock_args);
    let manager_type = data_parser::das_lock_args::get_manager_type(lock_args);
    let manager_pubkey_hash = data_parser::das_lock_args::get_manager_lock_args(lock_args);

    owner_type == manager_type && owner_pubkey_hash == manager_pubkey_hash
}

pub fn get_account_from_reader<'a>(account_reader: &Box<dyn AccountCellDataReaderMixer + 'a>) -> String {
    let mut account = account_reader.account().as_readable();
    account.extend(ACCOUNT_SUFFIX.as_bytes());

    String::from_utf8(account).unwrap()
}

pub fn get_account_id_from_account(account: &[u8]) -> [u8; ACCOUNT_ID_LENGTH] {
    let hash = blake2b_256(account);
    let mut account_id = [0u8; ACCOUNT_ID_LENGTH];

    account_id.copy_from_slice(&hash[0..ACCOUNT_ID_LENGTH]);

    account_id
}

pub fn get_sub_account_name_from_reader(sub_account_reader: das_packed::SubAccountReader) -> String {
    let mut account = sub_account_reader.account().as_readable();
    let suffix = sub_account_reader.suffix().raw_data();
    account.extend(suffix);

    String::from_utf8(account).unwrap()
}

pub fn parse_income_cell_witness(
    parser: &WitnessesParser,
    index: usize,
    source: Source,
) -> Result<das_packed::IncomeCellData, Box<dyn ScriptError>> {
    let (version, data_type, mol_bytes) = parser.verify_and_get(DataType::IncomeCellData, index, source)?;

    assert!(
        version == 1 && data_type == DataType::IncomeCellData,
        ErrorCode::WitnessVersionOrTypeInvalid,
        "{:?}[{}] The version or data_type of witness is invalid.",
        source,
        index
    );

    let ret = das_packed::IncomeCellData::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
        warn!("Decoding IncomeCellData failed");
        ErrorCode::WitnessEntityDecodingError
    })?;

    Ok(ret)
}

pub fn parse_proposal_cell_witness(
    parser: &WitnessesParser,
    index: usize,
    source: Source,
) -> Result<das_packed::ProposalCellData, Box<dyn ScriptError>> {
    let (version, data_type, mol_bytes) = parser.verify_and_get(DataType::ProposalCellData, index, source)?;

    assert!(
        version == 1 && data_type == DataType::ProposalCellData,
        ErrorCode::WitnessVersionOrTypeInvalid,
        "{:?}[{}] The version or data_type of witness is invalid.",
        source,
        index
    );

    let ret = das_packed::ProposalCellData::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
        warn!("Decoding ProposalCellData failed");
        ErrorCode::WitnessEntityDecodingError
    })?;

    Ok(ret)
}

pub fn parse_pre_account_cell_witness(
    parser: &WitnessesParser,
    index: usize,
    source: Source,
) -> Result<Box<dyn PreAccountCellDataMixer>, Box<dyn ScriptError>> {
    let (version, data_type, mol_bytes) = parser.verify_and_get(DataType::PreAccountCellData, index, source)?;

    assert!(
        data_type == DataType::PreAccountCellData,
        ErrorCode::WitnessVersionOrTypeInvalid,
        "{:?}[{}] The data_type of witness is invalid.",
        source,
        index
    );

    let ret: Box<dyn PreAccountCellDataMixer> = match version {
        1 => Box::new(
            das_packed::PreAccountCellDataV1::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
                warn!("{:?}[{}] Decoding PreAccountCellDataV1 failed", source, index);
                ErrorCode::WitnessEntityDecodingError
            })?,
        ),
        2 => Box::new(
            das_packed::PreAccountCellDataV2::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
                warn!("{:?}[{}] Decoding PreAccountCellDataV2 failed", source, index);
                ErrorCode::WitnessEntityDecodingError
            })?,
        ),
        3 => Box::new(
            das_packed::PreAccountCellData::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
                warn!("{:?}[{}] Decoding PreAccountCellData failed", source, index);
                ErrorCode::WitnessEntityDecodingError
            })?,
        ),
        _ => {
            warn!("{:?}[{}] The version of witness is invalid.", source, index);
            return Err(code_to_error!(ErrorCode::WitnessVersionOrTypeInvalid));
        }
    };

    Ok(ret)
}

pub fn parse_account_cell_witness(
    parser: &WitnessesParser,
    index: usize,
    source: Source,
) -> Result<Box<dyn AccountCellDataMixer>, Box<dyn ScriptError>> {
    let (version, data_type, mol_bytes) = parser.verify_and_get(DataType::AccountCellData, index, source)?;

    assert!(
        data_type == DataType::AccountCellData,
        ErrorCode::WitnessVersionOrTypeInvalid,
        "{:?}[{}] The data_type of witness is invalid.",
        source,
        index
    );

    let ret: Box<dyn AccountCellDataMixer> = match version {
        1 => {
            // CAREFUL! The early versions will no longer be supported.
            return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
        }
        2 => Box::new(
            das_packed::AccountCellDataV2::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
                warn!("{:?}[{}] Decoding AccountCellDataV2 failed", source, index);
                ErrorCode::WitnessEntityDecodingError
            })?,
        ),
        3 => Box::new(
            das_packed::AccountCellData::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
                warn!("{:?}[{}] Decoding AccountCellData failed", source, index);
                ErrorCode::WitnessEntityDecodingError
            })?,
        ),
        _ => {
            warn!("{:?}[{}] The version of witness is invalid.", source, index);
            return Err(code_to_error!(ErrorCode::WitnessVersionOrTypeInvalid));
        }
    };

    Ok(ret)
}

pub fn parse_account_sale_cell_witness(
    parser: &WitnessesParser,
    index: usize,
    source: Source,
) -> Result<Box<dyn AccountSaleCellDataMixer>, Box<dyn ScriptError>> {
    let (version, data_type, mol_bytes) = parser.verify_and_get(DataType::AccountSaleCellData, index, source)?;

    assert!(
        data_type == DataType::AccountSaleCellData,
        ErrorCode::WitnessVersionOrTypeInvalid,
        "{:?}[{}] The data_type of witness is invalid.",
        source,
        index
    );

    let ret: Box<dyn AccountSaleCellDataMixer> = match version {
        1 => Box::new(
            das_packed::AccountSaleCellDataV1::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
                warn!("{:?}[{}] Decoding AccountSaleCellDataV1 failed", source, index);
                ErrorCode::WitnessEntityDecodingError
            })?,
        ),
        2 => Box::new(
            das_packed::AccountSaleCellData::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
                warn!("{:?}[{}] Decoding AccountSaleCellData failed", source, index);
                ErrorCode::WitnessEntityDecodingError
            })?,
        ),
        _ => {
            warn!("{:?}[{}] The version of witness is invalid.", source, index);
            return Err(code_to_error!(ErrorCode::WitnessVersionOrTypeInvalid));
        }
    };

    Ok(ret)
}

pub fn parse_offer_cell_witness(
    parser: &WitnessesParser,
    index: usize,
    source: Source,
) -> Result<das_packed::OfferCellData, Box<dyn ScriptError>> {
    let (version, data_type, mol_bytes) = parser.verify_and_get(DataType::OfferCellData, index, source)?;

    assert!(
        version == 1 && data_type == DataType::OfferCellData,
        ErrorCode::WitnessVersionOrTypeInvalid,
        "{:?}[{}] The version or data_type of witness is invalid.",
        source,
        index
    );

    let ret = das_packed::OfferCellData::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
        warn!("{:?}[{}] Decoding OfferCellData failed", source, index);
        ErrorCode::WitnessEntityDecodingError
    })?;

    Ok(ret)
}

pub fn map_add<K, V>(btree_map: &mut BTreeMap<K, V>, key: K, value: V)
where
    K: Clone + Debug + PartialEq + core::cmp::Ord,
    V: Clone + Debug + PartialEq + core::ops::Add<Output = V>,
{
    match btree_map.get(&key) {
        Some(exist_value) => {
            let new_value = exist_value.to_owned() + value;
            btree_map.insert(key.clone(), new_value);
        }
        _ => {
            btree_map.insert(key.clone(), value);
        }
    }
}

pub fn exec_by_type_id(
    parser: &WitnessesParser,
    type_script: TypeScript,
    argv: &[&CStr],
) -> Result<u64, Box<dyn ScriptError>> {
    let type_id = get_type_id(parser, type_script.clone())?;

    debug!(
        "Execute script {:?} by type ID 0x{}",
        type_script,
        hex_string(type_id.raw_data())
    );

    high_level::exec_cell(type_id.raw_data(), ScriptHashType::Type, 0, 0, argv).map_err(|err| err.into())
}

pub fn get_timestamp_from_header(header: HeaderReader) -> u64 {
    u64::from(das_packed::Uint64Reader::new_unchecked(
        header.raw().timestamp().raw_data(),
    )) / 1000
}
