use alloc::{borrow::ToOwned, boxed::Box};
use ckb_std::{
    ckb_constants::Source,
    high_level::{self, load_cell_capacity, load_cell_lock, load_cell_type, load_script},
};
use core::{convert::TryFrom, result::Result};
use das_core::{
    assert,
    constants::*,
    data_parser,
    debug,
    error::Error,
    util, verifiers, warn,
    witness_parser::WitnessesParser,
};
use das_map::{map::Map, util as map_util};
use das_sorted_list::DasSortedList;
use das_types::{
    constants::*,
    mixer::{AccountCellDataMixer, PreAccountCellDataReaderMixer},
    packed::*,
    prelude::*,
    prettier::Prettier,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running proposal-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(Error::ActionNotSupported),
    };
    let action = action_cp.as_slice();

    util::is_system_off(&parser)?;

    debug!("Find out ProposalCell ...");

    // Find out PreAccountCells in current transaction.
    let this_type_script = load_script()?;
    let this_type_script_reader = this_type_script.as_reader();
    let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
    let dep_cells = util::find_cells_by_script(ScriptType::Type, this_type_script_reader, Source::CellDep)?;

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.to_vec()).map_err(|_| Error::ActionNotSupported)?
    );
    match action {
        b"propose" | b"extend_proposal" => {
            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            let config_proposal = parser.configs.proposal()?;

            if action == b"propose" {
                assert!(
                    dep_cells.len() == 0,
                    Error::InvalidTransactionStructure,
                    "There should be 0 ProposalCell in the cell_deps."
                );
            } else {
                assert!(
                    dep_cells.len() == 1,
                    Error::InvalidTransactionStructure,
                    "There should be 1 ProposalCell found in the cell_deps"
                );
            }

            verifiers::common::verify_cell_number("ProposalCell", &input_cells, 0, &output_cells, 1)?;
            verifiers::misc::verify_always_success_lock(output_cells[0], Source::Output)?;

            let dep_cell_witness;
            let dep_cell_witness_reader;
            let mut prev_slices_reader_opt = None;
            if action == b"extend_proposal" {
                dep_cell_witness = util::parse_proposal_cell_witness(&parser, dep_cells[0], Source::CellDep)?;
                dep_cell_witness_reader = dep_cell_witness.as_reader();
                prev_slices_reader_opt = Some(dep_cell_witness_reader.slices());
            }

            let output_cell_witness = util::parse_proposal_cell_witness(&parser, output_cells[0], Source::Output)?;
            let output_cell_witness_reader = output_cell_witness.as_reader();

            let required_cells_count = verify_slices(config_proposal, output_cell_witness_reader.slices())?;
            let dep_related_cells = find_proposal_related_cells(config_main, Source::CellDep)?;

            #[cfg(debug_assertions)]
            inspect_slices(output_cell_witness_reader.slices())?;
            #[cfg(debug_assertions)]
            inspect_related_cells(&parser, config_main, dep_related_cells.clone(), Source::CellDep)?;

            assert!(
                required_cells_count == dep_related_cells.len(),
                Error::ProposalSliceRelatedCellMissing,
                "Some of the proposal relevant cells are missing. (expected: {}, current: {})",
                required_cells_count,
                dep_related_cells.len()
            );

            verify_slices_relevant_cells(
                &parser,
                timestamp,
                config_main,
                output_cell_witness_reader.slices(),
                dep_related_cells,
                prev_slices_reader_opt,
            )?;
        }
        b"confirm_proposal" => {
            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            parser.parse_cell()?;
            let config_account = parser.configs.account()?;
            let config_main = parser.configs.main()?;
            let config_profit_rate = parser.configs.profit_rate()?;
            let config_proposal_reader = parser.configs.proposal()?;

            verifiers::common::verify_cell_number("ProposalCell", &input_cells, 1, &output_cells, 0)?;

            let input_cell_witness = util::parse_proposal_cell_witness(&parser, input_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();

            debug!("Check if the ProposalCell is able to be confirmed.");

            let height = util::load_oracle_data(OracleCellType::Height)?;
            let proposal_min_confirm_interval = u8::from(config_proposal_reader.proposal_min_confirm_interval()) as u64;
            let created_at_height = u64::from(input_cell_witness_reader.created_at_height());

            assert!(
                height >= created_at_height + proposal_min_confirm_interval,
                Error::ProposalConfirmNeedWaitLonger,
                "ProposalCell should be confirmed later, about {} block to wait.",
                created_at_height + proposal_min_confirm_interval - height
            );

            debug!("Check all AccountCells are updated or created base on proposal.");

            verify_proposal_execution_result(
                &parser,
                config_account,
                config_main,
                config_profit_rate,
                timestamp,
                input_cell_witness_reader,
            )?;

            verify_refund_correct(input_cells[0], input_cell_witness_reader, 0)?;
        }
        b"recycle_proposal" => {
            parser.parse_cell()?;
            let config_proposal_reader = parser.configs.proposal()?;

            verifiers::common::verify_cell_number("ProposalCell", &input_cells, 1, &output_cells, 0)?;

            debug!("Check if ProposalCell can be recycled.");

            let input_cell_witness = util::parse_proposal_cell_witness(&parser, input_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();

            let height = util::load_oracle_data(OracleCellType::Height)?;
            let proposal_min_recycle_interval = u8::from(config_proposal_reader.proposal_min_recycle_interval()) as u64;
            let created_at_height = u64::from(input_cell_witness_reader.created_at_height());

            assert!(
                height >= created_at_height + proposal_min_recycle_interval,
                Error::ProposalRecycleNeedWaitLonger,
                "ProposalCell should be recycled later, about {} block to wait.",
                created_at_height + proposal_min_recycle_interval - height
            );

            verify_refund_correct(input_cells[0], input_cell_witness_reader, 10000)?;
        }
        _ => return Err(Error::ActionNotSupported),
    }

    Ok(())
}

#[cfg(debug_assertions)]
fn inspect_slices(slices_reader: SliceListReader) -> Result<(), Error> {
    debug!("Inspect Slices [");
    for (sl_index, sl_reader) in slices_reader.iter().enumerate() {
        debug!("  Slice[{}] [", sl_index);
        for (index, item) in sl_reader.iter().enumerate() {
            let type_ = item.item_type().raw_data()[0];
            let item_type = match type_ {
                0 => "exist",
                1 => "proposed",
                _ => "new",
            };

            debug!(
                "    Item[{}] {{ account_id: {:?}, item_type: {}, next: {:?} }}",
                index,
                item.account_id(),
                item_type,
                item.next()
            );
        }
        debug!("  ]");
    }
    debug!("]");

    Ok(())
}

#[cfg(debug_assertions)]
fn inspect_related_cells(
    parser: &WitnessesParser,
    config_main: ConfigCellMainReader,
    related_cells: Vec<usize>,
    related_cells_source: Source,
) -> Result<(), Error> {
    debug!("Inspect {:?}{:?}:", related_cells_source, related_cells);

    for i in related_cells {
        let script = load_cell_type(i, related_cells_source)?.unwrap();
        let code_hash = Hash::from(script.code_hash());
        let data = util::load_cell_data(i, related_cells_source)?;

        if util::is_reader_eq(config_main.type_id_table().account_cell(), code_hash.as_reader()) {
            let (version, _, _) = parser.verify_and_get(DataType::AccountCellData, i, related_cells_source)?;
            debug!(
                "  {:?}[{}] AccountCell(v{}): {{ id: 0x{}, next: 0x{}, account: {} }}",
                related_cells_source,
                i,
                version,
                util::hex_string(data_parser::account_cell::get_id(&data)),
                util::hex_string(data_parser::account_cell::get_next(&data)),
                String::from_utf8(data_parser::account_cell::get_account(&data).to_vec()).unwrap()
            );
        } else if util::is_reader_eq(config_main.type_id_table().pre_account_cell(), code_hash.as_reader()) {
            let (version, _, _) = parser.verify_and_get(DataType::PreAccountCellData, i, related_cells_source)?;
            debug!(
                "  {:?}[{}] PreAccountCell(v{}): {{ id: 0x{} }}",
                related_cells_source,
                i,
                version,
                util::hex_string(pre_account_cell::get_id(&data))
            );
        }
    }

    Ok(())
}

fn verify_slices(config: ConfigCellProposalReader, slices_reader: SliceListReader) -> Result<usize, Error> {
    debug!("Check the data structure of proposal slices.");

    // debug!("slices_reader = {}", slices_reader);

    let mut required_cells_count: usize = 0;
    let mut account_cell_contained = 0;
    let mut pre_account_cell_contained = 0;

    assert!(
        slices_reader.len() > 0,
        Error::ProposalSlicesCanNotBeEmpty,
        "The slices of ProposalCell should not be empty."
    );

    let mut account_id_list = Vec::new();
    let mut account_id_list_with_next = Vec::new();
    let mut exist_next_list = Vec::new();
    for (sl_index, sl_reader) in slices_reader.iter().enumerate() {
        debug!("Check Slice[{}] ...", sl_index);

        assert!(
            sl_reader.len() > 1,
            Error::ProposalSliceMustContainMoreThanOneElement,
            "Slice[{}] must contain more than one element, but {} found.",
            sl_index,
            sl_reader.len()
        );

        // The "next" of last item is refer to an existing account, so we put it into the vector.
        let last_item = sl_reader.get(sl_reader.len() - 1).unwrap();
        let last_item_next = last_item.next().raw_data().to_vec();
        exist_next_list.push(last_item_next.clone());

        for (index, item) in sl_reader.iter().enumerate() {
            debug!("  Check if Item[{}] refer to correct next.", index);

            let account_id = item.account_id().raw_data().to_vec();

            if index == 0 {
                account_cell_contained += 1;
                assert!(
                    u8::from(item.item_type()) != ProposalSliceItemType::New as u8,
                    Error::ProposalCellTypeError,
                    "  Item[{}] The item_type of item[{}] should not be {:?}.",
                    index,
                    index,
                    ProposalSliceItemType::New
                );

                // Some account ID may be appear in next field and it is also an exist item in later slices,
                // we need to remove it from exist_next_list, because its uniqueness will be checked in account_id_list.
                if u8::from(item.item_type()) == ProposalSliceItemType::Exist as u8 {
                    let found = exist_next_list
                        .iter()
                        .enumerate()
                        .find(|(_i, next)| &account_id == *next);

                    if let Some((i, _)) = found {
                        exist_next_list.remove(i);
                    }
                }
            } else {
                pre_account_cell_contained += 1;
                assert!(
                    u8::from(item.item_type()) == ProposalSliceItemType::New as u8,
                    Error::ProposalCellTypeError,
                    "  Item[{}] The item_type of item[{}] should be {:?}.",
                    index,
                    index,
                    ProposalSliceItemType::New
                );
            }

            // Check the continuity of the items in the slice.
            if let Some(next_item) = sl_reader.get(index + 1) {
                assert!(
                    util::is_reader_eq(item.next(), next_item.account_id()),
                    Error::ProposalSliceIsDiscontinuity,
                    "  Item[{}].next should be {}, but it is {} now.",
                    index,
                    util::hex_string(next_item.account_id().raw_data()),
                    util::hex_string(item.next().raw_data())
                );
            }

            // Check if there is any account ID duplicate in the previous slices.
            for exist_account_id in account_id_list.iter() {
                assert!(
                    &account_id != exist_account_id,
                    Error::ProposalSliceItemMustBeUniqueAccount,
                    "Every item in slice should be unique, but item[0x{}] is found twice.",
                    util::hex_string(&account_id)
                )
            }

            // Store account IDs for order verification.
            account_id_list.push(account_id.clone());
            account_id_list_with_next.push(account_id);
            required_cells_count += 1;
        }

        account_id_list_with_next.push(last_item_next)
    }

    // Check if there is any next(it is account ID either) exist in the account_id_list.
    for next in exist_next_list.iter() {
        for account_id in account_id_list.iter() {
            assert!(
                next != account_id,
                Error::ProposalSliceItemMustBeUniqueAccount,
                "The next of any exist AccountCell should not be contained in the slices as an item, but the item[{}] is found.",
                util::hex_string(&next)
            )
        }
    }

    // Check the order of items in the slice.
    let sorted_account_id_list = DasSortedList::new(account_id_list_with_next.clone());
    assert!(
        sorted_account_id_list.cmp_order_with(&account_id_list_with_next),
        Error::ProposalSliceIsNotSorted,
        "The order of items in slices is incorrect."
    );

    let max_account_cell_count = u32::from(config.proposal_max_account_affect());
    assert!(
        account_cell_contained <= max_account_cell_count,
        Error::InvalidTransactionStructure,
        "The proposal should not contains more than {} AccountCells.",
        max_account_cell_count
    );

    let max_pre_account_cell_count = u32::from(config.proposal_max_pre_account_contain());
    assert!(
        pre_account_cell_contained <= max_pre_account_cell_count,
        Error::InvalidTransactionStructure,
        "The proposal should not contains more than {} PreAccountCells.",
        max_pre_account_cell_count
    );

    Ok(required_cells_count)
}

fn find_proposal_related_cells(config: ConfigCellMainReader, source: Source) -> Result<Vec<usize>, Error> {
    // Find related cells' indexes in cell_deps or inputs.
    let account_cell_type_id = config.type_id_table().account_cell();
    let account_cells = util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, source)?;
    let pre_account_cell_type_id = config.type_id_table().pre_account_cell();
    let pre_account_cells = util::find_cells_by_type_id(ScriptType::Type, pre_account_cell_type_id, source)?;

    assert!(
        pre_account_cells.len() > 0,
        Error::InvalidTransactionStructure,
        "There should be some PreAccountCells in {:?}.",
        source
    );

    // Merge cells' indexes in sorted order.
    let mut sorted = Vec::new();
    if account_cells.len() > 0 {
        let mut i = 0;
        let mut j = 0;
        let remain;
        let remain_idx;
        loop {
            if account_cells[i] < pre_account_cells[j] {
                sorted.push(account_cells[i]);
                i += 1;
                if i == account_cells.len() {
                    remain = pre_account_cells;
                    remain_idx = j;
                    break;
                }
            } else {
                sorted.push(pre_account_cells[j]);
                j += 1;
                if j == pre_account_cells.len() {
                    remain = account_cells;
                    remain_idx = i;
                    break;
                }
            }
        }

        for i in remain_idx..remain.len() {
            sorted.push(remain[i]);
        }
    } else {
        // The PreAccountCells in inputs is already sorted by their indexes, so no need to sort again.
        sorted = pre_account_cells;
    }

    debug!(
        "Inputs cells(AccountCell/PreAccountCell) sorted index list: {:?}",
        sorted
    );

    Ok(sorted)
}

