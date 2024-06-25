# Cell 结构协议

## 协议符号约定

本文档以下内容都会采用统一的结构描述一个 cell：

```
lock: ...
type: ...
data: ...
witness: ...
```

其中 `lock`, `type`, `outputs_data` 都是每个 cell 必定包含的信息，从 RPC 接口返回的数据结构中也可以看到，而 `data` 就是这笔交易中与 cell 对应的 `outputs_data` 。

在描述 cell 结构时可能看到以下符号：

| 符号                 | 说明                                                                                                                                            |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| lock: <...>         | 代表一个特定的 script ，其 code_hash, args, hash_type 都有简单的约定，所以就不列举明细了                                                        |
| type: <...>         | 同上                                                                                                                                            |
| hash(...)           | 指代这里存放的数据是通过什么计算得出的 hash 值                                                                                                  |
| ======              | 在描述 cell 结构的代码段类，此分隔符意味着下面的内容是对特定 molecule 结构的详细介绍，但最新的 schema 请以 das-types 仓库中的 schemas/ 目录为准 |
| ConfigCellXXXX.yyyy | 指代数据需要去某个 ConfigCell 的 witness 中的特定字段获取，详见 [ConfigCell](#ConfigCell)                                                       |

## 数据结构

### DidCell

此 Cell 为原来的 AccountCell 升级之后新生成的 Cell. DidCell 的 data 遵循 Dob/0 协议的格式。DidCell具体的 account 和 expireAt 信息在 Dob/0 规定的 content字段中。为了节省存储空间， 此 DidCell 对应的解析记录， 存储在 witness 中的 DidEntity 结构内， 而在 content 中只记录 DidCellWitnessDataV0 的 blake160 hash 的前 20 字节。 DidCell 的 lock 可以是任意lock， 以便和生态内的其他 lock 进行组合和交互。

#### 结构

```
lock: <any-valid-lock-script>
type: 
    code_hash: <did-cell-type>
    hash_type: Type
    args: <generated the same way as type id>
data:
  SporeData // molecule encoded
witness:
  DidEntity // molecule encoded
```

#### Molecule Schema

DidEntity 相关的Schema如下: 

```
table DidEntity {
    data: WitnessData,
    target: CellMetaOpt,  // Indicating which cell this witness should be used for
    hash: Byte20Opt,      // The first 20 bytes of hash(WitnessData). ( Mainly used for easier indexing inside contract code )
}

table DidCellWitnessDataV0 {
    records: Records,
}

// use union to implement version control
union WitnessData {
    DidCellWitnessDataV0,
}

table Record {
    record_type: Bytes,
    record_key: Bytes,
    record_label: Bytes,
    record_value: Bytes,
    record_ttl: Uint32,
}

vector Records <Record>;

struct CellMeta {
    source: byte, // input = 0/output = 1/deps = 2
    index: Uint64,
}

option CellMetaOpt (CellMeta);

```

---------

SporeData 相关的Schema如下:

```
table SporeData {
    content_type: Bytes,  // empty
    content: Bytes,       // <- account data will store here
    cluster_id: BytesOpt, // 32 bytes hash
}
```

#### SporeData 中的 content字段

```
[reserved; 1byte] // Reserved by Dob/0 protocol. value fixed to 0x00 indicating the content should be treated as raw bytes.
[version; 1byte] // Current value fixed to 0x01.
[witness_hash; 20bytes] // The first 20 bytes of hash(WitnessData) stored inside.
[expire_at; 8bytes] // Little endian u64 encoded time. Unit is second.
[account; n bytes] // The rest of the content is the account name. Utf-8 encoded.
```