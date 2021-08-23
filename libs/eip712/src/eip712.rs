use alloc::{format, vec};
use serde_json::{Map, Value};
use std::prelude::v1::*;

use super::error::*;
use super::types::*;
use super::util::*;

pub fn hash_json(json: &str) -> Result<Vec<u8>, EIP712EncodingError> {
    let data: TypedDataV4 = serde_json::from_str(json).unwrap();
    hash_data(data)
}

pub fn hash_data(typed_data: TypedDataV4) -> Result<Vec<u8>, EIP712EncodingError> {
    // The first part of EIP712 hash which is a constant `0x1901`.
    let part1 = vec![25u8, 1];
    let part2 = hash_message(&typed_data.types, "EIP712Domain", &typed_data.domain)?;

    let mut part3 = Vec::new();
    if typed_data.primary_type != "EIP712Domain" {
        part3 = hash_message(
            &typed_data.types,
            &typed_data.primary_type,
            &typed_data.message,
        )?;
    }

    // println!("part1: {:?}", hex::encode(part1.clone()));
    // println!("part2: {:?}", hex::encode(part2.clone()));
    // println!("part3: {:?}", hex::encode(part3.clone()));

    let bytes = vec![part1, part2, part3].concat();
    Ok(keccak256(&bytes))
}

pub fn hash_type(
    domain_types: &Map<String, Value>,
    primary_type: &str,
) -> Result<(&'static str, Vec<u8>), EIP712EncodingError> {
    let types_string = encode_type(domain_types, primary_type)?;
    Ok(("bytes32", keccak256(types_string.as_bytes())))
}