fn find_output_account_cells(config: ConfigCellMainReader) -> Result<Vec<usize>, Error> {
    // Find updated cells' indexes in outputs.
    let account_cell_type_id = config.type_id_table().account_cell();
    let mut account_cells = util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, Source::Output)?;
    account_cells.sort();

    assert!(
        account_cells.len() > 0,
        Error::InvalidTransactionStructure,
        "There should be some AccountCells in the outputs."
    );

    debug!("Outputs cells(AccountCell) sorted index list: {:?}", account_cells);

    Ok(account_cells)
}

fn verify_slices_relevant_cells(
    parser: &WitnessesParser,
    timestamp: u64,
    config: ConfigCellMainReader,
    slices_reader: SliceListReader,
    relevant_cells: Vec<usize>,
    prev_slices_reader_opt: Option<SliceListReader>,
) -> Result<(), Error> {
    debug!("Check the proposal slices relevant cells are real exist and in correct status.");

    let mut i = 0;
    for (_sl_index, sl_reader) in slices_reader.iter().enumerate() {
        debug!("Check slice {} ...", _sl_index);
        let mut next_of_first_cell = AccountId::default();
        for (item_index, item) in sl_reader.iter().enumerate() {
            let item_account_id = item.account_id();
            let item_type = u8::from(item.item_type());

            let cell_index = relevant_cells[i];

            // Check if the relevant cells has the same type as in the proposal.
            let cell_data = util::load_cell_data(cell_index, Source::CellDep)?;
            if item_type == ProposalSliceItemType::Exist as u8 {
                let expected_type_id = config.type_id_table().account_cell();
                verify_cell_type_id(item_index, cell_index, Source::CellDep, &expected_type_id)?;

                // Check if the relevant cells have the same account ID as in the proposal.
                verify_account_cell_account_id(
                    item_index,
                    &cell_data,
                    cell_index,
                    Source::CellDep,
                    item_account_id.raw_data(),
                )?;
            } else {
                let expected_type_id = config.type_id_table().pre_account_cell();
                verify_cell_type_id(item_index, cell_index, Source::CellDep, &expected_type_id)?;

                // Check if the relevant cells have the same account ID as in the proposal.
                verify_pre_account_cell_account_id(
                    item_index,
                    &cell_data,
                    cell_index,
                    Source::CellDep,
                    item_account_id.raw_data(),
                )?;

                let pre_account_cell_witness =
                    util::parse_pre_account_cell_witness(&parser, cell_index, Source::CellDep)?;
                let pre_account_cell_witness_reader = pre_account_cell_witness.as_reader();

                // For protecting register, do not allow PreAccountCell exists more than a week to be confirmed.
                let created_at = u64::from(pre_account_cell_witness_reader.created_at());
                assert!(
                    timestamp <= created_at + PRE_ACCOUNT_CELL_TIMEOUT,
                    Error::ProposalConfirmPreAccountCellExpired,
                    "The PreAccountCell has been expired.(created_at: {}, expired_at: {})",
                    created_at,
                    created_at + PRE_ACCOUNT_CELL_TIMEOUT
                );
            };

            // ⚠️ The first item is very very important, its "next" must be correct so that
            // AccountCells can form a linked list.
            if item_index == 0 {
                // If this is the first proposal in proposal chain, all slice must start with an AccountCell.
                if prev_slices_reader_opt.is_none() {
                    assert!(
                        item_type == ProposalSliceItemType::Exist as u8,
                        Error::ProposalSliceMustStartWithAccountCell,
                        "  In the first proposal of a proposal chain, all slice should start with an AccountCell."
                    );

                    // The correct "next" of first proposal is come from the cell's outputs_data.
                    next_of_first_cell =
                        AccountId::try_from(data_parser::account_cell::get_next(&cell_data)).map_err(|_| Error::InvalidCellData)?;

                // If this is the extended proposal in proposal chain, slice may starting with an
                // AccountCell/PreAccountCell included in previous proposal, or it may starting with
                // an AccountCell not included in previous proposal.
                } else {
                    assert!(
                        item_type == ProposalSliceItemType::Exist as u8 || item_type == ProposalSliceItemType::Proposed as u8,
                        Error::ProposalSliceMustStartWithAccountCell,
                        "  In the extended proposal of a proposal chain, slices should start with an AccountCell or a PreAccountCell which included in previous proposal."
                    );

                    let prev_slices_reader = prev_slices_reader_opt.as_ref().unwrap();
                    next_of_first_cell = match find_item_contains_account_id(prev_slices_reader, &item_account_id) {
                        // If the item is included in previous proposal, then we need to get its latest "next" from the proposal.
                        Ok(prev_item) => prev_item.next(),
                        // If the item is not included in previous proposal, then we get its latest "next" from the cell's outputs_data.
                        Err(_) => AccountId::try_from(data_parser::account_cell::get_next(&cell_data))
                            .map_err(|_| Error::InvalidCellData)?,
                    };
                }
            }

            i += 1;
        }

        // Check if the first item's "next" has pass to the last item correctly.
        let item = sl_reader.get(sl_reader.len() - 1).unwrap();
        let next_of_last_item = item.next();

        assert!(
            util::is_reader_eq(next_of_first_cell.as_reader(), next_of_last_item),
            Error::ProposalSliceNotEndCorrectly,
            "The next of first item should be pass to the last item correctly."
        );
    }

    Ok(())
}

