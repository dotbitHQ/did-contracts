use core::fmt;
use serde_json::{Map, Value};
use std::prelude::v1::*;

#[derive(Debug)]
pub struct TypedDataV4 {
    pub types: Map<String, Value>,
    pub primary_type: String,
    pub domain: Map<String, Value>,
    pub message: Map<String, Value>,
}

impl TypedDataV4 {
    pub fn new(
        types: Map<String, Value>,
        primary_type: &str,
        domain: Map<String, Value>,
        message: Map<String, Value>,
    ) -> Self {
        TypedDataV4 {
            types,
            primary_type: primary_type.to_string(),
            domain,
            message,
        }
    }

    pub fn digest(&mut self, digest: &str) {
        self.message.insert(String::from("digest"), Value::from(digest));
    }
}

#[cfg(debug_assertions)]
impl fmt::Display for TypedDataV4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            r#"{{ "types": {}, "primaryType": "{}", "domain": {}, "message": {} }}"#,
            serde_json::to_string(&self.types).unwrap(),
            self.primary_type,
            serde_json::to_string(&self.domain).unwrap(),
            serde_json::to_string(&self.message).unwrap()
        )
    }
}

pub type DomainType = Vec<DomainTypeField>;

#[derive(Debug)]
pub struct DomainTypeField {
    pub name: String,
    pub type_: String,
}

impl Into<Value> for DomainTypeField {
    fn into(self) -> Value {
        let mut map = Map::new();
        map.insert("name".to_string(), Value::from(self.name));
        map.insert("type".to_string(), Value::from(self.type_));

        Value::Object(map)
    }
}

pub struct Domain {
    // TODO The chain_id in EIP712 requires up to u256 size, we may need to support it when required.
    // https://eips.ethereum.org/EIPS/eip-712#definition-of-encodetype
    pub chain_id: u128,
    pub name: String,
    pub verifying_contract: String,
    pub version: String,
}

impl Domain {
    pub fn new(chain_id: u128, name: &str, verifying_contract: &str, version: &str) -> Self {
        return Domain {
            chain_id,
            name: name.to_string(),
            verifying_contract: verifying_contract.to_string(),
            version: version.to_string(),
        };
    }
}

pub struct Message {
    pub inputs: Vec<Cell>,
    pub outputs: Vec<Cell>,
    pub digest: String,
    pub plain_text: String,
}

impl Message {
    pub fn new(inputs: Vec<Cell>, outputs: Vec<Cell>, digest: &str, plain_text: &str) -> Self {
        return Message {
            inputs,
            outputs,
            digest: digest.to_string(),
            plain_text: plain_text.to_string(),
        };
    }
}

pub struct Action {
    pub action: String,
    pub params: String,
}

impl Action {
    pub fn new(action: &str, params: &str) -> Self {
        return Action {
            action: action.to_string(),
            params: params.to_string(),
        };
    }
}

impl Into<Value> for Action {
    fn into(self) -> Value {
        let mut map = Map::new();
        map.insert("action".to_string(), Value::from(self.action));
        map.insert("params".to_string(), Value::from(self.params));

        Value::Object(map)
    }
}

pub struct Cell {
    pub capacity: String,
    pub lock: String,
    pub type_: String,
    pub data: String,
    pub extra_data: String,
}

impl Cell {
    pub fn new(capacity: &str, lock: &str, type_: &str, data: &str, extra_data: &str) -> Self {
        return Cell {
            capacity: capacity.to_string(),
            lock: lock.to_string(),
            type_: type_.to_string(),
            data: data.to_string(),
            extra_data: extra_data.to_string(),
        };
    }
}

impl Into<Value> for Cell {
    fn into(self) -> Value {
        let mut map = Map::new();
        map.insert("capacity".to_string(), Value::from(self.capacity));
        map.insert("lock".to_string(), Value::from(self.lock));
        map.insert("type".to_string(), Value::from(self.type_));
        map.insert("data".to_string(), Value::from(self.data));
        map.insert("extraData".to_string(), Value::from(self.extra_data));

        Value::Object(map)
    }
}