pub fn encode_type(
    domain_types: &Map<String, Value>,
    primary_type: &str,
) -> Result<String, EIP712EncodingError> {
    let mut dep_types = Vec::new();
    find_type_dependencies(domain_types, primary_type, &mut dep_types)?;
    // Sort by ascii in ascending order
    dep_types.sort();

    // Push primary_type as the first element of the vector.
    dep_types = vec![vec![String::from(primary_type)], dep_types].concat();

    let mut result = String::new();
    for type_ in dep_types {
        let fields = domain_types[&type_]
            .as_array()
            .ok_or(EIP712EncodingError::FailedWhenEncodingTypes)?;
        // Concat fields of a type to string like `string value1,string value2` .
        let fields_str = fields
            .iter()
            .map(|field| -> Result<String, EIP712EncodingError> {
                let name = field["name"]
                    .as_str()
                    .ok_or(EIP712EncodingError::FailedWhenEncodingTypes)?;
                let type_ = field["type"]
                    .as_str()
                    .ok_or(EIP712EncodingError::FailedWhenEncodingTypes)?;
                return Ok(format!("{} {}", type_, name));
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Finally concat all types and their fields to string like `Transaction(TypeB layer1)TypeA(string value1,string value2)TypeB(TypeA layer2)` .
        result += format!("{}({})", type_, fields_str.join(",")).as_str()
    }

    // println!("Type encoding result: {:?}", result);
    Ok(result)
}

/// Recursively find all types declared in root.types field
///
/// The return is stored in the last param in type `Vec<String>`. Finally, it will be something like
/// `["Transaction", "TypeA", "TypeB"]`.
fn find_type_dependencies(
    domain_types: &Map<String, Value>,
    primary_type: &str,
    results: &mut Vec<String>,
) -> Result<(), EIP712EncodingError> {
    let types = domain_types
        .get(parse_type(primary_type))
        .ok_or(EIP712EncodingError::FailedWhenEncodingTypes)?;
    let types_vec = types
        .as_array()
        .ok_or(EIP712EncodingError::FailedWhenEncodingTypes)?;

    for field in types_vec {
        let sub_type = parse_type(
            field["type"]
                .as_str()
                .ok_or(EIP712EncodingError::FailedWhenEncodingTypes)?,
        );
        let sub_type_string = String::from(sub_type);

        if !results.contains(&sub_type_string) && domain_types.contains_key(sub_type) {
            results.push(sub_type_string);
            find_type_dependencies(domain_types, sub_type, results)?;
        }
    }

    Ok(())
}

pub fn hash_message(
    domain_types: &Map<String, Value>,
    primary_type: &str,
    message: &Map<String, Value>,
) -> Result<Vec<u8>, EIP712EncodingError> {
    let bytes = encode_message(domain_types, primary_type, message)?;
    Ok(keccak256(&bytes))
}

pub fn encode_message(
    domain_types: &Map<String, Value>,
    primary_type: &str,
    message: &Map<String, Value>,
) -> Result<Vec<u8>, EIP712EncodingError> {
    let (type_, data) = hash_type(domain_types, primary_type)?;
    let mut types = vec![type_];
    let mut values = vec![data];

    let fields = domain_types
        .get(primary_type)
        .ok_or(EIP712EncodingError::FailedWhenEncodingMessage)?
        .as_array()
        .ok_or(EIP712EncodingError::FailedWhenEncodingMessage)?;

    for field in fields {
        let name = field["name"]
            .as_str()
            .ok_or(EIP712EncodingError::FailedWhenEncodingMessage)?;
        let type_ = field["type"]
            .as_str()
            .ok_or(EIP712EncodingError::FailedWhenEncodingMessage)?;
        let value = message
            .get(name)
            .ok_or(EIP712EncodingError::FailedWhenEncodingMessage)?;
        let (type_, data) = encode_field(domain_types, name, type_, value)?;
        types.push(type_);
        values.push(data);
    }

    // println!("primary_type = {:?}", primary_type);
    // println!("types = {:#?}", types);
    // println!(
    //     "values = {:#?}",
    //     values_slices = values
    //         .iter()
    //         .map(|item| hex::encode(item))
    //         .collect::<Vec<_>>()
    // );

    let values_slices = values
        .iter()
        .map(|item| item.as_slice())
        .collect::<Vec<_>>();
    Ok(eth_abi_encode(types, values_slices)?)
}

fn encode_field(
    domain_types: &Map<String, Value>,
    name: &str,
    type_: &str,
    value: &Value,
) -> Result<(&'static str, Vec<u8>), EIP712EncodingError> {
    if domain_types.contains_key(type_) {
        let bytes = encode_message(
            domain_types,
            type_,
            value
                .as_object()
                .ok_or(EIP712EncodingError::TypeOfValueIsInvalid)?,
        )?;
        return Ok(("bytes32", keccak256(&bytes)));
    } else if type_ == "bytes" {
        let bytes: Vec<u8> = hex::decode(
            value
                .as_str()
                .ok_or(EIP712EncodingError::TypeOfValueIsInvalid)?
                .trim_start_matches("0x"),
        )
        .map_err(|_| EIP712EncodingError::HexDecodingError)?;
        return Ok(("bytes32", keccak256(&bytes)));
    } else if type_ == "string" {
        let text = value
            .as_str()
            .ok_or(EIP712EncodingError::TypeOfValueIsInvalid)?;
        return Ok(("bytes32", keccak256(text.as_bytes())));
    } else if type_.ends_with("[]") {
        let mut sub_types = Vec::new();
        let mut sub_values = Vec::new();
        for item in value
            .as_array()
            .ok_or(EIP712EncodingError::TypeOfValueIsInvalid)?
            .iter()
        {
            let (sub_type, sub_bytes) = encode_field(domain_types, name, parse_type(type_), item)?;
            sub_types.push(sub_type);
            sub_values.push(sub_bytes);
        }
        let bytes = eth_abi_encode(sub_types, sub_values.iter().map(AsRef::as_ref).collect())?;
        return Ok(("bytes32", keccak256(&bytes)));
    } else {
        if type_ == "bytes32" {
            let bytes: Vec<u8> = hex::decode(
                value
                    .as_str()
                    .ok_or(EIP712EncodingError::TypeOfValueIsInvalid)?
                    .trim_start_matches("0x"),
            )
            .map_err(|_| EIP712EncodingError::HexDecodingError)?;
            return Ok(("bytes32", bytes));
        } else if type_ == "address" {
            let bytes: Vec<u8> = hex::decode(
                value
                    .as_str()
                    .ok_or(EIP712EncodingError::TypeOfValueIsInvalid)?
                    .trim_start_matches("0x"),
            )
            .map_err(|_| EIP712EncodingError::HexDecodingError)?;
            return Ok(("address", bytes));
        } else if type_ == "uint256" {
            // WARNING! Here we can only support uint64 because of serde type limitation.
            let num = value
                .as_u64()
                .ok_or(EIP712EncodingError::TypeOfValueIsInvalid)?;
            return Ok(("uint256", num.to_be_bytes().to_vec()));
        } else {
            println!("name: {}, type: {}, value: {:?}", name, type_, value);
            return Err(EIP712EncodingError::UndefinedEIP712Type);
        }
    }
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
        // ⚠️ Because EIP712 encode most type into bytes and the message structure is predefined, so only a sub-set of all solidity types are supported here.
        "uint8" | "uint160" | "uint256" | "bytes32" => {
            let mut tmp = value.to_vec();
            if tmp.len() < 32 {
                let pad_length = 32 - tmp.len();
                ret = vec![0u8; pad_length];
                ret.append(&mut tmp);
            } else if tmp.len() == 32 {
                ret = tmp
            } else {
                return Err(EIP712EncodingError::InvalidEthABIType);
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
    use super::*;
    use hex;
    use serde_json::{Map, Value};

    fn gen_typed_data_v4() -> TypedDataV4 {
        let action: Value = Action::new("transfer_account", "0x01,0x00").into();
        let inputs = Value::from(vec![
            Cell::new(
                "999.99 CKB",
                "0x123456,0x01,0x123456",
                "0x123456,0x01,0x123456",
                "account: test.bit",
                "",
            ),
            Cell::new(
                "999.99 CKB",
                "0x123456,0x01,0x123456",
                "0x123456,0x01,0x123456",
                "",
                "",
            ),
        ]);
        let outputs = Value::from(vec![
            Cell::new(
                "999.99 CKB",
                "0x123456,0x01,0x123456",
                "0x123456,0x01,0x123456",
                "account: test.bit",
                "",
            ),
            Cell::new(
                "999.99 CKB",
                "0x123456,0x01,0x123456",
                "0x123456,0x01,0x123456",
                "",
                "",
            ),
        ]);

        let data = typed_data_v4!({
            types: {
                EIP712Domain: [
                    name: "string",
                    version: "string",
                    chainId: "uint256",
                    verifyingContract: "address"
                ],
                Action: [
                    action: "string",
                    params: "string"
                ],
                Cell: [
                  capacity: "string",
                  lock: "string",
                  type: "string",
                  data: "string",
                  extraData: "string"
                ],
                Transaction: [
                  action: "Action",
                  inputs: "Cell[]",
                  outputs: "Cell[]",
                  digest: "bytes32",
                  plainText: "string"
                ]
            },
            primaryType: "Transaction",
            domain: {
                chainId: 1,
                name: "da.systems",
                verifyingContract: "0xb3dc32341ee4bae03c85cd663311de0b1b122955",
                version: "1"
            },
            message: {
                action: action,
                inputs: inputs,
                outputs: outputs,
                digest: "0x4eb68a6707ae16ce24fde8e5964f9f04c5a4abf9884f67b9425a5e1e65968119",
                plainText: "Transfer account test.bit from A to B."
            }
        });

        data
    }

    // #[test]
    // fn test_hash_json() {
    //     let json = r#"{
    //       "types": {
    //         "EIP712Domain": [
    //           {
    //             "name": "name",
    //             "type": "string"
    //           },
    //           {
    //             "name": "version",
    //             "type": "string"
    //           },
    //           {
    //             "name": "chainId",
    //             "type": "uint256"
    //           },
    //           {
    //             "name": "verifyingContract",
    //             "type": "address"
    //           }
    //         ],
    //         "Action": [
    //           {
    //             "name": "action",
    //             "type": "string"
    //           },
    //           {
    //             "name": "params",
    //             "type": "string"
    //           }
    //         ],
    //         "Cell": [
    //           {
    //             "name": "capacity",
    //             "type": "string"
    //           },
    //           {
    //             "name": "lock",
    //             "type": "string"
    //           },
    //           {
    //             "name": "type",
    //             "type": "string"
    //           },
    //           {
    //             "name": "data",
    //             "type": "string"
    //           },
    //           {
    //             "name": "extraData",
    //             "type": "string"
    //           }
    //         ],
    //         "Transaction": [
    //           {
    //             "name": "action",
    //             "type": "Action"
    //           },
    //           {
    //             "name": "inputs",
    //             "type": "Cell[]"
    //           },
    //           {
    //             "name": "outputs",
    //             "type": "Cell[]"
    //           },
    //           {
    //             "name": "digest",
    //             "type": "string"
    //           },
    //           {
    //             "name": "plainText",
    //             "type": "string"
    //           }
    //         ]
    //       },
    //       "domain": {
    //         "chainId": 1,
    //         "name": "da.systems",
    //         "verifyingContract": "0xb3dc32341ee4bae03c85cd663311de0b1b122955",
    //         "version": "1"
    //       },
    //       "primaryType": "Transaction",
    //       "message": {
    //         "action": {
    //           "action": "transfer_account",
    //           "params": "0x01,0x00"
    //         },
    //         "inputs": [
    //           {
    //             "capacity": "999.99 CKB",
    //             "lock": "0x123456,0x01,0x123456",
    //             "type": "0x123456,0x01,0x123456",
    //             "data": "account: test.bit",
    //             "extraData": ""
    //           },
    //           {
    //             "capacity": "999.99 CKB",
    //             "lock": "0x123456,0x01,0x123456",
    //             "type": "0x123456,0x01,0x123456",
    //             "data": "",
    //             "extraData": ""
    //           }
    //         ],
    //         "outputs": [
    //           {
    //             "capacity": "999.99 CKB",
    //             "lock": "0x123456,0x01,0x123456",
    //             "type": "0x123456,0x01,0x123456",
    //             "data": "account: test.bit",
    //             "extraData": ""
    //           },
    //           {
    //             "capacity": "999.99 CKB",
    //             "lock": "0x123456,0x01,0x123456",
    //             "type": "0x123456,0x01,0x123456",
    //             "data": "",
    //             "extraData": ""
    //           }
    //         ],
    //         "digest": "0x4eb68a6707ae16ce24fde8e5964f9f04c5a4abf9884f67b9425a5e1e65968119",
    //         "plainText": "Transfer account test.bit from A to B."
    //       }
    //     }"#;
    //
    //     let expected = "2c96a222ea2a1e0e5815cd3a01d023a7518d68dc0e8311549df9f3fcf7d06065";
    //     let data = hash_json(json).unwrap();
    //
    //     assert_eq!(hex::encode(data).as_str(), expected);
    // }

    #[test]
    fn test_hash_data() {
        let typed_data = gen_typed_data_v4();

        let expected = "2c96a222ea2a1e0e5815cd3a01d023a7518d68dc0e8311549df9f3fcf7d06065";
        let data = hash_data(typed_data).unwrap();

        assert_eq!(hex::encode(data).as_str(), expected);
    }

    #[test]
    fn test_hash_type() {
        let typed_data = typed_data_v4!({
            types: {
                EIP712Domain: [
                    name: "string",
                    version: "string",
                    chainId: "uint256",
                    verifyingContract: "address"
                ],
                TypeA: [
                    value1: "string",
                    value2: "string"
                ],
                TypeB: [
                    layer2: "TypeA"
                ],
                Transaction: [
                    layer1: "TypeB"
                ]
            },
            primaryType: "Transaction",
            domain: {
                chainId: 1,
                name: "da.systems",
                verifyingContract: "0x23423534534645-1",
                version: "1"
            },
            message: {
                layer1: {
                    layer2: {
                        value: "test-nested-types"
                    }
                }
            }
        });

        let (type_, data) = hash_type(&typed_data.types, "Transaction").unwrap();
        assert_eq!(type_, "bytes32");
        assert_eq!(
            hex::encode(data).as_str(),
            "68fbcacb49eb1736e2e83075812d44628ec11fbf7543289f04d45eed0069ac10"
        )
    }

    #[test]
    fn test_encode_type() {
        let typed_data = typed_data_v4!({
            types: {
                EIP712Domain: [
                    name: "string",
                    version: "string",
                    chainId: "uint256",
                    verifyingContract: "address"
                ],
                TypeA: [
                    value1: "string",
                    value2: "string"
                ],
                TypeB: [
                    layer2: "TypeA"
                ],
                Transaction: [
                    layer1: "TypeB"
                ]
            },
            primaryType: "Transaction",
            domain: {
                chainId: 1,
                name: "da.systems",
                verifyingContract: "0xb3dc32341ee4bae03c85cd663311de0b1b122955",
                version: "1"
            },
            message: {
                layer1: {
                    layer2: {
                        value: "test-nested-types"
                    }
                }
            }
        });

        let expected = String::from(
            "Transaction(TypeB layer1)TypeA(string value1,string value2)TypeB(TypeA layer2)",
        );
        let types_string = encode_type(&typed_data.types, "Transaction").unwrap();
        assert_eq!(types_string, expected);
    }

    #[test]
    fn test_hash_message() {
        let typed_data = gen_typed_data_v4();

        let expected = "c6e1a436fd4e5e01092aba72e6856c2d045ab51f014e4bb5d3410849e367d707";
        let data = hash_message(&typed_data.types, "Transaction", &typed_data.message).unwrap();
        assert_eq!(hex::encode(data).as_str(), expected);
    }

    #[test]
    fn test_encode_message() {
        let typed_data = gen_typed_data_v4();

        let expected = "4ddab48cd954896ac22eb32b76c7104dcf84375c7ed70a467edda0c5511fe031c293aa03c4c3300b0ee452a0988dd18b08ab152328657ffb8954b6eae6564a1ab4e788529779901b8d9bd038e2a6bfd26938fa88c67557009afae43dcd20ee06b4e788529779901b8d9bd038e2a6bfd26938fa88c67557009afae43dcd20ee064eb68a6707ae16ce24fde8e5964f9f04c5a4abf9884f67b9425a5e1e6596811908e0229b71be5e528d9b3e217152cc55330e613c8d33a2d67c583950e78172ae";
        let data = encode_message(&typed_data.types, "Transaction", &typed_data.message).unwrap();
        assert_eq!(hex::encode(data).as_str(), expected);
    }

    #[test]
    fn test_encode_field() {
        let typed_data = gen_typed_data_v4();

        // Encoding bytes32 type
        let digest = typed_data.message.get("digest").unwrap();
        let (type_, data) = encode_field(&typed_data.types, "digest", "bytes32", digest).unwrap();
        assert_eq!(type_, "bytes32");
        assert_eq!(
            hex::encode(data).as_str(),
            "4eb68a6707ae16ce24fde8e5964f9f04c5a4abf9884f67b9425a5e1e65968119"
        );

        // Encoding string type
        let plain_text = typed_data.message.get("plainText").unwrap();
        let (type_, data) =
            encode_field(&typed_data.types, "plainText", "string", plain_text).unwrap();
        assert_eq!(type_, "bytes32");
        assert_eq!(
            hex::encode(data).as_str(),
            "08e0229b71be5e528d9b3e217152cc55330e613c8d33a2d67c583950e78172ae"
        );

        // Encoding uint256 type
        let chain_id = typed_data.domain.get("chainId").unwrap();
        let (type_, data) =
            encode_field(&typed_data.types, "chainId", "uint256", chain_id).unwrap();
        assert_eq!(type_, "uint256");
        assert_eq!(hex::encode(data).as_str(), "0000000000000001");

        // Encoding sub type
        let action = Action::new("transfer_account", "0x01,0x00").into();
        let (type_, data) = encode_field(&typed_data.types, "action", "Action", &action).unwrap();
        assert_eq!(type_, "bytes32");
        assert_eq!(
            hex::encode(data).as_str(),
            "c293aa03c4c3300b0ee452a0988dd18b08ab152328657ffb8954b6eae6564a1a"
        );
    }

    #[test]
    fn test_eth_abi_encode() {
        let types = vec!["bytes32", "bytes32", "bytes32", "uint256", "address"];
        let values = vec![
            hex::decode("8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f")
                .unwrap(),
            hex::decode("b6f30b130932fb5584f1644a542248dd6a18f3f873983c198e0dec0a324e840e")
                .unwrap(),
            hex::decode("c89efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc6")
                .unwrap(),
            hex::decode("0000000000000001").unwrap(),
            hex::decode("b3dc32341ee4bae03c85cd663311de0b1b122955").unwrap(),
        ];
        let values_ref = values.iter().map(|item| item.as_ref()).collect::<Vec<_>>();

        let expected = "8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400fb6f30b130932fb5584f1644a542248dd6a18f3f873983c198e0dec0a324e840ec89efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc60000000000000000000000000000000000000000000000000000000000000001000000000000000000000000b3dc32341ee4bae03c85cd663311de0b1b122955";
        let ret = hex::encode(eth_abi_encode(types, values_ref).unwrap());
        assert_eq!(ret, expected);
    }
}
