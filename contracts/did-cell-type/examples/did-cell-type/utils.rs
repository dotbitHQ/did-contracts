extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use ckb_std::ckb_constants::{CellField, Source};
use ckb_std::error::SysError;
use ckb_std::{debug, high_level, syscalls};
use das_core::constants::{ScriptType, ONE_CKB};
use das_core::data_parser::account_cell;
use das_core::util::{blake2b_160, calc_type_id, find_cells_by_type_id_in_inputs_and_outputs, load_type_args};
use das_types::constants::{get_account_cell_type_id, get_cluster_id, get_did_cell_type_id, get_recycle_indiator_tx};
use das_types::packed::{self, DidEntity, Records, SporeData, WitnessData};
use did_cell_type::error::ErrorCode;
use did_cell_type::parse_did_cell_account_info;
use molecule::hex_string;
use molecule::prelude::Entity;

#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub account: Vec<u8>,
    pub expire_at: u64,
    pub records: Option<Records>,
}

#[derive(Debug, Clone)]
pub struct DidCellInfo {
    pub cluster_id: Option<Vec<u8>>,
    pub index: usize,
    pub account: Vec<u8>,
    pub expire_at: u64,
    pub hash: [u8; 20],
    pub capacity: u64,
    pub lock_args_size: u64,
    pub witness_data: Option<WitnessData>,
}

#[derive(Debug, Clone)]
pub struct GroupInfo {
    pub input_account: Option<AccountInfo>,
    pub output_account: Option<AccountInfo>,
    pub input_did: Option<DidCellInfo>,
    pub output_did: Option<DidCellInfo>,
}

impl GroupInfo {
    fn new() -> GroupInfo {
        GroupInfo {
            input_account: None,
            output_account: None,
            input_did: None,
            output_did: None,
        }
    }

    // validate did-cell's hash to match witness
    pub fn validate_witness(&self) -> bool {
        let output_did = self.output_did.as_ref();
        let input_did = self.input_did.as_ref();

        // delete did-cell
        if output_did.is_none() {
            return true;
        }

        let output_did = output_did.unwrap();

        // did-cell hash changed
        if input_did.is_some() {
            let input_did = input_did.unwrap();
            if input_did.hash == output_did.hash && output_did.witness_data.is_none() {
                return true;
            }
        }

        // witness must present when create or update did-cell's witness_hash
        if output_did.witness_data.is_none() {
            return false;
        }

        let hash = blake2b_160(output_did.witness_data.as_ref().unwrap().as_slice());
        if output_did.hash != hash {
            debug!(
                "hash is not equal, hash(did-cell): {:?} hash(witness_data): {:?}",
                hex_string(&output_did.hash),
                hex_string(&hash)
            );
            return false;
        }

        return true;
    }

    // validate did-cell-type's args
    pub fn validate_args(&self) -> bool {
        // when create did-cell, args = hash(blake2b256(first_input_outpoint|output_index))
        if self.input_did.is_none() && self.output_did.is_some() {
            let idx = self.output_did.as_ref().unwrap().index;
            if verify_type_id(idx).is_none() {
                debug!("did-cell's args is wrong");
                return false;
            }
        }

        // when update did-cell, args should be the same
        if self.input_did.is_some() && self.output_did.is_some() {
            let input_args = load_type_args(self.input_did.as_ref().unwrap().index, Source::Input);
            let output_args = load_type_args(self.output_did.as_ref().unwrap().index, Source::Output);
            if input_args != output_args {
                debug!("did-cell-type's args is changed");
                return false;
            }
        }

        return true;
    }

