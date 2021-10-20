use super::{
    assert, constants::*, data_parser, debug, error::Error, types::ScriptLiteral, warn, witness_parser::WitnessesParser,
};
use blake2b_ref::{Blake2b, Blake2bBuilder};
use ckb_std::{
    ckb_constants::{CellField, Source},
    ckb_types::{bytes, packed::*, prelude::*},
    error::SysError,
    high_level, syscalls,
};
use core::convert::{TryFrom, TryInto};
use das_types::{
    constants::{DataType, LockRole, WITNESS_HEADER},
    packed as das_packed,
};
use std::prelude::v1::*;

#[cfg(test)]
use hex::FromHexError;

pub use das_types::util::{hex_string, is_entity_eq, is_reader_eq};

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
) -> Result<Vec<usize>, Error> {
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
                return Err(Error::from(err));
            }
        }

        i += 1;
    }

    Ok(cell_indexes)
}

pub fn find_cells_by_type_id_in_inputs_and_outputs(
    script_type: ScriptType,
    type_id: das_packed::HashReader,
) -> Result<(Vec<usize>, Vec<usize>), Error> {
    let input_cells = find_cells_by_type_id(script_type, type_id, Source::Input)?;
    let output_cells = find_cells_by_type_id(script_type, type_id, Source::Output)?;

    Ok((input_cells, output_cells))
}

pub fn find_cells_by_type_id_and_filter<F: Fn(usize, Source) -> Result<bool, Error>>(
    script_type: ScriptType,
    type_id: das_packed::HashReader,
    source: Source,
    filter: F,
) -> Result<Vec<usize>, Error> {
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
) -> Result<usize, Error> {
    let cells = find_cells_by_type_id(script_type, type_id, source)?;

    assert!(
        cells.len() == 1,
        Error::InvalidTransactionStructure,
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
) -> Result<Vec<usize>, Error> {
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
                return Err(Error::from(err));
            }
        }

        i += 1;
    }

    Ok(cell_indexes)
}

pub fn find_cells_by_script_in_inputs_and_outputs(
    script_type: ScriptType,
    script: ScriptReader,
) -> Result<(Vec<usize>, Vec<usize>), Error> {
    let input_cells = find_cells_by_script(script_type, script, Source::Input)?;
    let output_cells = find_cells_by_script(script_type, script, Source::Output)?;

    Ok((input_cells, output_cells))
}

pub fn find_cells_by_script_and_filter<F: Fn(usize, Source) -> Result<bool, Error>>(
    script_type: ScriptType,
    script: ScriptReader,
    source: Source,
    filter: F,
) -> Result<Vec<usize>, Error> {
    let cell_indexes = find_cells_by_script(script_type, script, source)?;
    let mut ret = Vec::new();
    for i in cell_indexes {
        if filter(i, source)? {
            ret.push(i);
        }
    }

    Ok(ret)
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

pub fn load_cell_data(index: usize, source: Source) -> Result<Vec<u8>, Error> {
    load_data(|buf, offset| syscalls::load_cell_data(buf, offset, index, source)).map_err(|err| Error::from(err))
}

pub fn load_oracle_data(type_: OracleCellType) -> Result<u64, Error> {
    let type_script;
    match type_ {
        OracleCellType::Height => {
            debug!("Reading HeightCell ...");
            type_script = height_cell_type();
        }
        OracleCellType::Time => {
            debug!("Reading TimeCell ...");
            type_script = time_cell_type();
        }
        OracleCellType::Quote => {
            debug!("Reading QuoteCell ...");
            type_script = quote_cell_type();
        }
    }

    // There must be one OracleCell in the cell_deps, no more and no less.
    let ret = find_cells_by_script(ScriptType::Type, type_script.as_reader(), Source::CellDep)?;
    assert!(
        ret.len() == 1,
        Error::OracleCellIsRequired,
        "There should be one cell of {:?} in cell_deps, no more and no less, but {} found.",
        type_,
        ret.len()
    );

    debug!("Reading outputs_data of the cell of {:?} ...", type_);

    // Read the passed timestamp from outputs_data of TimeCell
    let data = load_cell_data(ret[0], Source::CellDep)?;
    let data_in_uint = match data.get(2..) {
        Some(bytes) => {
            assert!(
                bytes.len() == 8,
                Error::OracleCellDataDecodingError,
                "Decoding data from cell of {:?} failed, uint64 with big-endian expected.",
                type_
            );
            u64::from_be_bytes(bytes.try_into().unwrap())
        }
        _ => {
            warn!("Decoding data from cell of {:?} failed, data is missing.", type_);
            return Err(Error::OracleCellDataDecodingError);
        }
    };

    Ok(data_in_uint as u64)
}

pub fn load_self_cells_in_inputs_and_outputs() -> Result<(Vec<usize>, Vec<usize>), Error> {
    let this_type_script = high_level::load_script().map_err(Error::from)?;
    let (input_cells, output_cells) =
        find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, this_type_script.as_reader())?;

    Ok((input_cells, output_cells))
}