fn find_item_contains_account_id(
    prev_slices_reader: &SliceListReader,
    account_id: &AccountIdReader,
) -> Result<ProposalItem, Error> {
    for slice in prev_slices_reader.iter() {
        for item in slice.iter() {
            if util::is_reader_eq(item.account_id(), *account_id) {
                return Ok(item.to_entity());
            }
        }
    }

    debug!("Can not find previous item: {}", account_id);
    Err(Error::PrevProposalItemNotFound)
}

fn verify_proposal_execution_result(
    parser: &WitnessesParser,
    config_account: ConfigCellAccountReader,
    config_main: ConfigCellMainReader,
    config_profit_rate: ConfigCellProfitRateReader,
    timestamp: u64,
    proposal_cell_data_reader: ProposalCellDataReader,
) -> Result<(), Error> {
    debug!("Check that all AccountCells/PreAccountCells have been converted according to the proposal.");

    #[cfg(debug_assertions)]
    inspect_slices(proposal_cell_data_reader.slices())?;

    let das_wallet_lock = das_wallet_lock();
    let proposer_lock_reader = proposal_cell_data_reader.proposer_lock();
    let slices_reader = proposal_cell_data_reader.slices();

    let account_cell_type_id = config_main.type_id_table().account_cell();
    let pre_account_cell_type_id = config_main.type_id_table().pre_account_cell();
    let input_related_cells = find_proposal_related_cells(config_main, Source::Input)?;
    let output_account_cells = find_output_account_cells(config_main)?;

    #[cfg(debug_assertions)]
    inspect_related_cells(&parser, config_main, input_related_cells.clone(), Source::Input)?;
    #[cfg(debug_assertions)]
    inspect_related_cells(&parser, config_main, output_account_cells.clone(), Source::Output)?;

    let mut profit_map = Map::new();
    let inviter_profit_rate = u32::from(config_profit_rate.inviter()) as u64;
    let channel_profit_rate = u32::from(config_profit_rate.channel()) as u64;
    let proposal_create_profit_rate = u32::from(config_profit_rate.proposal_create()) as u64;
    let proposal_confirm_profit_rate = u32::from(config_profit_rate.proposal_confirm()) as u64;

    let default_lock = Script::default();
    let default_lock_reader = default_lock.as_reader();

    let mut i = 0;
    for (_sl_index, sl_reader) in slices_reader.iter().enumerate() {
        debug!("Check Slice[{}] ...", _sl_index);

        let last_item = sl_reader.get(sl_reader.len() - 1).unwrap();
        let original_next_of_account_cell = last_item.next().raw_data();

        for (item_index, item) in sl_reader.iter().enumerate() {
            let item_account_id = item.account_id().raw_data();
            let item_type = u8::from(item.item_type());
            let item_next = item.next();

            let input_cell_data = util::load_cell_data(input_related_cells[i], Source::Input)?;
            let output_cell_data = util::load_cell_data(output_account_cells[i], Source::Output)?;

            if item_type == ProposalSliceItemType::Exist as u8 || item_type == ProposalSliceItemType::Proposed as u8 {
                debug!(
                    "  Item[{}] Check that the existing inputs[{}].AccountCell and outputs[{}].AccountCell is updated correctly.",
                    item_index, input_related_cells[i], output_account_cells[i]
                );

                // All cells' type is must be account-cell-type
                verify_cell_type_id(item_index, input_related_cells[i], Source::Input, &account_cell_type_id)?;
                verify_cell_type_id(
                    item_index,
                    output_account_cells[i],
                    Source::Output,
                    &account_cell_type_id,
                )?;

                verify_next_is_consistent_in_account_cell_and_item(
                    item_index,
                    input_related_cells[i],
                    &input_cell_data,
                    original_next_of_account_cell,
                )?;

                // All cells' account_id in data must be the same as the account_id in proposal.
                verify_account_cell_account_id(
                    item_index,
                    &input_cell_data,
                    input_related_cells[i],
                    Source::Input,
                    item_account_id,
                )?;
                verify_account_cell_account_id(
                    item_index,
                    &output_cell_data,
                    output_account_cells[i],
                    Source::Output,
                    item_account_id,
                )?;

                util::is_cell_capacity_equal(
                    (input_related_cells[i], Source::Input),
                    (output_account_cells[i], Source::Output),
                )?;
                util::is_cell_lock_equal(
                    (input_related_cells[i], Source::Input),
                    (output_account_cells[i], Source::Output),
                )?;

                // For the existing AccountCell, only the next field in data can be modified.
                // No need to check the witness of AccountCells here, because we check their hash instead.
                is_old_account_cell_data_consistent(item_index, &output_cell_data, &input_cell_data)?;
                is_next_correct(item_index, &output_cell_data, item_next)?;

                let input_cell_witness: Box<dyn AccountCellDataMixer> =
                    util::parse_account_cell_witness(&parser, input_related_cells[i], Source::Input)?;
                let input_cell_witness_reader = input_cell_witness.as_reader();

                let output_cell_witness: Box<dyn AccountCellDataMixer> =
                    util::parse_account_cell_witness(&parser, output_account_cells[i], Source::Output)?;
                let output_cell_witness_reader = output_cell_witness.as_reader();

                verifiers::account_cell::verify_account_witness_consistent(
                    input_related_cells[i],
                    output_account_cells[i],
                    &input_cell_witness_reader,
                    &output_cell_witness_reader,
                    vec![""],
                )?;
            } else {
                debug!(
                    "  Item[{}] Check that the inputs[{}].PreAccountCell and outputs[{}].AccountCell is converted correctly.",
                    item_index, input_related_cells[i], output_account_cells[i]
                );

                // All cells' type is must be pre-account-cell-type/account-cell-type
                verify_cell_type_id(
                    item_index,
                    input_related_cells[i],
                    Source::Input,
                    &pre_account_cell_type_id,
                )?;
                verify_cell_type_id(
                    item_index,
                    output_account_cells[i],
                    Source::Output,
                    &account_cell_type_id,
                )?;

                // All cells' account_id in data must be the same as the account_id in proposal.
                verify_pre_account_cell_account_id(
                    item_index,
                    &input_cell_data,
                    input_related_cells[i],
                    Source::Input,
                    item_account_id,
                )?;
                verify_account_cell_account_id(
                    item_index,
                    &output_cell_data,
                    output_account_cells[i],
                    Source::Output,
                    item_account_id,
                )?;

                let input_cell_witness =
                    util::parse_pre_account_cell_witness(&parser, input_related_cells[i], Source::Input)?;
                let input_cell_witness_reader = input_cell_witness.as_reader();

                let output_cell_witness =
                    util::parse_account_cell_witness(&parser, output_account_cells[i], Source::Output)?;
                let output_cell_witness_reader = if let Ok(reader) = output_cell_witness.as_reader().try_into_latest() {
                    reader
                } else {
                    warn!(
                        "  Item[{}] The AccouneCell in outputs is required to be latest data structure.",
                        item_index
                    );
                    return Err(Error::InvalidTransactionStructure);
                };

                let account_name_storage = data_parser::account_cell::get_account(&output_cell_data).len() as u64;
                let total_capacity = load_cell_capacity(input_related_cells[i], Source::Input)?;

                let lock = high_level::load_cell_lock(output_account_cells[i], Source::Output)?;
                let storage_capacity = util::calc_account_storage_capacity(
                    config_account,
                    account_name_storage,
                    lock.args().as_reader().into(),
                );
                // Allocate the profits carried by PreAccountCell to the wallets for later verification.
                let profit = total_capacity - storage_capacity;

                debug!(
                    "  Item[{}] The profit in PreAccountCell is: {}(profit) = {}(total_capacity) - {}(storage_capacity)",
                    item_index, profit, total_capacity, storage_capacity
                );

                is_cell_capacity_correct(item_index, output_account_cells[i], storage_capacity)?;
                is_new_account_cell_lock_correct(
                    item_index,
                    input_related_cells[i],
                    &input_cell_witness_reader,
                    output_account_cells[i],
                )?;

                // Check all fields in the data of new AccountCell.
                is_id_correct(item_index, &output_cell_data, &input_cell_data)?;
                is_account_correct(item_index, &output_cell_data)?;
                is_next_correct(item_index, &output_cell_data, item_next)?;
                is_expired_at_correct(
                    item_index,
                    profit,
                    timestamp,
                    &output_cell_data,
                    &input_cell_witness_reader,
                )?;

                // Check all fields in the witness of new AccountCell.
                verify_witness_id(item_index, &output_cell_data, output_cell_witness_reader)?;
                verify_witness_account(item_index, &output_cell_data, output_cell_witness_reader)?;
                verify_witness_registered_at(item_index, timestamp, output_cell_witness_reader)?;
                verify_witness_throttle_fields(item_index, output_cell_witness_reader)?;
                verify_witness_sub_account_fields(item_index, output_cell_witness_reader)?;
                verify_witness_initial_records(item_index, &input_cell_witness_reader, output_cell_witness_reader)?;
                verify_witness_initial_cross_chain_and_status(item_index, &input_cell_witness_reader, output_cell_witness_reader)?;

                let mut inviter_profit = 0;
                if input_cell_witness_reader.inviter_lock().is_some() {
                    let inviter_lock_reader = input_cell_witness_reader.inviter_lock().to_opt().unwrap();
                    // Skip default value for supporting transactions treat default value as None.
                    if !util::is_reader_eq(default_lock_reader, inviter_lock_reader) {
                        inviter_profit = profit * inviter_profit_rate / RATE_BASE;
                        debug!(
                            "  Item[{}] lock.args[{}]: {}(inviter_profit) = {}(profit) * {}(inviter_profit_rate) / {}(RATE_BASE)",
                            item_index, inviter_lock_reader.args(), inviter_profit, profit, inviter_profit_rate, RATE_BASE
                        );
                        map_util::add(&mut profit_map, inviter_lock_reader.as_slice().to_vec(), inviter_profit);
                    }
                };

                let mut channel_profit = 0;
                if input_cell_witness_reader.channel_lock().is_some() {
                    let channel_lock_reader = input_cell_witness_reader.channel_lock().to_opt().unwrap();
                    // Skip default value for supporting transactions treat default value as None.
                    if !util::is_reader_eq(default_lock_reader, channel_lock_reader) {
                        channel_profit = profit * channel_profit_rate / RATE_BASE;
                        debug!(
                            "  Item[{}] lock.args[{}]: {}(channel_profit) = {}(profit) * {}(channel_profit_rate) / {}(RATE_BASE)",
                            item_index, channel_lock_reader.args(), channel_profit, profit, channel_profit_rate, RATE_BASE
                        );
                        map_util::add(&mut profit_map, channel_lock_reader.as_slice().to_vec(), channel_profit);
                    }
                };

                let proposal_create_profit = profit * proposal_create_profit_rate / RATE_BASE;
                debug!(
                    "  Item[{}] lock.args[{}]: {}(proposal_create_profit) = {}(profit) * {}(proposal_create_profit_rate) / {}(RATE_BASE)",
                    item_index,
                    proposer_lock_reader.args(),
                    proposal_create_profit,
                    profit,
                    proposal_create_profit_rate,
                    RATE_BASE
                );
                map_util::add(
                    &mut profit_map,
                    proposer_lock_reader.as_slice().to_vec(),
                    proposal_create_profit,
                );

                let proposal_confirm_profit = profit * proposal_confirm_profit_rate / RATE_BASE;
                debug!(
                    "  Item[{}] {}(proposal_confirm_profit) = {}(profit) * {}(proposal_confirm_profit_rate) / {}(RATE_BASE) (! not included in IncomeCell)",
                    item_index, proposal_confirm_profit, profit, proposal_confirm_profit_rate, RATE_BASE
                );
                // No need to record proposal confirm profit, bacause the transaction creator can take its profit freely and this script do not know which lock script the transaction creator will use.

                let das_profit =
                    profit - inviter_profit - channel_profit - proposal_create_profit - proposal_confirm_profit;
                map_util::add(
                    &mut profit_map,
                    das_wallet_lock.as_reader().as_slice().to_vec(),
                    das_profit,
                );

                debug!(
                    "  Item[{}] lock.args[{}]: {}(das_profit) = {}(profit) - {}(inviter_profit) - {}(channel_profit) - {}(proposal_create_profit) - {}(proposal_confirm_profit)",
                    item_index, das_wallet_lock.as_reader().args(), das_profit, profit, inviter_profit, channel_profit, proposal_create_profit, proposal_confirm_profit
                );
            }

            i += 1;
        }
    }

    verifiers::income_cell::verify_income_cells(&parser, profit_map)?;

    Ok(())
}