    // validate cluster id
    pub fn validate_cluster_id(&self) -> bool {
        // create new did-cell
        if self.input_did.is_none() && self.output_did.is_some() {
            let did_cluster_id = self.output_did.as_ref().unwrap().cluster_id.as_ref();
            let expected_cluster_id = get_cluster_id();

            if let Some(did_cluster_id) = did_cluster_id {
                if did_cluster_id != expected_cluster_id {
                    return false;
                }
            }
        }

        // update did-cell
        if self.input_did.is_some() && self.output_did.is_some() {
            let input_did_cluster_id = self.input_did.as_ref().unwrap().cluster_id.as_ref();
            let output_did_cluster_id = self.output_did.as_ref().unwrap().cluster_id.as_ref();
            if input_did_cluster_id != output_did_cluster_id {
                return false;
            }
        }

        return true;
    }

    pub fn validate_capacity(&self) -> Result<bool, ErrorCode> {
        if self.input_did.is_none() || self.output_did.is_none() {
            return Ok(true);
        }

        let input_did = &self.input_did.as_ref().unwrap();
        let output_did = &self.output_did.as_ref().unwrap();
        let input_capacity = high_level::load_cell_capacity(input_did.index, Source::Input)?;
        let output_capacity = high_level::load_cell_capacity(output_did.index, Source::Output)?;
        let input_occupied_capacity = high_level::load_cell_occupied_capacity(input_did.index, Source::Input)?;
        let output_occupied_capacity = high_level::load_cell_occupied_capacity(output_did.index, Source::Output)?;

        debug!(
            "outputs[{}] The capacity change: {} -> {}",
            output_did.index, input_capacity, output_capacity
        );
        debug!(
            "outputs[{}] The occupied capacity change: {} -> {}",
            output_did.index, input_capacity, output_capacity
        );

        const AVAILABLE_FEE: u64 = 100_000;

        if output_occupied_capacity > input_occupied_capacity {
            // If input_capacity >= output_occupied_capacity, it is Ok to use existing capacity to hold the new data.
            // The existing capacity also can be either withdrawn or used to cover the transaction fee.

            // If input_capacity < output_occupied_capacity, It is Ok to use additional capacity to compensate the
            // insufficient part.
            return Ok(true);
        } else {
            if output_capacity >= input_capacity {
                // It is Ok to keep the existing capacity or add more capacity.
                debug!("Add more capacity: {}", output_capacity - input_capacity);
                return Ok(true);
            } else {
                if input_capacity - output_occupied_capacity > ONE_CKB {
                    // If there is more than 1 CKB left, it is Ok to withdraw the redundant capacity.
                    debug!("Withdraw the redundant capacity: {}", input_capacity - output_capacity);
                    return Ok((output_capacity - output_occupied_capacity) >= ONE_CKB);
                } else {
                    // If the remaining capacity is less than 1 CKB, it only can be used to cover the transaction fee.
                    debug!(
                        "Paied fee with the remaining capacity: {}",
                        input_capacity - output_capacity
                    );
                    return Ok((input_capacity - output_capacity) <= AVAILABLE_FEE);
                }
            }
        }
    }
}

// `Map` used to collect info
type Map = BTreeMap<Vec<u8>, GroupInfo>;

// group all the did-cell and account-cell from inputs and outputs based on account name
pub fn group_all_infos() -> Result<Map, ErrorCode> {
    let mut map = Map::new();

    // collect did-cell data
    let type_id = get_did_cell_type_id();
    // find all the index of did-cell
    let (inputs, outputs) = find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, *type_id).unwrap();
    process_did_cell_info(&mut map, inputs, Source::Input)?;
    process_did_cell_info(&mut map, outputs, Source::Output)?;

    // collect account-cell data
    let type_id = get_account_cell_type_id();
    // find all the index of account-cell
    let (inputs, outputs) = find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, *type_id).unwrap();
    process_account_cell_info(&mut map, inputs, Source::Input)?;
    process_account_cell_info(&mut map, outputs, Source::Output)?;

    process_witness(&mut map)?;
    Ok(map)
}

