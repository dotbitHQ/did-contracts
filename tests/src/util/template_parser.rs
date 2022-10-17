use std::cell::Cell;
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use std::{env, fs};

use ckb_chain_spec::consensus::TYPE_ID_CODE_HASH;
use ckb_hash::blake2b_256;
use ckb_mock_tx_types::*;
use ckb_script::TransactionScriptsVerifier;
use ckb_types::core::cell::{resolve_transaction, ResolvedTransaction};
use ckb_types::core::{Cycle, HeaderView, ScriptHashType, TransactionBuilder, TransactionView};
use ckb_types::packed::*;
use ckb_types::prelude::*;
use ckb_types::{bytes, H256};
use das_types_std::{
    constants::Source,
    // packed::{Script, ScriptOpt},
    prelude::{Builder, Entity},
};
use serde_json::Value;

use super::constants::*;
use super::error::*;
use super::util;
use crate::util::template_generator::TemplateGenerator;

const BINARY_VERSION: &str = "BINARY_VERSION";

pub enum BinaryVersion {
    Debug,
    Release,
}

impl FromStr for BinaryVersion {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(BinaryVersion::Debug),
            "release" => Ok(BinaryVersion::Release),
            _ => Err("Environment variable BINARY_VERSION only support \"debug\" and \"release\"."),
        }
    }
}

pub fn test_tx(tx: Value) {
    // println!("Transaction template: {}", serde_json::to_string_pretty(&tx).unwrap());
    let mut parser = TemplateParser::from_data(tx, 350_000_000);
    match parser.try_parse() {
        Ok(_) => match parser.execute_tx() {
            Ok((cycles, tx_view)) => {
                println!(
                    r#"︎↑︎======================================↑︎
Transaction size: {} bytes,
   Suggested fee: {} shannon(feeRate: 1)
          Cycles: {}
========================================"#,
                    tx_view.data().total_size(),
                    tx_view.data().total_size() + 4,
                    cycles
                );
            }
            Err(e) => {
                panic!(
                    "\n======\nThe transaction should pass the test, but it failed in script: {}\n======\n",
                    e.to_string()
                );
            }
        },
        Err(e) => {
            panic!(
                "\n======\nParse the template of transaction failed: {}\n======\n",
                e.to_string()
            );
        }
    }
}

pub fn challenge_tx(tx: Value, expected_error: impl Into<i8> + Clone + Debug) {
    // println!("Transaction template: {}", serde_json::to_string_pretty(&tx).unwrap());
    let mut parser = TemplateParser::from_data(tx, 350_000_000);
    let error_code: i8 = expected_error.clone().into();
    match parser.try_parse() {
        Ok(_) => match parser.execute_tx() {
            Ok(_) => {
                panic!(
                    "\n======\nThe test should failed with error code: {:?}({}), but it returns Ok.\n======\n",
                    expected_error, error_code
                );
            }
            Err(err) => {
                let msg = err.to_string();
                println!("Error message(single code): {}", msg);

                let search = format!("error code {}", error_code);
                assert!(
                    msg.contains(search.as_str()),
                    "\n======\nThe test should failed with error code: {:?}({})\n======\n",
                    expected_error,
                    error_code
                );
            }
        },
        Err(e) => {
            panic!(
                "\n======\nParse the template of transaction failed: {}\n======\n",
                e.to_string()
            );
        }
    }
}

// another style of text_tx/challenge_tx
pub fn test_tx2(tx: fn() -> TemplateGenerator) {
    test_tx(tx().as_json())
}

pub fn challenge_tx2(expected_error: ErrorCode, tx: fn() -> TemplateGenerator) {
    challenge_tx(tx().as_json(), expected_error)
}

pub struct TemplateParser {
    template: Value,
    type_id_map: HashMap<String, Byte32>,
    tx_builder: Cell<TransactionBuilder>,
    mock_header_deps: Vec<HeaderView>,
    mock_cell_deps: Vec<MockCellDep>,
    mock_inputs: Vec<MockInput>,
    max_cycles: u64,
}

impl TemplateParser {
    pub fn new(raw_json: &str, max_cycles: u64) -> Result<Self, Box<dyn StdError>> {
        let template = serde_json::from_str(raw_json)?;

        Ok(TemplateParser {
            template,
            type_id_map: TemplateParser::init_type_id_map(),
            tx_builder: Cell::new(TransactionBuilder::default()),
            mock_header_deps: vec![],
            mock_cell_deps: vec![],
            mock_inputs: vec![],
            max_cycles,
        })
    }

