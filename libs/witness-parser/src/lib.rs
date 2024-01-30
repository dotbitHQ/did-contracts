#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[macro_use]
mod macros;

pub mod constants;
pub mod error;
pub mod parsers;
pub mod traits;
pub mod types;
pub mod util;

pub use parsers::v1::witness_parser::WitnessesParser as WitnessesParserV1;
