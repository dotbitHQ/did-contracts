use super::{
    assert, constants::*, debug, error::Error, types::ScriptLiteral, warn,
    witness_parser::WitnessesParser,
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
    constants::{ConfigID, DataType, WITNESS_HEADER},
    packed as das_packed,
};
#[cfg(test)]
use hex::FromHexError;
use std::prelude::v1::*;

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
    let data = hex::decode(hex)?
        .into_iter()
        .map(Byte::new)
        .collect::<Vec<_>>();
    let mut inner = [Byte::new(0); 32];
    inner.copy_from_slice(&data);

    Ok(Byte32::new_builder().set(inner).build())
}

pub fn source_to_str(source: Source) -> &'static str {
    match source {
        Source::HeaderDep => "header_deps",
        Source::CellDep => "cell_deps",
        Source::Input => "inputs",
        Source::Output => "outputs",
        Source::GroupInput => "inputs",
        Source::GroupOutput => "outputs",
    }
}

pub fn script_literal_to_script(script: ScriptLiteral) -> Script {
    Script::new_builder()
        .code_hash(script.code_hash.pack())
        .hash_type(Byte::new(script.hash_type as u8))
        .args(bytes::Bytes::from(script.args).pack())
        .build()
}

pub fn is_unpacked_bytes_eq(a: &bytes::Bytes, b: &bytes::Bytes) -> bool {
    **a == **b
}

pub fn find_cells_by_type_id(
    script_type: ScriptType,
    type_id: das_packed::HashReader,
    source: Source,
) -> Result<Vec<usize>, Error> {
    let mut i = 0;
    let mut cell_indexes = Vec::new();
    loop {
        let mut buf = [0u8; 1000];
        let ret = match script_type {
            ScriptType::Lock => {
                syscalls::load_cell_by_field(&mut buf, 0, i, source, CellField::Lock)
            }
            ScriptType::Type => {
                syscalls::load_cell_by_field(&mut buf, 0, i, source, CellField::Type)
            }
        };

        if ret.is_err() {
            if script_type == ScriptType::Type && ret == Err(SysError::ItemMissing) {
                i += 1;
                continue;
            }

            match ret {
                Err(SysError::IndexOutOfBound) => break,
                _ => return Err(Error::from(ret.unwrap_err())),
            }
        } else {
            let cell_code_hash = buf.get(16..(16 + 32)).unwrap();
            if cell_code_hash == type_id.raw_data() {
                cell_indexes.push(i);
            }
            i += 1;
        }
    }

    Ok(cell_indexes)
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
        "Only one cell expected existing in this transaction, but found more: {:?}: {:?}",
        source,
        cells
    );

    Ok(cells[0])
}

pub fn find_cells_by_script(
    script_type: ScriptType,
    script: &Script,
    source: Source,
) -> Result<Vec<usize>, Error> {
    let mut i = 0;
    let mut cell_indexes = Vec::new();
    let expected_hash = blake2b_256(script.as_reader().as_slice());
    loop {
        let ret = match script_type {
            ScriptType::Lock => high_level::load_cell_lock_hash(i, source),
            _ => high_level::load_cell_type_hash(i, source).map(|hash_opt| match hash_opt {
                Some(hash) => hash,
                None => [0u8; 32],
            }),
        };

        if ret.is_err() {
            match ret {
                Err(SysError::IndexOutOfBound) => break,
                _ => return Err(Error::from(ret.unwrap_err())),
            }
        } else {
            let hash = ret.unwrap();
            if hash == expected_hash {
                cell_indexes.push(i);
            }
            i += 1;
        }
    }

    Ok(cell_indexes)
}

pub fn find_cells<F>(f: F, source: Source) -> Result<Vec<(CellOutput, usize)>, Error>
where
    F: Fn(&CellOutput, usize) -> bool,
{
    let mut i = 0;
    let mut cells = Vec::new();
    loop {
        let ret = high_level::load_cell(i, source);
        if let Err(e) = ret {
            if e == SysError::IndexOutOfBound {
                break;
            } else {
                return Err(Error::from(e));
            }
        }

        let cell = ret.unwrap();
        // debug!("{}", util::cmp_script(&input_lock, &super_lock));
        if f(&cell, i) {
            cells.push((cell, i));
        }

        i += 1;
    }

    Ok(cells)
}

pub fn find_cells_by_script_in_inputs_and_outputs(
    script_type: ScriptType,
    script: &Script,
) -> Result<(Vec<usize>, Vec<usize>), Error> {
    let input_cells = find_cells_by_script(script_type, script, Source::Input)?;
    let output_cells = find_cells_by_script(script_type, script, Source::Output)?;

    Ok((input_cells, output_cells))
}

