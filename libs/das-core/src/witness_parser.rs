use super::error::Error;
use super::util::{blake2b_256, is_entity_eq};
use ckb_std::ckb_constants::Source;
use das_types::{
    constants::{DataType, WITNESS_HEADER},
    packed::*,
    prelude::*,
};
use std::prelude::v1::*;

#[derive(Debug)]
pub struct WitnessesParser {
    pub action: Bytes,
    pub params: Bytes,
    pub dep: Vec<(u32, u32, u32, Hash, Bytes)>,
    pub old: Vec<(u32, u32, u32, Hash, Bytes)>,
    pub new: Vec<(u32, u32, u32, Hash, Bytes)>,
}

impl WitnessesParser {
    pub fn new(witnesses: Vec<Bytes>) -> Result<Self, Error> {
        // Parsing first witness as ActionData.
        let witness = witnesses.get(0).ok_or(Error::WitnessEmpty)?;
        Self::verify_das_header(&witness)?;
        let type_ = Self::parse_data_type(&witness)?;
        if type_ != DataType::ActionData as u32 {
            return Err(Error::WitnessActionIsNotTheFirst);
        }
        let action_data = Self::parse_action_data(&witness)?;

        let action = Bytes::from_slice(action_data.action().as_slice()).unwrap();
        let params = Bytes::from_slice(action_data.params().as_slice()).unwrap();

        let mut dep = Vec::new();
        let mut old = Vec::new();
        let mut new = Vec::new();
        for witness in witnesses.into_iter().skip(1) {
            Self::verify_das_header(&witness)?;

            let type_ = Self::parse_data_type(&witness)?;

            let data = Self::parse_data(&witness)?;
            if let Some(entity) = data.dep().to_opt() {
                dep.push(Self::parse_entity(entity, type_)?)
            }
            if let Some(entity) = data.old().to_opt() {
                old.push(Self::parse_entity(entity, type_)?)
            }
            if let Some(entity) = data.new().to_opt() {
                new.push(Self::parse_entity(entity, type_)?)
            }
        }

        Ok(WitnessesParser {
            action,
            params,
            dep,
            old,
            new,
        })
    }

    fn verify_das_header(witness: &Bytes) -> Result<(), Error> {
        if let Some(raw) = witness.as_slice().get(4..7) {
            if raw != &WITNESS_HEADER {
                return Err(Error::WitnessDasHeaderDecodingError);
            }
        } else {
            return Err(Error::WitnessDasHeaderDecodingError);
        };

        Ok(())
    }

    fn parse_data_type(witness: &Bytes) -> Result<u32, Error> {
        if let Some(raw) = witness.as_slice().get(7..11) {
            let type_in_int = match Uint32Reader::verify(raw, false) {
                Ok(()) => Uint32::new_unchecked(raw.to_vec().into()),
                Err(_) => return Err(Error::WitnessTypeDecodingError),
            };
            Ok(u32::from(type_in_int))
        } else {
            Err(Error::WitnessTypeDecodingError)
        }
    }

    fn parse_action_data(witness: &Bytes) -> Result<ActionData, Error> {
        if let Some(raw) = witness.as_slice().get(11..) {
            let action_data = match ActionData::from_slice(raw) {
                Ok(action_data) => action_data,
                Err(_) => return Err(Error::WitnessActionDecodingError),
            };
            Ok(action_data)
        } else {
            Err(Error::WitnessActionDecodingError)
        }
    }

    fn parse_data(witness: &Bytes) -> Result<Data, Error> {
        if let Some(raw) = witness.as_slice().get(11..) {
            let data = match Data::from_slice(raw) {
                Ok(data) => data,
                Err(_) => return Err(Error::WitnessActionDecodingError),
            };
            Ok(data)
        } else {
            Err(Error::WitnessActionDecodingError)
        }
    }

    fn parse_entity(entity: DataEntity, type_: u32) -> Result<(u32, u32, u32, Hash, Bytes), Error> {
        let index = u32::from(entity.index());
        let version = u32::from(entity.version());
        let data = entity.entity();

        let entity_data = data
            .as_slice()
            .get(4..)
            .ok_or(Error::WitnessEntityDecodingError)?;
        let hash = Hash::new_unchecked(blake2b_256(entity_data).to_vec().into());
        // eprintln!("entity = {:#?}", data);
        // eprintln!("hash = {:#?}", hash);

        Ok((
            index,
            version,
            type_,
            hash,
            Bytes::from_slice(data.as_slice()).unwrap(),
        ))
    }

    // pub fn action_str(&self) -> &str {}

