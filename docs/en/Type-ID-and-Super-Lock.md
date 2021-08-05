# Type ID and Super Lock


## Type ID

Type ID is a key concept of contract scripts on CKB blockchain, and all contract script can be divided to **type script** and **lock script** in terms of usage, for more information on these concepts please see [RFC-0022](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0022-transaction-structure/0022-transaction-structure.md#type-id) .

When you need to verify that if a cell uses the official DAS type script, the only correct way is to check `type.code_hash` field of the cell is one of the following values and its `type.hash_type` field is `type`.

- **always-success**: `0x303ead37be5eebfcf3504847155538cb623a26f237609df24bd296750c123078`
- **config-cell-type**: `0x903bff0221b72b2f5d549236b631234b294f10f53e6cc7328af07776e32a6640`
- **account-cell-type**: `0x4f170a048198408f4f4d36bdbcddcebe7a0ae85244d3ab08fd40a80cbfc70918`
- **apply-register-cell-type**: `0xc024b6efde8d49af665b3245223a8aa889e35ede15bc510392a7fea2dec0a758`
- **pre-account-cell-type**: `0x18ab87147e8e81000ab1b9f319a5784d4c7b6c98a9cec97d738a5c11f69e7254`
- **proposal-cell-type**: `0x6127a41ad0549e8574a25b4d87a7414f1e20579306c943c53ffe7d03f3859bbe`
- **income-cell-type**: `0x6c1d69a358954fc471a2ffa82a98aed5a4912e6002a5e761524f2304ab53bf39`

In addition to the above type scripts, DAS also deployed two lock scripts. The only correct way to determine whether a cell is using the official DAS lock script is to check whether its `lock.code_hash` is the following value and whether its `lock.code_hash` is `type`.

- **das-lock**: `0x9376c3b5811942960a846691e16e477cf43d7c7fa654067c9948dfcd09a32137`
- **always-success**: `0x303ead37be5eebfcf3504847155538cb623a26f237609df24bd296750c123078`


## Super Lock

In fact, it is a multi-sign lock script of CKB system, the source code is detailed in [secp256k1-blake160-multisig-all.c](https://github.com/nervosnetwork/ckb-system-scripts/blob/master/c/secp256k1_blake160_multisig_all.c), but its script structure in [molecule encoding](https://github.com/nervosnetwork/molecule) is hard-coded into all contract scripts of DAS, only the signatures that can unlock this lock script can update the DAS source code and configure DAS, so we named it **Super Lock**.

When you need to verify that a transaction is officially signed by DAS, the only correct way is to check if there is a cell in inputs with the following script structure in its `lock` field.

```
Code Hash: 0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8
     Args: 0xc126635ece567c71c50f7482c5db80603852c306
Hash Type: type
```