pub fn load_data<F: Fn(&mut [u8], usize) -> Result<usize, SysError>>(
    syscall: F,
) -> Result<Vec<u8>, SysError> {
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
    load_data(|buf, offset| syscalls::load_cell_data(buf, offset, index, source))
        .map_err(|err| Error::from(err))
}

pub fn load_cell_data_and_entity(
    parser: &WitnessesParser,
    index: usize,
    source: Source,
) -> Result<(Vec<u8>, &das_packed::Bytes), Error> {
    let data = load_data(|buf, offset| syscalls::load_cell_data(buf, offset, index, source))
        .map_err(|err| Error::from(err))?;
    let (_, _, entity) = parser.verify_and_get(index, source)?;

    Ok((data, entity))
}

pub fn load_timestamp() -> Result<u64, Error> {
    debug!("Reading TimeCell ...");

    // Define nervos official TimeCell type script.
    let type_script = time_cell_type();

    // There must be one TimeCell in the cell_deps, no more and no less.
    let ret = find_cells_by_script(ScriptType::Type, &type_script, Source::CellDep)?;
    assert!(
        ret.len() == 1,
        Error::TimeCellIsRequired,
        "There should be one TimeCell in cell_deps, no more and no less."
    );

    debug!("Reading outputs_data of the TimeCell ...");

    // Read the passed timestamp from outputs_data of TimeCell
    let data = load_cell_data(ret[0], Source::CellDep)?;
    let timestamp = match data.get(1..) {
        Some(bytes) => {
            assert!(
                bytes.len() == 4,
                Error::TimeCellDataDecodingError,
                "Decoding timestamp from TimeCell failed, uint32 with big-endian expected."
            );
            u32::from_be_bytes(bytes.try_into().unwrap())
        }
        _ => {
            warn!("Decoding timestamp from TimeCell failed, data is missing.");
            return Err(Error::TimeCellDataDecodingError);
        }
    };

    Ok(timestamp as u64)
}

pub fn load_height() -> Result<u64, Error> {
    debug!("Reading HeightCell ...");

    // Define nervos official TimeCell type script.
    let type_script = height_cell_type();

    // There must be one TimeCell in the cell_deps, no more and no less.
    let ret = find_cells_by_script(ScriptType::Type, &type_script, Source::CellDep)?;
    assert!(
        ret.len() == 1,
        Error::HeightCellIsRequired,
        "There should be one HeightCell in cell_deps, no more and no less."
    );

    debug!("Reading outputs_data of the HeightCell ...");

    // Read the passed timestamp from outputs_data of TimeCell
    let data = load_cell_data(ret[0], Source::CellDep)?;
    let height = match data.get(1..) {
        Some(bytes) => {
            assert!(
                bytes.len() == 8,
                Error::HeightCellDataDecodingError,
                "Decoding block number from HeightCell failed, uint64 with big-endian expected."
            );
            u64::from_be_bytes(bytes.try_into().unwrap())
        }
        _ => {
            warn!("Decoding block number from HeightCell failed, data is missing.");
            return Err(Error::HeightCellDataDecodingError);
        }
    };

    Ok(height)
}

pub fn load_quote() -> Result<u64, Error> {
    let quote_lock = oracle_lock();
    let quote_cells = find_cells_by_script(ScriptType::Lock, &quote_lock, Source::CellDep)?;

    assert!(
        quote_cells.len() == 1,
        Error::QuoteCellIsRequired,
        "There should be one QuoteCell in cell_deps, no more and no less."
    );

    let quote_cell_data = load_cell_data(quote_cells[0], Source::CellDep)?;
    let quote = u64::from_le_bytes(quote_cell_data.try_into().unwrap()); // y CKB/USD

    Ok(quote)
}

