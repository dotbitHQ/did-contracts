use alloc::boxed::Box;
use alloc::vec::Vec;

use ckb_std::ckb_types::packed::{CellOutput, Script};
use das_types::packed::ActionData;

use super::defult_structs::Action;
use crate::error::ScriptError;
use crate::witness_parser::general_witness_parser::WithMeta;

pub type CellWithMeta = WithMeta<CellOutput>;

pub trait Verification {
    fn verify(&self, contract: &mut dyn Contract) -> Result<(), Box<dyn ScriptError>>;
}

pub trait Contract {
    fn get_input_inner_cells(&self) -> &Vec<CellWithMeta>;
    fn get_input_outer_cells(&self) -> &Vec<CellWithMeta>;
    fn get_output_inner_cells(&self) -> &Vec<CellWithMeta>;
    fn get_output_outer_cells(&self) -> &Vec<CellWithMeta>;
    fn get_this_script(&self) -> &Script;
}

pub trait FSMContract: Contract + Sized {
    fn run_against_action(&mut self, action: &Action) -> Result<(), Box<dyn ScriptError>> {
        let verifications = &action.verifications;
        for v in verifications.iter() {
            v.verify(self)?;
        }

        Ok(())
    }

    fn get_action_data(&self) -> &ActionData;
}
