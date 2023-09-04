use alloc::collections::BTreeMap;
use alloc::{format, vec};
use core::cmp::Ordering;
#[cfg(debug_assertions)]
use core::fmt;
use std::prelude::v1::*;

use super::debug;
use super::error::*;
use super::util::*;

#[derive(Debug)]
pub struct TypedDataV4 {
    pub types: Types,
    pub primary_type: Value,
    pub domain: Value,
    pub message: Value,
}

impl TypedDataV4 {
    pub fn new(
        types: Types,
        primary_type: String,
        domain: (Vec<String>, BTreeMap<String, Value>),
        message: (Vec<String>, BTreeMap<String, Value>),
    ) -> Self {
        TypedDataV4 {
            types,
            primary_type: Value::String(primary_type),
            domain: Value::Object(domain),
            message: Value::Object(message),
        }
    }

    pub fn digest(&mut self, digest: String) {
        if let Value::Object((_, ref mut message)) = self.message {
            message.insert(String::from("digest"), Value::Byte32(digest));
        }
    }
}

#[cfg(debug_assertions)]
impl fmt::Display for TypedDataV4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> ::core::fmt::Result {
        macro_rules! types_to_json {
            ($name:expr) => {{
                let types_vec = self.types.get($name).unwrap();
                let mut json_str = String::from("[ ");
                let mut comma = "";
                for (name, type_) in types_vec {
                    json_str = json_str + comma + &format!(r#"{{ "name": "{}", "type": "{}" }}"#, name, type_);
                    comma = ", ";
                }
                json_str += " ]";
                json_str
            }};
        }

        let type_eip712domain = types_to_json!("EIP712Domain");
        let type_action = types_to_json!("Action");
        let type_cell = types_to_json!("Cell");
        let type_transaction = types_to_json!("Transaction");
        let types = format!(
            r#"{{ "EIP712Domain": {}, "Action": {}, "Cell": {}, "Transaction": {} }}"#,
            type_eip712domain, type_action, type_cell, type_transaction
        );

        write!(
            f,
            r#"{{ "types": {}, "primaryType": "Transaction", "domain": {}, "message": {} }}"#,
            types, self.domain, self.message
        )
    }
}

pub type Types = BTreeMap<String, Vec<(String, String)>>;

#[derive(Debug)]
pub enum Value {
    Array(Vec<Value>),
    String(String),
    Byte32(String),
    Bytes(String),
    Address(String),
    Uint256(String),
    Object((Vec<String>, BTreeMap<String, Value>)),
}

#[cfg(debug_assertions)]
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Value::Object((keys, value)) => {
                let mut json_str = String::from("{ ");
                let mut comma = "";
                for key in keys.iter() {
                    let item = value.get(key).unwrap();
                    json_str = json_str + comma + &format!(r#""{}": {}"#, key, item);
                    comma = ", "
                }
                json_str += " }";

                write!(f, "{}", json_str)
            }
            Value::Array(value) => {
                let mut json_str = String::from("[ ");
                let mut comma = "";
                for item in value.iter() {
                    json_str = json_str + comma + &item.to_string();
                    comma = ", "
                }
                json_str += " ]";

                write!(f, "{}", json_str)
            }
            Value::String(value)
            | Value::Bytes(value)
            | Value::Byte32(value)
            | Value::Address(value)
            | Value::Uint256(value) => {
                write!(f, r#""{}""#, value)
            }
        }
    }
}

