# Witness Data Structure


## Mint/Renew Method and flag Identifier

There are three main Mint methods for sub-accounts, and the Mint method is also the Renew method:
- **Sign Mint** signed by Owner or Manager;
- Self-minting by the user with a certain amount of USD. This Mint method is further divided into the following two categories based on the pricing source:
  - **Custom Script Mint** calculated based on custom scripts;
  - **Custom Rule Mint** calculated based on custom rules.

Correspondence between these different Mint methods and the SubAccountCell.data.flag status identifiers is as follows:

| Status Name | Status Value | Optional Mint Methods |
|:------------:|:------:|:-----------------------------:|
|    Manual    |  0x00  |           Sign Mint           |
| CustomScript |  0x01  | Sign Mint, Custom Script Mint |
|  CustomRule  |  0xff  |  Sign Mint, Custom Rule Mint  |

> When **Sign Mint** and other Mint methods are mixed in a single transaction, priority is given to matching the new account based on `SubAccountMintSign`:
> - If the matching is successful, the registration fee is calculated based on the minimum value provided in `ConfigCellSubAccount.new_sub_account_price`.
> - If the matching fails, the registration fee is calculated based on the logic of **Custom Script Mint** or **Custom Rule Mint**.


## witness Storage Structure

When transactions involve adding, modifying, or deleting sub-accounts, each sub-account needs to have a corresponding witness record. Its basic structure is the same as other DAS witness structures.

```
[
  Signatures Required by Lock Script,
  Signatures Required by Lock Script,
  Signatures Required by Lock Script,
  ...
  [das, type, raw/entity/table],
  [das, type, raw/entity/table],
  [das, type, sub_account_mint_sign],
  [das, type, sub_account_price_rule],
  [das, type, sub_account_price_rule],
  ...
  [das, type, sub_account_preserved_rule],
  [das, type, sub_account_preserved_rule],
  ...
  [das, type, sub_account],
  ...
]
```

The 4 bytes in little-endian encoding at [3:7] represent a u32 integer. It indicates the data type after the 8th byte, which is the sub-account type. For specific values, please refer to [Cell Structure Protocol.md/Type Constants/SubAccount](../system-constant.md).

The last section of data `sub_account` is divided into three categories:

- `SubAccountMintSign` designed for batch minting sub-accounts.
- `SubAccountPriceRule` designed for defining sub-account pricing.
- `SubAccountPreservedRule` designed for defining sub-account reserved lists.
- `SubAccount` designed for creating and editing sub-accounts SMT.

Due to the large data volume and the performance issues encountered with molecule encoding in handling long data in contracts, some types use the following **binary encoding based on LV (Length-Value)**.

```
[ length ][ field_1 ][ length ][ field_2 ] ...

[ length ][ field_1 ]
[ length ][ field_2 ]
...
```

In the described methods above, whether all fields are on the same line or different lines, **there are no separators/newlines in the actual binary data**. The formatting with line breaks is done solely for readability. `[ length ]` is a fixed 4-byte little-endian encoded u32 integer, representing the length of the subsequent data. For example, if the length of the `[ field_1 ]` data segment is 65 bytes, the binary data up to `[ field_1 ]` would appear as follows (**long portions of data are represented as `...` for brevity**):

```
0x00000041FFFFFF...

The above data can be considered in two parts:
0x00000041 0xFFFFFF...

0x00000041 is length of field_1
0xFFFFFF... is field_1 data
```


When the value of a certain segment data is empty, its `length` needs to be `0x00000000`. For example, when the `field_2` segment data is empty, then this piece of binary data is in the following form:
   ```
0x...FFF00000000

The above data can be viewed as 2 parts:
0x...FFF 0x00000000

0x...FFF is the data before length of field_2
0x00000000 is the length of field_2, which indicates that the value of field_2 is empty.
```

### SubAccountMintSign and SubAccountRenewSign Data Structure