    pub fn from_file(filepath: String, max_cycles: u64) -> Result<Self, Box<dyn StdError>> {
        let mut raw_json = String::new();
        File::open(filepath)?.read_to_string(&mut raw_json)?;
        let template = serde_json::from_str(&raw_json)?;

        Ok(TemplateParser {
            template,
            type_id_map: TemplateParser::init_type_id_map(),
            tx_builder: Cell::new(TransactionBuilder::default()),
            mock_header_deps: vec![],
            mock_cell_deps: vec![],
            mock_inputs: vec![],
            max_cycles,
        })
    }

    pub fn from_data(template: Value, max_cycles: u64) -> Self {
        TemplateParser {
            template,
            type_id_map: TemplateParser::init_type_id_map(),
            tx_builder: Cell::new(TransactionBuilder::default()),
            mock_header_deps: vec![],
            mock_cell_deps: vec![],
            mock_inputs: vec![],
            max_cycles,
        }
    }

    fn init_type_id_map() -> HashMap<String, Byte32, RandomState> {
        // The type IDs here are testing only.
        let mut type_id_map = HashMap::new();
        for (&key, &val) in TYPE_ID_TABLE.iter() {
            type_id_map.insert(key.to_string(), util::hex_to_byte32(val).unwrap());
        }

        type_id_map
    }

    pub fn try_parse(&mut self) -> Result<(), Box<dyn StdError>> {
        let to_owned = |v: &Vec<Value>| -> Vec<Value> { v.to_owned() };

        if let Some(header_deps) = self.template["header_deps"].as_array().map(to_owned) {
            self.parse_header_deps(header_deps)?
        }
        if let Some(cell_deps) = self.template["cell_deps"].as_array().map(to_owned) {
            self.parse_cell_deps(cell_deps)?
        }
        if let Some(inputs) = self.template["inputs"].as_array().map(to_owned) {
            self.parse_inputs(inputs)?
        }
        if let Some(outputs) = self.template["outputs"].as_array().map(to_owned) {
            self.parse_outputs(outputs)?
        }
        if let Some(witnesses) = self.template["witnesses"].as_array().map(to_owned) {
            self.parse_witnesses(witnesses)?
        }

        Ok(())
    }

    pub fn execute_tx(&mut self) -> Result<(Cycle, TransactionView), String> {
        let builder = self.tx_builder.take();
        let tx = builder.build();
        let mock_info = MockInfo {
            header_deps: self.mock_header_deps.drain(0..).collect(),
            cell_deps: self.mock_cell_deps.drain(0..).collect(),
            inputs: self.mock_inputs.drain(0..).collect(),
        };
        let mock_tx = MockTransaction {
            mock_info,
            tx: tx.data(),
        };

        let resource = Resource::from_both(&mock_tx, DummyResourceLoader {})?;
        let rtx: ResolvedTransaction = {
            let mut seen_inputs = HashSet::new();
            resolve_transaction(tx.clone(), &mut seen_inputs, &resource, &resource)
                .map_err(|err| format!("Resolve transaction error: {:?}", err))?
        };
        let mut verifier = TransactionScriptsVerifier::new(&rtx, &resource);
        verifier.set_debug_printer(Box::new(|hash: &Byte32, message: &str| {
            println!("Script(0x{}): {}", hex::encode(&hash.as_slice()[..6]), message);
        }));

        match verifier.verify(self.max_cycles) {
            Ok(cycles) => Ok((cycles, tx)),
            Err(err) => Err(format!("Verify script error: {:?}", err.to_string())),
        }
    }