impl Value {
    fn encode(
        &self,
        domain_types: &Types,
        name: &str,
        type_: &str,
        _deep: usize,
    ) -> Result<(&'static str, Vec<u8>), EIP712EncodingError> {
        let ret = match self {
            Value::Object((_, value)) => {
                debug!("{:-width$}Encoding {} ==========", "", name, width = _deep * 2);

                let bytes = encode_message(domain_types, type_, value, _deep + 1)?;
                let hash = keccak256(&bytes);

                debug!(
                    "{:-width$}encode_field: {}: {} -> {} {}",
                    "",
                    name,
                    type_,
                    "bytes32",
                    hex::encode(&hash),
                    width = _deep * 2
                );

                ("bytes32", hash)
            }
            Value::Array(value) => {
                debug!("{:-width$}Encoding {} ==========", "", name, width = _deep * 2);

                let mut sub_types = Vec::new();
                let mut sub_values = Vec::new();
                for item in value.iter() {
                    let (sub_type, sub_bytes) = item.encode(domain_types, name, parse_type(type_), _deep + 1)?;
                    sub_types.push(sub_type);
                    sub_values.push(sub_bytes);
                }
                let bytes = eth_abi_encode(sub_types, sub_values.iter().map(AsRef::as_ref).collect())?;
                let hash = keccak256(&bytes);

                debug!(
                    "{:-width$}encode_field: {}: {} -> {} {}",
                    "",
                    name,
                    type_,
                    "bytes32",
                    hex::encode(&hash),
                    width = _deep * 2
                );

                ("bytes32", hash)
            }
            Value::String(value) => {
                let hash = keccak256(value.as_bytes());

                debug!(
                    "{:-width$}encode_field: {}: {} -> {} {}",
                    "",
                    name,
                    type_,
                    "bytes32",
                    hex::encode(&hash),
                    width = _deep * 2
                );

                ("bytes32", hash)
            }
            Value::Bytes(value) => {
                let bytes: Vec<u8> = hex::decode(value.trim_start_matches("0x")).map_err(|_| {
                    debug!(
                        "{:-width$}encode_field failed: {}: {}",
                        "",
                        name,
                        type_,
                        width = _deep * 2
                    );
                    EIP712EncodingError::HexDecodingError
                })?;
                let hash = keccak256(&bytes);

                debug!(
                    "{:-width$}encode_field: {}: {} -> {} {}",
                    "",
                    name,
                    type_,
                    "bytes32",
                    hex::encode(&hash),
                    width = _deep * 2
                );

                ("bytes32", hash)
            }
            Value::Byte32(value) => {
                let bytes: Vec<u8> = hex::decode(value.trim_start_matches("0x")).map_err(|_| {
                    debug!(
                        "{:-width$}encode_field failed: {}: {}",
                        "",
                        name,
                        type_,
                        width = _deep * 2
                    );

                    EIP712EncodingError::HexDecodingError
                })?;

                debug!(
                    "{:-width$}encode_field: {}: {} -> {} {}",
                    "",
                    name,
                    type_,
                    "bytes32",
                    hex::encode(&bytes),
                    width = _deep * 2
                );

                ("bytes32", bytes)
            }
            Value::Address(value) => {
                let bytes: Vec<u8> = hex::decode(value.trim_start_matches("0x")).map_err(|_| {
                    debug!(
                        "{:-width$}encode_field failed: {}: {}",
                        "",
                        name,
                        type_,
                        width = _deep * 2
                    );
                    EIP712EncodingError::HexDecodingError
                })?;

                debug!(
                    "{:-width$}encode_field: {}: {} -> {} {}",
                    "",
                    name,
                    type_,
                    "address",
                    hex::encode(&bytes),
                    width = _deep * 2
                );

                ("address", bytes)
            }
            Value::Uint256(value) => {
                let num = value
                    .parse::<u64>()
                    .map_err(|_| EIP712EncodingError::FailedWhenEncodingTypes)?;
                let bytes = num.to_be_bytes().to_vec();

                debug!(
                    "{:-width$}encode_field: {}: {} -> {} {}",
                    "",
                    name,
                    type_,
                    "uint256",
                    hex::encode(&bytes),
                    width = _deep * 2
                );

                ("uint256", bytes)
            }
        };

        Ok(ret)
    }
}