To ensure that the transaction can still be executed when split into multiple transactions due to transaction size limitations, only requiring the user to sign once, this signature data structure has been designed for creating sub-accounts.
```
[ length ][ version ]
[ length ][ signature ]
[ length ][ sign_role ]
[ length ][ sign_expired_at ]
[ length ][ account_list_smt_root ]
```

- `version` is the version number of the current data structure, the type is a little-endian encoded u32 integer. When any subsequent fields are changed, this field will be `+1`;
- `signature` is the signature of part of the data of the current witness;
- `sign_role` indicates the role to which the `signature` signature belongs, `0x00` means verifying the signature as the owner, `0x01` means verifying the signature as the manager;
- `sign_expired_at` specifies the `signature` signature expiration period, the type is a little-endian encoded u64 integer. Since multiple split transactions take a while to complete on the chain, this signature can be reused within a period of time, **This The expiration time must be less than or equal to the minimum value between `expired_at` of the parent account and `expired_at` of all newly created sub-accounts**;
- `account_list_smt_root` is the root of [SMT](https://github.com/nervosnetwork/sparse-merkle-tree), and the key is the sub-account’s
  account hash, value is the address of the owner part in the `SubAccountData.lock.args` field after the sub-account is successfully created;
- 
#### sign_expired_at Security

Since this signature can be reused, here is a special explanation of its anti-replay and other security features:

- The expiration time of a sub-account is at least 1 year after it is created, that is, if you create it again within one year, you will find that the sub-account already exists;
- As long as `sign_expired_at` is less than 1 year, the newly created sub-account cannot be created again before it expires;
- The comparison object of `sign_expired_at` is block_header.timestamp of SubAccountCell, so unless SubAccountCell has not been used for a year, the validity of the time is unquestionable;
- 
#### Signature and verification

`signature` will eventually be used with `das-lock` for signature verification, so signature generation and signature verification are standard protocols for CKB, ETH, and BTC chains. The only difference is the generation of `digest`, which consists of concatenating the following fields in order:

- `from did: ` The binary bytes of the string;
- A 32-byte blake2b hash with `ckb-default-hash` as parameter. The creation method is to hash after concatenating the following fields in order:
  - `sign_expired_at`
  - `account_list_smt_root`
  
### SubAccountPriceRule 与 SubAccountPreservedRule 数据结构

These two types of data are structures with a relatively complex hierarchical relationship, so they are mainly stored using molecule encoding. Only a version field is reserved to identify the version number of the molecule structure:
```
[ length ][ version ]
[ length ][ rules ]
```

- `version` is the version number of the current data structure, the type is a little-endian encoded u32 integer. When any subsequent fields are changed, this field will be `+1`;
- `rules` is the `SubAccountRules` type shared by SubAccountPriceRule and SubAccountPreservedRule. The difference is that `SubAccountRule.price` in the SubAccountPreservedRule data structure will be ignored. For the definitions of these two types and their corresponding JSON descriptions, please see [Custom rules ](./custom-price-rule.md

### SubAccount data structure

All additions, deletions and modifications to sub-accounts can ultimately be summarized as modifications to `SubAccountCell.data.smt_root`. Therefore, each piece of this witness data structure can be understood as a modification record of `SubAccountCell.data.smt_root`.

When the `length != 4` of the first field, it needs to be processed according to the `1` version of the data structure:

```
[ length ][ signature ]
[ length ][ sign_role ]
[ length ][ prev_root ]
[ length ][ current_root ]
[ length ][ proof ]
[ length ][ version ]
[ length ][ sub_account ]
[ length ][ edit_key ]
[ length ][ edit_value ]
```


# Witness Data Structure


## Mint/Renew Method and flag Identifier

There are three main Mint methods for sub-accounts, and the Mint method is also the Renew method:
- **Sign Mint** signed by Owner or Manager;
- Self-minting by the user with a certain amount of USD. This Mint method is further divided into the following two categories based on the pricing source:
  - **Custom Script Mint** calculated based on custom scripts;
  - **Custom Rule Mint** calculated based on custom rules.

Correspondence between these different Mint methods and the SubAccountCell.data.flag status identifiers is as follows:

| Status Name | Status Value | Optional Mint Methods |
|:------------:|:------:|:-----------------------------:|
|    Manual    |  0x00  |           Sign Mint           |
| CustomScript |  0x01  | Sign Mint, Custom Script Mint |
|  CustomRule  |  0xff  |  Sign Mint, Custom Rule Mint  |

> When **Sign Mint** and other Mint methods are mixed in a single transaction, priority is given to matching the new account based on `SubAccountMintSign`:
> - If the matching is successful, the registration fee is calculated based on the minimum value provided in `ConfigCellSubAccount.new_sub_account_price`.
> - If the matching fails, the registration fee is calculated based on the logic of **Custom Script Mint** or **Custom Rule Mint**.


## witness Storage Structure

When transactions involve adding, modifying, or deleting sub-accounts, each sub-account needs to have a corresponding witness record. Its basic structure is the same as other DAS witness structures.

```
[
  Signatures Required by Lock Script,
  Signatures Required by Lock Script,
  Signatures Required by Lock Script,
  ...
  [das, type, raw/entity/table],
  [das, type, raw/entity/table],
  [das, type, sub_account_mint_sign],
  [das, type, sub_account_price_rule],
  [das, type, sub_account_price_rule],
  ...
  [das, type, sub_account_preserved_rule],
  [das, type, sub_account_preserved_rule],
  ...
  [das, type, sub_account],
  ...
]
```

The 4 bytes in little-endian encoding at [3:7] represent a u32 integer. It indicates the data type after the 8th byte, which is the sub-account type. For specific values, please refer to [Cell Structure Protocol.md/Type Constants/SubAccount](../system-constant.md).

The last section of data `sub_account` is divided into three categories:

- `SubAccountMintSign` designed for batch minting sub-accounts.
- `SubAccountPriceRule` designed for defining sub-account pricing.
- `SubAccountPreservedRule` designed for defining sub-account reserved lists.
- `SubAccount` designed for creating and editing sub-accounts SMT.

Due to the large data volume and the performance issues encountered with molecule encoding in handling long data in contracts, some types use the following **binary encoding based on LV (Length-Value)**.

```
[ length ][ field_1 ][ length ][ field_2 ] ...

[ length ][ field_1 ]
[ length ][ field_2 ]
...
```

In the described methods above, whether all fields are on the same line or different lines, **there are no separators/newlines in the actual binary data**. The formatting with line breaks is done solely for readability. `[ length ]` is a fixed 4-byte little-endian encoded u32 integer, representing the length of the subsequent data. For example, if the length of the `[ field_1 ]` data segment is 65 bytes, the binary data up to `[ field_1 ]` would appear as follows (**long portions of data are represented as `...` for brevity**):

```
0x00000041FFFFFF...

The above data can be considered in two parts:
0x00000041 0xFFFFFF...

0x00000041 is length of field_1
0xFFFFFF... is field_1 data
```


When the value of a certain segment data is empty, its `length` needs to be `0x00000000`. For example, when the `field_2` segment data is empty, then this piece of binary data is in the following form:
   ```
0x...FFF00000000

The above data can be viewed as 2 parts:
0x...FFF 0x00000000

0x...FFF is the data before length of field_2
0x00000000 is the length of field_2, which indicates that the value of field_2 is empty.
```

### SubAccountMintSign and SubAccountRenewSign Data Structure


To ensure that the transaction can still be executed when split into multiple transactions due to transaction size limitations, only requiring the user to sign once, this signature data structure has been designed for creating sub-accounts.
```
[ length ][ version ]
[ length ][ signature ]
[ length ][ sign_role ]
[ length ][ sign_expired_at ]
[ length ][ account_list_smt_root ]
```

- `version` is the version number of the current data structure, the type is a little-endian encoded u32 integer. When any subsequent fields are changed, this field will be `+1`;
- `signature` is the signature of part of the data of the current witness;
- `sign_role` indicates the role to which the `signature` signature belongs, `0x00` means verifying the signature as the owner, `0x01` means verifying the signature as the manager;
- `sign_expired_at` specifies the `signature` signature expiration period, the type is a little-endian encoded u64 integer. Since multiple split transactions take a while to complete on the chain, this signature can be reused within a period of time, **This The expiration time must be less than or equal to the minimum value between `expired_at` of the parent account and `expired_at` of all newly created sub-accounts**;
- `account_list_smt_root` is the root of [SMT](https://github.com/nervosnetwork/sparse-merkle-tree), and the key is the sub-account’s
  account hash, value is the address of the owner part in the `SubAccountData.lock.args` field after the sub-account is successfully created;
-
#### sign_expired_at Security

Since this signature can be reused, here is a special explanation of its anti-replay and other security features:

- The expiration time of a sub-account is at least 1 year after it is created, that is, if you create it again within one year, you will find that the sub-account already exists;
- As long as `sign_expired_at` is less than 1 year, the newly created sub-account cannot be created again before it expires;
- The comparison object of `sign_expired_at` is block_header.timestamp of SubAccountCell, so unless SubAccountCell has not been used for a year, the validity of the time is unquestionable;
-
#### Signature and verification

`signature` will eventually be used with `das-lock` for signature verification, so signature generation and signature verification are standard protocols for CKB, ETH, and BTC chains. The only difference is the generation of `digest`, which consists of concatenating the following fields in order:

- `from did: ` The binary bytes of the string;
- A 32-byte blake2b hash with `ckb-default-hash` as parameter. The creation method is to hash after concatenating the following fields in order:
  - `sign_expired_at`
  - `account_list_smt_root`

### SubAccountPriceRule 与 SubAccountPreservedRule 数据结构

These two types of data are structures with a relatively complex hierarchical relationship, so they are mainly stored using molecule encoding. Only a version field is reserved to identify the version number of the molecule structure:
```
[ length ][ version ]
[ length ][ rules ]
```

- `version` is the version number of the current data structure, the type is a little-endian encoded u32 integer. When any subsequent fields are changed, this field will be `+1`;
- `rules` is the `SubAccountRules` type shared by SubAccountPriceRule and SubAccountPreservedRule. The difference is that `SubAccountRule.price` in the SubAccountPreservedRule data structure will be ignored. For the definitions of these two types and their corresponding JSON descriptions, please see [Custom rules ](./custom-price-rule.md

### SubAccount data structure

All additions, deletions and modifications to sub-accounts can ultimately be summarized as modifications to `SubAccountCell.data.smt_root`. Therefore, each piece of this witness data structure can be understood as a modification record of `SubAccountCell.data.smt_root`.

When the `length != 4` of the first field, it needs to be processed according to the `1` version of the data structure:

```
[ length ][ signature ]
[ length ][ sign_role ]
[ length ][ prev_root ]
[ length ][ current_root ]
[ length ][ proof ]
[ length ][ version ]
[ length ][ sub_account ]
[ length ][ edit_key ]
[ length ][ edit_value ]
```

When the `length == 4` of the first field, the subsequent version of the data structure is processed according to the number specified by `version`. The current version number is `3`:
```
[ length ][ version ]
[ length ][ action ]
[ length ][ signature ]
[ length ][ sign_role ]
[ length ][ sign_expired_at ]
[ length ][ new_root ]
[ length ][ proof ]
[ length ][ old_sub_account_version ]
[ length ][ new_sub_account_version ]
[ length ][ sub_account ]
[ length ][ edit_key ]
[ length ][ edit_value ]
```

- `version` specifies the data structure version number of the following fields. The type is a little-endian encoded u32 integer. When any subsequent fields are changed, this field will be `+1`;
- `action` is the intention of the current witness. Since the SMT structure must have serious cell preemption problems when it is uploaded to the chain, all sub-account operation types are supported in the same transaction;
- `signature` is the signature field when the sub-account owner edits the sub-account, which contains the signature of `account_id, edit_key, edit_value, nonce` information;
- `sign_role` indicates the role to which the `signature` signature belongs, `0x00` means verifying the signature as the owner, `0x01` means verifying the signature as the manager;
- `sign_expired_at` specifies the expiration time of the `signature` signature, **This expiration time must be less than or equal to the minimum value between the `expired_at` of the parent account and the `expired_at` of the sub-account**;
- `new_root` is the new SMT root modified by the current witness pair `SubAccountCell.data.smt_root`;
- `proof` is the proof of SMT, used to prove that the current `SubAccountCell.data.smt_root` and the modified `SubAccountCell.data.smt_root` are correct;
- `old_sub_account_version` is the version number of the `sub_account` field in the current transaction before the transaction;
- `new_sub_account_version` is the version number of the `sub_account` field in the current transaction after the transaction;
- `sub_account` is a molecule encoding structure that stores sub-account information, also named SubAccount. The detailed data structure can be seen below;
- `edit_key` is a parameter field used with `action`;
- `edit_value` is also a parameter field used with `action`;
```
table SubAccount {
    // The lock of owner and manager
    lock: Script,
    // The first 160 bits of the hash of account.
    id: AccountId,
    // Separate chars of account.
    account: AccountChars,
    // The suffix of this sub-account, it is always .bit currently.
    suffix: Bytes,
    // The sub-account register timestamp.
    registered_at: Uint64,
    // The sub-account expiration timestamp.
    expired_at: Uint64,
    // The status of the account, 0x00 means normal, 0x01 means being sold, 0x02 means being auctioned.
    status: Uint8,
    // Resolving records of this sub-account.
    records: Records,
    // This is a count field, it mainly used to prevent replay attacks.
    nonce: Uint64,
    // If sub-account of sub-account is enabled.
    enable_sub_account: Uint8,
    // The price of renew sub-account of this sub-account.
    renew_sub_account_price: Uint64,
}
```

#### sub_account field data structure

In the witness of the entire sub-account, `sub_account` is a molecule-encoded data structure of the sub-account. The current version is **2** (**For the latest structure, please refer to [das-types](https://github. com/dotbitHQ/das-types) shall prevail**):
```
table SubAccount {
    // The lock of owner and manager
    lock: Script,
    // The first 160 bits of the hash of account.
    id: AccountId,
    // Separate chars of account.
    account: AccountChars,
    // The suffix of this sub-account, it is always .bit currently.
    suffix: Bytes,
    // The sub-account register timestamp.
    registered_at: Uint64,
    // The sub-account expiration timestamp.
    expired_at: Uint64,
    // The status of the account, 0x00 means normal, 0x01 means being sold, 0x02 means being auctioned.
    status: Uint8,
    // Resolving records of this sub-account.
    records: Records,
    // This is a count field, it mainly used to prevent replay attacks.
    nonce: Uint64,
    // If sub-account of sub-account is enabled.
    enable_sub_account: Uint8,
    // The price of renew sub-account of this sub-account.
    renew_sub_account_price: Uint64,
    // The approval that can be fulfilled in the future.
    approval: AccountApproval,
}
```

> Currently, the `lock` field only supports das-lock, that is, the `code_hash` and `hash_type` fields must be completely consistent with the das-lock used on other cells.
>
> The `nonce` field needs to be incremented by 1 every time a transaction that requires a sub-account signature is initiated, so as to prevent replay attacks. Since the value of witness.sub_account.nonce is always the current nonce value,
> If you need to sign a sub-account transaction, you can use the **current nonce value**. If you need to calculate the new sub-account information after the transaction is uploaded to the chain, you need to add **to the current nonce value** .
`, the subsequent version of the data structure will be processed according to the number specified by `version`. The current version number is `3`:
```
[ length ][ version ]
[ length ][ action ]
[ length ][ signature ]
[ length ][ sign_role ]
[ length ][ sign_expired_at ]
[ length ][ new_root ]
[ length ][ proof ]
[ length ][ old_sub_account_version ]
[ length ][ new_sub_account_version ]
[ length ][ sub_account ]
[ length ][ edit_key ]
[ length ][ edit_value ]
```

- `version` specifies the data structure version number of the following fields. The type is a little-endian encoded u32 integer. When any subsequent fields are changed, this field will be `+1`;
- `action` is the intention of the current witness. Since the SMT structure must have serious cell preemption problems when it is uploaded to the chain, all sub-account operation types are supported in the same transaction;
- `signature` is the signature field when the sub-account owner edits the sub-account, which contains the signature of `account_id, edit_key, edit_value, nonce` information;
- `sign_role` indicates the role to which the `signature` signature belongs, `0x00` means verifying the signature as the owner, `0x01` means verifying the signature as the manager;
- `sign_expired_at` specifies the expiration time of the `signature` signature, **This expiration time must be less than or equal to the minimum value between the `expired_at` of the parent account and the `expired_at` of the sub-account**;
- `new_root` is the new SMT root modified by the current witness pair `SubAccountCell.data.smt_root`;
- `proof` is the proof of SMT, used to prove that the current `SubAccountCell.data.smt_root` and the modified `SubAccountCell.data.smt_root` are correct;
- `old_sub_account_version` is the version number of the `sub_account` field in the current transaction before the transaction;
- `new_sub_account_version` is the version number of the `sub_account` field in the current transaction after the transaction;
- `sub_account` is a molecule encoding structure that stores sub-account information, also named SubAccount. The detailed data structure can be seen below;
- `edit_key` is a parameter field used with `action`;
- `edit_value` is also a parameter field used with `action`;
```
table SubAccount {
    // The lock of owner and manager
    lock: Script,
    // The first 160 bits of the hash of account.
    id: AccountId,
    // Separate chars of account.
    account: AccountChars,
    // The suffix of this sub-account, it is always .bit currently.
    suffix: Bytes,
    // The sub-account register timestamp.
    registered_at: Uint64,
    // The sub-account expiration timestamp.
    expired_at: Uint64,
    // The status of the account, 0x00 means normal, 0x01 means being sold, 0x02 means being auctioned.
    status: Uint8,
    // Resolving records of this sub-account.
    records: Records,
    // This is a count field, it mainly used to prevent replay attacks.
    nonce: Uint64,
    // If sub-account of sub-account is enabled.
    enable_sub_account: Uint8,
    // The price of renew sub-account of this sub-account.
    renew_sub_account_price: Uint64,
}
```

#### sub_account field data structure

In the witness of the entire sub-account, `sub_account` is a molecule-encoded data structure of the sub-account. The current version is **2** (**For the latest structure, please refer to [das-types](https://github.com/dotbitHQ/das-types) shall prevail**):
```
table SubAccount {
    // The lock of owner and manager
    lock: Script,
    // The first 160 bits of the hash of account.
    id: AccountId,
    // Separate chars of account.
    account: AccountChars,
    // The suffix of this sub-account, it is always .bit currently.
    suffix: Bytes,
    // The sub-account register timestamp.
    registered_at: Uint64,
    // The sub-account expiration timestamp.
    expired_at: Uint64,
    // The status of the account, 0x00 means normal, 0x01 means being sold, 0x02 means being auctioned.
    status: Uint8,
    // Resolving records of this sub-account.
    records: Records,
    // This is a count field, it mainly used to prevent replay attacks.
    nonce: Uint64,
    // If sub-account of sub-account is enabled.
    enable_sub_account: Uint8,
    // The price of renew sub-account of this sub-account.
    renew_sub_account_price: Uint64,
    // The approval that can be fulfilled in the future.
    approval: AccountApproval,
}
```

> Currently, the `lock` field only supports das-lock, that is, the `code_hash` and `hash_type` fields must be completely consistent with the das-lock used on other cells.
>
> The `nonce` field needs to be incremented by 1 every time a transaction that requires a sub-account signature is initiated, so as to prevent replay attacks. Since the value of witness.sub_account.nonce is always the current nonce value,
> If you need to sign a sub-account transaction, you can use the **current nonce value**. If you need to calculate the new sub-account information after the transaction is uploaded to the chain, you need to add **to the current nonce value** .