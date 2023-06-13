use ckb_types::prelude::{Builder, Entity};
use das_types_std::constants::Source;
use das_types_std::packed::{Byte10, DeviceKey, DeviceKeyList, DeviceKeyListCellData};

use super::{init, BuildRefundLock, DeviceKeyListCell};
use crate::util::template_parser::{test_tx, challenge_tx};
#[test]
fn should_pass_on_normal_add() {
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
fn should_pass_on_normal_remove() {
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


#[test]
fn should_fail_on_multiple_cells() {
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
    input_cell.clone().push(&mut template, Source::Input);
    output_cell.push(&mut template, Source::Output);

    challenge_tx(template.as_json(), 56);
}

#[test]
fn should_fail_on_too_much_capacity_change() {
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
        9_999_900_000,
        refund_lock.clone(),
        DeviceKeyListCellData::new_builder()
            .refund_lock(das_types_std::packed::Script::from_slice(refund_lock.as_slice()).unwrap())
            .keys(DeviceKeyList::new_builder().push(first_device_key).build())
            .build(),
    );

    input_cell.push(&mut template, Source::Input);
    output_cell.push(&mut template, Source::Output);

    challenge_tx(template.as_json(), 63);
}


#[test]
fn should_fail_on_inconsistent_lock() {
    let mut template = init("update_device_key_list");
    let first_device_key = DeviceKey::new_builder().build();
    let second_device_key = DeviceKey::new_builder()
        .cid(Byte10::new_builder().nth0(8.into()).build())
        .build();
    let refund_lock = first_device_key.build_default_refund_lock();

    let input_cell = DeviceKeyListCell::default_new(
        10_000_000_000,
        {
            let mut builder = refund_lock.args().clone().as_builder();
            builder.replace(0, 8.into());
            builder.build()
        },
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

    challenge_tx(template.as_json(), 54);
}


#[test]
fn should_fail_on_multipl_add() {
    let mut template = init("update_device_key_list");
    let first_device_key = DeviceKey::new_builder().build();
    let second_device_key = DeviceKey::new_builder()
        .cid(Byte10::new_builder().nth0(8.into()).build())
        .build();
    let third_device_key = DeviceKey::new_builder()
        .cid(Byte10::new_builder().nth0(9.into()).build())
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
                    .push(third_device_key)
                    .build(),
            )
            .build(),
    );

    input_cell.push(&mut template, Source::Input);
    output_cell.push(&mut template, Source::Output);

    challenge_tx(template.as_json(), 57);
}


#[test]
fn should_fail_on_duplicated_keys() {
    let mut template = init("update_device_key_list");
    let first_device_key = DeviceKey::new_builder().build();
    let second_device_key = first_device_key.clone();
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

    challenge_tx(template.as_json(), 64);
}

#[test]
fn should_fail_on_wrong_order() {
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
                    .push(second_device_key)
                    .push(first_device_key)
                    .build(),
            )
            .build(),
    );

    input_cell.push(&mut template, Source::Input);
    output_cell.push(&mut template, Source::Output);

    challenge_tx(template.as_json(), 58);
}


#[test]
fn should_fail_on_delete2_add1() {
    let mut template = init("update_device_key_list");
    let device_key_1 = DeviceKey::new_builder().build();
    let device_key_2 = DeviceKey::new_builder()
        .cid(Byte10::new_builder().nth0(2.into()).build())
        .build();
    let device_key_3 = DeviceKey::new_builder()
        .cid(Byte10::new_builder().nth0(3.into()).build())
        .build();
    let device_key_4 = DeviceKey::new_builder()
        .cid(Byte10::new_builder().nth0(4.into()).build())
        .build();
    let device_key_5 = DeviceKey::new_builder()
        .cid(Byte10::new_builder().nth0(5.into()).build())
        .build();
    let refund_lock = device_key_1.build_default_refund_lock();

    let input_cell = DeviceKeyListCell::default_new(
        10_000_000_000,
        refund_lock.args(),
        DeviceKeyListCellData::new_builder()
            .refund_lock(das_types_std::packed::Script::from_slice(refund_lock.as_slice()).unwrap())
            .keys(DeviceKeyList::new_builder().push(device_key_1.clone()).push(device_key_2.clone()).push(device_key_3.clone()).push(device_key_4.clone()).build())
            .build(),
    );

    let output_cell = DeviceKeyListCell::default_new(
        9_999_995_000,
        refund_lock.args(),
        DeviceKeyListCellData::new_builder()
            .refund_lock(das_types_std::packed::Script::from_slice(refund_lock.as_slice()).unwrap())
            .keys(
                DeviceKeyList::new_builder()
                    .push(device_key_1)
                    .push(device_key_2)
                    .push(device_key_5)
                    .build(),
            )
            .build(),
    );

    input_cell.push(&mut template, Source::Input);
    output_cell.push(&mut template, Source::Output);

    challenge_tx(template.as_json(), 58);
}