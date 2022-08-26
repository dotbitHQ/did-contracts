use super::{assert, data_parser, debug, error::Error, util, warn};
use alloc::vec::Vec;
use ckb_std::{ckb_constants::Source, error::SysError, syscalls};
use core::{
    convert::{TryFrom, TryInto},
    lazy::OnceCell,
};
use das_dynamic_libs::constants::DasLockType;
use das_types::{constants::*, packed::*, prelude::*};

#[cfg(all(debug_assertions))]
use alloc::string::String;
#[cfg(all(debug_assertions))]
use das_types::prettier::Prettier;

// Binary format: 'das'(3) + DATA_TYPE(4) + binary_data

#[derive(Debug)]
pub struct SubAccountWitness {
    // The index of the transaction's witnesses, this field is mainly used for debug.
    pub index: usize,
    // The rest is actually existing fields in the witness.
    pub signature: Vec<u8>,
    pub sign_role: Option<LockRole>,
    pub sign_type: Option<DasLockType>,
    pub sign_args: Vec<u8>,
    pub prev_root: Vec<u8>,
    pub current_root: Vec<u8>,
    pub proof: Vec<u8>,
    pub version: u32,
    pub sub_account: SubAccount,
    pub edit_key: Vec<u8>,
    pub edit_value: SubAccountEditValue,
    pub edit_value_bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum SubAccountEditValue {
    None,
    ExpiredAt(Uint64),
    Owner(Vec<u8>),
    Manager(Vec<u8>),
    Records(Records),
}

pub struct SubAccountWitnessesIter<'a> {
    parser: &'a SubAccountWitnessesParser,
    current: usize,
}

impl<'a> Iterator for SubAccountWitnessesIter<'a> {
    type Item = Result<&'a SubAccountWitness, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.parser.get(self.current);
        self.current += 1;

        ret
    }
}

#[derive(Debug)]
pub struct SubAccountWitnessesParser {
    pub indexes: Vec<usize>,
    pub witnesses: Vec<OnceCell<SubAccountWitness>>,
}

impl SubAccountWitnessesParser {
    pub fn new() -> Result<Self, Error> {
        let mut indexes = Vec::new();
        let mut i = 0;
        let mut das_witnesses_started = false;
        loop {
            let mut buf = [0u8; (WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES)];
            let ret = syscalls::load_witness(&mut buf, 0, i, Source::Input);

            match ret {
                // Data which length is too short to be DAS witnesses, so ignore it.
                Ok(_) => i += 1,
                Err(SysError::LengthNotEnough(_)) => {
                    if let Some(raw) = buf.get(..WITNESS_HEADER_BYTES) {
                        if das_witnesses_started {
                            // If it is parsing DAS witnesses currently, end the parsing.
                            if raw != &WITNESS_HEADER {
                                break;
                            }
                        } else {
                            // If it is not parsing DAS witnesses currently, continue to detect the next witness.
                            if raw != &WITNESS_HEADER {
                                i += 1;
                                continue;
                            }
                        }
                    }

                    let data_type_in_int = u32::from_le_bytes(
                        buf.get(WITNESS_HEADER_BYTES..(WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES))
                            .unwrap()
                            .try_into()
                            .unwrap(),
                    );
                    match DataType::try_from(data_type_in_int) {
                        Ok(DataType::SubAccount) => {
                            if !das_witnesses_started {
                                das_witnesses_started = true
                            }

                            indexes.push(i);
                        }
                        Ok(_) => {
                            // Ignore other witnesses in this parser.
                        }
                        Err(_) => {
                            // Ignore unknown DataTypes which will make adding new DataType much easier and no need to update every contracts.
                            debug!(
                                "Ignored unknown DataType {:?} for compatible purpose.",
                                data_type_in_int
                            );
                        }
                    }

                    i += 1;
                }
                Err(SysError::IndexOutOfBound) => break,
                Err(e) => return Err(Error::from(e)),
            }
        }

        let indexes_length = indexes.len();
        if indexes_length <= 0 {
            warn!("Can not find any sub-account witness in this transaction.");
            return Err(Error::WitnessEmpty);
        }

        let mut witnesses = Vec::with_capacity(indexes_length);
        for _ in indexes.iter() {
            let cell = OnceCell::new();
            witnesses.push(cell);
        }

        Ok(SubAccountWitnessesParser { indexes, witnesses })
    }

