use ckb_std::ckb_types::bytes;
use core::cmp::Ordering;
use std::prelude::v1::*;

pub fn cmp_by_byte(a: &bytes::Bytes, b: &bytes::Bytes) -> Ordering {
    for (i, a_byte) in a[..].iter().enumerate() {
        let b_byte = &b[i];
        if a_byte < b_byte {
            return Ordering::Less;
        } else if a_byte > b_byte {
            return Ordering::Greater;
        };
    }

    return Ordering::Equal;
}

pub fn cmp(a: &bytes::Bytes, b: &bytes::Bytes) -> Ordering {
    if a.len() < b.len() {
        Ordering::Less
    } else if a.len() > b.len() {
        Ordering::Greater
    } else {
        cmp_by_byte(a, b)
    }
}

#[cfg(test)]
pub fn hex_to_bytes(input: &str) -> bytes::Bytes {
    let hex = input.trim_start_matches("0x");
    if hex == "" {
        bytes::Bytes::default()
    } else {
        bytes::Bytes::from(hex::decode(hex).unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cmp_greater() {
        let a = hex_to_bytes("0x1000");
        let b = hex_to_bytes("0x10");
        assert!(cmp(&a, &b) == Ordering::Greater);

        let a = hex_to_bytes("0x0000");
        let b = hex_to_bytes("0x00");
        assert!(cmp(&a, &b) == Ordering::Greater);
    }

    #[test]
    fn test_cmp_less() {
        let a = hex_to_bytes("0x10");
        let b = hex_to_bytes("0x1000");
        assert!(cmp(&a, &b) == Ordering::Less);

        let a = hex_to_bytes("0x00");
        let b = hex_to_bytes("0x0000");
        assert!(cmp(&a, &b) == Ordering::Less);
    }

    #[test]
    fn test_cmp_equal() {
        let a = hex_to_bytes("0x1000");
        let b = hex_to_bytes("0x1000");
        assert!(cmp(&a, &b) == Ordering::Equal);
    }

    #[test]
    fn test_cmp_by_byte_greater() {
        let a = hex_to_bytes("0x0200");
        let b = hex_to_bytes("0x0100");
        assert!(cmp_by_byte(&a, &b) == Ordering::Greater);

        let a = hex_to_bytes("0x0200");
        let b = hex_to_bytes("0x01FF");
        assert!(cmp_by_byte(&a, &b) == Ordering::Greater);

        let a = hex_to_bytes("0xFF02");
        let b = hex_to_bytes("0xFF01");
        assert!(cmp_by_byte(&a, &b) == Ordering::Greater);
    }

    #[test]
    fn test_cmp_by_byte_less() {
        let a = hex_to_bytes("0x0100");
        let b = hex_to_bytes("0x0200");
        assert!(cmp_by_byte(&a, &b) == Ordering::Less);

        let a = hex_to_bytes("0x01FF");
        let b = hex_to_bytes("0x0200");
        assert!(cmp_by_byte(&a, &b) == Ordering::Less);

        let a = hex_to_bytes("0xFF01");
        let b = hex_to_bytes("0xFF02");
        assert!(cmp_by_byte(&a, &b) == Ordering::Less);
    }

    #[test]
    fn test_cmp_by_byte_equal() {
        let a = hex_to_bytes("0xFFFF");
        let b = hex_to_bytes("0xFFFF");
        assert!(cmp_by_byte(&a, &b) == Ordering::Equal);

        let a = hex_to_bytes("0x0000");
        let b = hex_to_bytes("0x0000");
        assert!(cmp_by_byte(&a, &b) == Ordering::Equal);
    }
}
