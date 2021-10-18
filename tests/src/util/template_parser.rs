use super::{constants::*, util};
use ckb_testtool::context::Context;
use ckb_tool::{
    ckb_error, ckb_jsonrpc_types as rpc_types,
    ckb_types::{
        bytes,
        bytes::Bytes,
        core::{Cycle, ScriptHashType, TransactionBuilder, TransactionView},
        h256, packed,
        packed::*,
        prelude::*,
        H160, H256,
    },
};
use serde_json::Value;
use std::{
    collections::{hash_map::RandomState, HashMap, HashSet},
    error::Error,
    fs::File,
    io::Read,
};

pub struct TemplateParser {
    pub context: Context,
    pub data: Value,
    pub header_deps: Vec<Byte32>,
    pub contracts: HashMap<String, Byte32>,
    pub deps: Vec<CellDep>,
    pub inputs: Vec<CellInput>,
    pub outputs: Vec<CellOutput>,
    pub outputs_data: Vec<packed::Bytes>,
    pub witnesses: Vec<packed::Bytes>,
}

impl std::fmt::Debug for TemplateParser {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("TemplateParser")
            .field("contracts", &self.contracts)
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs)
            .field("outputs_data", &self.outputs_data)
            .finish()
    }
}

impl TemplateParser {
    pub fn new(context: Context, raw_json: &str) -> Result<Self, Box<dyn Error>> {
        let data: Value = serde_json::from_str(raw_json)?;

        Ok(TemplateParser {
            context,
            data,
            header_deps: Vec::new(),
            contracts: TemplateParser::init_contracts(),
            deps: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            outputs_data: Vec::new(),
            witnesses: Vec::new(),
        })
    }

    pub fn from_file(context: Context, filepath: String) -> Result<Self, Box<dyn Error>> {
        let mut raw_json = String::new();
        File::open(filepath)?.read_to_string(&mut raw_json)?;
        let data: Value = serde_json::from_str(&raw_json)?;

        Ok(TemplateParser {
            context,
            data,
            header_deps: Vec::new(),
            contracts: TemplateParser::init_contracts(),
            deps: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            outputs_data: Vec::new(),
            witnesses: Vec::new(),
        })
    }

    pub fn from_data(context: Context, data: serde_json::Value) -> Self {
        TemplateParser {
            context,
            data,
            header_deps: Vec::new(),
            contracts: TemplateParser::init_contracts(),
            deps: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            outputs_data: Vec::new(),
            witnesses: Vec::new(),
        }
    }

    fn init_contracts() -> HashMap<String, Byte32, RandomState> {
        // The type IDs here are testing only.
        let mut contracts = HashMap::new();
        for (&key, &val) in TYPE_ID_TABLE.iter() {
            contracts.insert(key.to_string(), util::hex_to_byte32(val).unwrap());
        }

        contracts
    }

    pub fn parse(&mut self) -> () {
        if let Err(e) = self.try_parse() {
            panic!("{}", e.to_string());
        }
    }

    pub fn try_parse(&mut self) -> Result<(), Box<dyn Error>> {
        let to_owned = |v: &Vec<Value>| -> Vec<Value> { v.to_owned() };

        if let Some(header_deps) = self.data["header_deps"].as_array().map(to_owned) {
            self.parse_header_deps(header_deps)?
        }
        if let Some(cell_deps) = self.data["cell_deps"].as_array().map(to_owned) {
            self.parse_cell_deps(cell_deps)?
        }
        if let Some(inputs) = self.data["inputs"].as_array().map(to_owned) {
            self.parse_inputs(inputs)?
        }
        if let Some(outputs) = self.data["outputs"].as_array().map(to_owned) {
            self.parse_outputs(outputs)?
        }
        if let Some(witnesses) = self.data["witnesses"].as_array().map(to_owned) {
            self.parse_witnesses(witnesses)?
        }

        Ok(())
    }

    pub fn build_tx(&mut self) -> TransactionView {
        TransactionBuilder::default()
            .header_deps(self.header_deps.clone())
            .cell_deps(self.deps.clone())
            .inputs(self.inputs.clone())
            .outputs(self.outputs.clone())
            .outputs_data(self.outputs_data.clone())
            .set_witnesses(self.witnesses.clone())
            .build()
    }

