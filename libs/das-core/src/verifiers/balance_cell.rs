use alloc::boxed::Box;
use alloc::vec::Vec;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_types::constants::DasLockType;
use das_types::packed as das_packed;
use das_types::prelude::{Builder, Entity, Reader};

use crate::constants::*;
use crate::error::*;
use crate::util::{self};
use crate::{code_to_error, data_parser, warn};

pub fn verify_das_lock_always_with_type(
    config_main_reader: das_packed::ConfigCellMainReader,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Check if any cells with das-lock in outputs lack of one of balance-cell-type, account-cell-type, account-sale-cell-type, account-auction-cell-type.");

    let das_lock = das_lock();
    let das_lock_reader = das_lock.as_reader();

    let balance_cell_type_id = config_main_reader.type_id_table().balance_cell();
    let balance_cell_type = das_packed::Script::new_builder()
        .code_hash(balance_cell_type_id.to_entity())
        .hash_type(das_packed::Byte::new(ScriptHashType::Type as u8))
        .build();
    let balance_cell_type_reader = balance_cell_type.as_reader();

    // We need to find all BalanceCells even it has no type script, so we use das-lock as the finding condition.
    let output_cells =
        util::find_cells_by_type_id(ScriptType::Lock, das_lock_reader.code_hash().into(), Source::Output)?;

    let mut available_type_scripts: Vec<das_packed::Script> = Vec::new();
    for index in output_cells {
        let lock = high_level::load_cell_lock(index, Source::Output)?;
        let lock_args = lock.as_reader().args().raw_data();
        let owner_type = data_parser::das_lock_args::get_owner_type(lock_args);
        let manager_type = data_parser::das_lock_args::get_owner_type(lock_args);

        // Check if cells with das-lock in outputs also has the type script named balance-cell-type, account-cell-type, account-sale-cell-type, account-auction-cell-type..
        if owner_type == DasLockType::ETHTypedData as u8 || manager_type == DasLockType::ETHTypedData as u8 {
            let type_opt = high_level::load_cell_type(index, Source::Output)?;
            match type_opt {
                Some(type_) => {
                    let type_reader = type_.as_reader().into();
                    let mut pass = false;
                    if util::is_reader_eq(balance_cell_type_reader, type_reader) {
                        pass = true;
                    } else {
                        if available_type_scripts.is_empty() {
                            debug!("Try to load type ID table from ConfigCellMain, because found some cells with das-lock not using balance-cell-type.");

                            macro_rules! push_type_script {
                                ($type_id_name:ident) => {
                                    let type_id = config_main_reader.type_id_table().$type_id_name();
                                    let type_script = util::type_id_to_script(type_id);
                                    available_type_scripts.push(type_script);
                                };
                            }

                            push_type_script!(account_cell);
                            push_type_script!(account_sale_cell);
                            push_type_script!(account_auction_cell);
                            push_type_script!(offer_cell);
                            push_type_script!(reverse_record_cell);
                            push_type_script!(dpoint_cell);
                        }

                        for script in available_type_scripts.iter() {
                            if util::is_type_id_equal(script.as_reader().into(), type_reader.into()) {
                                pass = true;
                            }
                        }
                    }

                    if !pass {
                        warn!("Outputs[{}] This cell has das-lock, so it should also has one of the specific type scripts.", index);
                        return Err(code_to_error!(ErrorCode::BalanceCellFoundSomeOutputsLackOfType));
                    }
                }
                _ => {
                    warn!(
                        "Outputs[{}] This cell has das-lock, so it should also has one of the specific type scripts.",
                        index
                    );
                    return Err(code_to_error!(ErrorCode::BalanceCellFoundSomeOutputsLackOfType));
                }
            }
        }
    }

    Ok(())
}