fn verify_cell_type_id(
    item_index: usize,
    cell_index: usize,
    source: Source,
    expected_type_id: &HashReader,
) -> Result<(), Error> {
    let cell_type_id = load_cell_type(cell_index, source)?
        .map(|script| script.code_hash())
        .ok_or(Error::ProposalSliceRelatedCellNotFound)?;

    assert!(
        cell_type_id.as_reader().raw_data() == expected_type_id.raw_data(),
        Error::ProposalCellTypeError,
        "  The type ID of Item[{}] should be {}. (related_cell: {:?}[{}])",
        item_index,
        expected_type_id,
        source,
        cell_index
    );

    Ok(())
}

fn verify_next_is_consistent_in_account_cell_and_item(
    item_index: usize,
    cell_index: usize,
    cell_data: &[u8],
    original_next_of_account_cell: &[u8],
) -> Result<(), Error> {
    let next = data_parser::account_cell::get_next(cell_data);

    assert!(
        next == original_next_of_account_cell,
        Error::ProposalCellNextError,
        "  The next of Item[{}] should be {}, but it has changed. (related_cell: {:?}[{}])",
        item_index,
        util::hex_string(original_next_of_account_cell),
        Source::Input,
        cell_index
    );

    Ok(())
}

fn verify_account_cell_account_id(
    item_index: usize,
    cell_data: &[u8],
    cell_index: usize,
    source: Source,
    expected_account_id: &[u8],
) -> Result<(), Error> {
    let account_id = data_parser::account_cell::get_id(cell_data);

    assert!(
        account_id == expected_account_id,
        Error::ProposalCellAccountIdError,
        "  The account ID of Item[{}] should be {}. (related_cell: {:?}[{}])",
        item_index,
        util::hex_string(expected_account_id),
        source,
        cell_index
    );

    Ok(())
}

