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

    // Loading and parsing DAS witnesses.
    let witnesses = util::load_das_witnesses()?;
    let mut parser = WitnessesParser::new(witnesses)?;
    parser.parse_only_action()?;

    debug!("Find out ProposalCell ...");

    // Find out PreAccountCells in current transaction.
    let this_type_script = load_script().map_err(|e| Error::from(e))?;
    let old_cells = util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
    let new_cells =
        util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;
    let dep_cells =
        util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::CellDep)?;

    // Routing by ActionData in witness.
    let (action, _) = parser.action();
    if action == b"propose" {
        debug!("Route to propose action ...");

        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellMain])?;
        let config = parser.configs().main()?;

        if dep_cells.len() != 0 || old_cells.len() != 0 || new_cells.len() != 1 {
            return Err(Error::ProposalFoundInvalidTransaction);
        }

        // Read outputs_data and witness of the ProposalCell.
        let index = &new_cells[0];
        let (_, _, entity) = parser.verify_and_get(index.to_owned(), Source::Output)?;
        let proposal_cell_data = ProposalCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let proposal_cell_data_reader = proposal_cell_data.as_reader();

        verify_slices(proposal_cell_data_reader.slices())?;
        let related_cells = find_proposal_related_cells(config, Source::CellDep)?;
        verify_slices_relevant_cells(
            config,
            proposal_cell_data_reader.slices(),
            related_cells,
            None,
        )?;
    } else if action == b"extend_proposal" {
        debug!("Route to extend_proposal action ...");

        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellMain])?;
        let config = parser.configs().main()?;

        if dep_cells.len() != 1 || old_cells.len() != 0 || new_cells.len() != 1 {
            return Err(Error::ProposalFoundInvalidTransaction);
        }

        // Read outputs_data and witness of previous ProposalCell.
        let index = &dep_cells[0];
        let (_, _, entity) = parser.verify_and_get(index.to_owned(), Source::CellDep)?;
        let prev_proposal_cell_data = ProposalCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let prev_proposal_cell_data_reader = prev_proposal_cell_data.as_reader();

        // Read outputs_data and witness of the ProposalCell.
        let index = &new_cells[0];
        let (_, _, entity) = parser.verify_and_get(index.to_owned(), Source::Output)?;
        let proposal_cell_data = ProposalCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let proposal_cell_data_reader = proposal_cell_data.as_reader();

        verify_slices(proposal_cell_data_reader.slices())?;
        let related_cells = find_proposal_related_cells(config, Source::CellDep)?;
        verify_slices_relevant_cells(
            config,
            proposal_cell_data_reader.slices(),
            related_cells,
            Some(prev_proposal_cell_data_reader.slices()),
        )?;
    } else if action == b"confirm_proposal" {
        debug!("Route to confirm_proposal action ...");

        let timestamp = util::load_timestamp()?;

        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellMain, ConfigID::ConfigCellRegister])?;
        let config_main = parser.configs().main()?;
        let config_register = parser.configs().register()?;

        if dep_cells.len() != 0 || old_cells.len() != 1 || new_cells.len() != 0 {
            return Err(Error::ProposalFoundInvalidTransaction);
        }

        // Read outputs_data and witness of ProposalCell.
        let index = &old_cells[0];
        let (_, _, entity) = parser.verify_and_get(index.to_owned(), Source::Input)?;
        let proposal_cell_data = ProposalCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let proposal_cell_data_reader = proposal_cell_data.as_reader();

        debug!("Check all AccountCells are updated or created base on proposal.");

        let old_related_cells = find_proposal_related_cells(config_main, Source::Input)?;
        let new_account_cells = find_new_account_cells(config_main)?;
        verify_proposal_execution_result(
            &parser,
            config_main,
            config_register,
            timestamp,
            proposal_cell_data_reader.slices(),
            old_related_cells,
            new_account_cells,
        )?;

        debug!("Check that all revenues are correctly allocated to each roles in DAS.");
    } else if action == b"recycle_proposal" {
        debug!("Route to recycle_propose action ...");

        let height = util::load_height()?;

        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellRegister])?;
        let config_register = parser.configs().register()?;

        if dep_cells.len() != 0 || old_cells.len() != 1 || new_cells.len() != 0 {
            return Err(Error::ProposalFoundInvalidTransaction);
        }

        debug!("Check if ProposalCell can be recycled.");

        let index = &old_cells[0];
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