    pub fn execute_tx_directly(&mut self) -> Result<Cycle, ckb_error::Error> {
        let tx = self.build_tx();
        println!(
            "\nTransaction size: {} bytes, Suggested fee: {} shannon(feeRate: 1)",
            tx.data().total_size(),
            tx.data().total_size() + 4
        );
        self.context.verify_tx(&tx, MAX_CYCLES)
    }

    pub fn execute_tx(&mut self, tx: TransactionView) -> Result<Cycle, ckb_error::Error> {
        println!(
            "\nTransaction size: {} bytes, Suggested fee: {} shannon(feeRate: 1)",
            tx.data().total_size(),
            tx.data().total_size() + 4
        );
        self.context.verify_tx(&tx, MAX_CYCLES)
    }

    pub fn set_outputs_data(&mut self, i: usize, data: packed::Bytes) {
        self.outputs_data[i] = data;

        // eprintln!("Set self.outputs_data = {:#?}", self.outputs_data);
    }

    pub fn set_witnesses(&mut self, i: usize, data: packed::Bytes) {
        if self.witnesses.len() < i + 1 {
            let mut j = i + 1 - self.witnesses.len();
            loop {
                self.witnesses.push(packed::Bytes::default());
                j -= 1;

                if j <= 0 {
                    break;
                }
            }
        }

        self.witnesses[i] = data;

        // eprintln!("self.witnesses = {:#?}", self.witnesses);
    }

    pub fn sign_by_keys(&mut self, private_keys: Vec<&str>) -> Result<(), Box<dyn Error>> {
        // TODO Support sign transaction in tests
        for key in private_keys {
            self.sign_by_key(key)?
        }

        Ok(())
    }

    pub fn sign_by_key(&mut self, private_key: &str) -> Result<(), Box<dyn Error>> {
        // TODO Support sign transaction in tests
        let mut signer = util::get_privkey_signer(private_key);
        let input_size = self.inputs.len();

        let mut witnesses = if self.witnesses.len() <= 0 {
            self.inputs.iter().map(|_| packed::Bytes::default()).collect::<Vec<_>>()
        } else {
            self.witnesses.clone()
        };

        for ((code_hash, args), idxs) in self.group_inputs()?.into_iter() {
            if code_hash != SIGHASH_TYPE_HASH.pack() && code_hash != MULTISIG_TYPE_HASH.pack() {
                continue;
            }
            if args.len() != 20 && args.len() != 28 {
                return Err("SignErr: lock.args length is mismatched".into());
            }

            let mut lock_args: HashSet<H160> = HashSet::default();
            lock_args.insert(H160::from_slice(&args[..]).unwrap());

            if signer(&lock_args, &h256!("0x0"), &Transaction::default().into())?.is_some() {
                let transaction = self.build_tx();
                let signature = util::build_signature(
                    &transaction,
                    input_size,
                    &idxs,
                    &witnesses,
                    |message: &H256, tx: &rpc_types::Transaction| {
                        signer(&lock_args, message, tx).map(|sig| sig.unwrap())
                    },
                )?;

                if signature.len() != SECP_SIGNATURE_SIZE {
                    return Err("SignErr: Signature length is mismatched".into());
                }

                witnesses[idxs[0]] = WitnessArgs::new_builder()
                    .lock(Some(signature).pack())
                    .build()
                    .as_bytes()
                    .pack();
            }
        }

        self.witnesses = witnesses;
        // eprintln!("self.witnesses = {:#?}", self.witnesses);
        Ok(())
    }

    fn group_inputs(&self) -> Result<HashMap<(Byte32, Bytes), Vec<usize>>, Box<dyn Error>> {
        let mut groups: HashMap<(Byte32, Bytes), Vec<usize>> = HashMap::default();
        for (idx, cell_input) in self.inputs.iter().enumerate() {
            let (cell_output, _) = self.context.get_cell(&cell_input.previous_output()).unwrap();
            let code_hash = cell_output.lock().code_hash();
            let args = cell_output.lock().args().as_slice().get(4..).unwrap().to_owned();
            let lock_args = Bytes::from(args).to_owned();

            groups.entry((code_hash, lock_args)).or_default().push(idx);
        }

        Ok(groups)
    }

