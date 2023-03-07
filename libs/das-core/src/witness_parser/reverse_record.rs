use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::convert::{TryFrom, TryInto};
use core::str::FromStr;

use ckb_std::ckb_constants::Source;
use ckb_std::error::SysError;
use ckb_std::syscalls;
use das_types::constants::*;

use super::super::error::*;
use super::super::util;
use super::lv_parser::*;

// Binary format: 'das'(3) + DATA_TYPE(4) + binary_data

#[derive(Debug)]
pub struct ReverseRecordWitness {
    // The index of the transaction's witnesses, this field is mainly used for debug.
    pub index: usize,
    pub version: u32,
    pub action: ReverseRecordAction,
    pub signature: Vec<u8>,
    pub sign_type: DasLockType,
    pub address_payload: Vec<u8>,
    pub proof: Vec<u8>,
    pub prev_nonce: Option<u32>,
    pub prev_account: String,
    pub next_root: [u8; 32],
    pub next_account: String,
}

pub struct ReverseRecordWitnessesIter<'a> {
    parser: &'a ReverseRecordWitnessesParser,
    current: usize,
}

impl<'a> Iterator for ReverseRecordWitnessesIter<'a> {
    type Item = Result<ReverseRecordWitness, Box<dyn ScriptError>>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.parser.get(self.current);
        self.current += 1;

        ret
    }
}

#[derive(Debug)]
pub struct ReverseRecordWitnessesParser {
    pub contains_updating: bool,
    pub contains_removing: bool,
    pub reverse_record_indexes: Vec<usize>,
}

