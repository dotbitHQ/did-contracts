#![no_std]

extern crate alloc;
extern crate no_std_compat as std;

#[macro_use]
pub mod macros;

mod eip712;
pub mod error;
pub mod types;
pub mod util;

pub use crate::eip712::hash_data;
pub use crate::eip712::hash_json;
