use std::time::{SystemTime, UNIX_EPOCH};

use ckb_hash::{Blake2bBuilder, CKB_HASH_PERSONALIZATION};
use das_types::constants::CKB_HASH_DIGEST;
use das_types::packed::*;
use das_types::prelude::*;
use did_cell_type::{parse_did_cell_account_info, DidCellAccountInfo};

fn uint64_to_u64(v: Uint64) -> u64 {
    let mut buf = [0u8; 8];
    (&mut buf).copy_from_slice(&v.as_slice());
    return u64::from_le_bytes(buf);
}

fn byte20_to_array(v: Byte20) -> [u8; 20] {
    let mut buf = [0u8; 20];

    (&mut buf).copy_from_slice(v.as_slice());

    return buf;
}

fn array_to_byte20(v: &[u8; 20]) -> Byte20 {
    Byte20::from_slice(&v[..]).unwrap()
}

#[test]
fn test_parse_did_cell_data() {
    let data_str = "66000000100000001400000042000000000000002a0000000001a7d4860aaf1dc83daedf75d6022811d2c2ae250b1b666d660000000032303233303631362e62697420000000cdb443dd0f9d98f530fd8945b86f3ea946f56ee4d015882beb757571bbd529f1";
    let data = hex::decode(data_str).unwrap();
    let data = SporeData::from_slice(&data).unwrap();
    let content = data.content().raw_data().to_vec();

    let info = parse_did_cell_account_info(&content).unwrap();

    println!(">>>>>> {:?}", info);

    let data = gen_did_cell_data(b"alex.bit", 100, &[1u8; 20]);

    let content = data.content().raw_data().to_vec();

    let info = parse_did_cell_account_info(&content);
    assert!(info.is_ok(), "error parse account info");
    let DidCellAccountInfo {
        account,
        expire_at,
        hash,
    } = info.unwrap();
    let account = String::from_utf8(account).unwrap();
    assert!(account == "alex.bit", "wrong account");
    assert!(expire_at == 100, "wrong expire at");
    assert!(hash == [1u8; 20], "wrong hash");
}

#[test]
fn test_type_convert() {
    let v: Uint64 = 123u64.into();
    let v: u64 = uint64_to_u64(v);
    println!(">>>>>> {v}");
    assert_eq!(v, 123u64);
    let v = [12u8; 20];
    let v: Byte20 = Byte20::from_slice(&v[..]).unwrap();
    let v: [u8; 20] = byte20_to_array(v);
    println!(">>>>>> {v:?}");
    assert_eq!(v, [12u8; 20]);

    let v = [111u8; 20];
    let v = array_to_byte20(&v);
    let v = byte20_to_array(v);
    println!(">>>>>> {v:?}");
}

fn update_did_cell_data(dcd: &DidCellData, expire_at: Option<u64>, hash: Option<&[u8]>) -> DidCellData {
    let dcd = dcd.clone().to_enum();
    match dcd {
        DidCellDataUnion::DidCellDataV0(dcd) => {
            let mut builder = dcd.as_builder();
            if let Some(expire_at) = expire_at {
                builder = builder.expire_at(expire_at.into());
            }
            if let Some(hash) = hash {
                builder = builder.witness_hash(hash.into());
            }

            let data: DidCellDataUnion = builder.build().into();
            let data: DidCellData = DidCellDataBuilder::default().set(data).build();

            data
        }
    }
}

#[test]
fn test_utils() {
    let expire_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let witness_hash = [1u8; 20];
    let data = gen_did_cell_data(b"xxx.bit", expire_at, &witness_hash);

    let data_str = format!("{data:#x}");
    let data1: SporeData = parse_entity(&data_str);
    println!(">>>>>\n{data}\n{data1}");

    let records = gen_records(2);
    let record = &records[0];
    let record_str = format!("{record:#x}");
    let record1: Record = parse_entity(&record_str);
    println!(">>>>> record \n{record}\n{record1}");

    let (witness_data, _) = gen_witness_data(records);
    let witness_str = format!("{witness_data:#x}");
    let witness_data1: WitnessData = parse_entity(&witness_str);
    println!(">>>>> witness_data \n{witness_data}\n{witness_data1}");

    let (entity, _hash) = gen_did_entity(2);

    println!("{}", entity);
}

