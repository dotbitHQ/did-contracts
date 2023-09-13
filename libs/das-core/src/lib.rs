// #![feature(once_cell)]
#![feature(slice_pattern)]
#![feature(once_cell_try)]
#![no_std]

extern crate alloc;

#[macro_use]
pub mod macros;

pub mod constants;
pub mod data_parser;
pub mod error;
pub mod helpers;
pub mod inspect;
pub mod sign_util;
pub mod since_util;
pub mod traits;
pub mod types;
pub mod util;
pub mod verifiers;
pub mod witness_parser;
pub mod general_witness_parser;