fn verify_slices(slices_reader: SliceListReader) -> Result<(), Error> {
    debug!("Check the data structure of proposal slices.");

    for (sl_index, sl_reader) in slices_reader.iter().enumerate() {
        debug!("Check slice {} ...", sl_index);
        let mut account_id_list = Vec::new();
        for (index, item) in sl_reader.iter().enumerate() {
            // Check the continuity of the items in the slice.
            if let Some(next) = sl_reader.get(index + 1) {
                debug!(
                    "  Compare slice continuity: {} -> {}",
                    item.account_id(),
                    next.account_id()
                );
                if !util::is_reader_eq(item.next(), next.account_id()) {
                    return Err(Error::ProposalSliceIsDiscontinuity);
                }
            }

            account_id_list.push(bytes::Bytes::from(item.account_id().raw_data()))
        }

        // Check the order of items in the slice.
        let sorted_account_id_list = DasSortedList::new(account_id_list.clone());
        if !sorted_account_id_list.cmp_order_with(account_id_list) {
            return Err(Error::ProposalSliceIsNotSorted);
        }
    }

    Ok(())
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
        "AccountCell and PreAccountCell sorted index list: {:?}",
        sorted
    );

    Ok(sorted)
}

fn find_new_account_cells(config: ConfigCellMainReader) -> Result<Vec<usize>, Error> {
    // Find updated cells' indexes in outputs.
    let account_cell_type_id = config.type_id_table().account_cell();
    let mut account_cells =
        util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, Source::Output)?;
    account_cells.sort();

    if account_cells.len() <= 0 {
        return Err(Error::ProposalFoundInvalidTransaction);
    }

    debug!("AccountCell sorted index list: {:?}", account_cells);

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
        for (index, item) in sl_reader.iter().enumerate() {
            let item_account_id = item.account_id();
            let item_type = u8::from(item.item_type());

            let cell_index = relevant_cells[i];
            // Check if the relevant cells has the same type as in the proposal.
            let expected_type_id = if item_type == ProposalSliceItemType::Exist as u8 {
                config.type_id_table().account_cell()
            } else {
                config.type_id_table().pre_account_cell()
            };
            verify_cell_type_id(cell_index, Source::CellDep, &expected_type_id)?;

            // Check if the relevant cells have the same account ID.
            let cell_data =
                load_cell_data(cell_index, Source::CellDep).map_err(|e| Error::from(e))?;
            verify_cell_account_id(&cell_data, cell_index, Source::CellDep, item_account_id)?;

            // ⚠️ The first item is very very important, its "next" must be correct so that
            // AccountCells can form a linked list.
            if index == 0 {
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
    old_related_cells: Vec<usize>,
    new_account_cells: Vec<usize>,
) -> Result<(), Error> {
    debug!("Check that all AccountCells/PreAccountCells have been converted according to the proposal.");

    let account_cell_type_id = config_main.type_id_table().account_cell();
    let pre_account_cell_type_id = config_main.type_id_table().pre_account_cell();

    let mut wallet = Wallet::new();
    let inviter_profit_rate = u32::from(config_register.profit().profit_rate_of_inviter()) as u64;
    let channel_profit_rate = u32::from(config_register.profit().profit_rate_of_inviter()) as u64;

    let mut i = 0;
    for (sl_index, sl_reader) in slices_reader.iter().enumerate() {
        debug!("Check slice {} ...", sl_index);
        for (item_index, item) in sl_reader.iter().enumerate() {
            let item_account_id = item.account_id();
            let item_type = u8::from(item.item_type());
            let item_next = item.next();

            let old_cell_data =
                load_cell_data(old_related_cells[i], Source::Input).map_err(|e| Error::from(e))?;
            let (_, _, old_entity) =
                util::get_cell_witness(&parser, old_related_cells[i], Source::Input)?;
            let new_cell_data =
                load_cell_data(new_account_cells[i], Source::Output).map_err(|e| Error::from(e))?;
            let (_, _, new_entity) =
                util::get_cell_witness(&parser, new_account_cells[i], Source::Output)?;

            if item_type == ProposalSliceItemType::Exist as u8
                || item_type == ProposalSliceItemType::Proposed as u8
            {
                debug!(
                    "  [{}] Check that the existing AccountCell({}) is updated correctly.",
                    item_index, old_related_cells[i]
                );

                // All cells' type is must be account-cell-type
                verify_cell_type_id(old_related_cells[i], Source::Input, &account_cell_type_id)?;
                verify_cell_type_id(new_account_cells[i], Source::Output, &account_cell_type_id)?;

                // All cells' account_id in data must be the same as the account_id in proposal.
                verify_cell_account_id(
                    &old_cell_data,
                    old_related_cells[i],
                    Source::Input,
                    item_account_id,
                )?;
                verify_cell_account_id(
                    &new_cell_data,
                    new_account_cells[i],
                    Source::Output,
                    item_account_id,
                )?;

                let old_cell_witness =
                    AccountCellData::new_unchecked(old_entity.as_reader().raw_data().into());
                let old_cell_witness_reader = old_cell_witness.as_reader();
                let new_cell_witness =
                    AccountCellData::new_unchecked(new_entity.as_reader().raw_data().into());
                let new_cell_witness_reader = new_cell_witness.as_reader();

                // For the existing AccountCell, only the next field in data can be modified.
                is_id_same(item_index, &new_cell_data, &old_cell_data)?;
                is_account_same(item_index, &new_cell_data, &old_cell_data)?;
                is_expired_at_same(item_index, &new_cell_data, &old_cell_data)?;
                is_next_correct(item_index, &new_cell_data, item_next)?;

                // For the existing AccountCell, witness can not be modified.
                if !util::is_reader_eq(old_cell_witness_reader, new_cell_witness_reader) {
                    return Err(Error::ProposalWitnessCanNotBeModified);
                }
            } else {
                debug!(
                    "  [{}] Check that the PreAccountCell({}) is converted correctly.",
                    item_index, old_related_cells[i]
                );

                // All cells' type is must be account-cell-type
                verify_cell_type_id(
                    old_related_cells[i],
                    Source::Input,
                    &pre_account_cell_type_id,
                )?;
                verify_cell_type_id(new_account_cells[i], Source::Output, &account_cell_type_id)?;

                // All cells' account_id in data must be the same as the account_id in proposal.
                verify_cell_account_id(
                    &old_cell_data,
                    old_related_cells[i],
                    Source::Input,
                    item_account_id,
                )?;
                verify_cell_account_id(
                    &new_cell_data,
                    new_account_cells[i],
                    Source::Output,
                    item_account_id,
                )?;

                let old_cell_witness =
                    PreAccountCellData::new_unchecked(old_entity.as_reader().raw_data().into());
                let old_cell_witness_reader = old_cell_witness.as_reader();
                let new_cell_witness =
                    AccountCellData::new_unchecked(new_entity.as_reader().raw_data().into());
                let new_cell_witness_reader = new_cell_witness.as_reader();
                let income = load_cell_capacity(old_related_cells[i], Source::Input)
                    .map_err(|e| Error::from(e))?;

                // Check all fields in the data of new AccountCell.
                is_id_correct(item_index, &new_cell_data, &old_cell_data)?;
                is_account_correct(item_index, &new_cell_data)?;
                is_next_correct(item_index, &new_cell_data, item_next)?;
                is_expired_at_correct(
                    item_index,
                    income,
                    timestamp,
                    &new_cell_data,
                    old_cell_witness_reader,
                )?;

                // Check all fields in the witness of new AccountCell.
                verify_witness_id(item_index, &new_cell_data, new_cell_witness_reader)?;
                verify_witness_account(item_index, &new_cell_data, new_cell_witness_reader)?;
                verify_witness_locks(item_index, new_cell_witness_reader, old_cell_witness_reader)?;
                verify_witness_status(item_index, new_cell_witness_reader)?;
                verify_witness_records(item_index, new_cell_witness_reader)?;

                // Allocate the profits carried by PreAccountCell to the wallets for later verification.
                let account_capacity = ((new_cell_witness_reader.account().as_readable().len() + 4)
                    * 100_000_000) as u64;
                let total_capacity = load_cell_capacity(old_related_cells[i], Source::Input)
                    .map_err(|e| Error::from(e))?;
                let profit = total_capacity
                    - ACCOUNT_CELL_BASIC_CAPACITY
                    - REF_CELL_BASIC_CAPACITY
                    - account_capacity;
                // Only if inviter_wallet has 20 bytes, we will treat it as account ID and count profit.
                let mut inviter_profit = 0;
                if old_cell_witness_reader.inviter_wallet().len() == ACCOUNT_ID_LENGTH {
                    inviter_profit = profit * inviter_profit_rate / RATE_BASE;
                    debug!(
                        "  [{}] inviter_profit up: {} <- {} * {} / {}",
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
                        "  [{}] channel_profit up: {} <- {} * {} / {}",
                        item_index, channel_profit, profit, channel_profit_rate, RATE_BASE
                    );
                    wallet.add_balance(
                        old_cell_witness_reader.channel_wallet().raw_data(),
                        channel_profit,
                    );
                };

                let das_profit = profit - inviter_profit - channel_profit;
                debug!(
                    "  [{}] das_profit up: {} <- {} - {} - {}",
                    item_index, das_profit, profit, inviter_profit, channel_profit
                );
                wallet.add_balance(&DAS_WALLET_ID, das_profit);
            }

            i += 1;
        }
    }

    // Verify if the balance of all WalletCells have increased correctly.
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
                "Compare WalletCell account ID: inputs[{}] {:?} != outputs[{}] {:?}",
                old_wallet_index, old_wallet_id, new_wallet_index, new_wallet_id
            );
            return Err(Error::ProposalConfirmWalletMissMatch);
        }

        let old_balance =
            load_cell_capacity(old_wallet_index, Source::Input).map_err(|e| Error::from(e))?;
        let new_balance =
            load_cell_capacity(new_wallet_index, Source::Output).map_err(|e| Error::from(e))?;
        let current_profit = new_balance - old_balance;

        debug!("wallet_id: {:?}", new_wallet_id);
        let result = wallet
            .cmp_balance(new_wallet_id, current_profit)
            .map_err(|_| Error::ProposalConfirmWalletMissMatch)?;
        if !result {
            debug!(
                "Wallet balance variation: [{}]{} - [{}]{} = {}",
                new_wallet_index, new_balance, old_wallet_index, old_balance, current_profit
            );
            debug!(
                "Compare wallet balance with expected: {} != {} -> true",
                current_profit,
                wallet.get_balance(old_wallet_id).unwrap()
            );
            return Err(Error::ProposalConfirmWalletBalanceError);
        }
    }

    Ok(())
}

