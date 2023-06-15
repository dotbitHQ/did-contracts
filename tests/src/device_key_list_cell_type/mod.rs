use ckb_hash::blake2b_256;
use ckb_types::core::ScriptHashType;
// use ckb_types::bytes::Bytes;
use ckb_types::packed::{Byte32, Bytes, Script};
use ckb_types::prelude::{Builder, Entity};
use das_types_std::constants::{DataType, Source, WITNESS_HEADER};
use das_types_std::packed::{Data, DataEntity, DataEntityOpt, DeviceKey, DeviceKeyListCellData};
use hex::{ToHex, FromHex};
use serde_json::{json, Value};

use crate::util::constants::TYPE_ID_TABLE;
use crate::util::template_generator::{ContractType, TemplateGenerator};


mod create;
mod update;
mod destroy;

#[derive(Debug, Clone)]
pub struct DeviceKeyListCell {
    pub capacity: u64,
    // pub type_: Script,
    pub lock: Script,
    pub witness: DeviceKeyListCellData,
    pub data: Option<Byte32>,
}

impl DeviceKeyListCell {
    fn new(capacity: u64, lock: Script, witness: DeviceKeyListCellData) -> Self {
        Self {
            capacity,
            witness,
            lock,
            data: None,
        }
    }

    fn default_new(capacity: u64, lock_args: Bytes, witness: DeviceKeyListCellData) -> Self {
        Self::simple_new(capacity, "fake-das-lock", lock_args, witness)
    }

    fn simple_new(capacity: u64, lock_hash: impl AsRef<str>, lock_args: Bytes, witness: DeviceKeyListCellData) -> Self {
        let code_hash = Byte32::from_slice(
            Vec::<u8>::from_hex(TYPE_ID_TABLE.get(lock_hash.as_ref()).unwrap().trim_start_matches("0x"))
                .unwrap()
                .as_slice(),
        )
        .unwrap();

        let lock = Script::new_builder()
            .args(lock_args)
            .code_hash(code_hash)
            .hash_type(ScriptHashType::Type.into())
            .build();

        Self {
            capacity,
            witness,
            lock,
            data: None
        }
    }

    fn push(&self, template: &mut TemplateGenerator, source: Source) {
        let cell = json!({
            "capacity": self.capacity,
            "type": {
                "code_hash": "{{device-key-list-cell-type}}",
                "hash_type": "type"
            },
            "lock": {
                "args": format!("{:#x}", self.lock.args().raw_data()),
                "code_hash": format!("{:#x}", self.lock.code_hash().raw_data()),
                "hash_type": "type"
            },
            "tmp_data": format!("0x{}", blake2b_256(self.witness.as_slice()).encode_hex::<String>()),
            "tmp_type": "full"
        });

        let index = template.push_cell_json(cell, source, None);

        let data_entity_opt = DataEntityOpt::new_builder()
            .set(Some(
                DataEntity::new_builder()
                    .entity(self.witness.as_bytes().into())
                    .index(das_types_std::packed::Uint32::from(index as u32))
                    .build(),
            ))
            .build();

        let data = match source {
            Source::Input => Data::new_builder().old(data_entity_opt).build(),
            Source::Output => Data::new_builder().new(data_entity_opt).build(),
            Source::CellDep => Data::new_builder().dep(data_entity_opt).build(),
            _ => unreachable!(),
        };

        let mut outer_witness = Vec::new();
        outer_witness.extend(WITNESS_HEADER);
        outer_witness.extend((das_types_std::constants::DataType::DeviceKeyList as u32).to_le_bytes());
        outer_witness.extend(data.as_slice());

        template
            .outer_witnesses
            .push(format!("0x{}", outer_witness.encode_hex::<String>()));
    }
}

#[derive(Debug, Clone)]
pub struct BalanceCell {
    pub capacity: u64,
    pub lock: Script,
}

impl BalanceCell {
    fn new(capacity: u64, lock: Script) -> Self {
        Self { capacity, lock }
    }

    fn default_new(capacity: u64, lock_args: Bytes) -> Self {
        Self::simple_new(capacity, "always_success", lock_args)
    }

    fn simple_new(capacity: u64, lock_hash: impl AsRef<str>, lock_args: Bytes) -> Self {
        let code_hash = Byte32::from_slice(
            Vec::<u8>::from_hex(TYPE_ID_TABLE.get(lock_hash.as_ref()).unwrap().trim_start_matches("0x"))
                .unwrap()
                .as_slice(),
        )
        .unwrap();

        let lock = Script::new_builder()
            .args(lock_args)
            .code_hash(code_hash)
            .hash_type(ScriptHashType::Type.into())
            .build();

        Self {
            capacity,
            lock
        }
    }

    pub fn push(&self, template: &mut TemplateGenerator, source: Source) {
        let lock = json!({
            "code_hash": format!("0x{}", self.lock.code_hash().raw_data().encode_hex::<String>()),
            "args": format!("0x{}", self.lock.args().raw_data().encode_hex::<String>()),
            "hash_type": "type"
        });

        template.push_cell(self.capacity, lock, Value::Null, None, source);
    }
}

trait BuildLockArg {
    fn build_lock_arg(&self) -> Bytes;
}


impl BuildLockArg for DeviceKey {
    fn build_lock_arg(&self) -> Bytes {
        let args = Bytes::new_builder()
            .push(self.main_alg_id().nth0())
            .push(self.sub_alg_id().nth0())
            .extend(self.pubkey().as_slice().iter().map(|i| (*i).into()))
            .extend(self.cid().as_slice().iter().map(|i| (*i).into()))
            .push(self.main_alg_id().nth0())
            .push(self.sub_alg_id().nth0())
            .extend(self.pubkey().as_slice().iter().map(|i| (*i).into()))
            .extend(self.cid().as_slice().iter().map(|i| (*i).into()))
            .build();
        args
    }
}

trait BuildRefundLock {
    fn build_refund_lock(&self, code_hash: Byte32) -> Script;
    fn build_default_refund_lock(&self) -> Script {
        self.build_refund_lock(name_to_code_hash("always_success"))
    }
}

impl BuildRefundLock for Bytes {
    fn build_refund_lock(&self, code_hash: Byte32) -> Script {
        Script::new_builder()
            .args(self.clone())
            .code_hash(code_hash)
            .hash_type(ScriptHashType::Type.into())
            .build()
    }
}

impl BuildRefundLock for DeviceKey {
    fn build_refund_lock(&self, code_hash: Byte32) -> Script {
        let args = self.build_lock_arg();
        args.build_refund_lock(code_hash)
    }
}

fn init(action_name: impl AsRef<str>) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action_name.as_ref(), None);
    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("device-key-list-cell-type", ContractType::Contract);
    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);

    template
}


fn name_to_code_hash(name: impl AsRef<str>) -> Byte32 {
    Byte32::from_slice(
        Vec::<u8>::from_hex(TYPE_ID_TABLE.get(name.as_ref()).unwrap().trim_start_matches("0x"))
            .unwrap()
            .as_slice(),
    )
    .unwrap()
}

