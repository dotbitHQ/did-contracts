use super::{
    constants::*, debug, error::Error, types::ScriptLiteral, witness_parser::WitnessesParser,
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
    if cells.len() != 1 {
        return Err(Error::InvalidTransactionStructure);
    }

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
    let data = load_cell_data(ret[0], Source::CellDep)?;
    let timestamp = match data.get(1..) {
        Some(bytes) => {
            if bytes.len() != 4 {
                return Err(Error::TimeCellDataDecodingError);
            }
            u32::from_be_bytes(bytes.try_into().unwrap())
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
    let data = load_cell_data(ret[0], Source::CellDep)?;
    let height = match data.get(1..) {
        Some(bytes) => {
            if bytes.len() != 8 {
                return Err(Error::HeightCellDataDecodingError);
            }
            u64::from_be_bytes(bytes.try_into().unwrap())
        }
        _ => return Err(Error::HeightCellDataDecodingError),
    };

    Ok(height)
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

    if action_data_opt.is_none() {
        debug!("Can not found action in witnesses.");
        return Err(Error::WitnessActionNotFound);
    }

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

pub fn get_length_in_price(account_length: u64) -> u8 {
    if account_length > ACCOUNT_MAX_PRICED_LENGTH.into() {
        ACCOUNT_MAX_PRICED_LENGTH
    } else {
        account_length as u8
    }
}

pub fn get_account_storage_total(account_length: u64) -> u64 {
    ACCOUNT_CELL_BASIC_CAPACITY + (account_length * 100_000_000) + REF_CELL_BASIC_CAPACITY * 2
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
        TypeScript::RefCellType => config.type_id_table().ref_cell(),
        TypeScript::WalletCellType => config.type_id_table().wallet_cell(),
    };

    // Find out required cell in current transaction.
    let required_cells = find_cells_by_type_id(ScriptType::Type, type_id, source)?;

    // There must be some required cells in the transaction.
    if required_cells.len() <= 0 {
        return Err(err);
    }

    debug!("Require on: {:?}", type_script);

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