pub fn trim_empty_bytes(buf: &[u8]) -> &[u8] {
    let header = buf.get(..3);
    let length = buf
        .get(7..11)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()) as usize);

    if header.is_some() && header == Some(&WITNESS_HEADER) && length.is_some() {
        // debug!("Trim DAS witness with length: {}", 7 + length.unwrap());
        buf.get(..(7 + length.unwrap())).unwrap()
    } else {
        buf
    }
}

pub fn load_witnesses(index: usize) -> Result<Vec<u8>, Error> {
    let mut buf = [];
    let ret = syscalls::load_witness(&mut buf, 0, index, Source::Input);

    match ret {
        // Data which length is too short to be DAS witnesses, so ignore it.
        Ok(_) => Ok(buf.to_vec()),
        Err(SysError::LengthNotEnough(actual_size)) => {
            // debug!("Load witnesses[{}]: size: {} Bytes", index, actual_size);
            let mut buf = vec![0u8; actual_size];
            syscalls::load_witness(&mut buf, 0, index, Source::Input).map_err(Error::from)?;
            Ok(buf)
        }
        Err(e) => Err(Error::from(e)),
    }
}

pub fn load_das_witnesses(index: usize, data_type: DataType) -> Result<Vec<u8>, Error> {
    let mut buf = [0u8; 7];
    let ret = syscalls::load_witness(&mut buf, 0, index, Source::Input);

    match ret {
        // Data which length is too short to be DAS witnesses, so ignore it.
        Ok(_) => {
            warn!("The witnesses[{}] is too short to be DAS witness.", index);
            Err(Error::WitnessReadingError)
        }
        Err(SysError::LengthNotEnough(actual_size)) => {
            if let Some(raw) = buf.get(..3) {
                assert!(
                    raw == &WITNESS_HEADER,
                    Error::WitnessReadingError,
                    "The witness should start with \"das\" 3 bytes."
                );
            }

            let data_type_in_int = u32::from_le_bytes(buf.get(3..7).unwrap().try_into().unwrap());
            let parsed_data_type = DataType::try_from(data_type_in_int).unwrap();

            assert!(
                data_type == parsed_data_type,
                Error::WitnessReadingError,
                "The witnesses[{}] should be the {:?}, but {:?} found.",
                index,
                data_type,
                parsed_data_type
            );

            debug!("Load witnesses[{}]: {:?} size: {} Bytes", index, data_type, actual_size);

            if actual_size > 32000 {
                warn!("The witnesses[{}] should be less than 32KB because the signall lock do not support more than that.", index);
                Err(Error::from(SysError::LengthNotEnough(actual_size)))
            } else {
                let mut buf = vec![0u8; actual_size];
                syscalls::load_witness(&mut buf, 0, index, Source::Input).map_err(Error::from)?;
                Ok(buf)
            }
        }
        Err(e) => {
            warn!("Load witness[{}] failed: {:?}", index, e);
            Err(Error::from(e))
        }
    }
}

pub fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(CKB_HASH_DIGEST)
        .personal(CKB_HASH_PERSONALIZATION)
        .build()
}

pub fn blake2b_256(s: &[u8]) -> [u8; 32] {
    let mut result = [0u8; CKB_HASH_DIGEST];
    let mut blake2b = Blake2bBuilder::new(CKB_HASH_DIGEST)
        .personal(CKB_HASH_PERSONALIZATION)
        .build();
    blake2b.update(s);
    blake2b.finalize(&mut result);
    result
}

pub fn blake2b_das(s: &[u8]) -> [u8; 32] {
    let mut result = [0u8; CKB_HASH_DIGEST];
    let mut blake2b = Blake2bBuilder::new(CKB_HASH_DIGEST)
        .personal(b"2021-07-22 12:00")
        .build();
    blake2b.update(s);
    blake2b.finalize(&mut result);
    result
}

