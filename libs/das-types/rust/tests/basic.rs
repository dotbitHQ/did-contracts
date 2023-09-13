use core::convert::TryFrom;

use ckb_types::packed as ckb_packed;
use das_types::packed::*;
use das_types::util::*;

#[test]
fn should_support_u8() {
    let num: u8 = u8::MAX;
    let data = Uint8::from(num);
    let reader = data.as_reader();
    // println!("{:?}", data);

    assert_eq!(num, u8::from(reader));
    assert_eq!(num, u8::from(data));
}

#[test]
fn should_support_u32() {
    let num: u32 = u32::MAX;
    let data = Uint32::from(num);
    let reader = data.as_reader();
    // println!("{:?}", data);

    assert_eq!(num, u32::from(reader));
    assert_eq!(num, u32::from(data));
}

#[test]
fn should_support_u64() {
    let num: u64 = u64::MAX;
    let data = Uint64::from(num);
    let reader = data.as_reader();
    // println!("{:?}", data);

    assert_eq!(num, u64::from(reader));
    assert_eq!(num, u64::from(data));
}

#[test]
fn should_support_bytes() {
    // Convert from Bytes between Vec<u8>
    let text_in_vec = Vec::from("hello world");
    let data = Bytes::from(text_in_vec.clone());

    assert_eq!(Vec::from(data), text_in_vec);

    // Convert from Bytes to String
    let text = "hello world";
    let data = Bytes::from(text.as_bytes().to_vec());

    assert_eq!(String::try_from(data), Ok(String::from(text)));

    // Convert from ckb_std packed Bytes to das packed Bytes
    let ckb_bytes = ckb_packed::Bytes::default();
    let data = Bytes::default();

    assert!(is_entity_eq(&Bytes::from(ckb_bytes), &data));

    // Convert from das packed Bytes to ckb_std packed Bytes
    let ckb_bytes = ckb_packed::Bytes::default();
    let data = Bytes::default();

    assert!(is_entity_eq(&ckb_bytes, &data.into()));
}

#[test]
fn should_support_hash() {
    // Convert from Hash between Vec
    let expected = vec![
        160, 236, 23, 20, 166, 65, 57, 181, 240, 228, 109, 29, 29, 228, 242, 231, 179, 39, 53, 255, 237, 170, 179, 66,
        133, 196, 159, 46, 82, 105, 171, 218,
    ];
    let result = Hash::try_from(expected.clone()).unwrap();

    assert_eq!(Vec::from(result), expected);

    // Convert from ckb_std packed Bytes to das packed Bytes
    let ckb_byte32 = ckb_packed::Byte32::default();
    let result = Hash::from(ckb_byte32);
    let expected = Hash::default();

    assert!(is_entity_eq(&result, &expected));

    // Convert from das packed Bytes to ckb_std packed Bytes
    let ckb_byte32 = ckb_packed::Byte32::default();
    let data = Hash::default();

    assert!(is_entity_eq(&ckb_byte32.into(), &data));
}
