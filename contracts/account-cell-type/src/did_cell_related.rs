use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::cell::OnceCell;
use core::iter::Iterator;
use core::mem::transmute;
use core::ops::{Deref, Index, IndexMut};

use ckb_std::ckb_constants::Source;
use ckb_std::high_level::{self, load_cell_data, load_cell_type};
use das_core::data_parser::account_cell::get_expired_at;
use das_core::error::{AccountCellErrorCode, ErrorCode, ScriptError};
use das_core::util::parse_account_cell_witness;
use das_core::witness_parser::general_witness_parser::Meta;
use das_core::{code_to_error, das_assert, debug};
use das_types::constants::{get_account_cell_type_id, get_did_cell_type_id, AccountStatus, Action};
use das_types::data_parser::account_cell::get_account;
use das_types::packed::SporeData;
use das_types::prelude::Entity;
use did_cell_type::parse_did_cell_account_info;
use witness_parser::parsers::v1::witness_parser::WitnessesParser;

#[derive(Debug)]
pub(crate) struct Entry {
    input_account_cell: Option<Meta>,
    output_account_cell: Option<Meta>,
    input_did_cell: Option<Meta>,
    output_did_cell: Option<Meta>,
}

impl Entry {
    pub(crate) fn is_upgrade(&self) -> bool {
        match &self {
            Entry {
                input_account_cell: None,
                output_account_cell: None,
                ..
            } => {
                debug!("Not related to AccountCell, hence is not upgrade");
                false
            }
            Entry {
                input_account_cell: Some(_),
                output_account_cell: None,
                ..
            } => {
                debug!("Account recycling detected. Not related to DidCell.");
                false
            }
            Entry {
                input_account_cell: None,
                output_account_cell: Some(_),
                ..
            } => {
                debug!("Generated a AccountCell with status 0x99. Indicating a register + upgrade to DidCell.");
                unimplemented!()
            }
            Entry {
                input_account_cell: Some(input_meta),
                output_account_cell: Some(output_meta),
                ..
            } => {
                let account_data_in_input =
                    parse_account_cell_witness(input_meta.index, input_meta.source.into()).unwrap();
                let account_data_in_output =
                    parse_account_cell_witness(output_meta.index, output_meta.source.into()).unwrap();
                let original_status = AccountStatus::from_repr(u8::from(account_data_in_input.as_reader().status()))
                    .expect("Invalid account status");
                let current_status = AccountStatus::from_repr(u8::from(account_data_in_output.as_reader().status()))
                    .expect("Invalid account status");
                match (original_status, current_status) {
                    (AccountStatus::Normal, AccountStatus::Upgraded) => true,
                    _ => false,
                }
            }
        }
    }

    pub(crate) fn is_expire_consistent(&self, source: Source) -> Result<bool, Box<dyn ScriptError>> {
        let (account_cell_meta, did_cell_meta) = match source {
            Source::Input => (self.input_account_cell, self.input_did_cell),
            Source::Output => (self.output_account_cell, self.output_did_cell),
            _ => unreachable!(),
        };

        if account_cell_meta.is_none() {
            return Ok(false);
        }
        if did_cell_meta.is_none() {
            return Ok(false);
        }

        let account_cell_data = load_cell_data(
            account_cell_meta.unwrap().index,
            account_cell_meta.unwrap().source.into(),
        )?;
        let account_cell_expire_at = get_expired_at(&account_cell_data);
        let spore_data = SporeData::from_slice(
            load_cell_data(did_cell_meta.unwrap().index, did_cell_meta.unwrap().source.into())?.as_slice(),
        )?;

        let parsed =
            parse_did_cell_account_info(&spore_data.content().raw_data()).expect("Parsing did cell data error");
        let did_cell_expire_at = parsed.expire_at;
        Ok(account_cell_expire_at == did_cell_expire_at)
    }
}

type AccountName = Vec<u8>;
#[derive(Debug)]
pub(crate) struct Grouped(BTreeMap<AccountName, Entry>);

