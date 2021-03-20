use alloc::borrow::ToOwned;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::bytes,
    high_level::{load_cell_capacity, load_cell_data, load_cell_type, load_script},
};
use core::convert::TryFrom;
use core::result::Result;
use das_core::{
    account_cell_parser::{get_account, get_expired_at, get_id, get_next},
    constants::*,
    debug,
    error::Error,
    util,
    witness_parser::WitnessesParser,
};
use das_sorted_list::DasSortedList;
use das_types::{constants::*, packed::*, prelude::*};
use das_wallet::Wallet;

pub fn main() -> Result<(), Error> {
    debug!("====== Running proposal-cell-type ======");

    debug!("Find out ProposalCell ...");

    // Find out PreAccountCells in current transaction.
    let this_type_script = load_script().map_err(|e| Error::from(e))?;
    let input_cells =
        util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
    let output_cells =
        util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;
    let dep_cells =
        util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::CellDep)?;

    let action_data = util::load_das_action()?;
    let action = action_data.as_reader().action().raw_data();
    if action == b"propose" {
        debug!("Route to propose action ...");

        let mut parser = util::load_das_witnesses(None)?;
        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellMain])?;
        let config_main = parser.configs().main()?;

        if dep_cells.len() != 0 || input_cells.len() != 0 || output_cells.len() != 1 {
            return Err(Error::ProposalFoundInvalidTransaction);
        }

        // Read outputs_data and witness of the ProposalCell.
        let index = &output_cells[0];
        let (_, _, entity) = parser.verify_and_get(index.to_owned(), Source::Output)?;
        let proposal_cell_data = ProposalCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let proposal_cell_data_reader = proposal_cell_data.as_reader();

        let required_cells_count = verify_slices(proposal_cell_data_reader.slices())?;
        let dep_related_cells = find_proposal_related_cells(config_main, Source::CellDep)?;

        #[cfg(not(feature = "mainnet"))]
        inspect_slices(proposal_cell_data_reader.slices())?;
        #[cfg(not(feature = "mainnet"))]
        inspect_related_cells(
            &parser,
            config_main,
            dep_related_cells.clone(),
            Source::CellDep,
            None,
        )?;

        if required_cells_count != dep_related_cells.len() {
            return Err(Error::ProposalSliceRelatedCellMissing);
        }

        verify_slices_relevant_cells(
            config_main,
            proposal_cell_data_reader.slices(),
            dep_related_cells,
            None,
        )?;
    } else if action == b"extend_proposal" {
        debug!("Route to extend_proposal action ...");

        let mut parser = util::load_das_witnesses(None)?;
        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellMain])?;
        let config_main = parser.configs().main()?;

        if dep_cells.len() != 1 || input_cells.len() != 0 || output_cells.len() != 1 {
            return Err(Error::ProposalFoundInvalidTransaction);
        }

        // Read outputs_data and witness of previous ProposalCell.
        let index = &dep_cells[0];
        let (_, _, entity) = parser.verify_and_get(index.to_owned(), Source::CellDep)?;
        let prev_proposal_cell_data = ProposalCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let prev_proposal_cell_data_reader = prev_proposal_cell_data.as_reader();

        // Read outputs_data and witness of the ProposalCell.
        let index = &output_cells[0];
        let (_, _, entity) = parser.verify_and_get(index.to_owned(), Source::Output)?;
        let proposal_cell_data = ProposalCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let proposal_cell_data_reader = proposal_cell_data.as_reader();

        let required_cells_count = verify_slices(proposal_cell_data_reader.slices())?;
        let dep_related_cells = find_proposal_related_cells(config_main, Source::CellDep)?;

        #[cfg(not(feature = "mainnet"))]
        inspect_slices(proposal_cell_data_reader.slices())?;
        #[cfg(not(feature = "mainnet"))]
        inspect_related_cells(
            &parser,
            config_main,
            dep_related_cells.clone(),
            Source::CellDep,
            None,
        )?;

        if required_cells_count != dep_related_cells.len() {
            return Err(Error::ProposalSliceRelatedCellMissing);
        }

        verify_slices_relevant_cells(
            config_main,
            proposal_cell_data_reader.slices(),
            dep_related_cells,
            Some(prev_proposal_cell_data_reader.slices()),
        )?;
    } else if action == b"confirm_proposal" {
        debug!("Route to confirm_proposal action ...");

        let timestamp = util::load_timestamp()?;
        // let height = util::load_height()?;

        let mut parser = util::load_das_witnesses(None)?;
        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellMain, ConfigID::ConfigCellRegister])?;
        let config_main = parser.configs().main()?;
        let config_register = parser.configs().register()?;

        if dep_cells.len() != 0 || input_cells.len() != 1 || output_cells.len() != 0 {
            return Err(Error::ProposalFoundInvalidTransaction);
        }

        // Read outputs_data and witness of ProposalCell.
        let index = &input_cells[0];
        let (_, _, entity) = parser.verify_and_get(index.to_owned(), Source::Input)?;
        let proposal_cell_data = ProposalCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let proposal_cell_data_reader = proposal_cell_data.as_reader();

        debug!("Check all AccountCells are updated or created base on proposal.");

        let input_related_cells = find_proposal_related_cells(config_main, Source::Input)?;
        let output_account_cells = find_output_account_cells(config_main)?;

        #[cfg(not(feature = "mainnet"))]
        inspect_slices(proposal_cell_data_reader.slices())?;
        #[cfg(not(feature = "mainnet"))]
        inspect_related_cells(
            &parser,
            config_main,
            input_related_cells.clone(),
            Source::Input,
            Some(output_account_cells.clone()),
        )?;

        verify_proposal_execution_result(
            &parser,
            config_main,
            config_register,
            timestamp,
            proposal_cell_data_reader.slices(),
            input_related_cells,
            output_account_cells,
        )?;

        debug!("Check that all revenues are correctly allocated to each roles in DAS.");
    } else if action == b"recycle_proposal" {
        debug!("Route to recycle_propose action ...");

        let height = util::load_height()?;

        let mut parser = util::load_das_witnesses(None)?;
        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellRegister])?;
        let config_register = parser.configs().register()?;

        if dep_cells.len() != 0 || input_cells.len() != 1 || output_cells.len() != 0 {
            return Err(Error::ProposalFoundInvalidTransaction);
        }

        debug!("Check if ProposalCell can be recycled.");

        let index = &input_cells[0];
        let (_, _, entity) = parser.verify_and_get(index.to_owned(), Source::Input)?;
        let proposal_cell_data = ProposalCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let proposal_cell_data_reader = proposal_cell_data.as_reader();

        let proposal_min_recycle_interval =
            u8::from(config_register.proposal_min_recycle_interval()) as u64;
        let created_at_height = u64::from(proposal_cell_data_reader.created_at_height());
        if height - created_at_height < proposal_min_recycle_interval {
            return Err(Error::ProposalRecycleNeedWaitLonger);
        }

        debug!("Check if refund lock and amount is correct.");

        let refund_lock = proposal_cell_data_reader.proposer_lock().to_entity();
        let refund_cells =
            util::find_cells_by_script(ScriptType::Lock, &refund_lock.into(), Source::Output)?;
        if refund_cells.len() != 1 {
            return Err(Error::ProposalRecycleCanNotFoundRefundCell);
        }
        let proposal_capacity =
            load_cell_capacity(index.to_owned(), Source::Input).map_err(|e| Error::from(e))?;
        let refund_capacity =
            load_cell_capacity(refund_cells[0], Source::Output).map_err(|e| Error::from(e))?;
        if proposal_capacity > refund_capacity {
            return Err(Error::ProposalRecycleRefundAmountError);
        }
    } else {
        return Err(Error::ActionNotSupported);
    }

    Ok(())
}

