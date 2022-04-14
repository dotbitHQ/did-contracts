use ckb_std::ckb_constants::Source;
use das_core::{constants::*, debug, error::Error, util, witness_parser::WitnessesParser};

pub fn main() -> Result<(), Error> {
    debug!("====== Running account-auction-cell-type ======");
    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(Error::ActionNotSupported),
    };
    let action = action_cp.as_slice();

    util::is_system_off(&parser)?;
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
    } else if action == b"force_recover_account_status" {
        util::require_type_script(
            &parser,
            TypeScript::AccountCellType,
            Source::Input,
            Error::InvalidTransactionStructure,
        )?;
    } else {
        return Err(Error::ActionNotSupported);
    }
    Ok(())
}
