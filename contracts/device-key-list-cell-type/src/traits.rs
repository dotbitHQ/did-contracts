use core::ops::Deref;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed::{CellOutput, Script};
use ckb_std::debug;
use ckb_std::high_level::{load_cell, QueryIter};
use ckb_std::syscalls::SysError;
use das_core::code_to_error;
use das_core::constants::ScriptType;
use das_core::error::ScriptError;
use das_core::util::{self, find_cells_by_script_in_inputs_and_outputs};
use das_core::witness_parser::WitnessesParser;
use das_types::constants::{DataType, WITNESS_HEADER_BYTES, WITNESS_TYPE_BYTES};
use das_types::packed::{ActionData, DeviceKeyListCellData};
use molecule::prelude::Entity;

use crate::error::ErrorCode;
use crate::helpers::GetDataType;

pub struct Action {
    name: String,
    verifications: Vec<Box<dyn Verification>>,
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

pub trait Verification {
    fn verify(&self, contract: &mut MyContract) -> Result<(), Box<dyn ScriptError>>;
}

pub struct Rule<T> {
    desc: String,
    verification: T,
}

impl<T> Verification for Rule<T>
where
    T: Fn(&mut MyContract) -> Result<(), Box<dyn ScriptError>>,
{
    fn verify(&self, contract: &mut MyContract) -> Result<(), Box<dyn ScriptError>> {
        debug!("Start verify: {}", &self.desc);
        (self.verification)(contract)?;
        debug!("Finished verify: {}", &self.desc);
        Ok(())
    }
}

impl<T> Rule<T>
where
    T: Fn(&mut MyContract) -> Result<(), Box<dyn ScriptError>>,
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
    fn verify(&self, contract: &mut MyContract) -> Result<(), Box<dyn ScriptError>> {
        self()
    }
}

// impl<T> Verification for T
// where
//     T: Fn(&mut MyContract) -> Result<(), Box<dyn ScriptError>>,
// {
//     fn verify(&self, contract: &mut MyContract) -> Result<(), Box<dyn ScriptError>> {
//         self(contract)
//     }
// }

pub trait FSMContract {
    fn register_action(&mut self, action: Action);
    fn parse_action_with_params(&mut self) -> Result<(), Box<dyn ScriptError>>;
    fn get_cell_witness<T: Entity>(&self, cell: &CellWithMeta) -> Result<T, Box<dyn ScriptError>>;
    fn run(&mut self) -> Result<(), Box<dyn ScriptError>>;
    fn dispatch(&mut self) -> Option<Action>;
}

pub struct MyContract {
    pub registered_actions: Vec<Action>,
    pub action_data: ActionData,
    pub parser: WitnessesParser,
    pub this_script: Script,
    pub input_inner_cells: Vec<CellWithMeta>,
    pub input_outer_cells: Vec<CellWithMeta>,
    pub output_inner_cells: Vec<CellWithMeta>,
    pub output_outer_cells: Vec<CellWithMeta>,
}

pub struct CellWithMeta(pub usize, pub Source, pub CellOutput);

impl Deref for CellWithMeta {
    type Target = CellOutput;

    fn deref(&self) -> &Self::Target {
        &self.2
    }
}

impl FSMContract for MyContract {
    fn register_action(&mut self, action: Action) {
        self.registered_actions.push(action)
    }

    fn parse_action_with_params(&mut self) -> Result<(), Box<dyn ScriptError>> {
        let witness = util::load_das_witnesses(self.parser.witnesses[0].0)?;
        let action_data = ActionData::from_slice(witness.get(WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES..).unwrap())
            .map_err(|_| code_to_error!(ErrorCode::VerificationError))?;
        self.action_data = action_data;
        Ok(())
    }

    fn dispatch(&mut self) -> Option<Action> {
        while let Some(action) = self.registered_actions.pop() {
            if action.name.as_bytes() == self.action_data.action().as_slice() {
                return Some(action);
            }
        }
        None
    }

    fn get_cell_witness<T: Entity>(&self, cell: &CellWithMeta) -> Result<T, Box<dyn ScriptError>> {
        let data_type = T::get_type_constant();
        let (_, _, bytes) = self.parser.verify_and_get(data_type, cell.0, cell.1)?;
        let res =
            T::from_compatible_slice(bytes.as_slice()).map_err(|_| code_to_error!(ErrorCode::VerificationError))?;

        Ok(res)
    }

    fn run(&mut self) -> Result<(), Box<dyn ScriptError>> {
        let action = self.dispatch().ok_or(code_to_error!(ErrorCode::ActionNotSupported))?;
        let verifications = &action.verifications;

        for verification in verifications.iter() {
            verification.verify(self)?;
        }

        Ok(())
    }
}

impl MyContract {
    pub fn new() -> Result<Self, Box<dyn ScriptError>> {
        let mut parser = WitnessesParser::new()?;
        parser.parse_cell()?;

        fn load_cell_with_meta(index: usize, source: Source) -> Result<CellWithMeta, SysError> {
            load_cell(index, source).map(|cell|CellWithMeta(index, source, cell))
        }
        let this_script = ckb_std::high_level::load_script()?;
        let (input_inner_cells, input_outer_cells): (Vec<_>, Vec<_>) =
            QueryIter::new(load_cell_with_meta, Source::Input)
                .partition(|cell| cell.2.type_().as_slice() == this_script.as_slice());
        let (output_inner_cells, output_outer_cells): (Vec<_>, Vec<_>) =
            QueryIter::new(load_cell_with_meta, Source::Output)
                .partition(|cell| cell.2.type_().as_slice() == this_script.as_slice());

        let witness = util::load_das_witnesses(parser.witnesses[0].0)?;
        let action_data = ActionData::from_slice(witness.get(WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES..).unwrap())
            .map_err(|_| code_to_error!(ErrorCode::VerificationError))?;

        Ok(Self {
            registered_actions: Vec::new(),
            action_data,
            parser,
            this_script,
            input_inner_cells,
            input_outer_cells,
            output_inner_cells,
            output_outer_cells
        })
    }
}