#[cfg(not(feature = "mainnet"))]
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

#[cfg(not(feature = "mainnet"))]
fn inspect_related_cells(
    parser: &WitnessesParser,
    config_main: ConfigCellMainReader,
    related_cells: Vec<usize>,
    related_cells_source: Source,
    output_account_cells: Option<Vec<usize>>,
) -> Result<(), Error> {
    use das_core::inspect;

    debug!("Inspect inputs:");
    for i in related_cells {
        let script = load_cell_type(i, related_cells_source)
            .map_err(|e| Error::from(e))?
            .unwrap();
        let code_hash = Hash::from(script.code_hash());
        let (_, _, entity) = parser.verify_and_get(i, related_cells_source)?;
        let data = util::load_cell_data(i, related_cells_source)?;

        debug!(" Input[{}].cell.type: {}", i, script);

        if util::is_reader_eq(
            config_main.type_id_table().account_cell(),
            code_hash.as_reader(),
        ) {
            inspect::account_cell(Source::Input, i, &data, entity.to_owned());
        } else if util::is_reader_eq(
            config_main.type_id_table().pre_account_cell(),
            code_hash.as_reader(),
        ) {
            inspect::pre_account_cell(Source::Input, i, &data, entity.to_owned());
        }
    }

    if let Some(output_account_cells) = output_account_cells {
        for i in output_account_cells {
            let script = load_cell_type(i, Source::Output)
                .map_err(|e| Error::from(e))?
                .unwrap();
            let code_hash = Hash::from(script.code_hash());
            let (_, _, entity) = parser.verify_and_get(i, Source::Output)?;
            let data = util::load_cell_data(i, Source::Output)?;

            debug!(" Output[{}].cell.type: {}", i, script);

            if util::is_reader_eq(
                config_main.type_id_table().account_cell(),
                code_hash.as_reader(),
            ) {
                inspect::account_cell(Source::Output, i, &data, entity.to_owned());
            }
        }
    }

    Ok(())
}