impl Deref for Grouped {
    type Target = BTreeMap<AccountName, Entry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Grouped {
    pub(crate) fn get_group_by_meta(&self, index: usize, source: Source) -> Option<&Entry> {
        self.iter()
            .find(|e| match source {
                Source::Input => {
                    e.1.input_account_cell.is_some_and(|meta| meta.index == index)
                        || e.1.input_did_cell.is_some_and(|meta| meta.index == index)
                }
                Source::Output => {
                    e.1.output_account_cell.is_some_and(|meta| meta.index == index)
                        || e.1.output_did_cell.is_some_and(|meta| meta.index == index)
                }
                _ => unreachable!(),
            })
            .map(|e| e.1)
    }
}

pub(crate) fn group_cells_by_account_name() -> Result<&'static Grouped, Box<dyn ScriptError>> {
    static mut RES: OnceCell<Result<Grouped, Box<dyn ScriptError>>> = OnceCell::new();
    let res = unsafe {
        RES.get_or_init(|| {
            debug!("Start grouping account cell and did cell by account name");
            let account_cell_type_id = get_account_cell_type_id();
            let did_cell_type_id = get_did_cell_type_id();
            // [Option<Meta;> 4] is the index of account-cell and did-cell. 0 for input_account_cell, 1 for output_account_cell, 2 for input_did_cell, 3 for output_did_cell
            let mut map = BTreeMap::<AccountName, [Option<Meta>; 4]>::new();
            for source in [Source::Input, Source::Output] {
                let _ = high_level::QueryIter::new(
                    |index, source| load_cell_type(index, source).map(|r| (r, Meta { index, source })),
                    source,
                )
                .try_for_each(|(s, meta)| {
                    if let Some(script) = s {
                        let code_hash = script.code_hash();
                        let (account_name, insert_index) = if code_hash.as_slice() == account_cell_type_id.as_slice() {
                            let data = load_cell_data(meta.index, meta.source.into()).unwrap();
                            let account_name = get_account(&data).expect("Account name cannot be parsed from cell");
                            let index = match meta.source {
                                Source::Input => 0usize,
                                Source::Output => 1usize,
                                _ => unreachable!(),
                            };
                            (account_name.to_owned(), index)
                        } else if code_hash.as_slice() == did_cell_type_id.as_slice() {
                            let data = load_cell_data(meta.index, meta.source.into()).unwrap();
                            let spore_data = SporeData::from_slice(&data).expect("Parsing spore data error");
                            let parsed_did_data = parse_did_cell_account_info(&spore_data.content().raw_data())
                                .expect("Paring did data error");
                            let account_name = parsed_did_data.account;
                            let index = match meta.source {
                                Source::Input => 2usize,
                                Source::Output => 3usize,
                                _ => unreachable!(),
                            };
                            (account_name, index)
                        } else {
                            return Ok(());
                        };

                        let map_entry = map.entry(account_name.to_owned()).or_insert([None, None, None, None]);

                        if map_entry.index(insert_index).is_some() {
                            debug!("Found more than 1 account cell in input with same name");
                            return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
                            // return Err(Error::new(ErrorCode::InvalidTransactionStructure, Default::default()))?
                        } else {
                            *map_entry.index_mut(insert_index) = Some(meta);
                        }

                        Ok(())
                    } else {
                        Ok(())
                    }
                });
            }

            let res: BTreeMap<AccountName, Entry> = map
                .into_iter()
                .map(|(name, e)| {
                    (
                        name,
                        Entry {
                            input_account_cell: e[0],
                            output_account_cell: e[1],
                            input_did_cell: e[2],
                            output_did_cell: e[3],
                        },
                    )
                })
                .collect();

            debug!("Grouping succeeded");
            Ok(Grouped(res))
        })
    };

    match res.as_ref() {
        Ok(g) => Ok(g),
        Err(e) => {
            let err_code = unsafe { transmute::<i8, ErrorCode>(e.as_i8()) };
            Err(code_to_error!(err_code))
        }
    }
}

