#![allow(dead_code)]

#[macro_use]
mod util;
mod ckb_types_relay;

#[cfg(test)]
mod account_cell_type;
#[cfg(test)]
mod account_sale_cell_type;
#[cfg(test)]
mod apply_register_cell_type;
#[cfg(test)]
mod balance_cell_type;
#[cfg(test)]
mod config_cell_type;
#[cfg(test)]
mod income_cell_type;
#[cfg(test)]
mod offer_cell_type;
#[cfg(test)]
mod playground;
#[cfg(test)]
mod pre_account_cell_type;
#[cfg(test)]
mod proposal_cell_type;
#[cfg(test)]
mod reverse_record_cell_type;
#[cfg(test)]
mod sub_account_cell_type;
#[cfg(test)]
mod sub_account_witness_parser;
#[cfg(test)]
mod witness_parser;

#[cfg(test)]
mod gen_type_id_table;