fn trim_empty_bytes(buf: &mut [u8]) -> &[u8] {
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

pub fn load_das_action() -> Result<das_packed::ActionData, Error> {
    let mut i = 0;
    let mut action_data_opt = None;

    loop {
        let mut buf = [0u8; 7];
        let ret = syscalls::load_witness(&mut buf, 0, i, Source::Input);

        match ret {
            // Data which length is too short to be DAS witnesses, so ignore it.
            Ok(_) => i += 1,
            Err(SysError::LengthNotEnough(_actual_size)) => {
                if let Some(raw) = buf.get(..3) {
                    if raw != &WITNESS_HEADER {
                        i += 1;
                        continue;
                    }
                }

                let data_type = u32::from_le_bytes(buf.get(3..7).unwrap().try_into().unwrap());
                if data_type == DataType::ActionData as u32 {
                    debug!(
                        "Load witnesses[{}]: {:?} {} Bytes",
                        i,
                        DataType::ActionData,
                        _actual_size
                    );

                    let mut buf = [0u8; 1000];
                    syscalls::load_witness(&mut buf, 0, i, Source::Input)
                        .map_err(|e| Error::from(e))?;
                    let action_data = das_packed::ActionData::from_slice(
                        trim_empty_bytes(&mut buf).get(7..).unwrap(),
                    )
                    .map_err(|_| Error::WitnessActionDecodingError)?;

                    action_data_opt = Some(action_data);
                    break;
                }

                i += 1;
            }
            Err(SysError::IndexOutOfBound) => break,
            Err(e) => return Err(Error::from(e)),
        }
    }

    assert!(
        action_data_opt.is_some(),
        Error::WitnessActionNotFound,
        "There should be on ActionData in witnesses."
    );

    Ok(action_data_opt.unwrap())
}

pub fn load_das_witnesses(data_types_opt: Option<Vec<DataType>>) -> Result<WitnessesParser, Error> {
    let mut i = 0;
    let mut witnesses = Vec::new();

    fn load_witness(buf: &mut [u8], i: usize) -> Result<Vec<u8>, Error> {
        syscalls::load_witness(buf, 0, i, Source::Input).map_err(|e| Error::from(e))?;
        Ok(trim_empty_bytes(buf).to_vec())
    }

    // The following logic is specifically optimized for reading large amounts of data, do not modify it except you know what you are doing.
    loop {
        let mut buf = [0u8; 7];
        let data;
        let ret = syscalls::load_witness(&mut buf, 0, i, Source::Input);

        match ret {
            // Data which length is too short to be DAS witnesses, so ignore it.
            Ok(_) => i += 1,
            Err(SysError::LengthNotEnough(actual_size)) => {
                if let Some(raw) = buf.get(..3) {
                    if raw != &WITNESS_HEADER {
                        i += 1;
                        continue;
                    }
                }

                let data_type_in_int =
                    u32::from_le_bytes(buf.get(3..7).unwrap().try_into().unwrap());
                let data_type = DataType::try_from(data_type_in_int).unwrap();

                // Only parse action with load_das_action function.
                if data_type == DataType::ActionData {
                    i += 1;
                    continue;
                }

                if data_types_opt.is_none()
                    || (data_types_opt.is_some()
                        && data_types_opt.as_ref().unwrap().contains(&data_type))
                {
                    debug!(
                        "Load witnesses[{}]: {:?} {} Bytes",
                        i, data_type, actual_size
                    );

                    match actual_size {
                        x if x <= 2000 => {
                            let mut buf = [0u8; 2000];
                            data = load_witness(&mut buf, i)?;
                        }
                        x if x <= 4000 => {
                            let mut buf = [0u8; 4000];
                            data = load_witness(&mut buf, i)?;
                        }
                        x if x <= 8000 => {
                            let mut buf = [0u8; 8000];
                            data = load_witness(&mut buf, i)?;
                        }
                        x if x <= 16000 => {
                            let mut buf = [0u8; 16000];
                            data = load_witness(&mut buf, i)?;
                        }
                        x if x <= 32000 => {
                            let mut buf = [0u8; 32000];
                            data = load_witness(&mut buf, i)?;
                        }
                        x if x <= 64000 => {
                            let mut buf = [0u8; 64000];
                            data = load_witness(&mut buf, i)?;
                        }
                        _ => {
                            return Err(Error::from(SysError::LengthNotEnough(actual_size)));
                        }
                    }

                    witnesses.push(data);
                }

                i += 1;
            }
            Err(SysError::IndexOutOfBound) => break,
            Err(e) => return Err(Error::from(e)),
        }
    }

    Ok(WitnessesParser::new(witnesses)?)
}

pub fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(32)
        .personal(CKB_HASH_PERSONALIZATION)
        .build()
}

