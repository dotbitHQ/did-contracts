use ckb_types::{bytes, packed::*, prelude::*};

pub fn witness_args_new_builder() -> WitnessArgsBuilder {
    WitnessArgs::new_builder()
}

pub fn witness_args_build(builder: WitnessArgsBuilder) -> WitnessArgs {
    builder.build()
}

pub fn script_new_builder() -> ScriptBuilder {
    Script::new_builder()
}

pub fn script_build(builder: ScriptBuilder) -> Script {
    builder.build()
}

pub fn byte32_new(slice: &[u8]) -> Byte32 {
    Byte32::new_unchecked(bytes::Bytes::from(slice.to_vec()))
}

pub fn to_bytes_opt(bytes: &[u8]) -> BytesOpt {
    BytesOpt::new_builder().set(Some(bytes.pack())).build()
}

pub fn to_slice(entity: impl Entity) -> Vec<u8> {
    entity.as_slice().to_vec()
}