pub fn hash_data(typed_data: &TypedDataV4) -> Result<Vec<u8>, EIP712EncodingError> {
    // The first part of EIP712 hash which is a constant `0x1901`.
    let part1 = vec![25u8, 1];
    let part2;
    if let Value::Object((_, domain)) = &typed_data.domain {
        part2 = hash_message(&typed_data.types, "EIP712Domain", domain, 0)?;
    } else {
        unreachable!();
    }

    let primary_type;
    if let Value::String(val) = &typed_data.primary_type {
        primary_type = val;
    } else {
        unreachable!();
    }

    let mut part3 = Vec::new();
    if primary_type != "EIP712Domain" {
        if let Value::Object((_, message)) = &typed_data.message {
            part3 = hash_message(&typed_data.types, primary_type, message, 0)?;
        } else {
            unreachable!();
        }
    }

    // debug!("part1: {:?}", hex::encode(part1.clone()));
    // debug!("part2: {:?}", hex::encode(part2.clone()));
    // debug!("part3: {:?}", hex::encode(part3.clone()));

    let bytes = vec![part1, part2, part3].concat();
    Ok(keccak256(&bytes))
}

pub fn hash_type(
    domain_types: &Types,
    primary_type: &str,
    _deep: usize,
) -> Result<(&'static str, Vec<u8>), EIP712EncodingError> {
    let types_string = encode_type(domain_types, primary_type, _deep)?;
    let hash = keccak256(types_string.as_bytes());

    debug!(
        "{:-width$}hash_type: {} -> {} {}",
        "",
        primary_type,
        "bytes32",
        hex::encode(&hash),
        width = _deep * 2
    );

    Ok(("bytes32", hash))
}

pub fn encode_type(domain_types: &Types, primary_type: &str, _deep: usize) -> Result<String, EIP712EncodingError> {
    let mut dep_types = Vec::new();
    find_type_dependencies(domain_types, primary_type, &mut dep_types)?;
    // Sort by ascii in ascending order
    dep_types.sort();

    // Push primary_type as the first element of the vector.
    dep_types = vec![vec![String::from(primary_type)], dep_types].concat();

    let mut result = String::new();
    for type_ in dep_types {
        let fields = domain_types
            .get(&type_)
            .ok_or(EIP712EncodingError::FailedWhenEncodingTypes)?;
        // Concat fields of a type to string like `string value1,string value2` .
        let fields_str = fields
            .iter()
            .map(|(name, type_)| format!("{} {}", type_, name))
            .collect::<Vec<_>>();

        // Finally concat all types and their fields to string like `Transaction(TypeB layer1)TypeA(string value1,string value2)TypeB(TypeA layer2)` .
        result += format!("{}({})", type_, fields_str.join(",")).as_str()
    }

    debug!(
        "{:-width$}encode_type: {} -> {}",
        "",
        primary_type,
        result,
        width = _deep * 2
    );

    // debug!("Type encoding result: {:?}", result);
    Ok(result)
}

/// Recursively find all types declared in root.types field
///
/// The return is stored in the last param in type `Vec<String>`. Finally, it will be something like
/// `["Transaction", "TypeA", "TypeB"]`.
fn find_type_dependencies(
    domain_types: &Types,
    primary_type: &str,
    results: &mut Vec<String>,
) -> Result<(), EIP712EncodingError> {
    let types = domain_types
        .get(parse_type(primary_type))
        .ok_or(EIP712EncodingError::FailedWhenEncodingTypes)?;

    let mut types_vec = Vec::new();
    for (_, sub_type) in types {
        types_vec.push(sub_type);
    }

    for type_ in types_vec {
        let sub_type = parse_type(type_);
        let sub_type_string = String::from(sub_type);

        if !results.contains(&sub_type_string) && domain_types.contains_key(sub_type) {
            results.push(sub_type_string);
            find_type_dependencies(domain_types, sub_type, results)?;
        }
    }

    Ok(())
}