pub fn blake2b_256(s: &[u8]) -> [u8; 32] {
    let mut result = [0u8; 32];
    let mut blake2b = new_blake2b();
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

pub fn is_cell_only_lock_changed(
    cell_a: (usize, Source),
    cell_b: (usize, Source),
) -> Result<(), Error> {
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
    let a_lock_script =
        high_level::load_cell_lock_hash(cell_a.0, cell_a.1).map_err(|e| Error::from(e))?;
    let b_lock_script =
        high_level::load_cell_lock_hash(cell_b.0, cell_b.1).map_err(|e| Error::from(e))?;

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
        .map_err(|e| Error::from(e))?
        .unwrap();
    let b_type_script = high_level::load_cell_type_hash(cell_b.0, cell_b.1)
        .map_err(|e| Error::from(e))?
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
    let a_data = high_level::load_cell_data(cell_a.0, cell_a.1).map_err(|e| Error::from(e))?;
    let b_data = high_level::load_cell_data(cell_b.0, cell_b.1).map_err(|e| Error::from(e))?;

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
    let a_capacity =
        high_level::load_cell_capacity(cell_a.0, cell_a.1).map_err(|e| Error::from(e))?;
    let b_capacity =
        high_level::load_cell_capacity(cell_b.0, cell_b.1).map_err(|e| Error::from(e))?;

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
    let a_capacity =
        high_level::load_cell_capacity(cell_a.0, cell_a.1).map_err(|e| Error::from(e))?;
    let b_capacity =
        high_level::load_cell_capacity(cell_b.0, cell_b.1).map_err(|e| Error::from(e))?;

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

pub fn is_cell_capacity_equal(
    cell_a: (usize, Source),
    cell_b: (usize, Source),
) -> Result<(), Error> {
    let a_capacity =
        high_level::load_cell_capacity(cell_a.0, cell_a.1).map_err(|e| Error::from(e))?;
    let b_capacity =
        high_level::load_cell_capacity(cell_b.0, cell_b.1).map_err(|e| Error::from(e))?;

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

pub fn is_inputs_and_outputs_consistent(
    inputs_cells: Vec<usize>,
    outputs_cells: Vec<usize>,
) -> Result<(), Error> {
    for (i, input_cell_index) in inputs_cells.into_iter().enumerate() {
        let output_cell_index = outputs_cells[i];
        is_cell_capacity_equal(
            (input_cell_index, Source::Input),
            (output_cell_index, Source::Output),
        )?;
        is_cell_consistent(
            (input_cell_index, Source::Input),
            (output_cell_index, Source::Output),
        )?;
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

pub fn verify_account_length_and_years(
    account_length: usize,
    current_timestamp: u64,
    item_index: Option<usize>,
) -> Result<(), Error> {
    use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

    let current = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(current_timestamp as i64, 0),
        Utc,
    );

    if item_index.is_some() {
        debug!(
            "  Item[{}] Check if account is available for registration now. (length: {}, datetime: {:#?})",
            item_index.unwrap(), account_length, current
        );
    } else {
        debug!(
            "Check if account is available for registration now. (length: {}, datetime: {:#?})",
            account_length, current
        );
    }

    // On CKB main net, AKA Lina, accounts of less lengths can be registered only after a specific number of years.
    if cfg!(feature = "mainnet") {
        let start_from = 2021;
        let year_2 = Utc.ymd(start_from + 1, 1, 1).and_hms(0, 0, 0);
        let year_3 = Utc.ymd(start_from + 2, 1, 1).and_hms(0, 0, 0);
        let year_4 = Utc.ymd(start_from + 3, 1, 1).and_hms(0, 0, 0);
        if current < year_2 {
            if account_length <= 7 {
                return Err(Error::AccountStillCanNotBeRegister);
            }
        } else if current < year_3 {
            if account_length <= 6 {
                return Err(Error::AccountStillCanNotBeRegister);
            }
        } else if current < year_4 {
            if account_length <= 5 {
                return Err(Error::AccountStillCanNotBeRegister);
            }
        }
    // Otherwise, any account longer than two chars in length can be registered.
    } else {
        if account_length <= 1 {
            return Err(Error::AccountStillCanNotBeRegister);
        }
    }

    Ok(())
}

pub fn calc_account_storage_capacity(account_name_storage: u64) -> u64 {
    ACCOUNT_CELL_BASIC_CAPACITY + (account_name_storage * 100_000_000)
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
    parser.parse_only_config(&[ConfigID::ConfigCellMain])?;
    let config = parser.configs().main()?;

    let type_id = match type_script {
        TypeScript::AccountCellType => config.type_id_table().account_cell(),
        TypeScript::ApplyRegisterCellType => config.type_id_table().apply_register_cell(),
        TypeScript::PreAccountCellType => config.type_id_table().pre_account_cell(),
        TypeScript::ProposalCellType => config.type_id_table().proposal_cell(),
        TypeScript::WalletCellType => config.type_id_table().wallet_cell(),
    };

    debug!(
        "Require on: 0x{}({:?})",
        hex_string(type_id.raw_data()),
        TypeScript::AccountCellType
    );

    // Find out required cell in current transaction.
    let required_cells = find_cells_by_type_id(ScriptType::Type, type_id, source)?;

    assert!(
        required_cells.len() > 0,
        err,
        "The cells in {:?} which has type script 0x{}({:?}) is required in this transaction.",
        source,
        hex_string(type_id.raw_data()),
        TypeScript::AccountCellType
    );

    Ok(())
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
        let result =
            hex_to_byte32("0xe683b04139344768348499c23eb1326d5a52d6db006c0d2fece00a831f3660d7")
                .unwrap();

        let mut data = [Byte::new(0); 32];
        let v = vec![
            230, 131, 176, 65, 57, 52, 71, 104, 52, 132, 153, 194, 62, 177, 50, 109, 90, 82, 214,
            219, 0, 108, 13, 47, 236, 224, 10, 131, 31, 54, 96, 215,
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