fn verify_pre_account_cell_account_id(
    item_index: usize,
    cell_data: &[u8],
    cell_index: usize,
    source: Source,
    expected_account_id: &[u8],
) -> Result<(), Error> {
    let account_id = data_parser::pre_account_cell::get_id(cell_data);

    assert!(
        account_id == expected_account_id,
        Error::ProposalCellAccountIdError,
        "  The account ID of Item[{}] should be {}. (related_cell: {:?}[{}])",
        item_index,
        util::hex_string(expected_account_id),
        source,
        cell_index
    );

    Ok(())
}

fn is_new_account_cell_lock_correct<'a>(
    item_index: usize,
    input_cell_index: usize,
    input_cell_witness_reader: &Box<dyn PreAccountCellDataReaderMixer + 'a>,
    output_cell_index: usize,
) -> Result<(), Error> {
    debug!(
        "  Item[{}] Check if the lock script of new AccountCells is das-lock.",
        item_index
    );

    let das_lock = das_lock();
    let owner_lock_args = input_cell_witness_reader.owner_lock_args().raw_data().to_owned();
    let output_cell_lock = load_cell_lock(output_cell_index, Source::Output)?;

    let expected_lock = das_lock.as_builder().args(Bytes::from(owner_lock_args).into()).build();

    assert!(
        util::is_entity_eq(&expected_lock, &output_cell_lock),
        Error::ProposalConfirmAccountLockArgsIsInvalid,
        "  Item[{}] The outputs[{}].lock should come from the owner_lock_args of inputs[{}]. (expected: {}, current: {})",
        item_index,
        output_cell_index,
        input_cell_index,
        expected_lock,
        output_cell_lock
    );

    Ok(())
}

