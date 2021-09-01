use super::common::{init, init_without_apply};
use crate::util;
use crate::util::{
    constants::*,
    template_generator::{gen_account_chars, gen_das_lock_args, gen_fake_signhash_all_lock},
    template_parser::TemplateParser,
};
use ckb_testtool::context::Context;
use ckb_tool::ckb_hash::blake2b_256;
use das_core::error::Error;
use das_types::{packed::*, prelude::*};
use std::convert::TryFrom;

#[test]
fn gen_pre_register_simple() {
    let (mut template, account, timestamp) = init("✨das🎉001.bit");
    template.push_config_cell_derived_by_account("✨das🎉001", true, 0, Source::CellDep);

    let (cell_data, entity) = template.gen_pre_account_cell_data(
        account,
        "0x000000000000000000000000000000000000FFFF",
        "0x0000000000000000000000000000000000001100",
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        CKB_QUOTE,
        INVITED_DISCOUNT,
        timestamp,
    );
    template.push_pre_account_cell(
        cell_data,
        Some((1, 0, entity)),
        util::gen_register_fee(8, true),
        Source::Output,
    );

    template.write_template("pre_register.json");
}

test_with_template!(test_pre_register_simple, "pre_register.json");

challenge_with_generator!(
    challenge_pre_register_apply_still_need_wait,
    Error::ApplyRegisterNeedWaitLonger,
    || {
        let (mut template, account, timestamp, height) = init_without_apply("1234567890.bit");
        template.push_config_cell_derived_by_account("1234567890", true, 0, Source::CellDep);

        template.push_apply_register_cell(
            "0x9af92f5e690f4669ca543deb99af8385b12624cc",
            account,
            height,
            timestamp - 60,
            0,
            Source::Input,
        );

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            CKB_QUOTE,
            INVITED_DISCOUNT,
            timestamp - 1,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            util::gen_register_fee(10, true),
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_pre_register_apply_timeout,
    Error::ApplyRegisterHasTimeout,
    || {
        let (mut template, account, timestamp, height) = init_without_apply("1234567890.bit");
        template.push_config_cell_derived_by_account("1234567890", true, 0, Source::CellDep);

        template.push_apply_register_cell(
            "0x9af92f5e690f4669ca543deb99af8385b12624cc",
            account,
            height - 5761,
            timestamp - 60,
            0,
            Source::Input,
        );

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            CKB_QUOTE,
            INVITED_DISCOUNT,
            timestamp - 1,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            util::gen_register_fee(10, true),
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_pre_register_apply_hash_is_invalid,
    Error::PreRegisterApplyHashIsInvalid,
    || {
        let (mut template, account, timestamp, height) = init_without_apply("1234567890.bit");
        template.push_config_cell_derived_by_account("1234567890", true, 0, Source::CellDep);

        template.push_apply_register_cell(
            "0x9af92f5e690f4669ca543deb99af8385b12624cc",
            "000000000", // Different from the account in PreAccountCell, this will cause assertion of hash failure.
            height - 1,
            timestamp - 60,
            0,
            Source::Input,
        );

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            CKB_QUOTE,
            INVITED_DISCOUNT,
            timestamp - 1,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            util::gen_register_fee(10, true),
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_pre_register_invalid_account_id,
    Error::PreRegisterAccountIdIsInvalid,
    || {
        let (mut template, _account, timestamp) = init("1234567890.bit");
        template.push_config_cell_derived_by_account("1234567890", true, 0, Source::CellDep);

        let refund_lock_args = "0x0000000000000000000000000000000000002222";
        let owner_lock_args = "0x000000000000000000000000000000000000FFFF";
        let inviter_lock_args = "0x0000000000000000000000000000000000001111";
        let channel_lock_args = "0x0000000000000000000000000000000000002222";
        let quote = CKB_QUOTE;
        let invited_discount = INVITED_DISCOUNT;
        let created_at = timestamp - 1;

        let account_chars_raw = "1234567890".chars().map(|c| c.to_string()).collect::<Vec<String>>();
        let account_chars = gen_account_chars(account_chars_raw);
        let price = template.get_price(account_chars.len());
        let mut tmp = util::hex_to_bytes(&gen_das_lock_args(owner_lock_args, None));
        tmp.append(&mut tmp.clone());
        let owner_lock_args = Bytes::from(tmp);

        let entity = PreAccountCellData::new_builder()
            .account(account_chars.to_owned())
            .owner_lock_args(owner_lock_args)
            .refund_lock(gen_fake_signhash_all_lock(refund_lock_args))
            .inviter_id(Bytes::from(vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]))
            .inviter_lock(ScriptOpt::from(gen_fake_signhash_all_lock(inviter_lock_args)))
            .channel_lock(ScriptOpt::from(gen_fake_signhash_all_lock(channel_lock_args)))
            .price(price.to_owned())
            .quote(Uint64::from(quote))
            .invited_discount(Uint32::from(invited_discount as u32))
            .created_at(Timestamp::from(created_at))
            .build();

        // The account ID calculated from other account expected to be denied correctly.
        let id = util::account_to_id("0000000000");

        let hash = Hash::try_from(blake2b_256(entity.as_slice()).to_vec()).unwrap();
        let raw = [hash.as_reader().raw_data(), id.as_slice()].concat();
        let cell_data = Bytes::from(raw);

        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            util::gen_register_fee(10, true),
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_pre_register_created_at_mismatch,
    Error::PreRegisterCreateAtIsInvalid,
    || {
        let (mut template, account, timestamp) = init("1234567890.bit");
        template.push_config_cell_derived_by_account("1234567890", true, 0, Source::CellDep);

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            CKB_QUOTE,
            INVITED_DISCOUNT,
            timestamp - 1,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            util::gen_register_fee(10, true),
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_pre_register_invalid_owner_lock_args,
    Error::PreRegisterOwnerLockArgsIsInvalid,
    || {
        let (mut template, account, timestamp) = init("1234567890.bit");
        template.push_config_cell_derived_by_account("1234567890", true, 0, Source::CellDep);

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FF",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            CKB_QUOTE,
            INVITED_DISCOUNT,
            timestamp,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            util::gen_register_fee(10, true),
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_pre_register_quote_mismatch,
    Error::PreRegisterQuoteIsInvalid,
    || {
        let (mut template, account, timestamp) = init("1234567890.bit");
        template.push_config_cell_derived_by_account("1234567890", true, 0, Source::CellDep);

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            CKB_QUOTE - 1,
            INVITED_DISCOUNT,
            timestamp,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            util::gen_register_fee(10, true),
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_pre_register_exceed_account_max_length,
    Error::PreRegisterAccountIsTooLong,
    || {
        let (mut template, account, timestamp) = init("1234567890123456789012345678901234567890123.bit");
        template.push_config_cell_derived_by_account(
            "1234567890123456789012345678901234567890123",
            true,
            0,
            Source::CellDep,
        );

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            CKB_QUOTE,
            INVITED_DISCOUNT,
            timestamp,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            util::gen_register_fee(43, true),
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_pre_register_discount_not_zero_when_no_inviter,
    Error::PreRegisterDiscountIsInvalid,
    || {
        let (mut template, account, timestamp) = init("1234567890.bit");
        template.push_config_cell_derived_by_account("1234567890", true, 0, Source::CellDep);

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "",
            "0x0000000000000000000000000000000000002222",
            CKB_QUOTE,
            INVITED_DISCOUNT,
            timestamp,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            util::gen_register_fee(10, true),
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_pre_register_discount_incorrect,
    Error::PreRegisterDiscountIsInvalid,
    || {
        let (mut template, account, timestamp) = init("1234567890.bit");
        template.push_config_cell_derived_by_account("1234567890", true, 0, Source::CellDep);

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            CKB_QUOTE,
            INVITED_DISCOUNT - 1,
            timestamp,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            util::gen_register_fee(10, true),
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_pre_register_incorrect_price,
    Error::PreRegisterPriceInvalid,
    || {
        let (mut template, _account, timestamp) = init("1234567890.bit");
        template.push_config_cell_derived_by_account("1234567890", true, 0, Source::CellDep);

        let refund_lock_args = "0x0000000000000000000000000000000000002222";
        let owner_lock_args = "0x000000000000000000000000000000000000FFFF";
        let inviter_lock_args = "0x0000000000000000000000000000000000001111";
        let channel_lock_args = "0x0000000000000000000000000000000000002222";
        let quote = CKB_QUOTE;
        let invited_discount = INVITED_DISCOUNT;
        let created_at = timestamp - 1;

        let account_chars_raw = "1234567890".chars().map(|c| c.to_string()).collect::<Vec<String>>();
        let account_chars = gen_account_chars(account_chars_raw);
        let price = template.get_price(4);
        let mut tmp = util::hex_to_bytes(&gen_das_lock_args(owner_lock_args, None));
        tmp.append(&mut tmp.clone());
        let owner_lock_args = Bytes::from(tmp);

        let entity = PreAccountCellData::new_builder()
            .account(account_chars.to_owned())
            .owner_lock_args(owner_lock_args)
            .refund_lock(gen_fake_signhash_all_lock(refund_lock_args))
            .inviter_id(Bytes::from(vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]))
            .inviter_lock(ScriptOpt::from(gen_fake_signhash_all_lock(inviter_lock_args)))
            .channel_lock(ScriptOpt::from(gen_fake_signhash_all_lock(channel_lock_args)))
            .price(price.to_owned())
            .quote(Uint64::from(quote))
            .invited_discount(Uint32::from(invited_discount as u32))
            .created_at(Timestamp::from(created_at))
            .build();

        // The account ID calculated from other account expected to be denied correctly.
        let id = util::account_to_id("0000000000");

        let hash = Hash::try_from(blake2b_256(entity.as_slice()).to_vec()).unwrap();
        let raw = [hash.as_reader().raw_data(), id.as_slice()].concat();
        let cell_data = Bytes::from(raw);

        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            util::gen_register_fee(10, true),
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_pre_register_incorrect_capacity,
    Error::PreRegisterCKBInsufficient,
    || {
        let (mut template, account, timestamp) = init("1234567890.bit");
        template.push_config_cell_derived_by_account("1234567890", true, 0, Source::CellDep);

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            CKB_QUOTE,
            INVITED_DISCOUNT,
            timestamp,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            util::gen_register_fee(10, true) - 1,
            Source::Output,
        );

        template.as_json()
    }
);