    /// The header_deps should be an array of objects like below:
    ///
    /// ```json
    /// [
    ///     {
    ///         "version": "0x0"
    ///         "number": "0x1c526b",
    ///         "timestamp": "0x179f5e91cb9",
    ///         "epoch": "0x50903410008fb",
    ///         "transactions_root": "0xd5439ebffae718cab0fc837fb7b03a06253c250bcae8a2933ac820580a675560",
    ///         "hash": "0x24e33cfffb2658d32ed61b55ee156ad7288e0980b72416bf3a9ce11ecc2c737a",
    ///         "extra_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
    ///         "nonce": "0xb13000f771a12a98d82d62d2d6dfe382",
    ///         "parent_hash": "0x9fed43a51ae94c039e29602b46d25483c7b6a46cbce48559a3040440e6c12d5d",
    ///         "proposals_hash": "0x1bc5abcadf5b34bcc85b00c4c4afd0d6b01d0c0ca11f8ca34241838d12b9df04",
    ///         "compact_target": "0x1d17319c",
    ///         "dao": "0xece20aa8185bb5368aedb6e329ee24001603723bfefdae0100290badf10d4507",
    ///     },
    ///     ...
    /// ]
    /// ```
    ///
    /// These fields are all optional, and it will be compiled to a RawHeader object in molecule finally.
    fn parse_header_deps(&mut self, header_deps: Vec<Value>) -> Result<(), Box<dyn StdError>> {
        let mut mocked_header_deps = vec![];

        for (i, item) in header_deps.into_iter().enumerate() {
            let version = util::parse_json_u32(&format!("header_deps[{}].version", i), &item["version"], Some(0));
            let number = if item["number"].is_null() {
                util::parse_json_u64(&format!("header_deps[{}].height", i), &item["height"], Some(0))
            } else {
                util::parse_json_u64(&format!("header_deps[{}].number", i), &item["number"], Some(0))
            };
            let timestamp = util::parse_json_u64(&format!("header_deps[{}].timestamp", i), &item["timestamp"], Some(0));
            let epoch = util::parse_json_u64(&format!("header_deps[{}].epoch", i), &item["epoch"], Some(0));

            let transactions_root_raw = util::parse_json_hex_with_default(
                &format!("header_deps[{}].transactions_root", i),
                &item["transactions_root"],
                vec![0u8; 32],
            );
            let transactions_root = match Byte32::from_slice(&transactions_root_raw) {
                Ok(transactions_root) => transactions_root,
                Err(err) => return Err(format!("Parse transactions_root error: {:?}", err).into()),
            };

            let raw_header = RawHeaderBuilder::default()
                .version(version.pack())
                .number(number.pack())
                .timestamp(timestamp.pack())
                .epoch(epoch.pack())
                .transactions_root(transactions_root)
                .build();
            let header = Header::new_builder().raw(raw_header).nonce(Uint128::default()).build();
            let header_view = header.into_view();
            let hash = header_view.hash();
            self.mock_header_deps.push(header_view);

            mocked_header_deps.push(hash);
        }

        let builder = self.tx_builder.take();
        self.tx_builder.set(builder.set_header_deps(mocked_header_deps));

        Ok(())
    }

    fn parse_cell_deps(&mut self, cell_deps: Vec<Value>) -> Result<(), Box<dyn StdError>> {
        let mut mocked_cell_deps = vec![];

        for (i, item) in cell_deps.into_iter().enumerate() {
            match item["tmp_type"].as_str() {
                Some("contract") | Some("deployed_contract") | Some("shared_lib") | Some("deployed_shared_lib") => {
                    let tmp_type = item["tmp_type"].as_str().expect("The tmp_type field is required.");
                    let is_deployed = if tmp_type.contains("deployed") { true } else { false };
                    let is_shared_lib = if tmp_type.contains("shared_lib") { true } else { false };

                    let name = item["tmp_file_name"].as_str().unwrap();
                    let (_type_id, _out_point, cell_dep, cell_output, cell_data) =
                        self.mock_contract(name, is_deployed, is_shared_lib, Some(i));
                    // println!("{:>30}: {}", name, type_id);

                    let mock_cell_dep = MockCellDep {
                        cell_dep: cell_dep.clone(),
                        output: cell_output,
                        data: cell_data,
                        header: None,
                    };
                    self.mock_cell_deps.push(mock_cell_dep);

                    mocked_cell_deps.push(cell_dep);
                }
                Some("full") => {
                    // If we use {{...}} variable in cell_deps, then the contract need to be put in the cell_deps either.
                    // This is because variable is not a real code_hash, but everything needs code_hash here, so the
                    // contract need to be loaded for calculating hash.
                    let (capacity, lock_script, type_script, cell_data) = self
                        .parse_cell(item.clone(), Source::CellDep)
                        .map_err(|err| format!("Field `cell_deps[{}]` parse failed: {}", i, err.to_string()))?;

                    // Generate static out point for debugging purposes.
                    let out_point = self.mock_out_point(i);
                    let cell_dep = CellDep::new_builder().out_point(out_point.clone()).build();
                    let cell_output = CellOutput::new_builder()
                        .capacity(capacity.pack())
                        .lock(lock_script)
                        .type_(ScriptOpt::new_builder().set(type_script).build())
                        .build();

                    let mock_cell_dep = MockCellDep {
                        cell_dep: cell_dep.clone(),
                        output: cell_output,
                        data: cell_data,
                        header: None,
                    };
                    self.mock_cell_deps.push(mock_cell_dep);

                    mocked_cell_deps.push(cell_dep);
                }
                _ => {
                    return Err("Unsupported cell_deps type.".into());
                }
            }
        }

        let builder = self.tx_builder.take();
        self.tx_builder.set(builder.set_cell_deps(mocked_cell_deps));

        Ok(())
    }