    pub fn get(&self, index: &u32, hash: &Hash, source: Source) -> Result<&Bytes, Error> {
        let group = match source {
            Source::Input => &self.old,
            Source::Output => &self.new,
            Source::CellDep => &self.dep,
            _ => {
                return Err(Error::HardCodedError);
            }
        };

        let entity;
        if let Some((_, _, _, _hash, _entity)) = group.iter().find(|&(i, _, _, _h, _)| i == index) {
            if is_entity_eq(hash, _hash) {
                entity = _entity
            } else {
                return Err(Error::CellDataIsCorrupted);
            }
        } else {
            return Err(Error::CellDataMissing);
        }

        Ok(entity)
    }
}

#[cfg(test)]
mod test {
    use super::super::util::hex_to_byte32;
    use super::*;
    use das_types::util::is_entity_eq;

    fn restore_bytes_from_hex(input: &str) -> Bytes {
        let trimed_input = input.trim_start_matches("0x");
        Bytes::new_unchecked(hex::decode(trimed_input).unwrap().into())
    }

    fn before_each() -> WitnessesParser {
        let witnesses = vec![
            restore_bytes_from_hex("0x21000000646173000000001a0000000c0000001600000006000000636f6e66696700000000"),
            restore_bytes_from_hex("0x0501000064617301000000fe000000100000001000000010000000ee0000001000000014000000180000000000000001000000d2000000d2000000380000003c0000003d0000003e00000042000000460000004a0000004e0000005200000056000000c6000000ca000000ce00000000000000040232000000320000003c00000080510100e803000004000000700000001000000030000000500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c0100008051010080510100"),
        ];

        WitnessesParser::new(witnesses).unwrap()
    }

    #[test]
    #[should_panic]
    fn fn_new_should_failed_when_witness_has_no_das_header() {
        let witnesses = vec![
            restore_bytes_from_hex("0x21000000646173060000001a0000000c0000001600000006000000636f6e66696700000000"),
            restore_bytes_from_hex("0x00000000"),
            restore_bytes_from_hex("0x0501000064617301000000fe0000001000000010000000fe000000ee0000001000000014000000180000000000000001000000d2000000d2000000380000003c0000003d0000003e00000042000000460000004a0000004e0000005200000056000000c6000000ca000000ce00000000000000040232000000320000003c00000080510100e803000004000000700000001000000030000000500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c0100008051010080510100"),
        ];

        WitnessesParser::new(witnesses).unwrap();
    }

    #[test]
    fn fn_new_should_parse_witnesses_as_expected() {
        let parser = before_each();

        let config_in_bytes = "config"
            .as_bytes()
            .to_owned()
            .into_iter()
            .map(Byte::new)
            .collect::<Vec<_>>();

        assert!(
            is_entity_eq(
                &parser.action,
                &Bytes::new_builder().set(config_in_bytes).build()
            ),
            "Action should be \"config\"."
        );
        assert!(
            is_entity_eq(&parser.params, &Bytes::default()),
            "Params should be empty"
        );

        let (index, version, data_type, hash, _) = parser.new.get(0).unwrap();

        assert_eq!(0, index.to_owned());
        assert_eq!(1, version.to_owned());
        assert_eq!(DataType::ConfigCellData as u32, data_type.to_owned());
        assert!(is_entity_eq(
            &Hash::from(
                hex_to_byte32("0x04de45a843802e1c0bb8f1e382ee23be1434c36693eac143f61bbaf04dc901cb")
                    .unwrap()
            ),
            hash
        ));
    }

    #[test]
    #[should_panic]
    fn fn_get_should_check_hash_when_getting_data() {
        let parser = before_each();

        let hash = Hash::from(
            hex_to_byte32("0x000045a843802e1c0bb8f1e382ee23be1434c36693eac143f61bbaf04dc90000")
                .unwrap(),
        );
        parser.get(&0, &hash, Source::Output).unwrap();
    }

    #[test]
    fn fn_get_should_return_correctly() {
        let parser = before_each();

        let hash = Hash::from(
            hex_to_byte32("0x04de45a843802e1c0bb8f1e382ee23be1434c36693eac143f61bbaf04dc901cb")
                .unwrap(),
        );
        let entity = parser.get(&0, &hash, Source::Output).unwrap();

        let entity_data = entity.as_slice().get(4..).unwrap();
        let result = Hash::new_unchecked(blake2b_256(entity_data).to_vec().into());

        assert!(
            is_entity_eq(&result, &hash),
            "The hash of the returned data should match."
        )
    }
}