fn verify_cell_type_id(
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
            "  [{}] Verify type script at {}: {} != {} => {}",
            cell_index,
            util::source_to_str(source),
            cell_type_id,
            expected_type_id,
            !util::is_reader_eq(expected_type_id.to_owned(), cell_type_id.as_reader())
        );
        return Err(Error::ProposalCellTypeError);
    }

    Ok(())
}

fn verify_cell_account_id(
    cell_data: &Vec<u8>,
    cell_index: usize,
    source: Source,
    expected_account_id: AccountIdReader,
) -> Result<(), Error> {
    let account_id = AccountId::try_from(get_id(&cell_data)).map_err(|_| Error::InvalidCellData)?;

    if !util::is_reader_eq(account_id.as_reader(), expected_account_id) {
        debug!(
            "  [{}] Verify account_id at {}: {} != {} => {}",
            cell_index,
            util::source_to_str(source),
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
            "  [{}] Check outputs[].AccountCell.{}: {:x?} != {:x?} => {}",
            item_index,
            field,
            current_bytes,
            expected_bytes,
            current_bytes != expected_bytes
        );
        return Err(error_code);
    }

    Ok(())
}

fn is_id_same(
    item_index: usize,
    new_cell_data: &Vec<u8>,
    old_cell_data: &Vec<u8>,
) -> Result<(), Error> {
    is_bytes_eq(
        item_index,
        "id",
        get_id(new_cell_data),
        get_id(old_cell_data),
        Error::ProposalFieldCanNotBeModified,
    )
}