impl ReverseRecordWitnessesParser {
    pub fn new() -> Result<Self, Box<dyn ScriptError>> {
        let mut contains_updating = false;
        let mut contains_removing = false;
        let mut reverse_record_indexes = Vec::new();
        let mut i = 0;
        let mut das_witnesses_started = false;

        loop {
            let mut buf = [0u8; (WITNESS_HEADER_BYTES
                + WITNESS_TYPE_BYTES
                + REVERSE_RECORD_WITNESS_VERSION_BYTES
                + REVERSE_RECORD_WITNESS_ACTION_BYTES)];
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
                    das_witnesses_started = true;

                    let data_type_in_int = u32::from_le_bytes(
                        buf.get(WITNESS_HEADER_BYTES..(WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES))
                            .unwrap()
                            .try_into()
                            .unwrap(),
                    );
                    match DataType::try_from(data_type_in_int) {
                        Ok(DataType::ReverseRecord) => {
                            reverse_record_indexes.push(i);

                            let start = WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES;
                            // Every ReverseRecord witness has the next fields, here we parse it one by one.
                            let (start, _) = parse_field("version", &buf, start)?;
                            let (_, action_bytes) = parse_field("action", &buf, start)?;
                            if action_bytes == ReverseRecordAction::Update.to_string().as_bytes() {
                                contains_updating = true;
                            } else if action_bytes == ReverseRecordAction::Remove.to_string().as_bytes() {
                                contains_removing = true;
                            }
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
                Err(e) => return Err(e.into()),
            }
        }

        let indexes_length = reverse_record_indexes.len();
        if indexes_length <= 0 {
            warn!("Can not find any ReverseRecord witness in this transaction.");
            return Err(code_to_error!(ErrorCode::WitnessEmpty));
        }

        Ok(ReverseRecordWitnessesParser {
            contains_updating,
            contains_removing,
            reverse_record_indexes,
        })
    }

    fn parse_witness(index: usize) -> Result<ReverseRecordWitness, Box<dyn ScriptError>> {
        debug!("  witnesses[{:>2}] Parsing ReverseRecordWitness ...", index);

        let raw = util::load_das_witnesses(index)?;
        let start = WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES;

        // Every ReverseRecord witness has the next fields, here we parse it one by one.
        let (start, version) = parse_field("version", &raw, start)?;
        let (start, action) = parse_field("action", &raw, start)?;
        let (start, signature) = parse_field("signature", &raw, start)?;
        let (start, sign_type) = parse_field("sign_type", &raw, start)?;
        let (start, address_payload) = parse_field("address_payload", &raw, start)?;
        let (start, proof) = parse_field("proof", &raw, start)?;
        let (start, prev_nonce) = parse_field("prev_nonce", &raw, start)?;
        let (start, prev_account) = parse_field("prev_account", &raw, start)?;
        let (start, next_root) = parse_field("next_root", &raw, start)?;
        let (_, next_account) = parse_field("next_account", &raw, start)?;

        let version = u32::from_le_bytes(version.try_into().map_err(|_| {
            warn!(
                "  witnesses[{:>2}] ReverseRecordWitness.version should be 4 bytes.",
                index
            );
            ErrorCode::WitnessStructureError
        })?);
        if version != 1 {
            warn!(
                "  witnesses[{:>2}] ReverseRecordWitness.version is {} which is invalid for now.",
                index, version
            );
            return Err(code_to_error!(ErrorCode::WitnessVersionOrTypeInvalid));
        }

        let action = match String::from_utf8(action.to_vec()) {
            Ok(action) => match ReverseRecordAction::from_str(action.as_str()) {
                Ok(val) => val,
                Err(e) => {
                    warn!(
                        "  witnesses[{:>2}] ReverseRecordWitness.action field parse failed: {:?}",
                        index, e
                    );
                    return Err(code_to_error!(ErrorCode::WitnessStructureError));
                }
            },
            Err(e) => {
                warn!(
                    "  witnesses[{:>2}] ReverseRecordWitness.action field parse failed: {}",
                    index, e
                );
                return Err(code_to_error!(ErrorCode::WitnessStructureError));
            }
        };

        let sign_type = DasLockType::try_from(u8::from_le_bytes(sign_type.try_into().map_err(|_| {
            warn!(
                "  witnesses[{:>2}] ReverseRecordWitness.sign_type should be 1 byte.",
                index
            );
            ErrorCode::WitnessStructureError
        })?))
        .map_err(|_| {
            warn!(
                "  witnesses[{:>2}] ReverseRecordWitness.sign_type should be a valid DasLockType .",
                index
            );
            ErrorCode::WitnessStructureError
        })?;

        let prev_nonce = if prev_nonce.is_empty() {
            None
        } else {
            Some(u32::from_le_bytes(prev_nonce.try_into().map_err(|_| {
                warn!(
                    "  witnesses[{:>2}] ReverseRecordWitness.prev_nonce should be 4 bytes.",
                    index
                );
                ErrorCode::WitnessStructureError
            })?))
        };
        let prev_account = if prev_nonce.is_none() {
            String::from("")
        } else {
            String::from_utf8(prev_account.to_vec()).map_err(|_| {
                warn!(
                    "  witnesses[{:>2}] ReverseRecordWitness.prev_account should be a valid string.",
                    index
                );
                ErrorCode::WitnessStructureError
            })?
        };
        let next_root: [u8; 32] = next_root.try_into().map_err(|_| {
            warn!(
                "  witnesses[{:>2}] ReverseRecordWitness.next_root should be 32 bytes.",
                index
            );
            ErrorCode::WitnessStructureError
        })?;
        let next_account = String::from_utf8(next_account.to_vec()).map_err(|_| {
            warn!(
                "  witnesses[{:>2}] ReverseRecordWitness.next_account should be a valid string.",
                index
            );
            ErrorCode::WitnessStructureError
        })?;

        debug!(
            "  ReverseRecord witnesses[{:>2}]: {{ version: {}, action: {}, signature: 0x{}, sign_type: {}, address_payload: 0x{}, proof: 0x{}, prev_nonce: {:?}, prev_account: {}, next_root: 0x{}, next_account: {} }}",
            index,
            version,
            action.to_string(),
            util::hex_string(signature),
            sign_type.to_string(),
            util::hex_string(address_payload),
            util::hex_string(proof),
            prev_nonce,
            prev_account,
            util::hex_string(&next_root),
            next_account,
        );

        Ok(ReverseRecordWitness {
            index,
            version,
            action,
            signature: signature.to_vec(),
            sign_type,
            address_payload: address_payload.to_vec(),
            proof: proof.to_vec(),
            prev_nonce,
            prev_account,
            next_root,
            next_account,
        })
    }

    pub fn iter(&self) -> ReverseRecordWitnessesIter {
        ReverseRecordWitnessesIter {
            parser: self,
            current: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.reverse_record_indexes.len()
    }

    pub fn get(&self, index: usize) -> Option<Result<ReverseRecordWitness, Box<dyn ScriptError>>> {
        match self.reverse_record_indexes.get(index) {
            None => return None,
            Some(&i) => Some(Self::parse_witness(i)),
        }
    }
}
