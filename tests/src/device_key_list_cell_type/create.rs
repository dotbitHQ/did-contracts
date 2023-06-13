use ckb_types::prelude::{Builder, Entity};
use das_types_std::constants::Source;
use das_types_std::packed::{DeviceKey, DeviceKeyList, DeviceKeyListCellData};

use super::{init, BalanceCell, BuildRefundLock, DeviceKeyListCell};
use crate::util::template_parser::{test_tx, challenge_tx};

#[test]
fn should_pass_nomral_create() {
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

#[test]
fn should_fail_on_multiple_outputs() {
    let mut template = init("create_device_key_list");
    let device_key = DeviceKey::new_builder().build();
    let refund_lock = device_key.build_default_refund_lock();
    let witness_data = DeviceKeyListCellData::new_builder()
        .refund_lock(das_types_std::packed::Script::from_slice(refund_lock.as_slice()).unwrap())
        .keys(DeviceKeyList::new_builder().push(device_key).build())
        .build();
    DeviceKeyListCell::default_new(161 * 10u64.pow(8), refund_lock.clone().args(), witness_data.clone())
        .push(&mut template, Source::Output);
    DeviceKeyListCell::default_new(161 * 10u64.pow(8), refund_lock.clone().args(), witness_data.clone())
        .push(&mut template, Source::Output);
    let input_balance_cell = BalanceCell::default_new(180 * 10u64.pow(8), refund_lock.args());
    let output_balance_cell = BalanceCell::default_new(1 * 10u64.pow(8), refund_lock.args());
    input_balance_cell.push(&mut template, Source::Input);
    output_balance_cell.push(&mut template, Source::Output);

    challenge_tx(template.as_json(), 56);
}

#[test]
fn should_fail_on_insufficient_capacity() {
    let mut template = init("create_device_key_list");
    let device_key = DeviceKey::new_builder().build();
    let refund_lock = device_key.build_default_refund_lock();
    let witness_data = DeviceKeyListCellData::new_builder()
        .refund_lock(das_types_std::packed::Script::from_slice(refund_lock.as_slice()).unwrap())
        .keys(DeviceKeyList::new_builder().push(device_key).build())
        .build();
    DeviceKeyListCell::default_new(160 * 10u64.pow(8), refund_lock.clone().args(), witness_data.clone())
        .push(&mut template, Source::Output);
    let input_balance_cell = BalanceCell::default_new(180 * 10u64.pow(8), refund_lock.args());
    let output_balance_cell = BalanceCell::default_new(1 * 10u64.pow(8), refund_lock.args());
    input_balance_cell.push(&mut template, Source::Input);
    output_balance_cell.push(&mut template, Source::Output);

    challenge_tx(template.as_json(), 60);
}

#[test]
fn should_fail_on_inconsistent_refund_lock() {
    let mut template = init("create_device_key_list");
    let device_key = DeviceKey::new_builder().build();
    let refund_lock = device_key.build_default_refund_lock();
    let witness_data = DeviceKeyListCellData::new_builder()
        .refund_lock(das_types_std::packed::Script::from_slice(refund_lock.as_slice()).unwrap())
        .keys(DeviceKeyList::new_builder().push(device_key).build())
        .build();
    DeviceKeyListCell::default_new(161 * 10u64.pow(8), refund_lock.clone().args(), witness_data.clone())
        .push(&mut template, Source::Output);
    let input_balance_cell = BalanceCell::default_new(180 * 10u64.pow(8), refund_lock.clone().as_builder().args(Default::default()).build().args());
    let output_balance_cell = BalanceCell::default_new(1 * 10u64.pow(8), refund_lock.args());
    input_balance_cell.push(&mut template, Source::Input);
    output_balance_cell.push(&mut template, Source::Output);

    challenge_tx(template.as_json(), 62);
}


#[test]
fn should_fail_on_more_than_one_device_key() {
    let mut template = init("create_device_key_list");
    let first_device_key = DeviceKey::new_builder().build();
    let second_device_key = DeviceKey::new_builder().build();
    let refund_lock = first_device_key.build_default_refund_lock();
    let witness_data = DeviceKeyListCellData::new_builder()
        .refund_lock(das_types_std::packed::Script::from_slice(refund_lock.as_slice()).unwrap())
        .keys(DeviceKeyList::new_builder().push(first_device_key).push(second_device_key).build())
        .build();
    DeviceKeyListCell::default_new(161 * 10u64.pow(8), refund_lock.clone().args(), witness_data.clone())
        .push(&mut template, Source::Output);
    let input_balance_cell = BalanceCell::default_new(180 * 10u64.pow(8), refund_lock.clone().as_builder().args(Default::default()).build().args());
    let output_balance_cell = BalanceCell::default_new(1 * 10u64.pow(8), refund_lock.args());
    input_balance_cell.push(&mut template, Source::Input);
    output_balance_cell.push(&mut template, Source::Output);

    challenge_tx(template.as_json(), 57);
}

#[test]
fn should_fail_on_invalid_lock_arg() {
    let mut template = init("create_device_key_list");
    let device_key = DeviceKey::new_builder().build();
    let refund_lock = device_key.build_default_refund_lock();

    let witness_data = DeviceKeyListCellData::new_builder()
        .refund_lock(das_types_std::packed::Script::from_slice(refund_lock.as_slice()).unwrap())
        .keys(DeviceKeyList::new_builder().push(device_key).build())
        .build();

    let mut wrong_lock_arg_builder = refund_lock.clone().args().as_builder();
    wrong_lock_arg_builder.replace(0, 8.into());

    DeviceKeyListCell::default_new(161 * 10u64.pow(8),wrong_lock_arg_builder.build(), witness_data)
        .push(&mut template, Source::Output);
    let input_balance_cell = BalanceCell::default_new(180 * 10u64.pow(8), refund_lock.args());
    let output_balance_cell = BalanceCell::default_new(1 * 10u64.pow(8), refund_lock.args());
    input_balance_cell.push(&mut template, Source::Input);
    output_balance_cell.push(&mut template, Source::Output);

    challenge_tx(template.as_json(), 54);
}