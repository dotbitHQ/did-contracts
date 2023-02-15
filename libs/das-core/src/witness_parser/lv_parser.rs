use alloc::boxed::Box;

use das_types::constants::WITNESS_LENGTH_BYTES;

use crate::error::{ErrorCode, ScriptError};

pub fn parse_field<'a>(
    field_name: &str,
    bytes: &'a [u8],
    start: usize,
) -> Result<(usize, &'a [u8]), Box<dyn ScriptError>> {
    // Every field is start with 4 bytes of uint32 as its length.
    let length = match bytes.get(start..(start + WITNESS_LENGTH_BYTES)) {
        Some(bytes) => {
            assert!(
                bytes.len() == 4,
                ErrorCode::WitnessStructureError,
                "  [{}] Sub-account witness structure error, expect {}..{} to be bytes of LE uint32.",
                field_name,
                start,
                start + WITNESS_LENGTH_BYTES
            );

            u32::from_le_bytes(bytes.try_into().unwrap()) as usize
        }
        None => {
            warn!(
                "  [{}] Sub-account witness structure error, expect 4 bytes in {}..{} .",
                field_name,
                start,
                start + WITNESS_LENGTH_BYTES
            );
            return Err(code_to_error!(ErrorCode::WitnessStructureError));
        }
    };

    // Slice the field base on the start and length.
    let from = start + WITNESS_LENGTH_BYTES;
    let to = from + length;
    let field_bytes = match bytes.get(from..to) {
        Some(bytes) => bytes,
        None => {
            warn!(
                "  [{}] Sub-account witness structure error, expect {} bytes in {}..{} .",
                field_name, length, from, to
            );
            return Err(code_to_error!(ErrorCode::WitnessStructureError));
        }
    };

    let new_start = start + WITNESS_LENGTH_BYTES + length;
    Ok((new_start, field_bytes))
}