pub fn is_cell_consistent(cell_a: (usize, Source), cell_b: (usize, Source)) -> Result<(), Error> {
    debug!(
        "Compare if {:?}[{}] and {:?}[{}] are equal in every fields except capacity.",
        cell_a.1, cell_a.0, cell_b.1, cell_b.0
    );

    is_cell_lock_equal(cell_a, cell_b)?;
    is_cell_type_equal(cell_a, cell_b)?;
    is_cell_data_equal(cell_a, cell_b)?;

    Ok(())
}

pub fn is_cell_only_lock_changed(cell_a: (usize, Source), cell_b: (usize, Source)) -> Result<(), Error> {
    debug!(
        "Compare if only the cells' lock script are different: {:?}[{}] & {:?}[{}]",
        cell_a.1, cell_a.0, cell_b.1, cell_b.0
    );

    is_cell_capacity_equal(cell_a, cell_b)?;
    is_cell_type_equal(cell_a, cell_b)?;
    is_cell_data_equal(cell_a, cell_b)?;

    Ok(())
}

pub fn is_cell_lock_equal(cell_a: (usize, Source), cell_b: (usize, Source)) -> Result<(), Error> {
    let a_lock_script = high_level::load_cell_lock_hash(cell_a.0, cell_a.1).map_err(Error::from)?;
    let b_lock_script = high_level::load_cell_lock_hash(cell_b.0, cell_b.1).map_err(Error::from)?;

    assert!(
        a_lock_script == b_lock_script,
        Error::CellLockCanNotBeModified,
        "The lock script of {:?}[{}]({}) and {:?}[{}]({}) should be the same.",
        cell_a.1,
        cell_a.0,
        hex_string(&a_lock_script),
        cell_b.1,
        cell_b.0,
        hex_string(&b_lock_script)
    );

    Ok(())
}

pub fn is_cell_type_equal(cell_a: (usize, Source), cell_b: (usize, Source)) -> Result<(), Error> {
    let a_type_script = high_level::load_cell_type_hash(cell_a.0, cell_a.1)
        .map_err(Error::from)?
        .unwrap();
    let b_type_script = high_level::load_cell_type_hash(cell_b.0, cell_b.1)
        .map_err(Error::from)?
        .unwrap();

    assert!(
        a_type_script == b_type_script,
        Error::CellLockCanNotBeModified,
        "The type script of {:?}[{}]({}) and {:?}[{}]({}) should be the same.",
        cell_a.1,
        cell_a.0,
        hex_string(&a_type_script),
        cell_b.1,
        cell_b.0,
        hex_string(&b_type_script)
    );

    Ok(())
}

pub fn is_cell_data_equal(cell_a: (usize, Source), cell_b: (usize, Source)) -> Result<(), Error> {
    let a_data = high_level::load_cell_data(cell_a.0, cell_a.1).map_err(Error::from)?;
    let b_data = high_level::load_cell_data(cell_b.0, cell_b.1).map_err(Error::from)?;

    assert!(
        a_data == b_data,
        Error::CellLockCanNotBeModified,
        "The data of {:?}[{}]({}) and {:?}[{}]({}) should be the same.",
        cell_a.1,
        cell_a.0,
        hex_string(&a_data),
        cell_b.1,
        cell_b.0,
        hex_string(&b_data)
    );

    Ok(())
}

pub fn is_cell_capacity_lt(cell_a: (usize, Source), cell_b: (usize, Source)) -> Result<(), Error> {
    let a_capacity = high_level::load_cell_capacity(cell_a.0, cell_a.1).map_err(Error::from)?;
    let b_capacity = high_level::load_cell_capacity(cell_b.0, cell_b.1).map_err(Error::from)?;

    // ⚠️ Equal is not allowed here because we want to avoid abuse cell.
    assert!(
        a_capacity < b_capacity,
        Error::CellLockCanNotBeModified,
        "The capacity of {:?}[{}]({}) should be less than {:?}[{}]({}).",
        cell_a.1,
        cell_a.0,
        a_capacity,
        cell_b.1,
        cell_b.0,
        b_capacity
    );

    Ok(())
}

