#[cfg(feature = "no_std")]
use alloc::string::{String, ToString};
#[cfg(feature = "no_std")]
use alloc::vec;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec;

#[cfg(feature = "no_std")]
use ckb_std::high_level;
#[cfg(feature = "no_std")]
use ckb_std::syscalls::{self, SysError};
use das_types::constants::{DataType, Source, WITNESS_HEADER, WITNESS_HEADER_BYTES, WITNESS_TYPE_BYTES};
use das_types::packed::{self, Reader};
use das_types::util as types_util;
use molecule::prelude::Entity;

use crate::constants::ScriptType;
use crate::error::WitnessParserError;

pub fn load_das_witnesses(index: usize) -> Result<Vec<u8>, WitnessParserError> {
    let mut buf = [0u8; 7];
    let ret = syscalls::load_witness(&mut buf, 0, index, Source::Input.into());

    match ret {
        // Data which length is too short to be DAS witnesses, so ignore it.
        Ok(_) => Err(WitnessParserError::BasicDataStructureError {
            index,
            msg: String::from("The witness is too short to be DID witness."),
        }),
        Err(SysError::LengthNotEnough(actual_size)) => {
            if let Some(buf) = buf.get(..3) {
                err_assert!(
                    buf == &WITNESS_HEADER,
                    WitnessParserError::BasicDataStructureError {
                        index,
                        msg: String::from("The witness should start with \"das\" 3 bytes."),
                    }
                );
            }

            // WARNING This limit may not be accurate. It is advisable to adjust it based on the data available on the chain.
            if actual_size > 33000 {
                return Err(WitnessParserError::BasicDataStructureError {
                    index,
                    msg: String::from(
                        "The witness should be less than 32KB because the signall lock do not support more than that.",
                    ),
                });
            }

            let mut buf = vec![0u8; actual_size];
            syscalls::load_witness(&mut buf, 0, index, Source::Input.into())
                .map_err(|err| WitnessParserError::SysError { index, err })?;
            Ok(buf)
        }
        Err(e) => Err(WitnessParserError::SysError { index, err: e }),
    }
}

pub fn load_cell_data(index: usize, source: Source) -> Result<Vec<u8>, WitnessParserError> {
    let mut buf = vec![0u8; 32];
    let ret = syscalls::load_cell_data(&mut buf, 0, index, source.into());

    match ret {
        Ok(_) => Ok(buf),
        Err(SysError::LengthNotEnough(actual_size)) => {
            let mut buf = vec![0u8; actual_size];
            syscalls::load_cell_data(&mut buf, 0, index, source.into())
                .map_err(|err| WitnessParserError::SysError { index, err })?;
            Ok(buf)
        }
        Err(e) => Err(WitnessParserError::SysError { index, err: e }),
    }
}

pub fn parse_date_type_from_witness(index: usize, buf: &[u8]) -> Result<DataType, WitnessParserError> {
    let data_type_in_int = u32::from_le_bytes(
        buf.get(WITNESS_HEADER_BYTES..(WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES))
            .unwrap()
            .try_into()
            .unwrap(),
    );

    let data_type = DataType::try_from(data_type_in_int).map_err(|_err| WitnessParserError::UndefinedDataType {
        index,
        date_type: data_type_in_int,
    })?;

    Ok(data_type)
}

pub fn parse_data_from_witness(index: usize, buf: &[u8]) -> Result<packed::Data, WitnessParserError> {
    match buf.get((WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES)..) {
        Some(bytes) => match packed::Data::from_slice(&bytes) {
            Ok(data) => Ok(data),
            Err(err) => {
                return Err(WitnessParserError::DecodingDataFailed {
                    index,
                    err: err.to_string(),
                });
            }
        },
        None => Err(WitnessParserError::LoadDataBodyFailed { index }),
    }
}

pub fn parse_raw_from_witness(index: usize, buf: &[u8]) -> Result<Vec<u8>, WitnessParserError> {
    match buf.get((WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES)..) {
        Some(bytes) => Ok(bytes.to_vec()),
        None => Err(WitnessParserError::LoadDataBodyFailed { index }),
    }
}

pub fn find_cells_by_script(
    witness_index: usize,
    script_type: ScriptType,
    script: packed::ScriptReader,
    source: Source,
) -> Result<Vec<usize>, WitnessParserError> {
    let mut i = 0;
    let mut cell_indexes = Vec::new();
    let expected_hash = types_util::blake2b_256(script.as_slice());
    loop {
        // TODO replace the high_level functions with tx-resolver to support std environment.
        let ret = match script_type {
            ScriptType::Lock => high_level::load_cell_lock_hash(i, source.into()).map(Some),
            _ => high_level::load_cell_type_hash(i, source.into()),
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
                return Err(WitnessParserError::SysError {
                    index: witness_index,
                    err,
                });
            }
        }

        i += 1;
    }

    Ok(cell_indexes)
}