fn is_bytes_eq(
    item_index: usize,
    field: &str,
    current_bytes: &[u8],
    expected_bytes: &[u8],
    error_code: Error,
) -> Result<(), Error> {
    assert!(
        current_bytes == expected_bytes,
        error_code,
        "  Item[{}] The AccountCell.{} should be consist in inputs and outputs.(expected: {}, current: {})",
        item_index,
        field,
        util::hex_string(expected_bytes),
        util::hex_string(current_bytes)
    );

    Ok(())
}

fn is_old_account_cell_data_consistent(
    item_index: usize,
    output_cell_data: &Vec<u8>,
    input_cell_data: &Vec<u8>,
) -> Result<(), Error> {
    is_bytes_eq(
        item_index,
        "id",
        data_parser::account_cell::get_id(output_cell_data),
        data_parser::account_cell::get_id(input_cell_data),
        Error::ProposalFieldCanNotBeModified,
    )?;
    is_bytes_eq(
        item_index,
        "account",
        data_parser::account_cell::get_account(output_cell_data),
        data_parser::account_cell::get_account(input_cell_data),
        Error::ProposalFieldCanNotBeModified,
    )?;
    is_bytes_eq(
        item_index,
        "expired_at",
        &data_parser::account_cell::get_expired_at(output_cell_data).to_le_bytes(),
        &data_parser::account_cell::get_expired_at(input_cell_data).to_le_bytes(),
        Error::ProposalFieldCanNotBeModified,
    )?;

    Ok(())
}