pub fn hash_message(
    domain_types: &Types,
    primary_type: &str,
    message: &BTreeMap<String, Value>,
    _deep: usize,
) -> Result<Vec<u8>, EIP712EncodingError> {
    let bytes = encode_message(domain_types, primary_type, message, _deep)?;
    let hash = keccak256(&bytes);

    debug!(
        "{:-width$}hash_message: {} -> {}",
        "",
        primary_type,
        hex::encode(&hash),
        width = _deep * 2
    );

    Ok(hash)
}

pub fn encode_message(
    domain_types: &Types,
    primary_type: &str,
    message: &BTreeMap<String, Value>,
    _deep: usize,
) -> Result<Vec<u8>, EIP712EncodingError> {
    debug!(
        "{:-width$}encode_message: {} ==========",
        "",
        primary_type,
        width = _deep * 2
    );

    let (type_, data) = hash_type(domain_types, primary_type, _deep + 1)?;
    let mut types = vec![type_];
    let mut values = vec![data];

    let fields = domain_types
        .get(primary_type)
        .ok_or(EIP712EncodingError::FailedWhenEncodingMessage)?;

    for (name, type_) in fields {
        let value = message
            .get(name)
            .ok_or(EIP712EncodingError::FailedWhenEncodingMessage)?;
        let (encoded_type, encoded_data) = value.encode(domain_types, name, type_, _deep + 1)?;
        types.push(encoded_type);
        values.push(encoded_data);
    }

    let values_slices = values.iter().map(|item| item.as_slice()).collect::<Vec<_>>();
    eth_abi_encode(types, values_slices)
}

fn eth_abi_encode(types: Vec<&str>, values: Vec<&[u8]>) -> Result<Vec<u8>, EIP712EncodingError> {
    let mut ret = Vec::new();
    for (i, &type_) in types.iter().enumerate() {
        let value = values[i];
        let mut tmp = eth_abi_encode_single(type_, value)?;
        ret.append(&mut tmp);
    }

    Ok(ret)
}

fn eth_abi_encode_single(type_: &str, value: &[u8]) -> Result<Vec<u8>, EIP712EncodingError> {
    let mut ret;
    match type_ {
        "address" => ret = eth_abi_encode_single("uint160", value)?,
        "bool" => ret = eth_abi_encode_single("uint8", value)?,
        // CAREFUL: Because EIP712 encode most type into bytes and the message structure is predefined, so only a sub-set of all solidity types are supported here.
        "uint8" | "uint160" | "uint256" | "bytes32" => {
            let mut tmp = value.to_vec();
            match tmp.len().cmp(&32) {
                Ordering::Greater => return Err(EIP712EncodingError::InvalidEthABIType),
                Ordering::Less => {
                    let pad_length = 32 - tmp.len();
                    ret = vec![0u8; pad_length];
                    ret.append(&mut tmp);
                }
                Ordering::Equal => ret = tmp,
            }
        }
        _ => {
            return Err(EIP712EncodingError::InvalidEthABIType);
        }
    }

    Ok(ret)
}

#[cfg(test)]
mod test {
    use hex;

    use super::*;

