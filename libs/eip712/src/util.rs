use std::prelude::v1::*;
use tiny_keccak::{Hasher, Keccak};

pub fn keccak256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Keccak::v256();
    hasher.update(data);

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);

    output.to_vec()
}

pub fn parse_type(type_: &str) -> &str {
    if type_.ends_with("[]") {
        &type_[0..(type_.len() - 2)]
    } else {
        type_
    }
}
