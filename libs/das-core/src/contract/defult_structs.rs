use core::slice::SlicePattern;

use alloc::{string::String, boxed::Box};
use alloc::vec::Vec;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed::Script;
use ckb_std::high_level::{load_cell, QueryIter};
use ckb_std::syscalls::SysError;
use das_types::packed::ActionData;
use molecule::prelude::Entity;

use crate::error::ScriptError;
use crate::witness_parser::general_witness_parser::{WithMeta, Meta};

use super::traits::{Verification, FSMContract, Contract, CellWithMeta};

pub struct Action {
    pub(crate) name: String,
    pub(crate) verifications: Vec<Box<dyn Verification>>,
}

impl Action {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            verifications: Vec::new(),
        }
    }

    pub fn add_verification(&mut self, verification: impl Verification + 'static) {
        self.verifications.push(Box::new(verification))
    }
}

pub struct Rule<T> {
    desc: String,
    verification: T,
}

impl<T> Verification for Rule<T>
where
    T: Fn(&mut dyn Contract) -> Result<(), Box<dyn ScriptError>>,
{
    fn verify(&self, contract: &mut dyn Contract) -> Result<(), Box<dyn ScriptError>> {
        debug!("Start verify: {}", &self.desc);
        (self.verification)(contract)?;
        debug!("Finished verify: {}", &self.desc);
        Ok(())
    }
}

impl<T> Rule<T>
where
    T: Fn(&mut dyn Contract) -> Result<(), Box<dyn ScriptError>>,
{
    pub fn new(desc: impl Into<String>, verification: T) -> Self {
        Self {
            desc: desc.into(),
            verification,
        }
    }
}

impl<T> Verification for T
where
    T: Fn() -> Result<(), Box<dyn ScriptError>>,
{
    fn verify(&self, _contract: &mut dyn Contract) -> Result<(), Box<dyn ScriptError>> {
        self()
    }
}

pub struct MyContract {
    pub registered_actions: Vec<Action>,
    pub action_data: ActionData,
    pub this_script: Script,
    pub input_inner_cells: Vec<CellWithMeta>,
    pub input_outer_cells: Vec<CellWithMeta>,
    pub output_inner_cells: Vec<CellWithMeta>,
    pub output_outer_cells: Vec<CellWithMeta>,
}

// #[derive(Clone, Debug)]
// pub struct CellWithMeta<'a> {
//     pub cell: &'a CellOutput,
//     pub meta: CellMeta,
// }

// #[derive(Clone, Copy, Debug)]
// pub struct CellMeta {
//     pub index: usize,
//     pub source: Source,
// }

// impl <'a> CellWithMeta<'a> {
//     pub fn get_meta(&self) -> CellMeta {
//         self.meta
//     }

//     pub fn new(index: usize, source: Source, cell: &'a CellOutput) -> Self {
//         Self {
//             meta: CellMeta { index, source },
//             cell,
//         }
//     }
// }

// impl <'a> Deref for CellWithMeta<'a> {
//     type Target = CellOutput;

//     fn deref(&self) -> &Self::Target {
//         &self.cell
//     }
// }

impl FSMContract for MyContract {
    fn get_action_data(&self) -> &ActionData {
        &self.action_data
    }
}

impl MyContract {
    pub fn new(action_data: ActionData) -> Result<Self, Box<dyn ScriptError>> {
        fn load_cell_with_meta(index: usize, source: Source) -> Result<CellWithMeta, SysError> {
            load_cell(index, source).map(|cell| WithMeta::new(cell, Meta { index, source }))
        }
        let this_script = ckb_std::high_level::load_script()?;
        let (input_inner_cells, input_outer_cells): (Vec<_>, Vec<_>) =
            QueryIter::new(load_cell_with_meta, Source::Input)
                .partition(|cell| cell.type_().as_slice() == this_script.as_slice());
        let (output_inner_cells, output_outer_cells): (Vec<_>, Vec<_>) =
            QueryIter::new(load_cell_with_meta, Source::Output)
                .partition(|cell| cell.type_().as_slice() == this_script.as_slice());
        Ok(Self {
            registered_actions: Vec::new(),
            action_data,
            this_script,
            input_inner_cells,
            input_outer_cells,
            output_inner_cells,
            output_outer_cells,
        })
    }
}

// pub trait GetCellWitness {
//     fn get_cell_witness<T: Entity>(&self, meta: CellMeta) -> Result<T, Box<dyn ScriptError>>;
// }

// impl GetCellWitness for WitnessesParser {
//     fn get_cell_witness<T: Entity>(&self, meta: CellMeta) -> Result<T, Box<dyn ScriptError>> {
//         let data_type = T::get_type_constant();
//         let (_, _, bytes) = self.verify_and_get(data_type, meta.index, meta.source)?;
//         let res =
//             T::from_compatible_slice(&bytes.raw_data()).map_err(|_| code_to_error!(ErrorCode::VerificationError))?;
//         Ok(res)
//     }
// }

impl Contract for MyContract {
    fn get_input_inner_cells(&self) -> &Vec<CellWithMeta> {
        &self.input_inner_cells
    }

    fn get_input_outer_cells(&self) -> &Vec<CellWithMeta> {
        &self.input_outer_cells
    }

    fn get_output_inner_cells(&self) -> &Vec<CellWithMeta> {
        &self.output_inner_cells
    }

    fn get_output_outer_cells(&self) -> &Vec<CellWithMeta> {
        &self.output_outer_cells
    }

    fn get_this_script(&self) -> &Script {
        &self.this_script
    }

    // fn get_parser(&mut self) -> &mut WitnessesParser {
    //     &mut self.parser
    // }
}

#[derive(Default)]
pub struct RegisteredActions {
    registered_actions: Vec<Action>,
}

impl RegisteredActions {
    pub fn register_action(&mut self, action: Action) {
        self.registered_actions.push(action)
    }

    pub fn get_active_action(&mut self, action_data: &ActionData) -> Option<Action> {
        while let Some(action) = self.registered_actions.pop() {
            if action.name.as_bytes() == action_data.action().raw_data().as_slice() {
                return Some(action);
            }
        }
        None
    }
}