    fn parse_header_deps(&mut self, header_deps: Vec<Value>) -> Result<(), Box<dyn Error>> {
        for (i, item) in header_deps.iter().enumerate() {
            let header_hash = item["tmp_hash"]
                .as_str()
                .expect(&format!("Field `header_deps[{}].tmp_hash` is required.", i));

            let number: u64;
            if item["number"].is_number() {
                number = item["number"]
                    .as_u64()
                    .ok_or(format!("Field `header_deps[{}].number` is required.", i))?;
            } else {
                let hex = item["number"]
                    .as_str()
                    .ok_or(format!("Field `header_deps[{}].number` is required.", i))?;
                number = util::hex_to_u64(hex)
                    .expect(&format!("Field `header_deps[{}].number` is not valid u64 in hex.", i));
            }

            let timestamp: u64;
            if item["timestamp"].is_number() {
                timestamp = item["timestamp"]
                    .as_u64()
                    .ok_or(format!("Field `header_deps[{}].timestamp` is required.", i))?;
            } else {
                let hex = item["timestamp"]
                    .as_str()
                    .ok_or(format!("Field `header_deps[{}].timestamp` is required.", i))?;
                timestamp = util::hex_to_u64(hex).expect(&format!(
                    "Field `header_deps[{}].timestamp` is not valid u64 in hex.",
                    i
                ));
            }

            util::mock_header_deps(&mut self.context, util::hex_to_byte32(header_hash)?, number, timestamp);
        }

        Ok(())
    }

    fn parse_cell_deps(&mut self, cell_deps: Vec<Value>) -> Result<(), Box<dyn Error>> {
        for (i, item) in cell_deps.into_iter().enumerate() {
            match item["tmp_type"].as_str() {
                Some("contract") => {
                    let name = item["tmp_file_name"].as_str().unwrap();
                    let (type_id, _, cell_dep) = util::deploy_dev_contract(&mut self.context, name, Some(i));
                    // println!("{:>30}: {}", name, type_id);
                    self.deps.push(cell_dep);
                    self.contracts.insert(name.to_string(), type_id);
                }
                Some("deployed_contract") => {
                    let name = item["tmp_file_name"].as_str().unwrap();
                    let (type_id, _, cell_dep) = util::deploy_builtin_contract(&mut self.context, name, Some(i));
                    // println!("{:>30}: {}", name, type_id);
                    self.deps.push(cell_dep);
                    self.contracts.insert(name.to_string(), type_id);
                }
                Some("shared_lib") => {
                    let name = item["tmp_file_name"].as_str().unwrap();
                    let (code_hash, _, cell_dep) = util::deploy_shared_lib(&mut self.context, name, Some(i));
                    // println!("{:>30}: {}", name, type_id);
                    self.deps.push(cell_dep);
                    self.contracts.insert(name.to_string(), code_hash);
                }
                Some("full") => {
                    // If we use {{...}} variable in cell_deps, then the contract need to be put in the cell_deps either.
                    // This is because variable is not a real code_hash, but everything needs code_hash here, so the
                    // contract need to be loaded for calculating hash.
                    let (capacity, lock_script, type_script, data) = self
                        .parse_cell(item.clone(), Source::CellDep)
                        .map_err(|err| format!("Field `cell_deps[]` parse failed: {}", err.to_string()))?;
                    // Generate static out point for debugging purposes.
                    let out_point = util::mock_out_point(i);
                    util::mock_cell_with_outpoint(
                        &mut self.context,
                        out_point.clone(),
                        capacity,
                        lock_script,
                        type_script,
                        data,
                    );

                    let cell_dep = CellDep::new_builder().out_point(out_point).build();
                    self.deps.push(cell_dep);
                }
                _ => {
                    return Err("Unsupported cell_deps type.".into());
                }
            }
        }

        // eprintln!("Parse self.contracts = {:#?}", self.contracts);
        Ok(())
    }

