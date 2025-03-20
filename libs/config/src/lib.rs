#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[macro_use]
mod macros;

pub mod configs;
pub mod constants;
pub mod error;
pub mod traits;
pub mod util;

pub mod config;
pub use config::Config;