    fn parse_inputs(&mut self, inputs: Vec<Value>) -> Result<(), Box<dyn StdError>> {
        let mut mocked_inputs = vec![];

        for (i, item) in inputs.into_iter().enumerate() {
            match item["previous_output"]["tmp_type"].as_str() {
                Some("full") => {
                    // parse inputs[].previous_output as a mock cell
                    let (capacity, lock_script, type_script, cell_data) = self
                        .parse_cell(item["previous_output"].clone(), Source::Input)
                        .map_err(|err| {
                            format!(
                                "Field `inputs[{}].previous_output` parse failed: {}",
                                i,
                                err.to_string()
                            )
                        })?;
                    // parse inputs[].since
                    let since = util::parse_json_u64("cell.inputs.since", &item["since"], Some(0));

                    // Generate static out point for debugging purposes, and it use the space of 1_000_000 to u64::Max.
                    let out_point = self.mock_out_point(i + 1_000_000);
                    let cell_input = CellInput::new_builder()
                        .previous_output(out_point.clone())
                        .since(since.pack())
                        .build();
                    let cell_output = CellOutput::new_builder()
                        .capacity(capacity.pack())
                        .lock(lock_script)
                        .type_(ScriptOpt::new_builder().set(type_script).build())
                        .build();

                    let mock_input = MockInput {
                        input: cell_input.clone(),
                        output: cell_output,
                        data: cell_data,
                        header: None,
                    };
                    self.mock_inputs.push(mock_input);

                    mocked_inputs.push(cell_input);
                }
                _ => {
                    return Err("Unsupported inputs type.".into());
                }
            }
        }

        let builder = self.tx_builder.take();
        self.tx_builder.set(builder.set_inputs(mocked_inputs));

        // eprintln!("Parse self.inputs = {:#?}", self.inputs);
        Ok(())
    }

    fn parse_outputs(&mut self, outputs: Vec<Value>) -> Result<(), Box<dyn StdError>> {
        let mut mocked_outputs = Vec::new();
        let mut mocked_outputs_data = Vec::new();

        for (i, item) in outputs.into_iter().enumerate() {
            match item["tmp_type"].as_str() {
                Some("full") => {
                    // parse inputs[].previous_output as a mock cell
                    let (capacity, lock_script, type_script, cell_data) = self
                        .parse_cell(item.clone(), Source::Output)
                        .map_err(|err| format!("Field `outputs[{}]` parse failed: {}", i, err.to_string()))?;

                    let cell_output = CellOutput::new_builder()
                        .capacity(capacity.pack())
                        .lock(lock_script)
                        .type_(ScriptOpt::new_builder().set(type_script).build())
                        .build();

                    mocked_outputs.push(cell_output);
                    mocked_outputs_data.push(cell_data);
                }
                _ => {
                    return Err("Unsupported outputs type.".into());
                }
            }
        }

        let mut builder = self.tx_builder.take();
        builder = builder.set_outputs(mocked_outputs);
        builder = builder.set_outputs_data(mocked_outputs_data.into_iter().map(|data| data.pack()).collect());
        self.tx_builder.set(builder);

        // eprintln!("Parse self.outputs = {:#?}", self.outputs);
        // eprintln!("Parse self.outputs_data = {:#?}", self.outputs_data);
        Ok(())
    }