    fn parse_inputs(&mut self, inputs: Vec<Value>) -> Result<(), Box<dyn Error>> {
        for (i, item) in inputs.into_iter().enumerate() {
            match item["previous_output"]["tmp_type"].as_str() {
                Some("full") => {
                    // parse inputs[].previous_output as a mock cell
                    let (capacity, lock_script, type_script, data) = self
                        .parse_cell(item["previous_output"].clone(), Source::Input)
                        .map_err(|err| format!("Field `inputs[].previous_output` parse failed: {}", err.to_string()))?;
                    // Generate static out point for debugging purposes, and it use the space of 1_000_000 to u64::Max.
                    let out_point = util::mock_out_point(i + 1_000_000);
                    util::mock_cell_with_outpoint(
                        &mut self.context,
                        out_point.clone(),
                        capacity,
                        lock_script,
                        type_script,
                        data,
                    );

                    // parse input.since
                    let since;
                    if item["since"].is_number() {
                        since = item["since"].as_u64();
                    } else {
                        let hex = item["since"].as_str();
                        since = hex
                            .map(|hex| util::hex_to_u64(hex).expect("Field `inputs[].since` is not valid u64 in hex."));
                    }

                    // TODO implement context.link_cell_with_block

                    self.inputs.push(util::mock_input(out_point, since));
                }
                _ => {
                    return Err("Unsupported inputs type.".into());
                }
            }
        }

        // eprintln!("Parse self.inputs = {:#?}", self.inputs);
        Ok(())
    }

    fn parse_outputs(&mut self, outputs: Vec<Value>) -> Result<(), Box<dyn Error>> {
        for item in outputs {
            match item["tmp_type"].as_str() {
                Some("full") => {
                    // parse inputs[].previous_output as a mock cell
                    let (capacity, lock_script, type_script, data) = self
                        .parse_cell(item.clone(), Source::Output)
                        .map_err(|err| format!("Field `outputs[]` parse failed: {}", err.to_string()))?;

                    let cell: CellOutput = CellOutput::new_builder()
                        .capacity(capacity.pack())
                        .lock(lock_script)
                        .type_(ScriptOpt::new_builder().set(type_script).build())
                        .build();

                    self.outputs.push(cell);
                    self.outputs_data.push(data.unwrap_or_default().pack());
                }
                _ => {
                    return Err("Unsupported outputs type.".into());
                }
            }
        }

        // eprintln!("Parse self.outputs = {:#?}", self.outputs);
        // eprintln!("Parse self.outputs_data = {:#?}", self.outputs_data);
        Ok(())
    }

    fn parse_cell(
        &mut self,
        cell: Value,
        source: Source,
    ) -> Result<(u64, Script, Option<Script>, Option<Vec<u8>>), Box<dyn Error>> {
        // parse capacity of cell
        let capacity: u64;
        if cell["capacity"].is_number() {
            capacity = cell["capacity"].as_u64().ok_or("Field `cell.capacity` is required.")?;
        } else {
            let hex = cell["capacity"].as_str().ok_or("Field `cell.capacity` is required.")?;
            capacity = util::hex_to_u64(hex).expect("Field `cell.capacity` is not valid u64 in hex.");
        }

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
            data = Some(util::hex_to_bytes(hex))
        } else {
            data = None;
        }

        Ok((capacity, lock_script.unwrap(), type_script, data))
    }

    fn parse_script(&self, script_val: Value, source: Source) -> Result<Option<Script>, Box<dyn Error>> {
        if script_val.is_null() {
            return Ok(None);
        }

        let script: Option<Script>;
        if let Some(code_hash) = script_val["code_hash"].as_str() {
            // If code_hash is variable like {{xxx}}, then parse script field as deployed contract,
            let real_code_hash;
            if let Some(caps) = RE_VARIABLE.captures(code_hash) {
                let script_name = caps.get(1).map(|m| m.as_str()).unwrap();
                real_code_hash = match self.contracts.get(script_name) {
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
                // println!("Replace code_hash {} with {} .", code_hash, real_code_hash);

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
                        match self.contracts.get(script_name) {
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

    fn parse_witnesses(&mut self, witnesses: Vec<Value>) -> Result<(), Box<dyn Error>> {
        for (_i, witness) in witnesses.into_iter().enumerate() {
            let data = witness
                .as_str()
                .map(|hex| bytes::Bytes::from(util::hex_to_bytes(hex)))
                .unwrap();

            self.witnesses.push(data.pack());
        }

        // eprintln!("Parse self.witnesses = {:#?}", self.witnesses);
        Ok(())
    }
}
