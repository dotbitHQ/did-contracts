use super::constants::ScriptType;
use super::constants::CKB_HASH_PERSONALIZATION;
use super::error::Error;
use super::types::ScriptLiteral;
use blake2b_ref::{Blake2b, Blake2bBuilder};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes, packed::*, prelude::*},
    error::SysError,
    high_level, syscalls,
};
use das_types::{constants::WITNESS_HEADER, packed as das_packed};
use hex::FromHexError;
use std::prelude::v1::*;

pub use das_types::util::is_entity_eq;

pub fn hex_to_unpacked_bytes(input: &str) -> Result<bytes::Bytes, FromHexError> {
    let trimed_input = input.trim_start_matches("0x");
    if trimed_input == "" {
        Ok(bytes::Bytes::default())
    } else {
        Ok(bytes::Bytes::from(hex::decode(trimed_input)?))
    }
}

pub fn hex_to_bytes(input: &str) -> Result<Bytes, FromHexError> {
    let trimed_input = input.trim_start_matches("0x");
    if trimed_input == "" {
        Ok(Bytes::default())
    } else {
        Ok(bytes::Bytes::from(hex::decode(trimed_input)?).pack())
    }
}

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

pub fn script_literal_to_script(script: ScriptLiteral) -> Result<Script, FromHexError> {
    let code_hash = hex_to_byte32(script.code_hash)?;
    let args = hex_to_unpacked_bytes(script.args).map(|bytes| bytes.pack())?;
    Ok(Script::new_builder()
        .code_hash(code_hash)
        .hash_type(Byte::new(script.hash_type as u8))
        .args(args)
        .build())
}

pub fn is_unpacked_bytes_eq(a: &bytes::Bytes, b: &bytes::Bytes) -> bool {
    **a == **b
}

pub fn find_cell_by_script(
    script_type: ScriptType,
    script: &Script,
    source: Source,
) -> Result<Vec<(CellOutput, usize)>, Error> {
    find_cell(
        |cell, _| -> bool {
            match script_type {
                ScriptType::Lock => is_entity_eq(&cell.lock(), &script),
                _ => is_entity_eq(&cell.type_().to_opt().unwrap_or(Script::default()), &script),
            }
        },
        source,
    )
}

pub fn find_cell<F>(f: F, source: Source) -> Result<Vec<(CellOutput, usize)>, Error>
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
    let mut buf = [0u8; 1000];
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
        let witness = das_packed::Bytes::new_builder()
            .set(data.into_iter().map(das_packed::Byte::new).collect())
            .build();
        witnesses.push(witness);
    }

    Ok(witnesses)
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