fn is_id_correct(item_index: usize, output_cell_data: &Vec<u8>, input_cell_data: &Vec<u8>) -> Result<(), Error> {
    is_bytes_eq(
        item_index,
        "id",
        data_parser::account_cell::get_id(output_cell_data),
        data_parser::account_cell::get_id(input_cell_data),
        Error::ProposalConfirmNewAccountCellDataError,
    )
}

fn is_next_correct(item_index: usize, output_cell_data: &Vec<u8>, proposed_next: AccountIdReader) -> Result<(), Error> {
    let expected_next = proposed_next.raw_data();

    is_bytes_eq(
        item_index,
        "next",
        data_parser::account_cell::get_next(output_cell_data),
        expected_next,
        Error::ProposalConfirmNewAccountCellDataError,
    )
}

fn is_expired_at_correct<'a>(
    item_index: usize,
    profit: u64,
    current_timestamp: u64,
    output_cell_data: &Vec<u8>,
    pre_account_cell_witness: &Box<dyn PreAccountCellDataReaderMixer + 'a>,
) -> Result<(), Error> {
    let price = u64::from(pre_account_cell_witness.price().new());
    let quote = u64::from(pre_account_cell_witness.quote());
    let discount = u32::from(pre_account_cell_witness.invited_discount());
    let duration = util::calc_duration_from_paid(profit, price, quote, discount);
    let expired_at = data_parser::account_cell::get_expired_at(output_cell_data);
    let calculated_expired_at = current_timestamp + duration;

    debug!(
        "  Item[{}] Params of expired_at calculation: --profit={} --price={} --quote={} --discount={} --current={}",
        item_index, profit, price, quote, discount, current_timestamp
    );
    debug!(
        "  Item[{}] Critical value of expired_at calculation process: duration={}, calculated_expired_at={}",
        item_index, duration, calculated_expired_at
    );

    assert!(
        calculated_expired_at == expired_at,
        Error::ProposalConfirmNewAccountCellDataError,
        "  Item[{}] The AccountCell.expired_at should be {}, but {} found.",
        item_index,
        calculated_expired_at,
        expired_at
    );

    Ok(())
}

fn is_account_correct(item_index: usize, output_cell_data: &Vec<u8>) -> Result<(), Error> {
    let expected_account_id = data_parser::account_cell::get_id(output_cell_data);
    let account = data_parser::account_cell::get_account(output_cell_data);

    let hash = util::blake2b_256(account);
    let account_id = hash.get(..ACCOUNT_ID_LENGTH).unwrap();

    is_bytes_eq(
        item_index,
        "account",
        account_id,
        expected_account_id,
        Error::ProposalConfirmNewAccountCellDataError,
    )
}

fn is_cell_capacity_correct(item_index: usize, cell_index: usize, expected_capacity: u64) -> Result<(), Error> {
    let cell_capacity = load_cell_capacity(cell_index, Source::Output)?;

    assert!(
        expected_capacity == cell_capacity,
        Error::ProposalConfirmNewAccountCellCapacityError,
        "  Item[{}] The AccountCell.capacity should be {}, but {} found.",
        item_index,
        expected_capacity,
        cell_capacity
    );

    Ok(())
}

fn verify_witness_id(
    item_index: usize,
    output_cell_data: &Vec<u8>,
    output_cell_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    let account_id = output_cell_witness_reader.id().raw_data();
    let expected_account_id = data_parser::account_cell::get_id(output_cell_data);

    is_bytes_eq(
        item_index,
        "witness.id",
        account_id,
        expected_account_id,
        Error::ProposalConfirmNewAccountWitnessError,
    )
}

fn verify_witness_account(
    item_index: usize,
    output_cell_data: &Vec<u8>,
    output_cell_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    let mut account = output_cell_witness_reader.account().as_readable();
    account.append(&mut ACCOUNT_SUFFIX.as_bytes().to_vec());
    let expected_account = data_parser::account_cell::get_account(output_cell_data);

    is_bytes_eq(
        item_index,
        "witness.account",
        account.as_slice(),
        expected_account,
        Error::ProposalConfirmNewAccountWitnessError,
    )
}

fn verify_witness_registered_at(
    item_index: usize,
    timestamp: u64,
    output_cell_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    let registered_at = u64::from(output_cell_witness_reader.registered_at());

    assert!(
        registered_at == timestamp,
        Error::ProposalConfirmNewAccountWitnessError,
        "  Item[{}] The AccountCell.registered_at should be the same as the timestamp in TimeCell.(expected: {}, current: {})",
        item_index,
        timestamp,
        registered_at
    );

    Ok(())
}

