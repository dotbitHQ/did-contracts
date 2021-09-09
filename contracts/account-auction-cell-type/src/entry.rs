use alloc::{boxed::Box, vec, vec::Vec};
use ckb_std::high_level::{load_cell_capacity, load_cell_type};
use ckb_std::{ckb_constants::Source, ckb_types::prelude::*, debug, high_level};
use core::mem::{size_of, MaybeUninit};
use das_core::{
    assert, constants::*, data_parser, error::Error, parse_account_cell_witness, parse_witness, util, warn,
    witness_parser::WitnessesParser,
};
use das_map::{map::Map, util as map_util};
use das_types::{
    constants::{AccountStatus, DataType, LockRole},
    mixer::*,
    packed::*,
    prelude::*,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running account-auction-cell-type ======");
    let mut parser = WitnessesParser::new()?;

    util::is_system_off(&mut parser)?;
    parser.parse_cell()?;

    let action_data = parser.parse_action()?;
    let action = action_data.as_reader().action().raw_data();
    let params = action_data.as_reader().params().raw_data();

    if action == b"start_account_auction"
        || action == b"edit_account_auction"
        || action == b"cancel_account_auction"
        || action == b"bid_account_auction"
        || action == b"confirm_account_auction"
    {
        let timestamp = util::load_oracle_data(OracleCellType::Time)?;
        let config_main = parser.configs.main()?;
        let auction_config = parser.configs.auction()?;
        // let (input_auction_cell, output_auction_cell) = load_auction_cell()?;

        if action == b"start_account_auction" {
            debug!("Route to start_account_auction action ...");
        } else if action == b"cancel_account_auction" {
            debug!("Route to cancel_account_auction action ...");
        } else if action == b"bid_account_auction" {
            debug!("Route to bid_account_auction action ...");
        } else if action == b"confirm_account_auction" {
            debug!("Route to confirm_account_auction action ...");
        }
    } else {
        return Err(Error::ActionNotSupported);
    }
    Ok(())
}