fn verify_slices(slices_reader: SliceListReader) -> Result<usize, Error> {
    debug!("Check the data structure of proposal slices.");

    let mut required_cells_count: usize = 0;
    for (sl_index, sl_reader) in slices_reader.iter().enumerate() {
        debug!("Check Slice[{}] ...", sl_index);
        let mut account_id_list = Vec::new();
        for (index, item) in sl_reader.iter().enumerate() {
            // Check the continuity of the items in the slice.
            if let Some(next) = sl_reader.get(index + 1) {
                debug!("  Check Item[{}]", index);
                if !util::is_reader_eq(item.next(), next.account_id()) {
                    return Err(Error::ProposalSliceIsDiscontinuity);
                }
            }

            account_id_list.push(bytes::Bytes::from(item.account_id().raw_data()));
            required_cells_count += 1;
        }

        // Check the order of items in the slice.
        let sorted_account_id_list = DasSortedList::new(account_id_list.clone());
        if !sorted_account_id_list.cmp_order_with(account_id_list) {
            return Err(Error::ProposalSliceIsNotSorted);
        }
    }

    Ok(required_cells_count)
}

fn find_proposal_related_cells(
    config: ConfigCellMainReader,
    source: Source,
) -> Result<Vec<usize>, Error> {
    // Find related cells' indexes in cell_deps or inputs.
    let account_cell_type_id = config.type_id_table().account_cell();
    let account_cells =
        util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, source)?;
    let pre_account_cell_type_id = config.type_id_table().pre_account_cell();
    let pre_account_cells =
        util::find_cells_by_type_id(ScriptType::Type, pre_account_cell_type_id, source)?;

    if pre_account_cells.len() <= 0 {
        return Err(Error::ProposalFoundInvalidTransaction);
    }

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
        sorted = pre_account_cells;
        sorted.sort();
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
    let mut account_cells =
        util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, Source::Output)?;
    account_cells.sort();

    if account_cells.len() <= 0 {
        return Err(Error::ProposalFoundInvalidTransaction);
    }

    debug!(
        "Outputs cells(AccountCell) sorted index list: {:?}",
        account_cells
    );

    Ok(account_cells)
}

