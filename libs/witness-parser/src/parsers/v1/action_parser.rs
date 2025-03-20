#[cfg(feature = "no_std")]
use alloc::string::{String, ToString};
#[cfg(feature = "no_std")]
use alloc::vec;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;
#[cfg(feature = "no_std")]
use core::str::FromStr;

use das_types::constants::{
    Action, ActionParams, ActionParamsData, LockRole, WITNESS_HEADER_BYTES, WITNESS_TYPE_BYTES,
};
use das_types::packed::{self as packed, ActionDataReader};
use molecule::prelude::Entity;

use crate::error::WitnessParserError;

pub fn parse_action(
    index: usize,
    buf: Vec<u8>,
) -> Result<(packed::ActionData, Action, ActionParams, ActionParamsData), WitnessParserError> {
    let action_data = match buf.get((WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES)..) {
        Some(buf) => match packed::ActionData::from_slice(buf) {
            Ok(data) => data,
            Err(err) => {
                return Err(WitnessParserError::DecodingActionDataFailed {
                    index,
                    err: err.to_string(),
                });
            }
        },
        None => return Err(WitnessParserError::LoadActionDataBodyFailed { index }),
    };

    let action_str = String::from_utf8(action_data.as_reader().action().raw_data().to_vec()).map_err(|_| {
        WitnessParserError::DecodingActionDataFailed {
            index,
            err: String::from("The action is not a utf-8 string."),
        }
    })?;
    let action = match Action::from_str(&action_str) {
        Ok(action) => action,
        Err(_) => Action::Others,
    };

    let (action_params, action_params_data) = match action {
        Action::BuyAccount => parse_buy_account(index, action_data.as_reader())?,
        Action::UnitTest => parse_test_name(action_data.as_reader()),
        _ => {
            if action_data.params().is_empty() {
                (ActionParams::None, vec![])
            } else {
                let buf = action_data.as_reader().params().raw_data();
                let role = LockRole::try_from(*(buf.last().unwrap()))
                    .map_err(|_| WitnessParserError::DecodingActionParamsFailed { index })?;
                let mut action_params_data: ActionParamsData = vec![];
                action_params_data.push(vec![*(buf.last().unwrap())]);
                (ActionParams::Role(role), action_params_data)
            }
        }
    };

    Ok((action_data, action, action_params, action_params_data))
}

fn parse_buy_account(
    index: usize,
    action_data: ActionDataReader,
) -> Result<(ActionParams, ActionParamsData), WitnessParserError> {
    // TODO replace this implement with LV parser
    let bytes = action_data.params().raw_data();
    let first_header = bytes
        .get(..4)
        .ok_or(WitnessParserError::DecodingActionParamsFailed { index })?;
    let length_of_inviter_lock = u32::from_le_bytes(first_header.try_into().unwrap()) as usize;
    let inviter_lock_bytes = bytes
        .get(..length_of_inviter_lock)
        .ok_or(WitnessParserError::DecodingActionParamsFailed { index })?
        .to_vec();

    let second_header = bytes
        .get(length_of_inviter_lock..(length_of_inviter_lock + 4))
        .ok_or(WitnessParserError::DecodingActionParamsFailed { index })?;
    let length_of_channel_lock = u32::from_le_bytes(second_header.try_into().unwrap()) as usize;
    let channel_lock_bytes = bytes
        .get(length_of_inviter_lock..(length_of_inviter_lock + length_of_channel_lock))
        .ok_or(WitnessParserError::DecodingActionParamsFailed { index })?
        .to_vec();

    let bytes_of_role = bytes
        .get((length_of_inviter_lock + length_of_channel_lock)..)
        .ok_or(WitnessParserError::DecodingActionParamsFailed { index })?;
    let role = match LockRole::try_from(bytes_of_role[0]) {
        Ok(role) => role,
        Err(_) => {
            return Err(WitnessParserError::DecodingActionDataFailed {
                index,
                err: String::from("The role is undefined."),
            });
        }
    };

    // debug!("bytes_of_inviter_lock = 0x{}", hex::encode(bytes_of_inviter_lock));
    // debug!("bytes_of_channel_lock = 0x{}", hex::encode(bytes_of_channel_lock));

    let action_params = ActionParams::BuyAccount {
        inviter_lock_bytes: inviter_lock_bytes.clone(),
        channel_lock_bytes: channel_lock_bytes.clone(),
        role,
    };

    let mut action_params_data: ActionParamsData = vec![];
    action_params_data.push(inviter_lock_bytes);
    action_params_data.push(channel_lock_bytes);
    action_params_data.push(vec![bytes_of_role[0]]);

    Ok((action_params, action_params_data))
}

fn parse_test_name(action_data: ActionDataReader) -> (ActionParams, ActionParamsData) {
    let buf = action_data.params().raw_data();
    let name = String::from_utf8(buf.to_vec()).unwrap_or_default();

    let action_params = ActionParams::TestName(name);
    let mut action_params_data: ActionParamsData = vec![];
    action_params_data.push(buf.to_vec());

    (action_params, action_params_data)
}