    fn gen_typed_data_v4() -> TypedDataV4 {
        let data = typed_data_v4!({
            types: {
                EIP712Domain: {
                    chainId: "uint256",
                    name: "string",
                    verifyingContract: "address",
                    version: "string"
                },
                Action: {
                    action: "string",
                    params: "string"
                },
                Cell: {
                    capacity: "string",
                    lock: "string",
                    type: "string",
                    data: "string",
                    extraData: "string"
                },
                Transaction: {
                    DAS_MESSAGE: "string",
                    inputsCapacity: "string",
                    outputsCapacity: "string",
                    fee: "string",
                    action: "Action",
                    inputs: "Cell[]",
                    outputs: "Cell[]",
                    digest: "bytes32"
                }
            },
            primaryType: "Transaction",
            domain: {
                chainId: "5",
                name: "da.systems",
                verifyingContract: "0x0000000000000000000000000000000020210722",
                version: "1"
            },
            message: {
                DAS_MESSAGE: "Edit records of account tangzhihong005.bit .",
                inputsCapacity: "225 CKB",
                outputsCapacity: "224.9999 CKB",
                fee: "0.0001 CKB",
                action: {
                    action: "edit_records",
                    params: "0x01"
                },
                inputs: [
                    {
                        capacity: "225 CKB",
                        lock: "das-lock,0x01,0x0515a33588908cf8edb27d1abe3852bf287abd38...",
                        type: "account-cell-type,0x01,0x",
                        data: "{ account: tangzhihong005.bit, expired_at: 1662629612 }",
                        extraData: "{ status: 0, records_hash: 0x55478d76900611eb079b22088081124ed6c8bae21a05dd1a0d197efcc7c114ce }"
                    }
                ],
                outputs: [
                    {
                        capacity: "224.9999 CKB",
                        lock: "das-lock,0x01,0x0515a33588908cf8edb27d1abe3852bf287abd38...",
                        type: "account-cell-type,0x01,0x",
                        data: "{ account: tangzhihong005.bit, expired_at: 1662629612 }",
                        extraData: "{ status: 0, records_hash: 0x75e9c7a4725177c157b31d8a39f73e40ad328be5244a2a2fb6e478a24612c51a }"
                    }
                ],
                digest: "01bee5c80a6bd74440f0f96c983b1107f1a419e028bef7b33e77e8f968cbfae7"
            }
        });

        data
    }

    fn gen_typed_data_v4_with_objects() -> TypedDataV4 {
        let action = typed_data_v4!(@object {
            action: "edit_records",
            params: "0x01"
        });
        let inputs = typed_data_v4!(@array [
            {
                capacity: "225 CKB",
                lock: "das-lock,0x01,0x0515a33588908cf8edb27d1abe3852bf287abd38...",
                type: "account-cell-type,0x01,0x",
                data: "{ account: tangzhihong005.bit, expired_at: 1662629612 }",
                extraData: "{ status: 0, records_hash: 0x55478d76900611eb079b22088081124ed6c8bae21a05dd1a0d197efcc7c114ce }"
            }
        ]);
        let outputs = typed_data_v4!(@array [
            {
                capacity: "224.9999 CKB",
                lock: "das-lock,0x01,0x0515a33588908cf8edb27d1abe3852bf287abd38...",
                type: "account-cell-type,0x01,0x",
                data: "{ account: tangzhihong005.bit, expired_at: 1662629612 }",
                extraData: "{ status: 0, records_hash: 0x75e9c7a4725177c157b31d8a39f73e40ad328be5244a2a2fb6e478a24612c51a }"
            }
        ]);

        let data = typed_data_v4!({
            types: {
                EIP712Domain: {
                    chainId: "uint256",
                    name: "string",
                    verifyingContract: "address",
                    version: "string"
                },
                Action: {
                    action: "string",
                    params: "string"
                },
                Cell: {
                    capacity: "string",
                    lock: "string",
                    type: "string",
                    data: "string",
                    extraData: "string"
                },
                Transaction: {
                    DAS_MESSAGE: "string",
                    inputsCapacity: "string",
                    outputsCapacity: "string",
                    fee: "string",
                    action: "Action",
                    inputs: "Cell[]",
                    outputs: "Cell[]",
                    digest: "bytes32"
                }
            },
            primaryType: "Transaction",
            domain: {
                chainId: "5",
                name: "da.systems",
                verifyingContract: "0x0000000000000000000000000000000020210722",
                version: "1"
            },
            message: {
                DAS_MESSAGE: "Edit records of account tangzhihong005.bit .",
                inputsCapacity: "225 CKB",
                outputsCapacity: "224.9999 CKB",
                fee: "0.0001 CKB",
                action: action,
                inputs: inputs,
                outputs: outputs,
                digest: "01bee5c80a6bd74440f0f96c983b1107f1a419e028bef7b33e77e8f968cbfae7"
            }
        });

        data
    }

