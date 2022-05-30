use core::result::Result;
use das_core::{debug, error::Error};

pub fn main(argc: usize, argv: *const *const u8) -> Result<(), Error> {
    debug!("====== Running playground ======");

    Ok(())
}
