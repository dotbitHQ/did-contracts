use core::result::Result;
use das_core::{assert, debug, error::Error, warn};

use ckb_std::dynamic_loading::{CKBDLContext, Symbol};

pub fn main() -> Result<(), Error> {
    debug!("====== Running playground ======");

    let code_hash = [
        175, 146, 8, 214, 187, 74, 62, 167, 31, 178, 214, 207, 187, 168, 151, 8, 161, 157, 84, 85, 252, 192, 199, 248,
        109, 11, 195, 253, 232, 200, 115, 114,
    ];
    // Create a DL context with 64K buffer.
    let mut context = unsafe { CKBDLContext::<[u8; 32 * 1024]>::new() };
    // Load library
    let lib = context.load(&code_hash).expect("load shared lib");

    // get symbols

    Ok(())
}
