#Data storage solution

## Data storage location

In the early design, the data was designed to be stored in the `outputs_data` field of the cell. However, when some more complex standard transactions were designed, this plan was rejected. This is because the larger the cell size, the more CKB the user needs to pay. When the price of CKB rises, the cell size can easily reach 100+ Bytes, which will bring a serious financial burden to the user.

So now the `outputs_data` field of the cell only stores a small amount of fixed-length data, and the data content is directly parsed according to the fixed length. More complex data is stored using the `witnesses` field of the transaction, and only a hash is stored in the `outputs_data` field value of the cell as data verification. When data needs to be updated, because the cell will be used as input and will also appear in output, new and old data need to be provided at the same time to verify that the old data part provided in `witnesses` is indeed consistent with the data provided last time, and the new data part is consistent with the last data provided. The hash stored in the cell in the current output is consistent.

> See RFC [Transaction Data Structure](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0022-transaction-structure/0022-transaction-structure.md) for details.

## witness storage structure

Because the `witnesses` of the transaction is an array, and the current system script needs to use a one-to-one location corresponding to the inputs to store the signature, the following encoding method is adopted that is compatible with the existing rules of `witnesses`:

```
[
   The signature required by the lock script,
   The signature required by the lock script,
   The signature required by the lock script,
   ...
   [das, type, raw/entity/table],
   [das, type, raw/entity/table],
   [das, type, raw/entity/table],
   ...
]
```
As shown above, witnesses is an array, and signatures need to reserve space consistent with the number of inputs for storing signatures. Therefore, the relevant data of the DAS script needs to be calculated and reserved before signing, and then `push` to the end of the `witnesses` array** after calculating and reserving the location required for the signature script. Each array element represents an independent piece of data, and each piece of data follows the following structure:

