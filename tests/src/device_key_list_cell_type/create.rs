use ckb_types::prelude::{Builder, Entity};
use das_types_std::constants::Source;
use das_types_std::packed::{DeviceKey, DeviceKeyList, DeviceKeyListCellData};

use super::{init, BalanceCell, BuildRefundLock, DeviceKeyListCell};
use crate::util::template_parser::test_tx;

#[test]
pub fn test_create() {
    let mut template = init("create_device_key_list");
    let device_key = DeviceKey::new_builder().build();
    let refund_lock = device_key.build_default_refund_lock();

    let witness_data = DeviceKeyListCellData::new_builder()
        .refund_lock(das_types_std::packed::Script::from_slice(refund_lock.as_slice()).unwrap())
        .keys(DeviceKeyList::new_builder().push(device_key).build())
        .build();

    DeviceKeyListCell::default_new(161 * 10u64.pow(8), refund_lock.clone().args(), witness_data)
        .push(&mut template, Source::Output);
    let input_balance_cell = BalanceCell::default_new(180 * 10u64.pow(8), refund_lock.args());
    let output_balance_cell = BalanceCell::default_new(1 * 10u64.pow(8), refund_lock.args());
    input_balance_cell.push(&mut template, Source::Input);
    output_balance_cell.push(&mut template, Source::Output);

    test_tx(template.as_json());
}
