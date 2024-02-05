# Script and Cell Anti-counterfeiting


Because many transactions in DAS involve the collaboration of multiple contract scripts and multiple cells, this document describes how DAS addresses the following two issues in these collaborations:

- How to prevent attackers from using other contract scripts to impersonate DAS contract scripts;
- How to prevent attackers from counterfeiting cells to masquerade as cells controlled by DAS contracts;



## Principle

On the CKB blockchain, there are several rules that can help us address this issue:

- The contract script actually called at runtime is determined by `cell.lock.code_hash` or `cell.type.code_hash`;
- `code_hash` is essentially the blake2b hash value of the script's compiled executable file or a similar unique type ID value. Therefore, it can be considered the same script only when `code_hash` is consistent;
- The script pointed to by `cell.type.code_hash` will be executed when the cell is used as input or output;

> For more information about type ID and its underlying principles, please refer to: [RFC-0022](https://github.com/nervosnetwork/rfcs/blob/f0bf9fd6c6/rfcs/0022-transaction-structure/0022-transaction-structure.md#type-id) 。

## Solution

Based on the principles mentioned above, we have designed the following solution:

![DAS-anti-counterfeiting](../../images/DAS-anti-counterfeiting.png)

As shown in the diagram, the following rules are described:

- First, the DAS system will establish a **super lock** based on multi-signature.
- Then, it will be hardcoded into the source code of the **config-cell-type** script, restricting the creation and modification of the **config-cell-type**.
- After **config-cell-type** is deployed to the blockchain, it will create a **ConfigCell** with its type ID as `type.code_hash` in the witness field, which will store the type IDs of various other script types.
- Each of the other cells will need to hardcode the type ID of the **config-cell-type** within their scripts.

Next, based on these rules, let's examine how they address our issues. 

### Script Anti-counterfeiting

Because **ConfigCell**'s witness records the type IDs of various type scripts, when these type scripts need to cooperate with each other or when any other situation requires reliance on type scripts for validation, it is only necessary to securely confirm that other scripts are also included in the transaction by reading and parsing the type IDs in **ConfigCell**'s witness.

As for **ConfigCell** itself, its `type.code_hash`, which is the type ID of **config-cell-type**, is hardcoded into various type scripts. Therefore, there is no possibility of impersonation, and it also uses the **super lock**, making it impossible for others to create or modify it arbitrarily.


### Cell Anti-counterfeiting

Because, as described above, the type ID of type scripts cannot be forged, confirming that a cell has not been counterfeited becomes straightforward. All transactions in DAS will strictly read the cells in the transaction based on the type ID stored in **ConfigCell**. If the type ID is different, the cell's data will not even be read during execution.