    #[test]
    fn test_wip712_hash_data_macro() {
        let typed_data = gen_typed_data_v4();

        let expected = "e2d3286d053a3422c90ca48cb5bfcdb774d114283b5c98034fa407e57e317cd2";
        let data = hash_data(&typed_data).unwrap();

        assert_eq!(hex::encode(data).as_str(), expected);
    }

    #[test]
    fn test_wip712_hash_data_with_objects() {
        let typed_data = gen_typed_data_v4_with_objects();

        let expected = "e2d3286d053a3422c90ca48cb5bfcdb774d114283b5c98034fa407e57e317cd2";
        let data = hash_data(&typed_data).unwrap();

        assert_eq!(hex::encode(data).as_str(), expected);
    }

    #[test]
    fn test_eip712_hash_type() {
        let typed_data = gen_typed_data_v4();

        let (type_, data) = hash_type(&typed_data.types, "Transaction", 0).unwrap();
        assert_eq!(type_, "bytes32");
        assert_eq!(
            hex::encode(data).as_str(),
            "0d54929e2ad87db0194e2f1456aa05b3cb707b8a3c06aa559a7b7fcd8b35f4ed"
        )
    }

    #[test]
    fn test_eip712_encode_type() {
        let typed_data = gen_typed_data_v4();

        let expected = String::from("Transaction(string DAS_MESSAGE,string inputsCapacity,string outputsCapacity,string fee,Action action,Cell[] inputs,Cell[] outputs,bytes32 digest)Action(string action,string params)Cell(string capacity,string lock,string type,string data,string extraData)");
        let types_string = encode_type(&typed_data.types, "Transaction", 0).unwrap();

        assert_eq!(types_string, expected);
    }

