use alloc::boxed::Box;
use core::result::Result;

use ckb_std::high_level;
use das_core::debug;
use das_core::error::*;

pub fn main(_argc: usize, _argv: *const *const u8) -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running playground ======");

    let header = high_level::load_header(index, source);

    Ok(())
}