    fn parse_witnesses(&mut self, witnesses: Vec<Value>) -> Result<(), Box<dyn StdError>> {
        let mut mocked_witnesses = Vec::new();

        for (_i, witness) in witnesses.into_iter().enumerate() {
            let data: bytes::Bytes = witness.as_str().map(|hex| util::hex_to_bytes_2(hex)).unwrap();
            mocked_witnesses.push(data.pack());
        }

        let builder = self.tx_builder.take();
        self.tx_builder.set(builder.set_witnesses(mocked_witnesses));

        // eprintln!("Parse self.witnesses = {:#?}", self.witnesses);
        Ok(())
    }

    fn parse_cell(
        &self,
        cell: Value,
        source: Source,
    ) -> Result<(u64, Script, Option<Script>, bytes::Bytes), Box<dyn StdError>> {
        // parse capacity of cell
        let capacity = util::parse_json_u64("cell.capacity", &cell["capacity"], None);

        // parse lock script and type script of cell
        let lock_script = self
            .parse_script(cell["lock"].clone(), source)
            .map_err(|err| format!("Field `cell.lock` parse failed: {}", err.to_string()))?;
        let type_script = self
            .parse_script(cell["type"].clone(), source)
            .map_err(|err| format!("Field `cell.type` parse failed: {}", err.to_string()))?;

        // parse data of cell
        let data;
        if let Some(hex) = cell["tmp_data"].as_str() {
            data = util::hex_to_bytes_2(hex);
        } else {
            data = bytes::Bytes::new();
        }

        Ok((capacity, lock_script.unwrap(), type_script, data))
    }

    fn parse_script(&self, script_val: Value, source: Source) -> Result<Option<Script>, Box<dyn StdError>> {
        if script_val.is_null() {
            return Ok(None);
        }

        let script: Option<Script>;
        if let Some(code_hash) = script_val["code_hash"].as_str() {
            // If code_hash is variable like {{xxx}}, then parse script field as deployed contract,
            let real_code_hash;
            if let Some(caps) = RE_VARIABLE.captures(code_hash) {
                let script_name = caps.get(1).map(|m| m.as_str()).unwrap();
                real_code_hash = match self.type_id_map.get(script_name) {
                    Some(code_hash) => code_hash.to_owned(),
                    _ => {
                        if source == Source::CellDep {
                            Byte32::default()
                        } else {
                            return Err(format!("not found script {}", script_name).into());
                        }
                    }
                };
                // Tip: If contract can not find some cell by type ID, you can uncomment the following line to ensure transaction has correct type ID.
                // println!("Replace code_hash {} with {} .", script_name, real_code_hash);

                // else parse script field by field.
            } else {
                let code_hash_str: &str = script_val["code_hash"]
                    .as_str()
                    .expect("The code_hash field is required.");
                real_code_hash = util::hex_to_byte32(code_hash_str)?;
            }

            let mut args: String = script_val["args"].as_str().unwrap_or("").to_string();
            if !args.is_empty() {
                // If args is not empty, try to find and replace variables in args.
                if let Some(caps) = RE_VARIABLE.captures(&args) {
                    let script_name = caps.get(1).map(|m| m.as_str()).unwrap();
                    let code_hash = if source == Source::CellDep {
                        Byte32::default()
                    } else {
                        match self.type_id_map.get(script_name) {
                            Some(code_hash) => code_hash.to_owned(),
                            _ => return Err(format!("not found script {}", script_name).into()),
                        }
                    };

                    args = args.replace(
                        &format!("{{{{{}}}}}", script_name),
                        &hex_string(code_hash.as_reader().raw_data()),
                    );
                }
            }

            let hash_type = match script_val["hash_type"].as_str() {
                Some("data") => ScriptHashType::Data,
                _ => ScriptHashType::Type,
            };

            script = Some(
                Script::new_builder()
                    .code_hash(real_code_hash)
                    .hash_type(hash_type.into())
                    .args(bytes::Bytes::from(util::hex_to_bytes(&args)).pack())
                    .build(),
            );
        } else {
            return Err("The code_hash field is required.".into());
        }

        Ok(script)
    }

    fn mock_out_point(&self, index: usize) -> OutPoint {
        let tx_hash = index_to_byte32(index);
        OutPoint::new_builder().index(0u32.pack()).tx_hash(tx_hash).build()
    }

