use ckb_std::debug;
use das_core::{error::Error, util, witness_parser::WitnessesParser};

pub fn main() -> Result<(), Error> {
    debug!("====== Running account-auction-cell-type ======");
    let mut parser = WitnessesParser::new()?;
    let action_opt = parser.parse_action_with_params()?;
    if action_opt.is_none() {
        return Err(Error::ActionNotSupported);
    }

    let (action_raw, _params_raw) = action_opt.unwrap();
    let action = action_raw.as_reader().raw_data();
    // let params = params_raw.iter().map(|param| param.as_reader()).collect::<Vec<_>>();

    util::is_system_off(&mut parser)?;
    parser.parse_cell()?;

    if action == b"start_account_auction"
        || action == b"edit_account_auction"
        || action == b"cancel_account_auction"
        || action == b"bid_account_auction"
        || action == b"confirm_account_auction"
    {
        // let timestamp = util::load_oracle_data(OracleCellType::Time)?;
        // let config_main_reader = parser.configs.main()?;
        // let config_secondary_market_reader = parser.configs.secondary_market()?;
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
