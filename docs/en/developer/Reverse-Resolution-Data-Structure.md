# Reverse analysis data storage scheme


## Restrictions

This solution is the second version of the reverse analysis design solution, which meets the following requirements:


## SMT structure

[SMT](https://github.com/nervosnetwork/sparse-merkle-tree) is the full name of Sparse Merkle Tree. Since the current on-chain storage solution based on SMT has been further verified, it can meet the above constraints, so it is An SMT-based storage solution is adopted.

The key and value of the leaf of this SMT have a unified convention:

**key** must be the blake2b hash of the payload used to generate addresses for each chain. Considering the space size of 32 bytes, the source and algorithm of the public key are not distinguished here, that is, whether it is ECC, RSA or other unknown algorithms. Public keys can be used, and the only requirement that must be met is that das-lock supports the corresponding signature verification algorithm;

> **Requirements that payload must meet**
>
> Considering that some different chains may have completely different address generation and signature verification methods in the future, the definition of payload needs to be further clarified here. For currently known chains, the address generation process is basically: hash the public key -> intercept part of the hash result -> encode this part of the hash into a string. Because the public key can be deduced from the signature and the original text during the signature verification process, the hash of the public key can be restored during signature verification and compared with the hash in the address to achieve signature verification.
>
> It can be seen that the payload here must meet two restrictions:
>
> 1. It is the data that must be used when generating the address;
> 2. It is data that can be used for signature verification;

**value** , must also be a blake2b hash, its content consists of the following:
- 0..4 bytes, which is the little-endian encoded nonce value. This value is mainly used to prevent the signature from being reused. The value must be +1 for each operation on the same public key;
- 4.. bytes, which is a utf-8 encoded account;

## witness storage structure

Every time a user creates, edits, or deletes a reverse parsing record operation, it corresponds to a witness record. Its basic structure is the same as other witness structures of DAS:

```
[
   The signature required by the lock script,
   The signature required by the lock script,
   The signature required by the lock script,
   ...
   [das, type, raw/entity/table],
   [das, type, raw/entity/table],
   [das, type, reverse_record],
   [das, type, reverse_record],
   ...
]
```

Among them, [3:7] 4 bytes are little-endian encoded u32 integers, which indicate that the data type after the 8th bytes is the sub-account type. For details, see [Witness Type Value DataType] (system-constant.md) .

The last piece of data `reverse_record` is `ReverseRecord` type data that stores the user's reverse parsing operation record. The specific data structure adopts and [sub-account/witness data structure](sub-account/witness-structure.md) The same ** is based on the binary ** form of LV encoding (Length-Value).

### ReverseRecord data structure

The operation on the reverse parsing record is actually an operation on SMT, so it is essentially an update operation on the SMT leaf. Therefore, a witness data structure expressing SMT operation records is as follows:

```
[length][version]
[length][action]
[length][signature]
[ length ][ sign_type ]
[ length ][ address_payload ]
[length][proof]
[ length ][ prev_nonce ]
[ length ][ prev_account ]
[ length ][ next_root ]
[ length ][ next_account ]
```

- `version` is a little-endian encoded u32 integer, indicating the data structure version number of subsequent fields. When any subsequent fields are changed, this field will be `+1`;
- `action` is the intention of the current witness. Since the SMT structure must have serious Cell preemption problems when it is uploaded to the chain, all reverse parsing operation types are supported in the same transaction:
    - `action == update`, indicating that this record is an update operation for reverse parsing, including creation and editing;
    - `action == remove`, indicating that this record is a deletion operation for reverse parsing;
- `signature` is the signature field that verifies the user's ownership of the public key contained in key, which contains the signature of `next_nonce, next_account` information;
- `sign_type` 1 byte identifier, indicating which das-lock algorithm should be used for signature verification:
- `address_payload` is the payload information corresponding to the public key updated in this record. After blake2b hashing, this field should be the key of the leaf of the current SMT;
- `proof` is a proof to verify the existence of `prev_nonce, prev_account` and `prev_nonce + 1, next_account`;
- `prev_nonce` little-endian encoded u32 integer, which is the current nonce value;
- `prev_account` is the current account value;
- `next_root` The new SMT root value when the new nonce and account are written to SMT;
- `next_account` is the new account value to be updated;

> `prev_root`: Since the first prev_root can be obtained from the outputs_data of ReverseRecordRootCell, and the subsequent `prev_root` is the `next_root` of the previous operation record, this field is omitted;
> `next_nonce`: This field is omitted because `next_nonce = prev_nonce + 1`;

#### Create and delete

When a public key is created for reverse parsing, because the original leaf is empty, `prev_nonce` and `prev_account` must also be empty, and `next_nonce` must be 1.

When a public key is deleted and reverse parsed, `next_nonce` and `next_account` must also be empty because there is a security issue with `nonce` resetting TODO.

#### Signature and Verification

`signature` will eventually use signature verification based on `das-lock`, so signature generation and signature verification are standard protocols for CKB, ETH, and BTC chains. The only difference is the generation of `digest`, which consists of concatenating the following fields in order, and finally encoding/decoding them into a Hex type:

- `from did: ` string;
- A 32-byte blake2b hash with `ckb-default-hash` as parameter. The creation method is to hash after concatenating the following fields in order:
    - `prev_nonce + 1`
    - `next_account`