fn is_account_same(
    item_index: usize,
    new_cell_data: &Vec<u8>,
    old_cell_data: &Vec<u8>,
) -> Result<(), Error> {
    is_bytes_eq(
        item_index,
        "account",
        get_account(new_cell_data),
        get_account(old_cell_data),
        Error::ProposalFieldCanNotBeModified,
    )
}

fn is_expired_at_same(
    item_index: usize,
    new_cell_data: &Vec<u8>,
    old_cell_data: &Vec<u8>,
) -> Result<(), Error> {
    is_bytes_eq(
        item_index,
        "expired_at",
        get_expired_at(new_cell_data),
        get_expired_at(old_cell_data),
        Error::ProposalFieldCanNotBeModified,
    )
}

fn is_id_correct(
    item_index: usize,
    new_cell_data: &Vec<u8>,
    old_cell_data: &Vec<u8>,
) -> Result<(), Error> {
    is_bytes_eq(
        item_index,
        "id",
        get_id(new_cell_data),
        get_id(old_cell_data),
        Error::ProposalConfirmIdError,
    )
}

fn is_next_correct(
    item_index: usize,
    new_cell_data: &Vec<u8>,
    proposed_next: AccountIdReader,
) -> Result<(), Error> {
    let expected_next = proposed_next.raw_data();

    is_bytes_eq(
        item_index,
        "next",
        get_next(new_cell_data),
        expected_next,
        Error::ProposalConfirmNextError,
    )
}