pub fn is_cell_capacity_gt(cell_a: (usize, Source), cell_b: (usize, Source)) -> Result<(), Error> {
    let a_capacity = high_level::load_cell_capacity(cell_a.0, cell_a.1).map_err(Error::from)?;
    let b_capacity = high_level::load_cell_capacity(cell_b.0, cell_b.1).map_err(Error::from)?;

    // ⚠️ Equal is not allowed here because we want to avoid abuse cell.
    assert!(
        a_capacity > b_capacity,
        Error::CellLockCanNotBeModified,
        "The capacity of {:?}[{}]({}) should be greater than {:?}[{}]({}).",
        cell_a.1,
        cell_a.0,
        a_capacity,
        cell_b.1,
        cell_b.0,
        b_capacity
    );

    Ok(())
}

pub fn is_cell_capacity_equal(cell_a: (usize, Source), cell_b: (usize, Source)) -> Result<(), Error> {
    let a_capacity = high_level::load_cell_capacity(cell_a.0, cell_a.1).map_err(Error::from)?;
    let b_capacity = high_level::load_cell_capacity(cell_b.0, cell_b.1).map_err(Error::from)?;

    assert!(
        a_capacity == b_capacity,
        Error::CellCapacityMustConsistent,
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

pub fn is_inputs_and_outputs_consistent(inputs_cells: Vec<usize>, outputs_cells: Vec<usize>) -> Result<(), Error> {
    for (i, input_cell_index) in inputs_cells.into_iter().enumerate() {
        let output_cell_index = outputs_cells[i];
        is_cell_capacity_equal((input_cell_index, Source::Input), (output_cell_index, Source::Output))?;
        is_cell_consistent((input_cell_index, Source::Input), (output_cell_index, Source::Output))?;
    }

    Ok(())
}

pub fn is_cell_use_always_success_lock(index: usize, source: Source) -> Result<(), Error> {
    let lock = high_level::load_cell_lock(index, source).map_err(Error::from)?;
    let lock_reader = lock.as_reader();
    let always_success_lock = always_success_lock();
    let always_success_lock_reader = always_success_lock.as_reader();

    assert!(
        is_reader_eq(lock_reader.code_hash(), always_success_lock_reader.code_hash())
            && lock_reader.hash_type() == always_success_lock_reader.hash_type(),
        Error::AlwaysSuccessLockIsRequired,
        "The cell at {:?}[{}] should use always-success lock.(expected_code_hash: {})",
        source,
        index,
        always_success_lock.as_reader().code_hash()
    );

    Ok(())
}

pub fn is_cell_use_signall_lock(index: usize, source: Source) -> Result<(), Error> {
    let lock = high_level::load_cell_lock(index, source).map_err(Error::from)?;
    let signall_lock = signall_lock();

    assert!(
        is_reader_eq(lock.as_reader().code_hash(), signall_lock.as_reader().code_hash()),
        Error::SignallLockIsRequired,
        "The cell at {:?}[{}] should use signall lock.(expected_code_hash: {})",
        source,
        index,
        signall_lock.as_reader().code_hash()
    );

    Ok(())
}

pub fn is_system_off(parser: &mut WitnessesParser) -> Result<(), Error> {
    parser.parse_config(&[DataType::ConfigCellMain])?;
    let config_main = parser.configs.main()?;
    let status = u8::from(config_main.status());
    if status == 0 {
        warn!("The DAS system is currently off.");
        return Err(Error::SystemOff);
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

pub fn is_init_day(current_timestamp: u64) -> Result<(), Error> {
    use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

    let current = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(current_timestamp as i64, 0), Utc);

    // On CKB main net, AKA Lina, some actions can be only executed at or before the initialization day of DAS.
    if cfg!(feature = "mainnet") {
        let init_day = Utc.ymd(2021, 7, 22).and_hms(12, 00, 00);
        // Otherwise, any account longer than two chars in length can be registered.
        assert!(
            current <= init_day,
            Error::InitDayHasPassed,
            "The day of DAS initialization has passed."
        );
    }

    Ok(())
}

pub fn calc_account_storage_capacity(
    config_account: das_packed::ConfigCellAccountReader,
    account_name_storage: u64,
) -> u64 {
    let basic_capacity = u64::from(config_account.basic_capacity());
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

pub fn require_type_script(
    parser: &mut WitnessesParser,
    type_script: TypeScript,
    source: Source,
    err: Error,
) -> Result<(), Error> {
    parser.parse_config(&[DataType::ConfigCellMain])?;
    let config = parser.configs.main()?;

    let type_id = match type_script {
        TypeScript::AccountCellType => config.type_id_table().account_cell(),
        TypeScript::ApplyRegisterCellType => config.type_id_table().apply_register_cell(),
        TypeScript::BalanceCellType => config.type_id_table().balance_cell(),
        TypeScript::IncomeCellType => config.type_id_table().income_cell(),
        TypeScript::AccountSaleCellType => config.type_id_table().account_sale_cell(),
        TypeScript::AccountAuctionCellType => config.type_id_table().account_auction_cell(),
        TypeScript::PreAccountCellType => config.type_id_table().pre_account_cell(),
        TypeScript::ProposalCellType => config.type_id_table().proposal_cell(),
    };

    debug!(
        "Require on: 0x{}({:?}) in {:?}",
        hex_string(type_id.raw_data()),
        type_script,
        source
    );

    // Find out required cell in current transaction.
    let required_cells = find_cells_by_type_id(ScriptType::Type, type_id, source)?;

    assert!(
        required_cells.len() > 0,
        err,
        "The cells in {:?} which has type script 0x{}({:?}) is required in this transaction.",
        source,
        hex_string(type_id.raw_data()),
        type_script
    );

    Ok(())
}

pub fn require_super_lock() -> Result<(), Error> {
    let super_lock = super_lock();
    let has_super_lock = find_cells_by_script(ScriptType::Lock, super_lock.as_reader(), Source::Input)?.len() > 0;

    assert!(has_super_lock, Error::SuperLockIsRequired, "Super lock is required.");

    Ok(())
}

/// Get the role required by each action
pub fn get_action_required_role(action: das_packed::BytesReader) -> Option<LockRole> {
    match action.raw_data() {
        // account-cell-type
        b"transfer_account" => Some(LockRole::Owner),
        b"edit_manager" => Some(LockRole::Owner),
        b"edit_records" => Some(LockRole::Manager),
        // account-sale-cell-type
        b"start_account_sale" => Some(LockRole::Owner),
        b"edit_account_sale" => Some(LockRole::Owner),
        b"cancel_account_sale" => Some(LockRole::Owner),
        b"buy_account" => Some(LockRole::Owner),
        _ => None,
    }
}

pub fn derive_owner_lock_from_cell(input_cell: usize, source: Source) -> Result<Script, Error> {
    let lock = high_level::load_cell_lock(input_cell, source).map_err(Error::from)?;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hex_to_unpacked_bytes() {
        let result = hex_to_unpacked_bytes("0x00FF").unwrap();
        let expect = bytes::Bytes::from(vec![0u8, 255u8]);

        // eprintln!("result = {:#?}", result);
        // eprintln!("expect = {:#?}", expect);
        assert!(
            is_unpacked_bytes_eq(&result, &expect),
            "Expect generated bytes to be [0u8, 255u8]"
        );
    }

    #[test]
    fn test_hex_to_bytes() {
        let result = hex_to_bytes("0x00FF").unwrap();
        let expect = bytes::Bytes::from(vec![0u8, 255u8]).pack();

        // eprintln!("result = {:#?}", result);
        // eprintln!("expect = {:#?}", expect);
        assert!(
            is_entity_eq(&result, &expect),
            "Expect generated bytes to be [0u8, 255u8]"
        );
    }

    #[test]
    fn test_hex_to_byte32() {
        let result = hex_to_byte32("0xe683b04139344768348499c23eb1326d5a52d6db006c0d2fece00a831f3660d7").unwrap();

        let mut data = [Byte::new(0); 32];
        let v = vec![
            230, 131, 176, 65, 57, 52, 71, 104, 52, 132, 153, 194, 62, 177, 50, 109, 90, 82, 214, 219, 0, 108, 13, 47,
            236, 224, 10, 131, 31, 54, 96, 215,
        ]
        .into_iter()
        .map(Byte::new)
        .collect::<Vec<_>>();
        data.copy_from_slice(&v);
        let expect: Byte32 = Byte32::new_builder().set(data).build();

        // eprintln!("result = {:#?}", result);
        // eprintln!("expect = {:#?}", expect);
        assert!(is_entity_eq(&result, &expect));
    }

    #[test]
    fn test_is_unpacked_bytes_eq() {
        let a = hex_to_unpacked_bytes("0x0102").unwrap();
        let b = hex_to_unpacked_bytes("0x0102").unwrap();
        let c = hex_to_unpacked_bytes("0x0103").unwrap();

        assert!(is_unpacked_bytes_eq(&a, &b), "Expect a == b return true");
        assert!(!is_unpacked_bytes_eq(&a, &c), "Expect a == c return false");
    }
}