fn process_witness(map: &mut Map) -> Result<(), ErrorCode> {
    // collect witnesses
    for idx in 0.. {
        match load_witness(idx) {
            Ok(data) => {
                match data {
                    WitnessDataRaw::Did(data) => {
                        let data: &[u8] = &data[3..]; // remove 'DID' from start
                        if let Ok(ent) = DidEntity::from_slice(data) {
                            attach_witness_to_account(ent, map);
                        }
                    }
                    // we will collect grace period from config-cell
                    // TODO Remove the following branch and related code completely
                    WitnessDataRaw::Das(..) => {}
                }
            }
            Err(ErrorCode::IndexOutOfBound) => break,
            Err(ErrorCode::NotDidWitness) => {}
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

fn process_did_cell_info(map: &mut Map, idxs: Vec<usize>, src: Source) -> Result<(), ErrorCode> {
    for idx in idxs {
        let info = load_did_cell_info(idx, src)?;
        let account = info.account.clone();
        let group = map.entry(account).or_insert(GroupInfo::new());

        let did = {
            match src {
                Source::Input => &mut group.input_did,
                Source::Output => &mut group.output_did,
                _ => unimplemented!(),
            }
        };

        if did.is_some() {
            debug!("only one did-cell with same account name is allowed: {:?}", src);
            return Err(ErrorCode::InvalidTransactionStructure);
        }
        *did = Some(info);
    }

    Ok(())
}

fn process_account_cell_info(map: &mut Map, idxs: Vec<usize>, src: Source) -> Result<(), ErrorCode> {
    for idx in idxs {
        let info = load_account_cell_info(idx, src)?;
        let group = map.entry(info.account.clone()).or_insert(GroupInfo::new());
        let account = {
            match src {
                Source::Input => &mut group.input_account,
                Source::Output => &mut group.output_account,
                _ => unimplemented!(),
            }
        };
        if account.is_some() {
            debug!("only one account-cell with same account name is allowed: {:?}", src);

            return Err(ErrorCode::InvalidTransactionStructure);
        }
        *account = Some(info);
    }

    Ok(())
}

// attach witness data to account
fn attach_witness_to_account(ent: DidEntity, map: &mut Map) {
    if let Some(hash) = ent.hash().to_opt() {
        let hash: [u8; 20] = hash.into();
        for (_k, v) in map {
            // collect input did-cell witness
            if let Some(did) = &mut v.input_did {
                if did.hash == hash {
                    did.witness_data = Some(ent.data());
                }
            }
            // collect output did-cell witness
            if let Some(did) = &mut v.output_did {
                if did.hash == hash {
                    did.witness_data = Some(ent.data());
                }
            }
        }
    }
}

// load account-cell data to create a AccountInfo
pub fn load_account_cell_info(index: usize, source: Source) -> Result<AccountInfo, ErrorCode> {
    let data = high_level::load_cell_data(index, source)?;

    let account: Vec<u8> = account_cell::get_account(&data).into();
    let expire_at = account_cell::get_expired_at(&data);
    Ok(AccountInfo {
        account,
        expire_at,
        records: None,
    })
}

// load did-cell info
pub fn load_did_cell_info(idx: usize, src: Source) -> Result<DidCellInfo, ErrorCode> {
    let data = high_level::load_cell_data(idx, src)?;
    let data = SporeData::from_slice(&data).map_err(|e| {
        debug!("decoding SporeData error: {}", e);
        ErrorCode::Encoding
    })?;

    // load capacity of did-cell
    let mut buf = [0u8; 8];

    let len = syscalls::load_cell_by_field(&mut buf, 0, idx, src, CellField::Capacity).map_err(|e| {
        debug!("error load capacity of cell");
        e
    })?;

    if len != buf.len() {
        debug!("error load capacity of cell");
        return Err(ErrorCode::InvalidTransactionStructure);
    }
    let capacity = u64::from_le_bytes(buf);

    // load lock args size of did-cell
    let lock = high_level::load_cell_lock(idx, src)?;
    let lock_args_size = lock.args().raw_data().len() as u64;

    let cluster_id = {
        let cluster_id = data.cluster_id().to_opt();
        match cluster_id {
            Some(cid) => Some(cid.raw_data().to_vec()),
            None => None,
        }
    };
    let content = data.content();
    let info = parse_did_cell_account_info(content.as_reader().raw_data())?;

    Ok(DidCellInfo {
        lock_args_size,
        capacity,
        cluster_id,
        index: idx,
        account: info.account,
        expire_at: info.expire_at,
        hash: info.hash,
        witness_data: None,
    })
}

#[allow(dead_code)]
pub enum WitnessDataRaw {
    Did(Vec<u8>), // witness with prefix 'DID'
    Das(Vec<u8>), // witness with prefix 'das'
}

// load 'DID' or 'das' prefixed witness data
pub fn load_witness(index: usize) -> Result<WitnessDataRaw, ErrorCode> {
    let mut buf = [0u8; 7];
    let ret = syscalls::load_witness(&mut buf, 0, index, Source::Input);

    match ret {
        Ok(_) => Err(ErrorCode::NotDidWitness),
        Err(SysError::LengthNotEnough(actual_size)) => {
            const WITNESS_SIZE_LIMIT: usize = 33000;
            if actual_size > WITNESS_SIZE_LIMIT {
                return Err(ErrorCode::LengthNotEnough);
            }

            if buf.starts_with(b"DID") {
                let mut buf = vec![0u8; actual_size];
                syscalls::load_witness(&mut buf, 0, index, Source::Input)?;
                return Ok(WitnessDataRaw::Did(buf));
            }

            if buf.starts_with(b"das") {
                let mut buf = vec![0u8; actual_size];
                syscalls::load_witness(&mut buf, 0, index, Source::Input)?;
                return Ok(WitnessDataRaw::Das(buf));
            }

            return Err(ErrorCode::NotDidWitness);
        }
        Err(e) => Err(e.into()),
    }
}

pub fn verify_type_id(output_index: usize) -> Option<[u8; 32]> {
    let first_input = match high_level::load_input(0, Source::Input) {
        Ok(cell_input) => cell_input,
        Err(_) => return None,
    };

    let expected_id = calc_type_id(first_input.as_slice(), output_index);
    let type_id_args = load_type_args(output_index, Source::Output);

    debug!("wanted: {expected_id:?}");
    debug!("got({output_index}): {type_id_args:?}");
    if type_id_args.len() < 32 {
        return None;
    }
    if type_id_args.as_ref()[..32] == expected_id {
        return Some(expected_id);
    }

    None
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Create,
    Update,
    Delete,
}

pub fn validate_action(pre_action: &mut Option<Action>, current: Action) -> Result<(), ErrorCode> {
    if *pre_action != None && *pre_action != Some(current) {
        debug!(
            "different action in groups, pre_action: {:?}, current: {:?}",
            pre_action, current
        );

        return Err(ErrorCode::DifferentAction);
    }
    *pre_action = Some(current);

    return Ok(());
}

pub fn validate_delete_dep_cell() -> Result<(), ErrorCode> {
    let tx_hash = get_recycle_indiator_tx();
    let tx = high_level::load_transaction()?;

    let mut has_delete_dep_cell = false;
    for cell_dep in tx.raw().cell_deps() {
        let out_point = cell_dep.out_point();
        debug!(">>> out_point: {}", out_point);
        let hash: [u8; 32] = Into::<packed::Hash>::into(out_point.tx_hash()).into();
        let idx: u32 = Into::<packed::Uint32>::into(out_point.index()).into();
        if hash.as_slice() == tx_hash && idx == 0 {
            has_delete_dep_cell = true;
            break;
        }
    }

    if has_delete_dep_cell {
        Ok(())
    } else {
        Err(ErrorCode::InvalidTransactionStructure)
    }
}