    fn parse_witness(i: usize) -> Result<SubAccountWitness, Error> {
        debug!("Parsing sub-accounts witnesses[{}] ...", i);

        let raw = util::load_das_witnesses(i)?;
        let start = WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES;

        // Every sub-account witness has the next fields, here we parse it one by one.
        let (start, signature) = Self::parse_field(&raw, start)?;
        let (start, sign_role_byte) = Self::parse_field(&raw, start)?;
        let (start, prev_root) = Self::parse_field(&raw, start)?;
        let (start, current_root) = Self::parse_field(&raw, start)?;
        let (start, proof) = Self::parse_field(&raw, start)?;
        let (start, version_bytes) = Self::parse_field(&raw, start)?;
        let (start, sub_account_bytes) = Self::parse_field(&raw, start)?;
        let (start, edit_key) = Self::parse_field(&raw, start)?;
        let (_, edit_value_bytes) = Self::parse_field(&raw, start)?;

        assert!(
            version_bytes.len() == 4,
            Error::WitnessStructureError,
            "  witnesses[{}] Sub-account witness structure error, the version field should be 4 bytes",
            i
        );
        let version = u32::from_le_bytes(version_bytes.try_into().unwrap());

        assert!(
            version == 1,
            Error::WitnessVersionOrTypeInvalid,
            "  witnesses[{}] The version of sub-account witness is invalid.",
            i
        );

        let sub_account = match SubAccount::from_slice(sub_account_bytes) {
            Ok(val) => val,
            Err(e) => {
                warn!(
                    "  witnesses[{}] Sub-account witness structure error, the sub_account field parse failed: {}",
                    i, e
                );
                return Err(Error::WitnessStructureError);
            }
        };

        // The actual type of the edit_value field is base what the edit_key field is.
        let edit_value = match edit_key {
            b"expired_at" => {
                let expired_at = match Uint64::from_slice(edit_value_bytes) {
                    Ok(val) => val,
                    Err(e) => {
                        warn!(
                            "  witnesses[{}] Sub-account witness structure error, decoding expired_at failed: {}",
                            i, e
                        );
                        return Err(Error::WitnessStructureError);
                    }
                };

                SubAccountEditValue::ExpiredAt(expired_at)
            }
            b"owner" => SubAccountEditValue::Owner(edit_value_bytes.to_vec()),
            b"manager" => SubAccountEditValue::Manager(edit_value_bytes.to_vec()),
            b"records" => {
                let records = match Records::from_slice(edit_value_bytes) {
                    Ok(val) => val,
                    Err(e) => {
                        warn!(
                            "  witnesses[{}] Sub-account witness structure error, decoding records failed: {}",
                            i, e
                        );
                        return Err(Error::WitnessStructureError);
                    }
                };

                SubAccountEditValue::Records(records)
            }
            _ => SubAccountEditValue::None,
        };

        let mut sign_type = None;
        let mut sign_args = Vec::new();
        let mut sign_role = None;
        if sign_role_byte.len() == 1 {
            let sign_role_int = u8::from_le_bytes(sign_role_byte.try_into().unwrap());
            let args = sub_account.as_reader().lock().args();
            let args_bytes = args.raw_data();

            let sign_type_int;
            let sign_args_ref;
            if sign_role_int == LockRole::Owner as u8 {
                sign_type_int = data_parser::das_lock_args::get_owner_type(args_bytes);
                sign_args_ref = data_parser::das_lock_args::get_owner_lock_args(args_bytes);
                sign_role = Some(LockRole::Owner);
            } else {
                sign_type_int = data_parser::das_lock_args::get_manager_type(args_bytes);
                sign_args_ref = data_parser::das_lock_args::get_manager_lock_args(args_bytes);
                sign_role = Some(LockRole::Manager);
            };

            sign_type = DasLockType::try_from(sign_type_int).ok();
            sign_args = sign_args_ref.to_vec();
        }

        debug!(
            "  Sub-account witnesses[{}]: {{ signature: 0x{}, sign_role: 0x{}, prev_root: 0x{}, current_root: 0x{}, version: {}, sub_account: {}, edit_key: {}, sign_args: {} }}",
            i, util::hex_string(signature), util::hex_string(sign_role_byte), util::hex_string(prev_root), util::hex_string(current_root), version, sub_account.account().as_prettier(), String::from_utf8(edit_key.to_vec()).unwrap(), util::hex_string(&sign_args)
        );

        Ok(SubAccountWitness {
            index: i,
            signature: signature.to_vec(),
            sign_role,
            sign_type,
            sign_args,
            prev_root: prev_root.to_vec(),
            current_root: current_root.to_vec(),
            proof: proof.to_vec(),
            version,
            sub_account,
            edit_key: edit_key.to_vec(),
            edit_value,
            edit_value_bytes: edit_value_bytes.to_vec(),
        })
    }

    fn parse_field(bytes: &[u8], start: usize) -> Result<(usize, &[u8]), Error> {
        // Every field is start with 4 bytes of uint32 as its length.
        let length = match bytes.get(start..(start + WITNESS_LENGTH_BYTES)) {
            Some(bytes) => {
                assert!(
                    bytes.len() == 4,
                    Error::WitnessStructureError,
                    "  Sub-account witness structure error, expect {}..{} to be bytes of LE uint32.",
                    start,
                    start + WITNESS_LENGTH_BYTES
                );

                u32::from_le_bytes(bytes.try_into().unwrap()) as usize
            }
            None => {
                warn!(
                    "  Sub-account witness structure error, expect 4 bytes in {}..{} .",
                    start,
                    start + WITNESS_LENGTH_BYTES
                );
                return Err(Error::WitnessStructureError);
            }
        };

        // Slice the field base on the start and length.
        let from = start + WITNESS_LENGTH_BYTES;
        let to = from + length;
        let field_bytes = match bytes.get(from..to) {
            Some(bytes) => bytes,
            None => {
                warn!(
                    "  Sub-account witness structure error, expect {} bytes in {}..{} .",
                    length, from, to
                );
                return Err(Error::WitnessStructureError);
            }
        };

        let new_start = start + WITNESS_LENGTH_BYTES + length;
        Ok((new_start, field_bytes))
    }

    pub fn iter(&self) -> SubAccountWitnessesIter {
        SubAccountWitnessesIter {
            parser: self,
            current: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.indexes.len()
    }

    pub fn get(&self, index: usize) -> Option<Result<&SubAccountWitness, Error>> {
        match self.indexes.get(index) {
            None => return None,
            Some(&i) => self
                .witnesses
                .get(index)
                .map(|cell| cell.get_or_try_init(|| -> Result<SubAccountWitness, Error> { Self::parse_witness(i) })),
        }
    }
}
