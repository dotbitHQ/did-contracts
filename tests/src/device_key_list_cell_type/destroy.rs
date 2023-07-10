use ckb_types::prelude::{Builder, Entity};
use das_types_std::constants::Source;
use das_types_std::packed::{DeviceKey, DeviceKeyList, DeviceKeyListCellData};

use super::{init, BalanceCell, BuildRefundLock, DeviceKeyListCell};
use crate::util::template_parser::test_tx;
#[test]
fn test_destory() {
    let mut template = init("destroy_device_key_list");
    let device_key = DeviceKey::new_builder().build();
    let refund_lock = device_key.build_default_refund_lock();

    let input_cell = DeviceKeyListCell::default_new(
        10_000_000_000,
        refund_lock.args(),
        DeviceKeyListCellData::new_builder()
            .refund_lock(das_types_std::packed::Script::from_slice(refund_lock.as_slice()).unwrap())
            .keys(DeviceKeyList::new_builder().push(device_key.clone()).build())
            .build(),
    );

    let output_balance_cell = BalanceCell::default_new(90_000_000_000, refund_lock.args());
    input_cell.push(&mut template, Source::Input);
    output_balance_cell.push(&mut template, Source::Output);

    test_tx(template.as_json());
}
