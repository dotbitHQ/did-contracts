# Type ID 以及 Super Lock


## Type ID

Type ID 是 CKB 链上合约相关的一个关键概念，关于什么是 Type ID 请详见 [RFC-0022](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0022-transaction-structure/0022-transaction-structure.md#type-id) 。

当需要验证一个 Cell 是否使用了 DAS 官方合约时，唯一正确的方式就是检查其 `type.code_hash` 是否为下列值且 `type.hash_type` 是否为 `type` 。

- **config-cell-type**: `0x903bff0221b72b2f5d549236b631234b294f10f53e6cc7328af07776e32a6640`
- **account-cell-type**: `0x4f170a048198408f4f4d36bdbcddcebe7a0ae85244d3ab08fd40a80cbfc70918`
- **apply-register-cell-type**: `0xc024b6efde8d49af665b3245223a8aa889e35ede15bc510392a7fea2dec0a758`
- **pre-account-cell-type**: `0x18ab87147e8e81000ab1b9f319a5784d4c7b6c98a9cec97d738a5c11f69e7254`
- **proposal-cell-type**: `0x6127a41ad0549e8574a25b4d87a7414f1e20579306c943c53ffe7d03f3859bbe`
- **income-cell-type**: `0x6c1d69a358954fc471a2ffa82a98aed5a4912e6002a5e761524f2304ab53bf39`


## Super Lock

实际上这就是一个 CKB 系统的多签 lock script，源码详见 [secp256k1-blake160-multisig-all.c](https://github.com/nervosnetwork/ckb-system-scripts/blob/master/c/secp256k1_blake160_multisig_all.c) ，只是它的 Script 结构被硬编码到了 DAS 合约脚本中，只有能够解锁这个 lock script 的签名可以更新 DAS 源码，配置 DAS ，所以我们将其命名为 **Super Lock**。

当需要验证一笔交易是否为 DAS 官方签发时，唯一正确的方式就是检查 inputs 中是否存在某个 Cell 的 `lock` 字段为下列结构。

```
Code Hash: 0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8
     Args: 0xc126635ece567c71c50f7482c5db80603852c306
Hash Type: type
```
