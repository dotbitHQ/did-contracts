# Cell structure protocol

## Protocol symbol convention

The following contents of this document will use a unified structure to describe a cell:

```
lock: ...
type: ...
data: ...
witness: ...
```

Among them, `lock`, `type`, `outputs_data` are information that each cell must contain, which can also be seen from the data structure returned by the RPC interface, and `data` is the `outputs_data' corresponding to the cell in this transaction. `. 

You may see the following symbols when describing the cell structure:

| Symbol                 | Description                                                                                                                                                                                                                                                 |
| ------------------- |-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| lock: <...>         | represents a specific script, and its code_hash, args, hash_type have simple conventions, so the details will not be listed                                                                                                                                 |
| type: <...>         | Same as above                                                                                                                                                                                                                                               |
| hash(...)           | refers to the hash value calculated by the data stored here                                                                                                                                                                                                 |
| ======              | In the code segment class describing the cell structure, this delimiter means that the following content is a detailed introduction to the specific molecule structure, but the latest schema please refer to the schemas/ directory in the das-types repo. |
| ConfigCellXXXX.yyyy | Refers to data that needs to be obtained from a specific field in the witness of a ConfigCell. For details, see [ConfigCell](#ConfigCell)                                                                                                                                                                                       |

## Data structure

### DidCell

This is the Cell created after you upgrade a AccountCell. The data of DidCell follows Dob/0 protocol. The account, expire_at and witness_hash information are stored inside the content field defined by Dob/0. The records for DidCell resides in Witness, with its first 20 bytes of blake160 hash stored in cell data. The lock for DidCell can be any valid lock. 


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

DidEntity Schema:

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

SporeData Schema:

```
table SporeData {
    content_type: Bytes,  // empty
    content: Bytes,       // <- account data will store here
    cluster_id: BytesOpt, // 32 bytes hash
}
```

#### The content field encoding in SporeData

```
[reserved; 1byte] // Reserved by Dob/0 protocol. value fixed to 0x00 indicating the content should be treated as raw bytes.
[version; 1byte] // Current value fixed to 0x01.
[witness_hash; 20bytes] // The first 20 bytes of hash(WitnessData) stored inside.
[expire_at; 8bytes] // Little endian u64 encoded time. Unit is second.
[account; n bytes] // The rest of the content is the account name. Utf-8 encoded.
```