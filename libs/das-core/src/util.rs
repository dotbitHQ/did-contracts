use super::{
    constants::{ScriptType, CKB_HASH_PERSONALIZATION},
    error::Error,
    types::ScriptLiteral,
    witness_parser::WitnessesParser,
};
use blake2b_ref::{Blake2b, Blake2bBuilder};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes, packed::*, prelude::*},
    debug,
    error::SysError,
    high_level, syscalls,
};
use das_types::{constants::WITNESS_HEADER, packed as das_packed};
#[cfg(test)]
use hex::FromHexError;
use std::prelude::v1::*;

use crate::constants::{CONFIG_CELL_TYPE, TIME_CELL_TYPE};
use core::convert::TryInto;
pub use das_types::util::{is_entity_eq, is_reader_eq};

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

// 68575 cycles
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
        // TODO Need optimization, load_cell_type and load_cell_lock will cost lots of cycles.
        let ret = match script_type {
            ScriptType::Lock => high_level::load_cell_lock(i, source),
            _ => high_level::load_cell_type(i, source).map(|hash_opt| match hash_opt {
                Some(hash) => hash,
                None => Script::default(),
            }),
        };

        if ret.is_err() {
            match ret {
                Err(SysError::IndexOutOfBound) => break,
                _ => return Err(Error::from(ret.unwrap_err())),
            }
        } else {
            let script = ret.unwrap();
            // debug!(
            //     "{} {}: {:x?} == {:x?}",
            //     i,
            //     is_entity_eq(&script.code_hash(), &type_id.to_entity().into()),
            //     script.code_hash(),
            //     type_id.to_entity()
            // );
            if is_entity_eq(&script.code_hash(), &type_id.to_entity().into()) {
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
    if cells.len() != 1 {
        return Err(Error::InvalidTransactionStructure);
    }

    Ok(cells[0])
}

// 229893 cycles
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
            // debug!(
            //     "{} {}: {:x?} == {:x?}",
            //     i,
            //     hash == expected_hash,
            //     hash,
            //     expected_hash
            // );
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

fn load_data<F: Fn(&mut [u8], usize) -> Result<usize, SysError>>(
    syscall: F,
) -> Result<Vec<u8>, SysError> {
    // The buffer length should be a little bigger than the size of the biggest data.
    let mut buf = [0u8; 2000];
    match syscall(&mut buf, 0) {
        Ok(len) => Ok(buf[..len].to_vec()),
        Err(SysError::LengthNotEnough(actual_size)) => {
            let mut data = Vec::with_capacity(actual_size);
            data.resize(actual_size, 0);
            let loaded_len = buf.len();
            data[..loaded_len].copy_from_slice(&buf);
            let len = syscall(&mut data[loaded_len..], loaded_len)?;
            debug_assert_eq!(len + loaded_len, actual_size);
            Ok(data)
        }
        Err(err) => Err(err),
    }
}

pub fn load_timestamp() -> Result<u64, Error> {
    debug!("Reading TimeCell ...");

    // Define nervos official TimeCell type script.
    let time_cell_type = script_literal_to_script(TIME_CELL_TYPE);

    // There must be one TimeCell in the cell_deps, no more and no less.
    let ret = find_cells_by_script(ScriptType::Type, &time_cell_type, Source::CellDep)?;
    if ret.len() != 1 {
        return Err(Error::TimeCellIsRequired);
    }

    debug!("Reading outputs_data of the TimeCell ...");

    // Read the passed timestamp from outputs_data of TimeCell
    let data = high_level::load_cell_data(ret[0], Source::CellDep).map_err(|e| Error::from(e))?;
    let timestamp = match data.get(1..) {
        Some(bytes) => {
            if bytes.len() != 8 {
                return Err(Error::TimeCellDataDecodingError);
            }
            u64::from_le_bytes(bytes.try_into().unwrap())
        }
        _ => return Err(Error::TimeCellDataDecodingError),
    };

    Ok(timestamp)
}

pub fn load_config(parser: &WitnessesParser) -> Result<das_packed::ConfigCellData, Error> {
    debug!("Reading ConfigCell ...");

    let config_cell_type = script_literal_to_script(CONFIG_CELL_TYPE);
    // There must be one ConfigCell in the cell_deps, no more and no less.
    let ret = find_cells_by_script(ScriptType::Type, &config_cell_type, Source::CellDep)?;
    if ret.len() != 1 {
        return Err(Error::ConfigCellIsRequired);
    }

    debug!("Reading witness of the ConfigCell ...");

    // Read and decode the witness of ConfigCell.
    let (_, _, entity) = get_cell_witness(parser, ret[0], Source::CellDep)?;
    let config_cell_data =
        das_packed::ConfigCellData::new_unchecked(entity.as_reader().raw_data().to_owned().into());

    Ok(config_cell_data)
}

// 1251831 cycles
pub fn load_das_witnesses() -> Result<Vec<das_packed::Bytes>, Error> {
    let mut i = 0;
    let mut start_reading_das_witness = false;
    let mut witnesses = Vec::new();
    loop {
        let data;
        let ret = load_data(|buf, offset| syscalls::load_witness(buf, offset, i, Source::Input));
        match ret {
            Ok(_data) => {
                i += 1;
                data = _data;
            }
            Err(SysError::IndexOutOfBound) => break,
            Err(e) => return Err(Error::from(e)),
        }

        // Check DAS header in witness until one witness with DAS header found.
        if !start_reading_das_witness {
            if let Some(raw) = data.as_slice().get(..3) {
                if raw != &WITNESS_HEADER {
                    continue;
                } else {
                    start_reading_das_witness = true;
                }
            } else {
                continue;
            }
        }

        // Start reading DAS witnesses, it is a convention that all DAS witnesses stay together in the end of the witnesses vector.
        let witness = das_packed::Bytes::from(bytes::Bytes::from(data).pack());
        witnesses.push(witness);
    }

    Ok(witnesses)
}

pub fn verify_cells_witness(
    parser: &WitnessesParser,
    index: usize,
    source: Source,
) -> Result<(), Error> {
    let data = high_level::load_cell_data(index, source).map_err(|e| Error::from(e))?;
    let hash = match data.get(..32) {
        Some(bytes) => bytes.to_vec(),
        _ => return Err(Error::InvalidCellData),
    };
    parser.get(index as u32, &hash, source)?;

    Ok(())
}

// 108040 cycles
pub fn get_cell_witness(
    parser: &WitnessesParser,
    index: usize,
    source: Source,
) -> Result<(u32, u32, &das_packed::Bytes), Error> {
    let data = high_level::load_cell_data(index, source).map_err(|e| Error::from(e))?;
    let hash = match data.get(..32) {
        Some(bytes) => bytes.to_vec(),
        _ => return Err(Error::InvalidCellData),
    };

    Ok(parser.get(index as u32, &hash, source)?)
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

pub fn verify_if_cell_consistent(old_index: usize, new_index: usize) -> Result<(), Error> {
    verify_if_cell_lock_consistent(old_index, new_index)?;
    verify_if_cell_type_consistent(old_index, new_index)?;
    verify_if_cell_data_consistent(old_index, new_index)?;

    Ok(())
}

pub fn verify_if_cell_lock_consistent(old_index: usize, new_index: usize) -> Result<(), Error> {
    let old_lock_script =
        high_level::load_cell_lock_hash(old_index, Source::Input).map_err(|e| Error::from(e))?;
    let new_lock_script =
        high_level::load_cell_lock_hash(new_index, Source::Output).map_err(|e| Error::from(e))?;

    if old_lock_script != new_lock_script {
        debug!(
            "Compare cell lock script: [{}]{:?} != [{}]{:?} => {}",
            old_index,
            old_lock_script,
            new_index,
            new_lock_script,
            old_lock_script != new_lock_script
        );
        return Err(Error::CellLockCanNotBeModified);
    }

    Ok(())
}

pub fn verify_if_cell_type_consistent(old_index: usize, new_index: usize) -> Result<(), Error> {
    let old_type_script = high_level::load_cell_type_hash(old_index, Source::Input)
        .map_err(|e| Error::from(e))?
        .unwrap();
    let new_type_script = high_level::load_cell_type_hash(new_index, Source::Output)
        .map_err(|e| Error::from(e))?
        .unwrap();

    if old_type_script != new_type_script {
        debug!(
            "Compare cell type script: [{}]{:?} != [{}]{:?} => {}",
            old_index,
            old_type_script,
            new_index,
            new_type_script,
            old_type_script != new_type_script
        );
        return Err(Error::CellTypeCanNotBeModified);
    }

    Ok(())
}

pub fn verify_if_cell_data_consistent(old_index: usize, new_index: usize) -> Result<(), Error> {
    let old_data =
        high_level::load_cell_data(old_index, Source::Input).map_err(|e| Error::from(e))?;
    let new_data =
        high_level::load_cell_data(new_index, Source::Output).map_err(|e| Error::from(e))?;

    if old_data != new_data {
        debug!(
            "Compare cell capacity: [{}]{:?} != [{}]{:?} => {}",
            old_index,
            old_data,
            new_index,
            new_data,
            old_data != new_data
        );
        return Err(Error::CellDataCanNotBeModified);
    }

    Ok(())
}

pub fn verify_if_cell_capacity_reduced(old_index: usize, new_index: usize) -> Result<(), Error> {
    let old_capacity =
        high_level::load_cell_capacity(old_index, Source::Input).map_err(|e| Error::from(e))?;
    let new_capacity =
        high_level::load_cell_capacity(new_index, Source::Output).map_err(|e| Error::from(e))?;

    if old_capacity <= new_capacity {
        debug!(
            "Compare cell capacity: [{}]{:?} <= [{}]{:?} => {}",
            old_index,
            old_capacity,
            new_index,
            new_capacity,
            old_capacity != new_capacity
        );
        return Err(Error::CellCapacityMustReduced);
    }

    Ok(())
}

pub fn verify_if_cell_capacity_increased(old_index: usize, new_index: usize) -> Result<(), Error> {
    let old_capacity =
        high_level::load_cell_capacity(old_index, Source::Input).map_err(|e| Error::from(e))?;
    let new_capacity =
        high_level::load_cell_capacity(new_index, Source::Output).map_err(|e| Error::from(e))?;

    if old_capacity >= new_capacity {
        debug!(
            "Compare cell capacity: [{}]{:?} >= [{}]{:?} => {}",
            old_index,
            old_capacity,
            new_index,
            new_capacity,
            old_capacity != new_capacity
        );
        return Err(Error::CellCapacityMustIncreased);
    }

    Ok(())
}

pub fn verify_if_cell_capacity_consistent(old_index: usize, new_index: usize) -> Result<(), Error> {
    let old_capacity =
        high_level::load_cell_capacity(old_index, Source::Input).map_err(|e| Error::from(e))?;
    let new_capacity =
        high_level::load_cell_capacity(new_index, Source::Output).map_err(|e| Error::from(e))?;

    if old_capacity != new_capacity {
        debug!(
            "Compare cell capacity: [{}]{:?} != [{}]{:?} => {}",
            old_index,
            old_capacity,
            new_index,
            new_capacity,
            old_capacity != new_capacity
        );
        return Err(Error::CellCapacityMustConsistent);
    }

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
    fn test_account_to_id() {
        let account = bytes::Bytes::from("das.bit".as_bytes());
        let id = account_to_id(account.pack());
        let expect = hex_to_bytes("0xb7526803f67ebe70aba631ae3e9560e0cd969c2d").unwrap();

        assert!(
            is_entity_eq(&id, &expect),
            "Expect account ID of das.bit is equal to 0xb7526803f67ebe70aba631ae3e9560e0cd969c2d"
        );
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