    fn mock_contract(
        &self,
        binary_name: &str,
        is_deployed: bool,
        is_shared_lib: bool,
        index_opt: Option<usize>,
    ) -> (Byte32, OutPoint, CellDep, CellOutput, bytes::Bytes) {
        let file = self.load_binary(binary_name, is_deployed);

        let code_hash;
        let cell_output;
        if is_shared_lib {
            let hash = blake2b_256(file.clone());
            let mut inner = [Byte::new(0); 32];
            for (i, item) in hash.iter().enumerate() {
                inner[i] = Byte::new(*item);
            }
            code_hash = Byte32::new_builder().set(inner).build();
            cell_output = CellOutput::new_builder()
                .capacity(0u64.pack())
                .lock(Script::default())
                .type_(ScriptOpt::new_builder().set(None).build())
                .build();
        } else {
            let args = {
                // Padding args to 32 bytes, because it is convenient to use 32 bytes as the real args are also 32 bytes.
                let mut buf = [0u8; 32];
                let len = buf.len();
                let bytes = binary_name.as_bytes();
                if bytes.len() >= len {
                    buf.copy_from_slice(&bytes[..32]);
                } else {
                    let (_, right) = buf.split_at_mut(len - bytes.len());
                    right.copy_from_slice(bytes);
                }

                buf
            };
            let args_bytes = args.iter().map(|v| Byte::new(*v)).collect::<Vec<_>>();
            let type_ = Script::new_builder()
                .code_hash(Byte32::new_unchecked(bytes::Bytes::from(TYPE_ID_CODE_HASH.as_bytes())))
                .hash_type(ScriptHashType::Type.into())
                .args(Bytes::new_builder().set(args_bytes).build())
                .build();

            code_hash = type_.calc_script_hash();
            cell_output = CellOutput::new_builder()
                .capacity(0u64.pack())
                .lock(Script::default())
                .type_(ScriptOpt::new_builder().set(Some(type_)).build())
                .build();
            // Uncomment the line below can print type ID of each script in unit tests.
            // println!(
            //     "script: {}, type_id: {}, args: {}",
            //     binary_name,
            //     type_id,
            //     hex_string(binary_name.as_bytes())
            // );
        }

        let out_point = self.mock_out_point(index_opt.unwrap_or(rand::random::<usize>()));
        let cell_dep = CellDep::new_builder().out_point(out_point.clone()).build();

        (code_hash, out_point, cell_dep, cell_output, file)
    }

    fn load_binary(&self, name: &str, is_deployed: bool) -> bytes::Bytes {
        let current_dir = env::current_dir().unwrap();
        let mut file_path = PathBuf::new();

        if is_deployed {
            file_path.push(current_dir);
            file_path.push("..");
            file_path.push("deployed-scripts");
        } else {
            let binary_version = match env::var(BINARY_VERSION) {
                Ok(val) => val.parse().expect("Binary version should be one of debug and release."),
                Err(_) => BinaryVersion::Debug,
            };
            let binary_dir = match binary_version {
                BinaryVersion::Debug => "debug",
                BinaryVersion::Release => "release",
            };

            file_path.push(current_dir);
            file_path.push("..");
            file_path.push("build");
            file_path.push(binary_dir);
        }

        file_path.push(name);

        fs::read(file_path.as_path())
            .expect(&format!(
                "Can not load binary of {} from path {}.",
                name,
                file_path.display()
            ))
            .into()
    }
}

fn index_to_byte32(index: usize) -> Byte32 {
    let index_bytes = (index as u64).to_be_bytes().to_vec();
    let padding_bytes = [
        vec![0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        index_bytes,
    ]
    .concat();

    Byte32::from_slice(&padding_bytes).expect("The Byte32::from_slice(&tx_hash_bytes) should always succeed.")
}

pub struct DummyResourceLoader {}

impl MockResourceLoader for DummyResourceLoader {
    fn get_header(&mut self, hash: H256) -> Result<Option<HeaderView>, String> {
        return Err(format!("Header {:x} is missing!", hash));
    }

    fn get_live_cell(
        &mut self,
        out_point: OutPoint,
    ) -> Result<Option<(CellOutput, bytes::Bytes, Option<Byte32>)>, String> {
        return Err(format!("Cell: {:?} is missing!", out_point));
    }
}
