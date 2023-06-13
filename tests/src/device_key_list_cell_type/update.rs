use ckb_types::prelude::{Builder, Entity};
use das_types_std::constants::Source;
use das_types_std::packed::{Byte10, DeviceKey, DeviceKeyList, DeviceKeyListCellData};

use super::{init, BuildRefundLock, DeviceKeyListCell};
use crate::util::template_parser::test_tx;
#[test]
fn test_update_add() {
    let mut template = init("update_device_key_list");
    let first_device_key = DeviceKey::new_builder().build();
    let second_device_key = DeviceKey::new_builder()
        .cid(Byte10::new_builder().nth0(8.into()).build())
        .build();
    let refund_lock = first_device_key.build_default_refund_lock();

    let input_cell = DeviceKeyListCell::default_new(
        10_000_000_000,
        refund_lock.args(),
        DeviceKeyListCellData::new_builder()
            .refund_lock(das_types_std::packed::Script::from_slice(refund_lock.as_slice()).unwrap())
            .keys(DeviceKeyList::new_builder().push(first_device_key.clone()).build())
            .build(),
    );

    let output_cell = DeviceKeyListCell::default_new(
        9_999_995_000,
        refund_lock.args(),
        DeviceKeyListCellData::new_builder()
            .refund_lock(das_types_std::packed::Script::from_slice(refund_lock.as_slice()).unwrap())
            .keys(
                DeviceKeyList::new_builder()
                    .push(first_device_key)
                    .push(second_device_key)
                    .build(),
            )
            .build(),
    );

    input_cell.push(&mut template, Source::Input);
    output_cell.push(&mut template, Source::Output);

    test_tx(template.as_json());
}

#[test]
fn test_update_remove() {
    let mut template = init("update_device_key_list");
    let first_device_key = DeviceKey::new_builder().build();
    let second_device_key = DeviceKey::new_builder()
        .cid(Byte10::new_builder().nth0(8.into()).build())
        .build();
    let refund_lock = first_device_key.build_default_refund_lock();

    let input_cell = DeviceKeyListCell::new(
        10_000_000_000,
        refund_lock.clone(),
        DeviceKeyListCellData::new_builder()
            .refund_lock(das_types_std::packed::Script::from_slice(refund_lock.as_slice()).unwrap())
            .keys(
                DeviceKeyList::new_builder()
                    .push(first_device_key.clone())
                    .push(second_device_key)
                    .build(),
            )
            .build(),
    );

    let output_cell = DeviceKeyListCell::new(
        9_999_995_000,
        refund_lock.clone(),
        DeviceKeyListCellData::new_builder()
            .refund_lock(das_types_std::packed::Script::from_slice(refund_lock.as_slice()).unwrap())
            .keys(DeviceKeyList::new_builder().push(first_device_key).build())
            .build(),
    );

    input_cell.push(&mut template, Source::Input);
    output_cell.push(&mut template, Source::Output);

    test_tx(template.as_json());
}