pub(crate) fn verify_did_cell_related_logic() -> Result<(), Box<dyn ScriptError>> {
    debug!("Start verifying did cell related logic");
    let group = group_cells_by_account_name()?;
    let witness_parser = WitnessesParser::get_instance();
    for e in group.iter() {
        match e.1 {
            Entry {
                input_account_cell: None,
                output_account_cell: None,
                ..
            } => {
                debug!("did-cell's own logic. Not related to AccountCell");
            }
            Entry {
                input_account_cell: Some(_),
                output_account_cell: None,
                ..
            } => {
                debug!("Account recycling detected. Not related to DidCell.")
            }
            Entry {
                input_account_cell: None,
                output_account_cell: Some(output_meta),
                ..
            } => {
                let account_data_in_output = parse_account_cell_witness(output_meta.index, output_meta.source.into())?;
                let current_status = AccountStatus::from_repr(u8::from(account_data_in_output.as_reader().status()))
                    .expect("Invalid account status");
                if current_status == AccountStatus::Upgraded {
                    debug!("Generated an AccountCell with status 0x99. Indicating a register + upgrade to DidCell.");
                    unimplemented!()
                }
            }
            entry @ Entry {
                input_account_cell: Some(input_meta),
                output_account_cell: Some(output_meta),
                ..
            } => {
                let account_data_in_input = parse_account_cell_witness(input_meta.index, input_meta.source.into())?;
                let account_data_in_output = parse_account_cell_witness(output_meta.index, output_meta.source.into())?;
                let original_status = AccountStatus::from_repr(u8::from(account_data_in_input.as_reader().status()))
                    .expect("Invalid account status");
                let current_status = AccountStatus::from_repr(u8::from(account_data_in_output.as_reader().status()))
                    .expect("Invalid account status");
                match (original_status, current_status) {
                    (AccountStatus::Normal, AccountStatus::Upgraded) => {
                        debug!("Upgrade detected");
                        das_assert!(
                            entry.input_did_cell.is_none(),
                            AccountCellErrorCode::InvalidUpgradeTxStructure,
                            "DidCell not allowed in input"
                        );
                        das_assert!(
                            entry.output_did_cell.is_some(),
                            AccountCellErrorCode::InvalidUpgradeTxStructure,
                            "Require DidCell in output"
                        );
                        let action = witness_parser.action;
                        das_assert!(
                            [
                                Action::UpgradeDid,
                                Action::EditRecords,
                                Action::RenewAccount,
                                Action::TransferAccount,
                                Action::BidExpiredAccountDutchAuction
                            ]
                            .into_iter()
                            .find(|a| a == &action)
                            .is_some(),
                            AccountCellErrorCode::InvalidUpgradeAction,
                            "This action is not supported for upgrade"
                        );
                        das_assert!(
                            entry.is_expire_consistent(Source::Output)?,
                            AccountCellErrorCode::InvalidUpgradeTxStructure,
                            "The expire_at in output is not consistent for AccountCell and DidCell"
                        )
                    }
                    (AccountStatus::Upgraded, AccountStatus::Upgraded) => {
                        debug!(
                            "Only renew action and confirm proposal are allowed for upgraded account, start verifing"
                        );
                        if witness_parser.action == Action::RenewAccount {
                            debug!("Renew check");
                            das_assert!(
                                entry.input_did_cell.is_some(),
                                AccountCellErrorCode::InvalidUpgradeTxStructure,
                                "No DidCell in input"
                            );
                            das_assert!(
                                entry.output_did_cell.is_some(),
                                AccountCellErrorCode::InvalidUpgradeTxStructure,
                                "No DidCell in output"
                            );
                            das_assert!(
                                entry.is_expire_consistent(Source::Output)?,
                                AccountCellErrorCode::InvalidUpgradeTxStructure,
                                "The expire_at in output is not consistent for AccountCell and DidCell"
                            )
                        } else if witness_parser.action == Action::ConfirmProposal {
                            debug!("Confirm Proposal check. Do nothing.");
                        } else if witness_parser.action == Action::RecycleExpiredAccount {
                            debug!("inputs[{}] Ignore the first AccountCell; the verification logic is protected in account-cell-type.", input_meta.index);
                        } else {
                            return Err(code_to_error!(AccountCellErrorCode::InvalidUpgradeAction));
                        }
                    }
                    (_, AccountStatus::Upgraded) => {
                        unreachable!("Only 0x00 to 0x99 is allowed")
                    }
                    (_, _) => {
                        debug!("Not related to DidCell")
                    }
                }
            }
        }
    }

    debug!("DidCell related logic verified");
    Ok(())
}