fn verify_witness_throttle_fields(
    item_index: usize,
    output_cell_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    let last_transfer_account_at = u64::from(output_cell_witness_reader.last_transfer_account_at());
    let last_edit_manager_at = u64::from(output_cell_witness_reader.last_edit_manager_at());
    let last_edit_records_at = u64::from(output_cell_witness_reader.last_edit_records_at());

    assert!(
        last_transfer_account_at == 0 && last_edit_manager_at == 0 && last_edit_records_at == 0,
        Error::ProposalConfirmNewAccountWitnessError,
        "  Item[{}] The AccountCell.last_transfer_account_at/last_edit_manager_at/last_edit_records_at should be 0 .",
        item_index
    );

    Ok(())
}

fn verify_witness_sub_account_fields(
    item_index: usize,
    output_cell_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    let enable_sub_account = u8::from(output_cell_witness_reader.enable_sub_account());
    let renew_sub_account_price = u64::from(output_cell_witness_reader.renew_sub_account_price());

    assert!(
        enable_sub_account == SubAccountEnableStatus::Off as u8,
        Error::ProposalConfirmNewAccountWitnessError,
        "  Item[{}] The AccountCell.enable_sub_account should be off. (expected: 0, current: {})",
        item_index,
        enable_sub_account
    );

    assert!(
        renew_sub_account_price == 0,
        Error::ProposalConfirmNewAccountWitnessError,
        "  Item[{}] The AccountCell.renew_sub_account_price should be 0. (expected: 0, current: {})",
        item_index,
        renew_sub_account_price
    );

    Ok(())
}

fn verify_witness_initial_records<'a>(
    item_index: usize,
    pre_account_cell_reader: &Box<dyn PreAccountCellDataReaderMixer + 'a>,
    account_cell_reader: AccountCellDataReader,
) -> Result<(), Error> {
    let current_records = account_cell_reader.records();
    if pre_account_cell_reader.version() >= 2 {
        debug!(
            "  Item[{}] The PreAccountCell's version is >= 2, start verifying the field initial_records.",
            item_index
        );

        let expected_records ;
        if let Ok(reader) = pre_account_cell_reader.try_into_latest() {
            expected_records = reader.initial_records();
        } else if let Ok(reader) = pre_account_cell_reader.try_into_v2() {
            expected_records = reader.initial_records();
        } else {
            warn!("  Item[{}] Some version of PreAccountCell is unhandled. It is required to verify the field initial_records.", item_index);
            return Err(Error::HardCodedError);
        }

        assert!(
            util::is_reader_eq(expected_records, current_records),
            Error::ProposalConfirmInitialRecordsMismatch,
            "  Item[{}] The AccountCell.records should be the same as the PreAccountCell.initial_records . (expected: {}, current: {})",
            item_index,
            expected_records.as_prettier(),
            current_records.as_prettier()
        );
    } else {
        debug!(
            "  Item[{}] The PreAccountCell's version is < 2, start verifying if the new AccountCell.records is empty.",
            item_index
        );

        assert!(
            current_records.is_empty(),
            Error::ProposalConfirmInitialRecordsMismatch,
            "  Item[{}] The AccountCell.records should be empty.",
            item_index
        );
    }

    Ok(())
}

fn verify_witness_initial_cross_chain_and_status<'a>(
    item_index: usize,
    pre_account_cell_reader: &Box<dyn PreAccountCellDataReaderMixer + 'a>,
    account_cell_reader: AccountCellDataReader,
) -> Result<(), Error> {
    let status = u8::from(account_cell_reader.status());
    if pre_account_cell_reader.version() >= 3 {
        debug!(
            "  Item[{}] The PreAccountCell's version is >= 3, start verifying the field initial_cross_chain.",
            item_index
        );

        let checked ;
        if let Ok(reader) = pre_account_cell_reader.try_into_latest() {
            checked = u8::from(reader.initial_cross_chain().checked());
        } else {
            warn!("  Item[{}] Some version of PreAccountCell is unhandled. It is required to verify the field initial_cross_chain.", item_index);
            return Err(Error::HardCodedError);
        }

        if checked == 1 {
            assert!(
                status == AccountStatus::LockedForCrossChain as u8,
                Error::ProposalConfirmNewAccountWitnessError,
                "  Item[{}] The AccountCell.status should be LockedForCrossChain in outputs. (expected: {:?}, current: {})",
                item_index,
                AccountStatus::LockedForCrossChain as u8,
                status
            );
        } else {
            assert!(
                status == AccountStatus::Normal as u8,
                Error::ProposalConfirmNewAccountWitnessError,
                "  Item[{}] The AccountCell.status should be Normal in outputs. (expected: {:?}, current: {})",
                item_index,
                AccountStatus::Normal as u8,
                status
            );
        }
    } else {
        debug!(
            "  Item[{}] The PreAccountCell's version is <= 2, start verifying if the new AccountCell.status is Normal.",
            item_index
        );

        assert!(
            status == AccountStatus::Normal as u8,
            Error::ProposalConfirmNewAccountWitnessError,
            "  Item[{}] The AccountCell.status should be Normal in outputs. (expected: {:?}, current: {})",
            item_index,
            AccountStatus::Normal as u8,
            status
        );
    }

    Ok(())
}

fn verify_refund_correct(
    proposal_cell_index: usize,
    proposal_cell_data_reader: ProposalCellDataReader,
    available_for_fee: u64,
) -> Result<(), Error> {
    debug!("Check if the refund amount to proposer_lock is correct.");

    let proposer_lock = proposal_cell_data_reader.proposer_lock();
    let refund_cells = util::find_cells_by_script(ScriptType::Lock, proposer_lock.into(), Source::Output)?;

    assert!(
        refund_cells.len() >= 1,
        Error::ProposalConfirmRefundError,
        "There should be at least 1 cell in outputs with the lock of the proposer. (expected_lock: {})",
        proposer_lock
    );

    let mut refund_capacity = 0;
    for index in refund_cells {
        refund_capacity += load_cell_capacity(index, Source::Output)?;
    }

    let proposal_capacity = load_cell_capacity(proposal_cell_index.to_owned(), Source::Input)?;
    assert!(
        proposal_capacity <= refund_capacity + available_for_fee,
        Error::ProposalConfirmRefundError,
        "There refund of proposer should be at least {}, but {} found.",
        proposal_capacity - available_for_fee,
        refund_capacity
    );

    Ok(())
}
