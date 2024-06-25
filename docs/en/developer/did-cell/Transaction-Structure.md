# Transaction Structure

Since contract script on CKB do not have things like interface, we defined a series of transaction structures here instead of interface, they should be understood in terms of interface. These structures require that the final on-chain transactions must implement something such as：

- The type and number of cells in `CellDeps`, `Inputs` and `Outpus` must be correct.
- The order of the cells must also be correct in transactions with explicit requirements.
- `Action` must be explicitly provided in `transaction.witnesses`.

> For more technical details of each type of Cell, see [Cell Structure](./Cell-Structure.md)

> The CellDeps required for CKB official signall lock script and multisigh lock script are not listed in the transactions below, but you should still add them to the transaction's CellDeps.

> Unlike [AccountCell related transactions](../Cell-Protocol.md#accountcell), to allow better composability, DidCell related cells 和 witnesses are not required to appear in strict order。Howerver, when DidCell and AccountCell show in the same transaction, you need to follow the transaction structure of AccountCell, meaning you should append DidCell related Cells and Witnesses after the ones for AccountCell。 The operation on DidCell does not require an Action in Witness. did-cell-type only verifies if the content transition is valid.


## Terms and Conventions

> Everything in this document needs to be understood on the basis of [RFC-0022 CKB Transaction Structure](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0022-transaction-structure/0022-transaction-structure.md) . Lack of sufficient understanding of transaction structure of CKB may be an obstacle to understanding this document.

|   Term    |                                                      Description                                                       |
| ---------- | ---------------------------------------------------------------------------------------------------------------------- |
| ActionData | A segment of data that must be carried by transaction of DAS , as detailed in [Data Storage](./Data-Storage.md) .      |
| NormalCell | All cells in CKB have lock, type, outputs_data attributes, this means the cells whose type and outputs_data are empty. |
| FeeCell    | The NormalCell to pay the various fees required by transactions.                                                       |
| ChangeCell | The NormalCell to store changes of transactions.                                                                       |
| ScriptCell | The cell where executable of contract script is deployed.                                                              |

|      Symbol    |                                                                                        Description                                                                                 |
| -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [XxxYyyCell]   | Means XxxYyyCell is optional. |
| XxxYyyCell (n) | Means XxxYyyCell is ordered and this is nth XxxYyyCell. |
| XxxYyyCell {n} | {4} means there are and only 4 of XxxYyyCell<br>{3,} means there are at least 3 XxxYyyCell<br/>{,2} means there are at most 2 XxxYyyCell<br/>{1,4} means there are 1 to 4 XxxYyyCell. |
| XxxYyyCell [A] | Means multiple XxxYyyCell in differet parts of transaction should have consistent sequence.<br/>If XxxYyyCell in Inputs/Output/CellDeps with the same `[A]` mark then they should have consistent sequence.<br>The `[A]` means that some sorting method A, it could also be B, C and so on |
| ConfigCellXXXX.yyyy | Means the data needs to be retrieved from a field in the witness of a ConfigCell, see [ConfigCell](./Cell-Structure.md#ConfigCell) for details. |

> All hash are using the same hash algorithm, i.e. [ckbhash algorithm](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0022-transaction-structure/0022-transaction-structure.md#crypto-primitives) .

## Supported Transactions

### Upgrade to DidCell

When the status of [AccountCell](../Cell-Protocol.md#accountcell) changes from Normal(0x00) to Upgraded(0x99), did-cell-type allows the creation of DidCell. The expire_at of DidCell should be equal to the one of corresponding AccountCell in output.


**Transaction Structure**

```
CellDeps:
  ...,     // Deps required by AccountCell
  did-cell-type
Inputs:
  ...,     // Inputs required by AccountCell
  FeeCell // NormalCell used to pay for storage fee for the created DidCell
Outputs:
  AccountCell
  DidCell
  [ChangeCell]
```


### Transfer DidCell

Transfering DidCell does not require extra Witnesses of CellDeps. Notice that expired DidCells are also allowed to be transferred. The sender and recerver should both be aware of that.

DidCell can be transferred to any lock.

**Transaction Structure**

```
CellDeps:
  did-cell-type
  ...     // Any other deps
Inputs:
  DidCell,
  ...     // Any other cells
Outputs:
  DidCell,
  ...     // Any other cells
```


### Edit Records in DidCell

You can change the witness_hash of DidCell's content as long as you provide a valid WitnessData whose hash is the same as the one in the content. Please refer to [Cell-Protocol](./Cell-Protocol.md)

**Transaction Structure**

```
CellDeps:
  did-cell-type,
  ...     // Any other deps
Inputs:
  DidCell,
  ...     // Any other cells
Outputs:
  DidCell,
  ...     // Any other cells
```


### Destry DidCell

Only expired DidCells are allowed to be destroyed. When did-cell-type sees there's a DidCell in the Inputs but no corresponding DidCell found in the Outputs, it will enter destroy logic, which requires extra CellDeps.

**Transaction Structure**

```
CellDeps:
  did-cell-type,
  TimeCell,
  ConfigCellMain.
  ...     // Any other deps
Inputs:
  DidCell,
  ...     // Any other cells
Outputs:
  DidCell,
  ...     // Any other cells
```