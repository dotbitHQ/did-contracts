use super::{
    constants::{height_cell_type, time_cell_type, ScriptType, CKB_HASH_PERSONALIZATION},
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
use core::convert::TryInto;
use das_types::{constants::WITNESS_HEADER, packed as das_packed};
#[cfg(test)]
use hex::FromHexError;
use std::prelude::v1::*;

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

pub fn load_data<F: Fn(&mut [u8], usize) -> Result<usize, SysError>>(
    syscall: F,
) -> Result<Vec<u8>, SysError> {
    // The buffer length should be a little bigger than the size of the biggest data.
    let mut buf = [0u8; 2000];
    let extend_buf = [0u8; 5000];
    match syscall(&mut buf, 0) {
        Ok(len) => Ok(buf[..len].to_vec()),
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

pub fn load_timestamp() -> Result<u64, Error> {
    debug!("Reading TimeCell ...");

    // Define nervos official TimeCell type script.
    let type_script = time_cell_type();

    // There must be one TimeCell in the cell_deps, no more and no less.
    let ret = find_cells_by_script(ScriptType::Type, &type_script, Source::CellDep)?;
    if ret.len() != 1 {
        return Err(Error::TimeCellIsRequired);
    }

    debug!("Reading outputs_data of the TimeCell ...");

    // Read the passed timestamp from outputs_data of TimeCell
    let data = high_level::load_cell_data(ret[0], Source::CellDep).map_err(|e| Error::from(e))?;
    let timestamp = match data.get(1..) {
        Some(bytes) => {
            if bytes.len() != 4 {
                return Err(Error::TimeCellDataDecodingError);
            }
            u32::from_le_bytes(bytes.try_into().unwrap())
        }
        _ => return Err(Error::TimeCellDataDecodingError),
    };

    Ok(timestamp as u64)
}

pub fn load_height() -> Result<u64, Error> {
    debug!("Reading HeightCell ...");

    // Define nervos official TimeCell type script.
    let type_script = height_cell_type();

    // There must be one TimeCell in the cell_deps, no more and no less.
    let ret = find_cells_by_script(ScriptType::Type, &type_script, Source::CellDep)?;
    if ret.len() != 1 {
        return Err(Error::HeightCellIsRequired);
    }

    debug!("Reading outputs_data of the HeightCell ...");

    // Read the passed timestamp from outputs_data of TimeCell
    let data = high_level::load_cell_data(ret[0], Source::CellDep).map_err(|e| Error::from(e))?;
    let height = match data.get(1..) {
        Some(bytes) => {
            if bytes.len() != 8 {
                return Err(Error::TimeCellDataDecodingError);
            }
            u64::from_le_bytes(bytes.try_into().unwrap())
        }
        _ => return Err(Error::TimeCellDataDecodingError),
    };

    Ok(height)
}

// 1251831 cycles
pub fn load_das_witnesses() -> Result<Vec<Vec<u8>>, Error> {
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
        witnesses.push(data);
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

pub fn is_cell_consistent(cell_a: (usize, Source), cell_b: (usize, Source)) -> Result<(), Error> {
    debug!(
        "Compare if the cells' are consistent: {:?}[{}] & {:?}[{}]",
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

    if a_lock_script != b_lock_script {
        debug!(
            "Compare cell lock script: {:?}[{}] {:?} != {:?}[{}] {:?} => true",
            cell_a.1, cell_a.0, a_lock_script, cell_b.1, cell_b.0, b_lock_script
        );
        return Err(Error::CellLockCanNotBeModified);
    }

    Ok(())
}

pub fn is_cell_type_equal(cell_a: (usize, Source), cell_b: (usize, Source)) -> Result<(), Error> {
    let a_type_script = high_level::load_cell_type_hash(cell_a.0, cell_a.1)
        .map_err(|e| Error::from(e))?
        .unwrap();
    let b_type_script = high_level::load_cell_type_hash(cell_b.0, cell_b.1)
        .map_err(|e| Error::from(e))?
        .unwrap();

    if a_type_script != b_type_script {
        debug!(
            "Compare cell type script: {:?}[{}] {:?} != {:?}[{}] {:?} => true",
            cell_a.1, cell_a.0, a_type_script, cell_b.1, cell_b.0, b_type_script
        );
        return Err(Error::CellTypeCanNotBeModified);
    }

    Ok(())
}

pub fn is_cell_data_equal(cell_a: (usize, Source), cell_b: (usize, Source)) -> Result<(), Error> {
    let a_data = high_level::load_cell_data(cell_a.0, cell_a.1).map_err(|e| Error::from(e))?;
    let b_data = high_level::load_cell_data(cell_b.0, cell_b.1).map_err(|e| Error::from(e))?;

    if a_data != b_data {
        debug!(
            "Compare cell data: {:?}[{}] {:?} != {:?}[{}] {:?} => true",
            cell_a.1, cell_a.0, a_data, cell_b.1, cell_b.0, b_data
        );
        return Err(Error::CellDataCanNotBeModified);
    }

    Ok(())
}

pub fn is_cell_capacity_lte(cell_a: (usize, Source), cell_b: (usize, Source)) -> Result<(), Error> {
    let a_capacity =
        high_level::load_cell_capacity(cell_a.0, cell_a.1).map_err(|e| Error::from(e))?;
    let b_capacity =
        high_level::load_cell_capacity(cell_b.0, cell_b.1).map_err(|e| Error::from(e))?;

    if a_capacity <= b_capacity {
        debug!(
            "Compare cell capacity: {:?}[{}] {:?} <= {:?}[{}] {:?} => true",
            cell_a.1, cell_a.0, a_capacity, cell_b.1, cell_b.0, b_capacity
        );
        return Err(Error::CellCapacityMustReduced);
    }

    Ok(())
}

pub fn is_cell_capacity_gte(cell_a: (usize, Source), cell_b: (usize, Source)) -> Result<(), Error> {
    let a_capacity =
        high_level::load_cell_capacity(cell_a.0, cell_a.1).map_err(|e| Error::from(e))?;
    let b_capacity =
        high_level::load_cell_capacity(cell_b.0, cell_b.1).map_err(|e| Error::from(e))?;

    if a_capacity >= b_capacity {
        debug!(
            "Compare cell capacity: {:?}[{}] {:?} >= {:?}[{}] {:?} => true",
            cell_a.1, cell_a.0, a_capacity, cell_b.1, cell_b.0, b_capacity
        );
        return Err(Error::CellCapacityMustIncreased);
    }

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

    if a_capacity != b_capacity {
        debug!(
            "Compare cell capacity: {:?}[{}] {:?} != {:?}[{}] {:?} => true",
            cell_a.1, cell_a.0, a_capacity, cell_b.1, cell_b.0, b_capacity
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
    fn test_is_unpacked_bytes_eq() {
        let a = hex_to_unpacked_bytes("0x0102").unwrap();
        let b = hex_to_unpacked_bytes("0x0102").unwrap();
        let c = hex_to_unpacked_bytes("0x0103").unwrap();

        assert!(is_unpacked_bytes_eq(&a, &b), "Expect a == b return true");
        assert!(!is_unpacked_bytes_eq(&a, &c), "Expect a == c return false");
    }
}