fn is_expired_at_correct(
    item_index: usize,
    income: u64,
    current_timestamp: u64,
    new_cell_data: &Vec<u8>,
    pre_account_cell_witness: PreAccountCellDataReader,
) -> Result<(), Error> {
    let account_size = get_account(new_cell_data).len() as u64;
    let cell_storage = ACCOUNT_CELL_BASIC_CAPACITY + (account_size * 100_000_000);
    let price = u64::from(pre_account_cell_witness.price().new());
    let quote = u64::from(pre_account_cell_witness.quote());
    let duration = ((income - cell_storage) / (price / quote * 100_000_000)) * 365 * 86400;

    let current_expired_at = get_expired_at(new_cell_data);
    let mut buf = [0u8; 8];
    buf.copy_from_slice(current_expired_at);
    let expired_at = u64::from_le_bytes(buf);

    if current_timestamp + duration != expired_at {
        debug!(
            "  [{}] Check if outputs[].AccountCell.expired_at: current({}) + duration({}) != expired_at({})",
            item_index,
            current_timestamp,
            duration,
            expired_at
        );
        return Err(Error::ProposalConfirmExpiredAtError);
    }

    Ok(())
}

fn is_account_correct(item_index: usize, new_cell_data: &Vec<u8>) -> Result<(), Error> {
    let expected_account_id = get_id(new_cell_data);
    let account = get_account(new_cell_data);

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
    new_cell_data: &Vec<u8>,
    new_cell_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    let account_id = new_cell_witness_reader.id().raw_data();
    let expected_account_id = get_id(new_cell_data);

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
    new_cell_data: &Vec<u8>,
    new_cell_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    let mut account = new_cell_witness_reader.account().as_readable();
    account.append(&mut ACCOUNT_SUFFIX.as_bytes().to_vec());
    let expected_account = get_account(new_cell_data);

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
    new_cell_witness_reader: AccountCellDataReader,
    old_cell_witness_reader: PreAccountCellDataReader,
) -> Result<(), Error> {
    let owner_lock = new_cell_witness_reader.owner_lock();
    let manager_lock = new_cell_witness_reader.manager_lock();
    let expected_lock = old_cell_witness_reader.owner_lock();

    if !util::is_reader_eq(owner_lock, expected_lock) {
        debug!(
            "  [{}] Check outputs[].AccountCell.owner: {:x?} != {:x?} => {}",
            item_index,
            owner_lock,
            expected_lock,
            !util::is_reader_eq(owner_lock, expected_lock)
        );
        return Err(Error::ProposalConfirmWitnessOwnerError);
    }

    if !util::is_reader_eq(manager_lock, expected_lock) {
        debug!(
            "  [{}] Check outputs[].AccountCell.owner: {:x?} != {:x?} => {}",
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
    new_cell_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    let status = u8::from(new_cell_witness_reader.status());

    if status != AccountStatus::Normal as u8 {
        debug!(
            "  [{}] Check if outputs[].AccountCell.status is normal. (Result: {}, expected: 0)",
            item_index, status
        );
        return Err(Error::ProposalConfirmWitnessManagerError);
    }

    Ok(())
}

fn verify_witness_records(
    item_index: usize,
    new_cell_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    let records = new_cell_witness_reader.records();

    if !records.is_empty() {
        debug!(
            "  [{}] Check if outputs[].AccountCell.records is empty. (Result: {}, expected: true)",
            item_index,
            records.is_empty()
        );
        return Err(Error::ProposalConfirmWitnessRecordsError);
    }

    Ok(())
}