fn verify_slices_relevant_cells(
    config: ConfigCellMainReader,
    slices_reader: SliceListReader,
    relevant_cells: Vec<usize>,
    prev_slices_reader_opt: Option<SliceListReader>,
) -> Result<(), Error> {
    debug!("Check the proposal slices relevant cells are real exist and in correct status.");

    let mut i = 0;
    for (sl_index, sl_reader) in slices_reader.iter().enumerate() {
        debug!("Check slice {} ...", sl_index);
        let mut next_of_first_cell = AccountId::default();
        for (item_index, item) in sl_reader.iter().enumerate() {
            let item_account_id = item.account_id();
            let item_type = u8::from(item.item_type());

            let cell_index = relevant_cells[i];
            // Check if the relevant cells has the same type as in the proposal.
            let expected_type_id = if item_type == ProposalSliceItemType::Exist as u8 {
                config.type_id_table().account_cell()
            } else {
                config.type_id_table().pre_account_cell()
            };
            verify_cell_type_id(item_index, cell_index, Source::CellDep, &expected_type_id)?;

            // Check if the relevant cells have the same account ID.
            let cell_data =
                load_cell_data(cell_index, Source::CellDep).map_err(|e| Error::from(e))?;
            verify_cell_account_id(
                item_index,
                &cell_data,
                cell_index,
                Source::CellDep,
                item_account_id,
            )?;

            // ⚠️ The first item is very very important, its "next" must be correct so that
            // AccountCells can form a linked list.
            if item_index == 0 {
                // If this is the first proposal in proposal chain, all slice must start with an AccountCell.
                if prev_slices_reader_opt.is_none() {
                    if item_type != ProposalSliceItemType::Exist as u8 {
                        return Err(Error::ProposalSliceMustStartWithAccountCell);
                    }

                    // The correct "next" of first proposal is come from the cell's outputs_data.
                    next_of_first_cell = AccountId::try_from(get_next(&cell_data))
                        .map_err(|_| Error::InvalidCellData)?;

                // If this is the extended proposal in proposal chain, slice may starting with an
                // AccountCell/PreAccountCell included in previous proposal, or it may starting with
                // an AccountCell not included in previous proposal.
                } else {
                    if item_type != ProposalSliceItemType::Exist as u8
                        && item_type != ProposalSliceItemType::Proposed as u8
                    {
                        return Err(Error::ProposalSliceMustStartWithAccountCell);
                    }

                    let prev_slices_reader = prev_slices_reader_opt.as_ref().unwrap();
                    next_of_first_cell =
                        match find_item_contains_account_id(prev_slices_reader, &item_account_id) {
                            // If the item is included in previous proposal, then we need to get its latest "next" from the proposal.
                            Ok(prev_item) => prev_item.next(),
                            // If the item is not included in previous proposal, then we get its latest "next" from the cell's outputs_data.
                            Err(_) => AccountId::try_from(get_next(&cell_data))
                                .map_err(|_| Error::InvalidCellData)?,
                        };
                }
            }

            i += 1;
        }

        // Check if the first item's "next" has pass to the last item correctly.
        let item = sl_reader.get(sl_reader.len() - 1).unwrap();
        let next_of_last_item = item.next();

        debug!(
            "  Compare the last next of slice: {} != {} => {}",
            next_of_first_cell,
            next_of_last_item,
            !util::is_reader_eq(next_of_first_cell.as_reader(), next_of_last_item)
        );
        if !util::is_reader_eq(next_of_first_cell.as_reader(), next_of_last_item) {
            return Err(Error::ProposalSliceNotEndCorrectly);
        }
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
    config_main: ConfigCellMainReader,
    config_register: ConfigCellRegisterReader,
    timestamp: u64,
    slices_reader: SliceListReader,
    input_related_cells: Vec<usize>,
    output_account_cells: Vec<usize>,
) -> Result<(), Error> {
    debug!("Check that all AccountCells/PreAccountCells have been converted according to the proposal.");

    let account_cell_type_id = config_main.type_id_table().account_cell();
    let pre_account_cell_type_id = config_main.type_id_table().pre_account_cell();

    let mut wallet = Wallet::new();
    let inviter_profit_rate = u32::from(config_register.profit().profit_rate_of_inviter()) as u64;
    let channel_profit_rate = u32::from(config_register.profit().profit_rate_of_inviter()) as u64;

    let mut i = 0;
    for (sl_index, sl_reader) in slices_reader.iter().enumerate() {
        debug!("Check Slice[{}] ...", sl_index);
        for (item_index, item) in sl_reader.iter().enumerate() {
            let item_account_id = item.account_id();
            let item_type = u8::from(item.item_type());
            let item_next = item.next();

            let input_cell_data = load_cell_data(input_related_cells[i], Source::Input)
                .map_err(|e| Error::from(e))?;
            let (_, _, old_entity) =
                parser.verify_and_get(input_related_cells[i], Source::Input)?;
            let output_cell_data = load_cell_data(output_account_cells[i], Source::Output)
                .map_err(|e| Error::from(e))?;
            let (_, _, new_entity) =
                parser.verify_and_get(output_account_cells[i], Source::Output)?;

            let mut new_account_ids = Vec::new();
            if item_type == ProposalSliceItemType::Exist as u8
                || item_type == ProposalSliceItemType::Proposed as u8
            {
                debug!(
                    "  Item[{}] Check that the existing AccountCell({}) is updated correctly.",
                    item_index, input_related_cells[i]
                );

                // All cells' type is must be account-cell-type
                verify_cell_type_id(
                    item_index,
                    input_related_cells[i],
                    Source::Input,
                    &account_cell_type_id,
                )?;
                verify_cell_type_id(
                    item_index,
                    output_account_cells[i],
                    Source::Output,
                    &account_cell_type_id,
                )?;

                // All cells' account_id in data must be the same as the account_id in proposal.
                verify_cell_account_id(
                    item_index,
                    &input_cell_data,
                    input_related_cells[i],
                    Source::Input,
                    item_account_id,
                )?;
                verify_cell_account_id(
                    item_index,
                    &output_cell_data,
                    output_account_cells[i],
                    Source::Output,
                    item_account_id,
                )?;

                let old_cell_witness =
                    AccountCellData::new_unchecked(old_entity.as_reader().raw_data().into());
                let old_cell_witness_reader = old_cell_witness.as_reader();
                let new_cell_witness =
                    AccountCellData::new_unchecked(new_entity.as_reader().raw_data().into());
                let output_cell_witness_reader = new_cell_witness.as_reader();

                // For the existing AccountCell, only the next field in data can be modified.
                is_id_same(item_index, &output_cell_data, &input_cell_data)?;
                is_account_same(item_index, &output_cell_data, &input_cell_data)?;
                is_expired_at_same(item_index, &output_cell_data, &input_cell_data)?;
                is_next_correct(item_index, &output_cell_data, item_next)?;

                // For the existing AccountCell, witness can not be modified.
                if !util::is_reader_eq(old_cell_witness_reader, output_cell_witness_reader) {
                    return Err(Error::ProposalWitnessCanNotBeModified);
                }
            } else {
                debug!(
                    "  Item[{}] Check that the PreAccountCell({}) is converted correctly.",
                    item_index, input_related_cells[i]
                );

                // All cells' type is must be account-cell-type
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
                verify_cell_account_id(
                    item_index,
                    &input_cell_data,
                    input_related_cells[i],
                    Source::Input,
                    item_account_id,
                )?;
                verify_cell_account_id(
                    item_index,
                    &output_cell_data,
                    output_account_cells[i],
                    Source::Output,
                    item_account_id,
                )?;

                // Store account IDs of all new accounts for later RefCell verification.
                new_account_ids.push(item_account_id.raw_data().to_vec());

                let old_cell_witness =
                    PreAccountCellData::new_unchecked(old_entity.as_reader().raw_data().into());
                let old_cell_witness_reader = old_cell_witness.as_reader();
                let new_cell_witness =
                    AccountCellData::new_unchecked(new_entity.as_reader().raw_data().into());
                let output_cell_witness_reader = new_cell_witness.as_reader();

                let account_length = get_account(&output_cell_data).len() as u64;
                let total_capacity = load_cell_capacity(input_related_cells[i], Source::Input)
                    .map_err(|e| Error::from(e))?;
                let storage_capacity = util::get_account_storage_total(account_length);
                // Allocate the profits carried by PreAccountCell to the wallets for later verification.
                let profit = total_capacity - storage_capacity;

                // Check all fields in the data of new AccountCell.
                is_id_correct(item_index, &output_cell_data, &input_cell_data)?;
                is_account_correct(item_index, &output_cell_data)?;
                is_next_correct(item_index, &output_cell_data, item_next)?;
                is_expired_at_correct(
                    item_index,
                    profit,
                    timestamp,
                    &output_cell_data,
                    old_cell_witness_reader,
                )?;

                // Check all fields in the witness of new AccountCell.
                verify_witness_id(item_index, &output_cell_data, output_cell_witness_reader)?;
                verify_witness_account(item_index, &output_cell_data, output_cell_witness_reader)?;
                verify_witness_locks(
                    item_index,
                    output_cell_witness_reader,
                    old_cell_witness_reader,
                )?;
                verify_witness_status(item_index, output_cell_witness_reader)?;
                verify_witness_records(item_index, output_cell_witness_reader)?;

                // Only when inviter_wallet's length is equal to account ID it will be count in profit.
                let mut inviter_profit = 0;
                if old_cell_witness_reader.inviter_wallet().len() == ACCOUNT_ID_LENGTH {
                    inviter_profit = profit * inviter_profit_rate / RATE_BASE;
                    debug!(
                        "  Item[{}] {}(inviter_profit) = {}(profit) * {}(inviter_profit_rate) / {}(RATE_BASE)",
                        item_index, inviter_profit, profit, inviter_profit_rate, RATE_BASE
                    );
                    wallet.add_balance(
                        old_cell_witness_reader.inviter_wallet().raw_data(),
                        inviter_profit,
                    );
                };
                let mut channel_profit = 0;
                if old_cell_witness_reader.channel_wallet().len() == ACCOUNT_ID_LENGTH {
                    channel_profit = profit * channel_profit_rate / RATE_BASE;
                    debug!(
                        "  Item[{}] {}(channel_profit) = {}(profit) * {}(channel_profit_rate) / {}(RATE_BASE)",
                        item_index, channel_profit, profit, channel_profit_rate, RATE_BASE
                    );
                    wallet.add_balance(
                        old_cell_witness_reader.channel_wallet().raw_data(),
                        channel_profit,
                    );
                };

                let das_profit = profit - inviter_profit - channel_profit;
                debug!(
                    "  Item[{}] {}(das_profit) = {}(profit) - {}(inviter_profit) - {}(channel_profit)",
                    item_index, das_profit, profit, inviter_profit, channel_profit
                );
                wallet.add_balance(&DAS_WALLET_ID, das_profit);
            }

            i += 1;
        }
    }

    debug!("Check if RefCells have been created correctly.");

    let ref_cell_type_id = config_main.type_id_table().wallet_cell();
    let old_ref_cells =
        util::find_cells_by_type_id(ScriptType::Type, ref_cell_type_id, Source::Input)?;
    let new_ref_cells =
        util::find_cells_by_type_id(ScriptType::Type, ref_cell_type_id, Source::Output)?;

    if old_ref_cells.len() != 0 || new_ref_cells.len() == 0 {
        return Err(Error::ProposalFoundInvalidTransaction);
    }

    // for ref_cell in new_ref_cells {}

    debug!("Check if the balance of all WalletCells have increased correctly.");

    let wallet_cell_type_id = config_main.type_id_table().wallet_cell();
    let old_wallet_cells =
        util::find_cells_by_type_id(ScriptType::Type, wallet_cell_type_id, Source::Input)?;
    let new_wallet_cells =
        util::find_cells_by_type_id(ScriptType::Type, wallet_cell_type_id, Source::Output)?;

    if old_wallet_cells.len() != new_wallet_cells.len() {
        debug!(
            "Compare WalletCells number: inputs({}) != outputs({})",
            old_wallet_cells.len(),
            new_wallet_cells.len()
        );
        return Err(Error::ProposalFoundInvalidTransaction);
    }

    for (i, old_wallet_index) in old_wallet_cells.into_iter().enumerate() {
        let new_wallet_index = new_wallet_cells.get(i).unwrap().to_owned();

        let type_of_old_wallet = load_cell_type(old_wallet_index, Source::Input)
            .map_err(|e| Error::from(e))?
            .unwrap();
        let old_wallet_id = type_of_old_wallet.as_reader().args().raw_data();
        let type_of_new_wallet = load_cell_type(new_wallet_index, Source::Output)
            .map_err(|e| Error::from(e))?
            .unwrap();
        let new_wallet_id = type_of_new_wallet.as_reader().args().raw_data();

        // The WalletCells in inputs must have the same order as those in outputs.
        if old_wallet_id != new_wallet_id {
            debug!(
                "Compare WalletCells order: inputs[{}] {:?} != outputs[{}] {:?}",
                old_wallet_index, old_wallet_id, new_wallet_index, new_wallet_id
            );
            return Err(Error::ProposalConfirmWalletMissMatch);
        }

        let old_balance =
            load_cell_capacity(old_wallet_index, Source::Input).map_err(|e| Error::from(e))?;
        let new_balance =
            load_cell_capacity(new_wallet_index, Source::Output).map_err(|e| Error::from(e))?;
        let current_profit = new_balance - old_balance;

        debug!(
            "Check if WalletCell[0x{}] has updated balance correctly.",
            util::hex_string(new_wallet_id)
        );

        // Balance in wallet instance do not contains cell occupied capacities, so it is pure profit.
        let result = wallet
            .cmp_balance(new_wallet_id, current_profit)
            .map_err(|_| Error::ProposalConfirmWalletMissMatch)?;
        if !result {
            debug!(
                "Wallet balance variation: {}(current_profit) = {}(0x{}) - {}(0x{})",
                current_profit,
                new_balance,
                util::hex_string(new_wallet_id),
                old_balance,
                util::hex_string(old_wallet_id)
            );
            debug!(
                "Compare profit with expected: {}(current_profit) != {} -> true",
                current_profit,
                wallet.get_balance(old_wallet_id).unwrap()
            );
            return Err(Error::ProposalConfirmWalletBalanceError);
        }
    }

    Ok(())
}

fn verify_cell_type_id(
    item_index: usize,
    cell_index: usize,
    source: Source,
    expected_type_id: &HashReader,
) -> Result<(), Error> {
    let cell_type_id = load_cell_type(cell_index, source)
        .map_err(|e| Error::from(e))?
        .map(|script| Hash::from(script.code_hash()))
        .ok_or(Error::ProposalSliceRelatedCellNotFound)?;

    if !util::is_reader_eq(expected_type_id.to_owned(), cell_type_id.as_reader()) {
        debug!(
            "  Item[{}] Verify type script at {:?}[{}]: {} != {} => {}",
            item_index,
            source,
            cell_index,
            cell_type_id,
            expected_type_id,
            !util::is_reader_eq(expected_type_id.to_owned(), cell_type_id.as_reader())
        );
        return Err(Error::ProposalCellTypeError);
    }

    Ok(())
}

fn verify_cell_account_id(
    item_index: usize,
    cell_data: &Vec<u8>,
    cell_index: usize,
    source: Source,
    expected_account_id: AccountIdReader,
) -> Result<(), Error> {
    let account_id = AccountId::try_from(get_id(&cell_data)).map_err(|_| Error::InvalidCellData)?;

    if !util::is_reader_eq(account_id.as_reader(), expected_account_id) {
        debug!(
            "  Item[{}] Verify account_id at {:?}[{}]: {} != {} => {}",
            item_index,
            source,
            cell_index,
            account_id,
            expected_account_id,
            !util::is_reader_eq(account_id.as_reader(), expected_account_id)
        );
        return Err(Error::ProposalCellAccountIdError);
    }

    Ok(())
}

fn is_bytes_eq(
    item_index: usize,
    field: &str,
    current_bytes: &[u8],
    expected_bytes: &[u8],
    error_code: Error,
) -> Result<(), Error> {
    if current_bytes != expected_bytes {
        debug!(
            "  Item[{}] Check outputs[].AccountCell.{}: 0x{} != 0x{} => true",
            item_index,
            field,
            util::hex_string(current_bytes),
            util::hex_string(expected_bytes)
        );
        return Err(error_code);
    }

    Ok(())
}

fn is_id_same(
    item_index: usize,
    output_cell_data: &Vec<u8>,
    input_cell_data: &Vec<u8>,
) -> Result<(), Error> {
    is_bytes_eq(
        item_index,
        "id",
        get_id(output_cell_data),
        get_id(input_cell_data),
        Error::ProposalFieldCanNotBeModified,
    )
}

fn is_account_same(
    item_index: usize,
    output_cell_data: &Vec<u8>,
    input_cell_data: &Vec<u8>,
) -> Result<(), Error> {
    is_bytes_eq(
        item_index,
        "account",
        get_account(output_cell_data),
        get_account(input_cell_data),
        Error::ProposalFieldCanNotBeModified,
    )
}

fn is_expired_at_same(
    item_index: usize,
    output_cell_data: &Vec<u8>,
    input_cell_data: &Vec<u8>,
) -> Result<(), Error> {
    let input_expired_at = get_expired_at(input_cell_data);
    let output_expired_at = get_expired_at(output_cell_data);

    if input_expired_at != output_expired_at {
        debug!(
            "  Item[{}] Check outputs[].AccountCell.expired_at: {:x?} != {:x?} => true",
            item_index, input_expired_at, output_expired_at
        );
        return Err(Error::ProposalFieldCanNotBeModified);
    }

    Ok(())
}

fn is_id_correct(
    item_index: usize,
    output_cell_data: &Vec<u8>,
    input_cell_data: &Vec<u8>,
) -> Result<(), Error> {
    is_bytes_eq(
        item_index,
        "id",
        get_id(output_cell_data),
        get_id(input_cell_data),
        Error::ProposalConfirmIdError,
    )
}

fn is_next_correct(
    item_index: usize,
    output_cell_data: &Vec<u8>,
    proposed_next: AccountIdReader,
) -> Result<(), Error> {
    let expected_next = proposed_next.raw_data();

    is_bytes_eq(
        item_index,
        "next",
        get_next(output_cell_data),
        expected_next,
        Error::ProposalConfirmNextError,
    )
}

fn is_expired_at_correct(
    item_index: usize,
    profit: u64,
    current_timestamp: u64,
    output_cell_data: &Vec<u8>,
    pre_account_cell_witness: PreAccountCellDataReader,
) -> Result<(), Error> {
    let price = u64::from(pre_account_cell_witness.price().new());
    let quote = u64::from(pre_account_cell_witness.quote());
    let duration = profit * 365 * 86400 / (price / quote * 100_000_000);
    let expired_at = get_expired_at(output_cell_data);

    debug!(
        "  Item[{}] Check if outputs[].AccountCell.expired_at: expired_at({}) != {} = current({}) + duration({})",
        item_index,
        expired_at,
        current_timestamp + duration,
        current_timestamp,
        duration
    );

    if current_timestamp + duration != expired_at {
        debug!(
            "  Item[{}] duration({}) = profit({}) * 365 * 86400 / (price({}) / quote({}) * 100_000_000)",
            item_index,
            duration,
            profit,
            price,
            quote
        );
        return Err(Error::ProposalConfirmExpiredAtError);
    }

    Ok(())
}

fn is_account_correct(item_index: usize, output_cell_data: &Vec<u8>) -> Result<(), Error> {
    let expected_account_id = get_id(output_cell_data);
    let account = get_account(output_cell_data);

    let hash = util::blake2b_256(account);
    let account_id = hash.get(..ACCOUNT_ID_LENGTH).unwrap();

    is_bytes_eq(
        item_index,
        "account",
        account_id,
        expected_account_id,
        Error::ProposalConfirmAccountError,
    )
}

fn verify_witness_id(
    item_index: usize,
    output_cell_data: &Vec<u8>,
    output_cell_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    let account_id = output_cell_witness_reader.id().raw_data();
    let expected_account_id = get_id(output_cell_data);

    is_bytes_eq(
        item_index,
        "witness.id",
        account_id,
        expected_account_id,
        Error::ProposalConfirmWitnessIDError,
    )
}

fn verify_witness_account(
    item_index: usize,
    output_cell_data: &Vec<u8>,
    output_cell_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    let mut account = output_cell_witness_reader.account().as_readable();
    account.append(&mut ACCOUNT_SUFFIX.as_bytes().to_vec());
    let expected_account = get_account(output_cell_data);

    is_bytes_eq(
        item_index,
        "witness.account",
        account.as_slice(),
        expected_account,
        Error::ProposalConfirmWitnessAccountError,
    )
}

fn verify_witness_locks(
    item_index: usize,
    output_cell_witness_reader: AccountCellDataReader,
    old_cell_witness_reader: PreAccountCellDataReader,
) -> Result<(), Error> {
    let owner_lock = output_cell_witness_reader.owner_lock();
    let manager_lock = output_cell_witness_reader.manager_lock();
    let expected_lock = old_cell_witness_reader.owner_lock();

    if !util::is_reader_eq(owner_lock, expected_lock) {
        debug!(
            "  Item[{}] Check outputs[].AccountCell.owner: {:x?} != {:x?} => {}",
            item_index,
            owner_lock,
            expected_lock,
            !util::is_reader_eq(owner_lock, expected_lock)
        );
        return Err(Error::ProposalConfirmWitnessOwnerError);
    }

    if !util::is_reader_eq(manager_lock, expected_lock) {
        debug!(
            "  Item[{}] Check outputs[].AccountCell.owner: {:x?} != {:x?} => {}",
            item_index,
            manager_lock,
            expected_lock,
            !util::is_reader_eq(manager_lock, expected_lock)
        );
        return Err(Error::ProposalConfirmWitnessManagerError);
    }

    Ok(())
}

fn verify_witness_status(
    item_index: usize,
    output_cell_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    let status = u8::from(output_cell_witness_reader.status());

    if status != AccountStatus::Normal as u8 {
        debug!(
            "  Item[{}] Check if outputs[].AccountCell.status is normal. (Result: {}, expected: 0)",
            item_index, status
        );
        return Err(Error::ProposalConfirmWitnessManagerError);
    }

    Ok(())
}

fn verify_witness_records(
    item_index: usize,
    output_cell_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    let records = output_cell_witness_reader.records();

    if !records.is_empty() {
        debug!(
            "  Item[{}] Check if outputs[].AccountCell.records is empty. (Result: {}, expected: true)",
            item_index,
            records.is_empty()
        );
        return Err(Error::ProposalConfirmWitnessRecordsError);
    }

    Ok(())
}