    #[test]
    fn test_hash_message() {
        let typed_data = gen_typed_data_v4();

        let expected = "42d5bf292f447fc13bc0a4c67a622a64e298128fa8f7ccf2591245122a2d2e51";
        if let Value::Object((_, message)) = typed_data.message {
            let data = hash_message(&typed_data.types, "Transaction", &message, 0).unwrap();
            assert_eq!(hex::encode(data).as_str(), expected);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_eip712_encode_message() {
        let typed_data = gen_typed_data_v4();

        let expected = "0d54929e2ad87db0194e2f1456aa05b3cb707b8a3c06aa559a7b7fcd8b35f4edd4fc696c3d28d758fedbba5bf47ba0c737bab4d047c4d023d6fd13f9fddf564d5c77a8da76ade7feff107bcc5433d88d0c2ce6f6c745ccba6439afca13b001249aa7fc86e69e74b4faa4f7af930bde719742f1a730f5cae0122d881c2ea5e75f890e6a9632fdbb0d41dc68fa3cf66e218c32f08d5aaf6ff15c717066176c293be844fbde0e91f5eb2b8cd24091c44f0df279c0f3d6fc5de242b7daa860cfbd4195334d7f8e64c9c9d44908471f5cf2fb0b1b575d33d320b7150521b399fbe0779ab40417c527f8531a70bb97783a2b013046a77acca17901605863e99de3ffba01bee5c80a6bd74440f0f96c983b1107f1a419e028bef7b33e77e8f968cbfae7";
        if let Value::Object((_, message)) = typed_data.message {
            let data = encode_message(&typed_data.types, "Transaction", &message, 0).unwrap();
            assert_eq!(hex::encode(data).as_str(), expected);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_eip712_encode_array_field() {
        let typed_data = gen_typed_data_v4();

        if let Value::Object((_, message)) = typed_data.message {
            let (type_, data) = message
                .get("inputs")
                .unwrap()
                .encode(&typed_data.types, "inputs", "Cell[]", 0)
                .unwrap();
            assert_eq!(type_, "bytes32");
            assert_eq!(
                hex::encode(data).as_str(),
                "95334d7f8e64c9c9d44908471f5cf2fb0b1b575d33d320b7150521b399fbe077"
            );
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_eip712_encode_object_field() {
        let typed_data = gen_typed_data_v4();

        if let Value::Object((_, message)) = typed_data.message {
            let (type_, data) = message
                .get("action")
                .unwrap()
                .encode(&typed_data.types, "action", "Action", 0)
                .unwrap();
            assert_eq!(type_, "bytes32");
            assert_eq!(
                hex::encode(data).as_str(),
                "e844fbde0e91f5eb2b8cd24091c44f0df279c0f3d6fc5de242b7daa860cfbd41"
            );
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_eip712_encode_address_field() {
        let typed_data = gen_typed_data_v4();

        if let Value::Object((_, domain)) = typed_data.domain {
            let (type_, data) = domain
                .get("verifyingContract")
                .unwrap()
                .encode(&typed_data.types, "verifyingContract", "address", 0)
                .unwrap();
            assert_eq!(type_, "address");
            assert_eq!(hex::encode(data).as_str(), "0000000000000000000000000000000020210722");
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_eip712_encode_bytes32_field() {
        let typed_data = gen_typed_data_v4();

        if let Value::Object((_, message)) = typed_data.message {
            let (type_, data) = message
                .get("digest")
                .unwrap()
                .encode(&typed_data.types, "digest", "bytes32", 0)
                .unwrap();
            assert_eq!(type_, "bytes32");
            assert_eq!(
                hex::encode(data).as_str(),
                "01bee5c80a6bd74440f0f96c983b1107f1a419e028bef7b33e77e8f968cbfae7"
            );
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_eip712_encode_string_field() {
        let typed_data = gen_typed_data_v4();

        if let Value::Object((_, message)) = typed_data.message {
            let (type_, data) = message
                .get("DAS_MESSAGE")
                .unwrap()
                .encode(&typed_data.types, "DAS_MESSAGE", "string", 0)
                .unwrap();
            assert_eq!(type_, "bytes32");
            assert_eq!(
                hex::encode(data).as_str(),
                "d4fc696c3d28d758fedbba5bf47ba0c737bab4d047c4d023d6fd13f9fddf564d"
            );
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_eip712_encode_uint_field() {
        let typed_data = gen_typed_data_v4();

        if let Value::Object((_, domain)) = typed_data.domain {
            let (type_, data) = domain
                .get("chainId")
                .unwrap()
                .encode(&typed_data.types, "chainId", "uint256", 0)
                .unwrap();
            assert_eq!(type_, "uint256");
            assert_eq!(hex::encode(data).as_str(), "0000000000000005");
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_eip712_eth_abi_encode() {
        let types = vec!["bytes32", "bytes32", "bytes32", "uint256", "address"];
        let values = vec![
            hex::decode("8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f").unwrap(),
            hex::decode("b6f30b130932fb5584f1644a542248dd6a18f3f873983c198e0dec0a324e840e").unwrap(),
            hex::decode("c89efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc6").unwrap(),
            hex::decode("0000000000000001").unwrap(),
            hex::decode("b3dc32341ee4bae03c85cd663311de0b1b122955").unwrap(),
        ];
        let values_ref = values.iter().map(|item| item.as_ref()).collect::<Vec<_>>();

        let expected = "8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400fb6f30b130932fb5584f1644a542248dd6a18f3f873983c198e0dec0a324e840ec89efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc60000000000000000000000000000000000000000000000000000000000000001000000000000000000000000b3dc32341ee4bae03c85cd663311de0b1b122955";
        let ret = hex::encode(eth_abi_encode(types, values_ref).unwrap());
        assert_eq!(ret, expected);
    }
}
