use alloc::boxed::Box;
use core::result::Result;
use das_core::{debug, error::*};

pub fn main(_argc: usize, _argv: *const *const u8) -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running playground ======");

    Ok(())
}
