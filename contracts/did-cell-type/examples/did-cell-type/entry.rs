use alloc::string::String;

use config::Config;
use das_core::constants::OracleCellType;
use das_core::{debug, util};
use did_cell_type::error::ErrorCode;

use crate::utils::{group_all_infos, validate_action, validate_delete_dep_cell, Action, GroupInfo};

/*
===== design =====
    # group cells
        * how to
            * load all inputs and outputs and witnesses for did-cell-type cell and account-cell-type, group then by account name
        * grouped structure
            account name
                * did-cell-type in inputs and outputs
                * account-cell-type in inputs and outputs
                * witness of account-cell-type
                * witness of did-cell-type


    # prerequisites
        account name
            * no more than one did-cell-type in inputs or outputs
            * no more than one account-cell-type in inputs or outputs
            * no more than one witness with prefix 'DID', referencing did-cell-type
                * did-cell-type's DidCellData's hash matches DidCellData
                * did-cell-type's DidCellData's hash equals DidEntity's option hash if any
            * no more than one withness with prefix 'das', referencing account-cell-type
                * the hash in account-cell-type's data matches account-cell-type's witness

    # action based on group data:
        * upgrade account
            account name:
                * no did-cell-type in inputs and one did-cell-type in outputs
                    * handle by did-cell-type
                    * `expire_at` should be equal in did-cell-type and account-cell-type
                    * validate witness
                    * [no need?] records in witness of did-cell-type and account-cell-typeshould be equal

        * renewal | edit records | transfer owner
            account name
                * one did-cell-type in inputs and one did-cell-type in outputs
                    * handle by did-cell-type
                    * if `hash` changed, then action is editting records, then validate witness
                    * if `expire_at` changed, then action is renewal account, then make sure `expire_at`  equals `account-cell-type`'s `expire_at`
                    * if nothing changed, then action is transfer owner, then validate witness

        * destroy account
            account name:
                * one did-cell-type in inputs and no did-cell-type in outputs
                    * handle by did-cell-type
                    * validate witness
                    * time <= expire_at + 3 months
*/

pub fn main() -> Result<(), ErrorCode> {
    debug!("====== Running did-cell-type ======");

    // group all the infos by account name, according to the design above.
    // as we group all the infos by account name, so we don't need to verify account name,
    // because different account will be dispatch to different group.
    // as account-cell-type already has many validations,
    // did-cell-type only need to make itself is valid and its info is syncnized with account-cell
    let map = group_all_infos()?;
    let mut pre_action: Option<Action> = None;
    for (_acc, info) in map {
        debug!("operate on account: {}", String::from_utf8(_acc).unwrap());
        // validate capacity
        if !info.validate_capacity()? {
            debug!("validate did cell capacity failed!");
            return Err(ErrorCode::WrongCapacity);
        }

        // validate all the did-cell's witness.
        // as we validate all did-cell's in outputs here,
        // we don't need to verify their witness anymore.
        if !info.validate_witness() {
            debug!("validate did cell witness hash failed!");
            return Err(ErrorCode::InvalidDidCellWitnessHash);
        }

        if !info.validate_args() {
            debug!("validate did-cell-type args failed!");
            return Err(ErrorCode::WrongTypeArgs);
        }

        if !info.validate_cluster_id() {
            debug!("validate did-cell cluster id failed!");
            return Err(ErrorCode::WrongClusterID);
        }

        // dispatch action based on the grouped info.
        let GroupInfo {
            input_did,
            output_did,
            input_account,
            output_account,
        } = info;

        let delete_dep_cell_validated = false;
        match (input_account, output_account, input_did, output_did) {
            // no did-cell in inputs, one did-cell in outputs, means upgrading account
            (Some(_), Some(account), None, Some(did)) => {
                // when upgrade a account, we will assume that
                // the account-did-type will make sure the account-cell is valid.
                // so we only need to make sure the expire time in did-cell is equal to account-cell.
                // this way, its possible to combine different actions for did-cell together.
                debug!("[action]: upgrade account");
                validate_action(&mut pre_action, Action::Create)?;
                debug!(
                    "account-cell's expire_at: {} did-cell's expire_at: {}.",
                    account.expire_at, did.expire_at
                );
                if account.expire_at != did.expire_at {
                    return Err(ErrorCode::ExpireAtNotEqual);
                }
            }
            // one account-cell in inputs, one account-cell in inputs,
            // one did-cell in inputs, one did-cell in outputs,
            // means renew account
            (Some(_), Some(account), Some(_), Some(did)) => {
                // when renew account, we will assume that
                // the account-did-type will make sure the renewl is valid.
                // so we only need to make sure the expire time in did-cell is equal to account-cell
                debug!("[action]: renewl account");
                validate_action(&mut pre_action, Action::Update)?;
                if account.expire_at != did.expire_at {
                    debug!(
                        "error expire_at not equal, account-cell: {}, did-cell: {}",
                        account.expire_at, did.expire_at
                    );
                    return Err(ErrorCode::ExpireAtNotEqual);
                }
            }
            // no account-cell, one did-cell in inputs, one did-cell in outputs,
            // means we are doing did-cell only actions, aka. edit records or transfer owner.
            (None, None, Some(i_did), Some(o_did)) => {
                debug!("[action]: edit records | transfer owner");
                validate_action(&mut pre_action, Action::Update)?;
                // because we already verified witness and account name (by grouping them),
                // we only need to make sure the expire time not changed.
                if i_did.expire_at != o_did.expire_at {
                    debug!(
                        "error expire_at not equal, account-cell: {}, did-cell: {}",
                        i_did.expire_at, o_did.expire_at
                    );
                    return Err(ErrorCode::ExpireAtNotEqual);
                }
            }
            // on did-cell in inputs, no did-cell in outputs,
            // means we are destroying did-cell
            (_, _, Some(did), None) => {
                debug!("[action]: destroy did-cell");
                validate_action(&mut pre_action, Action::Delete)?;
                if !delete_dep_cell_validated {
                    validate_delete_dep_cell()?;
                    // delete_dep_cell_validated = true;
                }
                let now = util::load_oracle_data(OracleCellType::Time).unwrap();
                let config_account = Config::get_instance().account()?;
                let grace_period = config_account.expiration_grace_period() as u64;

                debug!("the grace period is: {}", grace_period);
                // we need make sure user has grace period to regrate
                if did.expire_at + grace_period > now {
                    return Err(ErrorCode::NotValidToDestroy);
                }
            }
            // invalid cell patterns in inputs and outputs
            (a, b, c, d) => {
                debug!("invalid transaction structure.");
                debug!("{:?} {:?} {:?} {:?}", a, b, c, d);
                return Err(ErrorCode::InvalidTransactionStructure);
            }
        }

        debug!("\n\n");
    }
    Ok(())
}
