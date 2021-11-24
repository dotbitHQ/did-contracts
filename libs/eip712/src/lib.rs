#![no_std]

extern crate alloc;
extern crate no_std_compat as std;

#[macro_use]
pub mod macros;

pub mod eip712;
pub mod error;
pub mod util;

pub use crate::eip712::hash_data;