pub fn now_in_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}
pub fn expire_in_secs(n: u64) -> u64 {
    now_in_secs() + n
}

pub fn gen_record(type_: &[u8], key: &[u8], label: &[u8], value: &[u8], record_ttl: u32) -> Record {
    let builder = RecordBuilder::default();
    let builder = builder.record_type(type_.into());
    let builder = builder.record_key(key.into());
    let builder = builder.record_label(label.into());
    let builder = builder.record_value(value.into());
    let builder = builder.record_ttl(record_ttl.into());

    builder.build()
}

pub fn parse_entity<E: Entity>(data_str: impl AsRef<str>) -> E {
    let mut data_str = data_str.as_ref();
    if data_str.starts_with("0x") {
        data_str = &data_str[2..];
    }
    let data = hex::decode(data_str).unwrap();
    Entity::new_unchecked(data.into())
}

pub fn gen_records(n: usize) -> Vec<Record> {
    let n = n as u32;
    let mut records: Vec<Record> = Vec::new();
    for i in 0..n {
        let type_ = format!("type_{i}");
        let key = format!("key_{i}");
        let label = format!("label_{i}");
        let value = format!("value_{i}");
        let record_ttl = i + 1234;
        records.push(gen_record(
            type_.as_bytes(),
            key.as_bytes(),
            label.as_bytes(),
            value.as_bytes(),
            record_ttl,
        ));
    }

    records
}

pub fn gen_witness_data(records: Vec<Record>) -> (WitnessData, [u8; 20]) {
    let records = RecordsBuilder::default().set(records).build();
    let data: WitnessDataUnion = DidCellWitnessDataV0Builder::default().records(records).build().into();
    let data: WitnessData = WitnessDataBuilder::default().set(data).build();
    let hash = blake2b_160(data.as_slice());
    (data, hash)
}

pub fn gen_did_entity(n: usize) -> (DidEntity, [u8; 20]) {
    let records = gen_records(n);
    let (witness_data, hash) = gen_witness_data(records);
    let builder = DidEntityBuilder::default();
    let hash_: Byte20 = (&hash).into();
    let hash_opt = Byte20OptBuilder::default().set(Some(hash_)).build();
    let builder = builder.hash(hash_opt);
    let builder = builder.data(witness_data);

    (builder.build(), hash)
}

// generate DidCellData
pub fn gen_did_cell_data(account: &[u8], expire_at: u64, witness_hash: &[u8; 20]) -> SporeData {
    let ver = [1u8];
    let expired_at = expire_at.to_le_bytes();
    let content: Bytes = [&[0u8][..], &ver[..], &witness_hash[..], &expired_at[..], account]
        .concat()
        .into();
    // let content: Bytes = content.into();
    let data = SporeDataBuilder::default()
        .cluster_id(BytesOpt::default())
        .content(content)
        .content_type(Bytes::default())
        .build();

    data
}

const HASH_DIGEST_160: usize = 20;
pub fn blake2b_160<T: AsRef<[u8]>>(s: T) -> [u8; HASH_DIGEST_160] {
    let mut result = [0u8; CKB_HASH_DIGEST];
    let mut blake2b = Blake2bBuilder::new(CKB_HASH_DIGEST)
        .personal(CKB_HASH_PERSONALIZATION)
        .build();
    blake2b.update(s.as_ref());
    blake2b.finalize(&mut result);

    let mut ret = [0u8; HASH_DIGEST_160];
    ret.copy_from_slice(&result[..20]);

    ret
}