- Use simple binary encoding for each piece of data;
- [0:3] The first 3 bytes have a fixed value of `0x646173`, which is the three-letter ascii encoding of `das`, indicating that the following data is DAS system data;
- [3:7] 4 bytes are little-endian encoded u32 integer, which is the identification of the data type after the 8th bytes. For specific values, see [Cell Structure Protocol.md/Type constant list] (#Cell Structure Protocol .md);
- [7:] Starting from the 8th byte, there are variable data structures fixed in several types. The specific type will be parsed based on the previous 4 bytes;

In addition to the above structure, DAS-related witnesses must also follow the following rules:

1. The first DAS data block appearing in `witnesses` must be a special ActionData structure:

```
// At this time, the [3:7] bytes value of this data block is 0x00000000, that is: ActionData, and then the data content of [7:] is:
witness:
   table Action {
     action: Bytes,
     params: Bytes,
   }
```

2. The second and subsequent data in `witnesses` must all be DAS data blocks, and there must be no other data;
### Parsing of variable data structures

For the variable data after [7:] bytes of the DAS data block, there are three main types:

- The raw type, that is, a pure binary data type, is a data type specifically optimized for individual ConfigCells that need to store extremely large amounts of data. The data content parsing method is not fixed. Fortunately, there is no need to parse such data off-chain;
- Entity type, that is, a simple molecule structure, is used to store ConfigCell, and due to its particularity, the external packaging data type is omitted. It can be parsed according to the corresponding [Type constant] (#Cell structure protocol.md);
- table type, this is a structure that wraps the entity type, mainly adding the location and version number of the cell corresponding to the entity data;

For the raw type, only the contract needs to be parsed, so generally there is no need to pay attention to its content; for the entity type, it can be directly encoded and decoded according to the molecule; for the table type, it can also be directly encoded and decoded according to the molecule. What needs to be mentioned additionally is This type is described in the documentation in the following form:

```
// At this time, the [3:7] bytes value of this data block can only be values other than ActionData and ConfigCellXXX
witness:
   table Data {
     dep: DataEntityOpt,
     old: DataEntityOpt,
     new: DataEntityOpt,
   }
```

for example:
- When a cell only appears in the cell\_deps of a transaction, we consider it to be referenced, and its data is stored in the `dep` field in the above structure, and the other two fields are empty;
- When a cell only exists in the outputs of a transaction, we consider it to be created, and its data is stored in the `new` field of the above structure, and the other two fields are empty;
- When a cell only exists in the inputs of the transaction, we consider it to be destroyed, and its data is stored in the `old` field of the above structure, and the other two fields are empty;
- When a cell exists in both inputs and outputs, we think it has been edited, and its data **should** be stored in the `old` and `new` fields of the above structure, only `dep` Is empty;

> The use of **should** here means that the contract does not impose this requirement, but the off-chain service should try its best to ensure this correlation so that the contract can check it when needed.
## Data verification

Because the data in the `witnesses` field can be filled in arbitrarily when constructing a transaction, the cell will store the hash of the witness data in the `outputs_data` field to ensure that the data provided by the user is indeed the data generated by the last transaction.

Suppose there is the following cell, whose index in inputs is 0:

```
inputs:
   [0]:
     lock: <super_lock>
     type: <config_cell_type>
     data: hash(StateCellData)
```

This transaction also puts the new cell generated after the cell is modified at the position where the index of outputs is 1:

```
outputs:
   [1]:
     lock: <super_lock>
     type: <config_cell_type>
     data: hash(StateCellData)
```

Because all data types use molecule encoding, we can directly obtain the binary bytes of this data structure. Suppose now the following data chunk is discovered by parsing `witnesses`:

```
witness:
    das,
    ConfigCellMain,
    table Data {
        dep: table DataEntityOpt {
            index: 0,
            version: 1,
            entity: StateCellData
        },
        old: table DataEntityOpt {
            index: 0,
            version: 1,
            entity: StateCellData
        },
        new: table DataEntityOpt {
          index: 1,
          version: 1,
          entity: StateCellData
        },
    }
```
- Because this cell does not exist in cell\_deps, dep must be None;
- Because it is found that old has data, it is necessary to check whether the `data.hash` of the corresponding cell of inputs is equal to `hash(old.entity)` according to the index;
- Because it is found that new has data, it is necessary to check whether the `data.hash` of the outputs corresponding cell is equal to `hash(new.entity)` according to the index;

> What is actually stored in inputs is the OutPoint structure, which is a pointer to the live cell. When the actual contract is running, it will be treated as a cell, so it is simply called the cell in inputs here.

> If the data block of a specific cell is not found in `witnesses`, and the document clearly stipulates that this cell needs to store additional data in the witness, in this case, the data verification is considered to have failed.
## Data encoding method

When selecting the encoding method, the following points are mainly considered:

- Data structures should be compact;
- There are ready-made standards and libraries available for data encoding and decoding methods;
- CKB on-chain scripts can perform data encoding and decoding;

The result is a coding scheme that may not be the best but is good enough: **[Molecule](https://github.com/nervosnetwork/molecule)** .

### Molecule coding related key points

>For official documentation of Molecule, please see https://github.com/nervosnetwork/molecule

Molecule is a compact binary encoding method designed by the Nervos Foundation for storing data on the chain. Compared with **Protobuf** and other solutions, it is a better solution for CKB on-chain storage. Official details Benchmarks and feature comparisons can be found in the documentation. However, the main reason why we use it is because this coding method is officially supported by Nervos and provides a relatively complete library to support the use of languages ​​such as Rust, Go, and Js.

The main points that need to be paid attention to when using Molecule coding are:

- Remember that there is only one basic type in Molecule, `byte`, so parsing any Molecule-encoded data is actually manipulating bytes;
- It is necessary to clarify whether the data type is **static type** or **dynamic type**, that is, whether the data is fixed-length data or variable-length data;

### Extend basic types

Because the metatype of Molecule is only `byte`, we customize some additional basic types:

```
array Uint8   [byte; 1];
array Uint32  [byte; 4];
array Uint64  [byte; 8];

// The following byte vector represents data of arbitrary length, whether it is utf-8 encoded or
// another structure will depend on the meaning of the field.
vector Bytes <byte>;

// The following byte array represents an 8-byte fixed timestamp, using the little-endian.
array Timestamp [byte; 8];

// The following array represents a fixed-length 32-byte array that is always used to store hashes.
array Hash [byte; 32];

option HashOpt (Hash);

// This represents the Script type in CKB.
table Script {
    code_hash: Hash,
    hash_type: byte,
    args:      Bytes,
}

option ScriptOpt (Script);

// This represents the OutPoint type in CKB, which is used to indicate the location of live cells.
struct OutPoint {
    tx_hash: Hash,
    index:   Uint32,
}

table Data {
    // when cell is in cell_deps its data will be stored at here
    dep: DataEntityOpt,
    // when cell is in inputs its data will be stored at here
    old: DataEntityOpt,
    // when cell is in outputs its data will be stored at here
    new: DataEntityOpt,
}

table DataEntity {
    // Indicates the cell in cell_deps/inputs/outputs to which this entity data belongs.
    index: Uint32,
    // Indicates the version of the entity data structure.
    version: Uint32,
    // Indicates the data of the entity.
    entity: Bytes,
}

option DataEntityOpt (DataEntity);
```
So **when operating these data, remember that you need to first implement conversion and reverse conversion from bytes to the language's own type** before using it.

#### Distinguish between static types and dynamic types

`byte` is the metatype of Molecule. In addition, Molecule also has composite types, as follows:

| Type | byte  | array | struct | vector  | table   | option  | union   |
| ---- | ----- | ----- | ------ | ------- | ------- | ------- | ------- |
| Size | Fixed | Fixed | Fixed  | Dynamic | Dynamic | Dynamic | Dynamic |

The key to understanding the combined type is to understand whether it is a **Fixed** fixed-length type or a **Dynamic** variable-length type. It is also because of this that some data uses `table` instead of `struct`, using ` vector` instead of `array`. For many problems that do not need to be distinguished in high-level languages, it is very important in Molecule.

### Schemas and libraries

The complete Schemas definition has been stored in the [das-types](https://github.com/DA-Services/das-types) project. When new languages need to be supported, they should also be added to the source code of this library.

### USD units

In DAS, all user-facing fees are billed in US dollars, but because CKB's contract can only handle integers, 1 USD is uniformly stored here as an integer of `1_000_000`, so that when the calculation cannot produce an integer You can use integer to represent 6 decimal places. If 6 digits are not enough, rounding is used to retain 6 decimal digits.

### Percent unit

There is no floating point type in molecule encoding and CKB VM, so here we agree on `100% == 10000`, so that it can be accurate to the last two digits of the percentage. For example, `0.99%` is equivalent to `99`.