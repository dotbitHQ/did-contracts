# Transaction Structure

Since contract script on CKB do not have things like interface, we defined a series of transaction structures here instead of interface, they should be understood in terms of interface. These structures require that the final on-chain transactions must implement something such as：

- The type and number of cells in `CellDeps`, `Inputs` and `Outpus` must be correct.
- The order of the cells must also be correct in transactions with explicit requirements.
- `Action` must be explicitly provided in `transaction.witnesses`.

> For more technical details of each type of Cell, see [Cell Structure](./Cell-Structure.md)

> The CellDeps required for CKB official signall lock script and multisigh lock script are not listed in the transactions below, but you should still add them to the transaction's CellDeps.

> All transaction fees are paid by the creator of the transaction.


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


## Transactions of Keeper

### Proposal Transactions

#### Propose

This transaction will create a ProposalCell and verifie the uniqueness of the account name in related PreAccountCells. The `witness.slices` field of the ProposalCell is a special `SliceList` structure, which is a description of multiple slices of the account chain, so the AccountCell and PreAccountCell in CellDeps must be sorted according to the `SliceList` structure, which in brief means AccountCell and PreAccountCell must be pushed in an array and sorted by bytes of account ID.

**Action structure**

```
table ActionData {
  action: "propose",
  params: [],
}
```

**Transaction structure**

```
CellDeps:
  always-success
  proposal-cell-type
  TimeCell
  HeightCell
  ConfigCellMain
  ConfigCellProposal
  // Require all AccountCells and PreAccountCells listed in the witnesses.slices
  AccountCell [A] {1,}
  PreAccountCell [A] {1,}
Inputs:
  FeeCell
Outputs:
  ProposalCell
  [ChangeCell]
```

#### ExtendProposal

This transaction means to create a new proposal based on an existing proposal, not to consume the existing proposal, so the existing ProposalCell needs to be placed in CellDeps of the transaction. This transaction requires the same sorting method for AccountCells and PreAccountCells as the "propose" transaction.

**Action structure**

```
table ActionData {
  action: "extend_proposal",
  params: [],
}
```

**Transaction structure**

```
CellDeps:
  always-success
  proposal-cell-type
  TimeCell
  HeightCell
  ConfigCellMain
  ConfigCellProposal
  ProposalCell (n)
  // Require all AccountCells and PreAccountCells listed in the witnesses.slices by ProposalCell (n + 1)
  AccountCell [A] {1,}
  PreAccountCell [A] {1,}
Inputs:
  FeeCell
Outputs:
  ProposalCell (n + 1)
  [ChangeCell]
```

#### ConfirmProposal

The proposal must wait for n blocks before being confirmed, n can be obtained from `ConfigCellProposal.proposal_min_confirm_interval`.

**Action structure**

```
table ActionData {
  action: "confirm_proposal",
  params: [],
}
```

**Transaction structure**

```
CellDeps:
  always-success
  proposal-cell-type
  account-cell-type
  pre-account-cell-type
  income-cell-type
  TimeCell
  HeightCell
  ConfigCellAccount
  ConfigCellMain
  ConfigCellProfitRate
Inputs:
  ProposalCell
  AccountCell [A] {1,}
  PreAccountCell [A] {1,}
  [IncomeCell] // If the total profit is less than the occupied capacity of the IncomeCell, then an empty IncomeCell can be placed in Inputs.
  [FeeCell]
Outputs:
  AccountCell [A]
  IncomeCell {1,}
  ChangeCell // There must be a ChangeCell to return ProposalCell.capacity to the proposer.
  [ChangeCell] // The rest of changes can be arranged according to needs.
```

##### Sorting Method of AccountCell and PreAccountCell

As the "propose" transaction, the AccountCell and PreAccountCell in Inputs must be pushed in an array and sorted by bytes of account ID, and the AccountCell in Outputs must also be sorted in the same way.

##### Profit Allocation

When the proposal is confirmed successfully the new account is registered successfully, the registration fee carried by PreAccountCells also become the essential source of profit for DAS. Therefore, this transaction requires that the profits be allocated according to the following rules:

- A portion of the profit belonging to the proposer which comes from `ConfigCellProfitRate.proposal_create` .
- A portion of the profit belonging to who confirmed proposal which comes from `ConfigCellProfitRate.proposal_confirm` .
- A portion of the profit belonging to the channel role which comes from `ConfigCellProfitRate.channel` .
- A portion of the profit belonging to the inviter role which comes from `ConfigCellProfitRate.inviter` .
- A portion of the profit belonging to DAS which is **total_profit - total_profit_of_above_roles**.

In the above profit, except for the profit belongs to who confirmed proposal, the profit of other roles need to be placed in IncomeCell to avoid the problem of not being able to transfer due to insufficient 61 CKB. Since the keeper who constructs the transaction is who confirmed proposal, it can put some NormalCells in the Inputs to make up the 61 CKB for its profit.



> ⚠️ Note that the proposer's profit is recorded in IncomeCell, but ProposalCell.capacity must return to the proposer directly. The purpose of this design is to ensure that the proposer can get back the cost of creating the proposal sooner and avoid pledging too much CKB to run a keeper.

#### RecycleProposal

When a proposal related PreAccountCell is spent for some reason, the proposal cannot be confirmed anymore. The proposer can therefore recover the CKB occupied by the ProposalCell by recycling the failed proposal. **n** blocks must be waited before the proposal is recycled, and **n** can be obtained from `ConfigCellProposal.proposal_min_recycle_interval`.

**Action structure**

```
table ActionData {
  action: "recycle_proposal",
  params: [],
}
```

**Transaction structure**

```
CellDeps:
  always-success
  proposal-cell-type
  TimeCell
  HeightCell
  ConfigCellRegister
Inputs:
  ProposalCell
  [FeeCell]
Outputs:
  ChangeCell // There must be a ChangeCell to return ProposalCell.capacity to the proposer
  [ChangeCell] // The rest of changes can be arranged according to needs.
```

### IncomeCell Transactions

#### CreateIncome

Anyone can execute this transaction to create IncomeCells, but there is no financial incentive to do so, so this transaction is primarily a way for DAS official to dynamically create and maintain a certain number of empty IncomeCells depending on the situation of the blockchain. These empty IncomeCells cannot be used for consolidating, but can be used to receive profits in a "propose" transaction, solving the problem of insufficient profits in a proposal transaction for basic required capacity of IncomeCell.

**Action structure**

```
table ActionData {
  action: "create_income",
  params: [],
}
```

**Transaction structure**

```
CellDeps:
  always-success
  income-cell-type
  ConfigCellMain
  ConfigCellIncome
Inputs:
  FeeCell
Outputs:
  IncomeCell
  [ChangeCell]
```

#### ConsolidateIncome

The main function of this transaction is to release the profit deposited in IncomeCell to the owner of the profit, and the transaction must comply with the following constraints:

- For an empty IncomeCell **which has only one record and is paied by its creator for storage, then this IncomeCell cannot be consolidated**.
- The inputs of the transaction must have at least 2 IncomeCells.
- The total number of records of all IncomeCells in outputs must be less than which in inputs.
- The record with the same lock script must be unique in all IncomeCells in outputs.
- For **a record whose capacity meets the minimum transfer out threshold which can be retrieve from `ConfigCellIncome.min_transfer_capacity`**, it must be transferred directly to the record's lock script, unless IncomeCells in outputs are lack sufficient capacity for storage after the transfer.
- When transferring to some lock script, the creator of the "consolidate_income" transaction can take away a portion of capacity base on `ConfigCellProfitRate.income_consolidate` as a fee.
- If the lock script to transfer is the creator of some IncomeCells in inputs or the lock script is belong to DAS, then the creator of the "consolidate_income" transaction cannot take any fee from the transfer.
- Transactions that are lack of sufficient capacity for storage of IncomeCells due to transfer are called **Transactions that need to be padded**.
- For **transactions that need to be padded**, a portion of capacity that should be transferred can be taken for padding, the amount is depending on the creator of the transaction.

> The capacity which IncomeCell requires for storage can retrieve from `ConfigCellIncome.basic_capacity` .

**Example**

For example that current `ConfigCellIncome.basic_capacity` is 200 CKB and current `ConfigCellIncome.min_transfer_capacity` is 100 CKB, there are two following IncomeCells:

```
IncomeCell A:
  creator: lock-a
  lock-a: 200 CKB
  lock-b: 99 CKB
  lock-c: 99 CKB
  lock-d: 5 CKB

IncomeCell B:
  creator: lock-a
  lock-a: 200 CKB
  lock-b: 1 CKB
  lock-c: 1 CKB
  lock-e: 5 CKB
```

Then one posible consolidating result is:

```
IncomeCell C:
  creator: null // Use Script::default() instead
  lock-a: 190 CKB // In fact, any one or more of lock-a, lock-b, lock-c can leave 190 CKB at total
  lock-d: 5 CKB
  lock-e: 5 CKB

Cell:
	lock-a: 210 CKB
Cell:
	lock-b: 100 CKB
Cell:
	lock-c: 100 CKB
```

Here any one or more of a, b or c can leave enough CKB for the IncomeCell storage, because the capacity of lock-d and lock-e is too small to transfer out, resulting in the IncomeCell can not be destroyed. There is no constraints in contracts about how much capacity shoule be leave for storage, but the more creator can transfer out, the more fees the creator of the transaction can get.

**Action structure**

```
table ActionData {
  action: "consolidate_income",
  params: [],
}
```

**Transaction structure**

```
CellDeps:
  always-success
  income-cell-type
  ConfigCellIncome
  ConfigCellProfitRate
Inputs:
  IncomeCell {2,}
  [FeeCell]
Outputs:
  IncomeCell {1,}
  [ChangeCell]
```

### Maintaince Transactions

#### RecycleExpiredAccountByKeeper

**Action structure**

```
table ActionData {
  action: "recycle_expired_account_by_keeper",
  params: [],
}
```

**Transaction structure**

```
CellDeps:
  das-lock
  account-cell-type
  TimeCell
  HeightCell
  ConfigCellMain
  ConfigCellAccount
Inputs:
  AccountCell (n - 1)  // The previous account of the overdue account, its next pointer needs to be modified
  AccountCell (n)      // The overdue account, its AccountCell will be consumed
  [FeeCell]
Outputs:
  AccountCell (n - 1)
  ChangeCell // The capacity for storage of AccountCell (n) must be refund to the owner lock of AccountCell (n)
  [ChangeCell]
```


## Transactions of User

### Register Transactions

#### ApplyRegister

This is the first transaction in the registration process. To prevent the user's registering account name to be robbery in registeration process, only hash of the account name is provided in this transaction.
The ApplyRegisterCell in outputs is the credential that must wait for a certain blocks before taking it to the next pre-register step. The blocks to wait for can be obtained from `ConfigCellApply.apply_min_waiting_block_number`.

**Action structure**

```
table ActionData {
  action: "apply_register",
  params: [],
}
```

**Transaction structure**

```
CellDeps:
  apply-register-cell-type
  TimeCell
  HeightCell
  ConfigCellMain
  ConfigCellApply
Inputs:
  FeeCell
Outputs:
  ApplyRegisterCell
  [ChangeCell]
```

#### RefundApply

If an ApplyRegisterCell created by a user has not been taken by the Keeper for pre-register after the maximum wait time, then the ApplyRegisterCell can be refunded with this transaction.
The blocks to wait for before ApplyRegisterCell can be refund is stored in `ConfigCellApply.apply_max_waiting_block_number`.

> Since the ApplyRegisterCell can use any lock script, an ApplyRegisterCell created with the user's own lock script needs to be signed by the user in this transaction.

**Action structure**

```
table ActionData {
  action: "refund_apply",
  params: [],
}
```

**Transaction structure**

```
CellDeps:
  apply-register-cell-type
  TimeCell
  HeightCell
  ConfigCellMain
  ConfigCellApply
Inputs:
  ApplyRegisterCell
Outputs:
  ChangeCell
```

#### PreRegister

This is the second transaction in the registration process, the user must provide the plain text of the registering account and pay the registration fee. The ApplyRegisterCell in inputs must wait for n blocks and cannot wait for more than m blocks. The n can be obtained from `ConfigCellApply.apply_min_waiting_block_number` and the m from `ConfigCellApply.apply_max_waiting_block_number`.

**Action structure**

```
table ActionData {
  action: "pre_register",
  params: [],
}
```

**Transaction structure**

```
CellDeps:
  always-success
  apply-register-cell-type
  pre-account-cell-type
  TimeCell
  HeightCell
  QuoteCell
  ConfigCellMain
  ConfigCellAccount
  ConfigCellApply
  ConfigCellPreservedAccountXX
  ConfigCellCharSetXxxx {1,}
Inputs:
  ApplyRegisterCell
  {FeeCell}
Outputs:
  PreAccountCell
  {ChangeCell}
```

**ConfigCellPreservedAccountXX**

There are a total of 20 ConfigCellPreservedAccountXX which comes from ConfigCellPreservedAccount00 to ConfigCellPreservedAccount19. They are used to store preserved account names. But due to the size limitation of the `transaction.witnesses` field by CKB's lock script, they are split, so the creator of the transaction only need to select the correct one when using them. The selection is done as following rules:

- Remove the `.bit` suffix from the account name and hash it.
- Takes the first byte of hash as a u8 type integer.
- Modulo 20 with the number obtained above.
- Select the ConfigCellPreservedAccountXX with the corresponding end number according to the result obtained from above modelling.

**ConfigCellCharSetXxxx**

ConfigCellCharSetXxxx does not exist in a fixed number, they are used to store the character sets supported by DAS, and are also split due to the size limitation of the `transaction.witnesses` field. The correct one need to be select for the transaction and the selection must follow these rules:

- Unserialize the witness of the PreAccountCell, iterate over `account` field.
- Every `account` is `AccountChar` type, the value of its `char_set_name` field is called **Charset ID**.
- Add the **Charset ID** to `100000` to get the **Config ID** corresponding to ConfigCellCharSetXxxx.
- Finally, one or more ConfigCellCharSetXxxx can be select based on these Config IDs.

> Details of **Charset ID** and **Config ID** please see [Cell Structure](./Cell-Structure.md) 。

#### RefundPreRegister

If by chance there are multiple PreAccountCells with the same account name on blockchain, only one of them can eventually be registered as an account through the proposal, and the remaining unregistered PreAccountCells can be recycled through this transaction and the user's registration fee will be refunded.

**Action structure**

```
table ActionData {
  action: "refund_pre_register",
  params: [],
}
```

**Transaction structure**

```
CellDeps:
  ScriptCells
  TimeCell
  HeightCell
  ConfigCellMain
  AccountCell // The reference here is to prevent the malicious refunds
Inputs:
  PreAccountCell
Outputs:
  ChangeCell // There must be a ChangeCell in outputs to refund the registration fee to the user, and Keeper can take up to 10,000 shannon as transaction fee
```

### Account Management Transactions

All account management transactions have an additional `permission` parameter in the `ActionData` which indicates the required permission for the current transaction and is checked by both the account-cell-type and das-lock scripts, so it must be provided correctly. The optional values for `permission` are:

- 0x00 indicates the transaction requires owner permission.
- 0x01 indicates the transaction requires manager permission.

> The account's owner and manager permissions are mutually exclusive, i.e., the owner is not authorized to perform operations that require manager permissions.

> No transactions other than [RenewAccount](#RenewAccount) can be executed after the account has entered the grace period.

#### TransferAccount

This transaction allows the user to transfer the AccountCell to another person, i.e. permanently transfer ownership of the account. The transaction modifies the `lock.args` field of the AccountCell and requires that the `lock.args` of the AccountCell in the output must have the same lock hash for both owner and manager, as the transfer of the account to another person with the manager still belonging to the original user may lead to potential risks.

There is a limit to how often this transaction can be executed on the same account, and the time interval for each transaction can be retrieved from `ConfigCellAccount.transfer_account_throttle`. The transaction fee can be deducted from the AccountCell. The maximum capacity of the transaction fee can be retrieved from `ConfigCellAccount.transfer_account_fee`.

**Action structure**

```
table ActionData {
  action: "transfer_account",
  params: [0x00],
}
```

**Transaction structure**

```
CellDeps:
  das-lock
  account-cell-type
  TimeCell
  HeightCell
  ConfigCellMain
  ConfigCellAccount
Inputs:
  AccountCell
  [FeeCell]
Outputs:
  AccountCell
  [ChangeCell]
```

#### EditManager

This transaction allows the user to specify some one else to edit records of the account. The transaction modifies the `lock.args` field of the AccountCell, and requires that the manager part of `lock.args` of the AccountCell in outputs must be different from which in inputs.

There is a limit to how often this transaction can be executed on the same account, and the time interval for each transaction can be retrieved from `ConfigCellAccount.edit_manager_throttle`. The transaction fee can be deducted from the AccountCell. The maximum capacity of the transaction fee can be retrieved from `ConfigCellAccount.edit_manager_fee`.

**Action structure**

```
table ActionData {
  action: "edit_manager",
  params: [0x00],
}
```

**Transaction structure**

```
CellDeps:
  das-lock
  account-cell-type
  TimeCell
  HeightCell
  ConfigCellAccount
  ConfigCellMain
Inputs:
  AccountCell
  [FeeCell]
Outputs:
  AccountCell
  [ChangeCell]
```

#### EditRecords

The records of the account are modified here, which is the information that is most frequently read in the daily use of an account. The information stored in `witness.records` of the AccountCell will be modified after the transaction.

There is a limit to how often this transaction can be executed on the same account, and the time interval for each transaction can be retrieved from `ConfigCellAccount.edit_records_throttle`. The transaction fee can be deducted from the AccountCell. The maximum capacity of the transaction fee can be retrieved from `ConfigCellAccount.edit_records_fee`.


> Since the owner and manager permissions are mutually exclusive as mentioned earlier, **this transaction is only available to the manager**.

**Action structure**

```
table ActionData {
  action: "edit_records",
  params: [0x01],
}
```

**Transaction structure**

```
CellDeps:
  das-lock
  account-cell-type
  TimeCell
  HeightCell
  ConfigCellMain
  ConfigCellAccount
  ConfigCellRecordKeyNamespace
Inputs:
  AccountCell
  [FeeCell]
Outputs:
  AccountCell
  [ChangeCell]
```

#### RenewAccount

The AccountCell for each account has an expiration time stored in `data`, and the only way to extend this expiration time is to push this transaction. After the transaction, the expiration time stored in the `data` of the AccountCell can be updated to a new value which depending on the capacity user paied, the minimum capacity must as mush as more than one year.

> Anyone can renew for any account and the contract will not limit the source of funds.

**Action structure**

```
table ActionData {
  action: "renew_account",
  params: [],
}
```

**Transaction structure**

```
CellDeps:
  das-lock
  account-cell-type
  income-cell-type
  TimeCell
  HeightCell
  QuoteCell
  ConfigCellMain
  ConfigCellPrice
  ConfigCellAccount
Inputs:
  AccountCell
  [IncomeCell] // If the capacity for renew is not enough for the IncomeCell's storage, the creator of the transaction can put an empty IncomeCell in inputs
  FeeCell // Users need to pay capacities through a NormalCell
Outputs:
  AccountCell
  IncomeCell // This is used to store fees paid by users
  [ChangeCell]
```

## Special Transactions

#### Contract Deploy

The Keeper needs to listen to this transaction in order to update the OutPoint of each contract script in time so that the CellDeps field can be constructed correctly when a transaction is created.

> ⚠️ Because this is a transaction that deploys contract script, the execution of it will not be protected by any contract script of DAS! To avoid forged contract update transactions, **be sure to check the following two items:**
> - Is DAS offical multi-sign lock script in inputs?
> - Is the Type ID calculated from the ScriptCell's type field consistent with before?

**Action structure**

```
table ActionData {
  action: "deploy",
  params: [],
}
```

**Transaction structure**

```
CellDeps:
Inputs:
  [ScriptCell] // If there is already a ScriptCell in inputs, it means updating an exist one, if not, it means creating a new one.
  [FeeCell]
Outputs:
  ScriptCell
  [ChangeCell]
```

#### Initialize Account Chain

This transaction can be only pushed before **2021-07-22T12:00:00Z** and is required for the initialization of the account chain of DAS on blockchain. It will created a special AccountCell which named RootAccountCell, its `data.account_id` is `0x0000000000000000000000000000000000000000` and its `data.next` is `0xffffffffffffffffffffffffffffffffffffffff`.

The RootAccountCell stores three Merkle roots which represent the DAS gratitude list, the development team, and some other message leaved by kinds of people, and the position to store these information is where other AccountCell to store data of `data.account`.

**Action structure**

```
table ActionData {
  action: "init_account_chain",
  params: [],
}
```

**Transaction structure**

```
CellDeps:
  das-lock
  account-cell-type
  TimeCell
  HeightCell
  ConfigCellMain
  ConfigCellAccount
Inputs:
  [FeeCell]
Outputs:
  RootAccountCell
  [ChangeCell]
```

#### Config

This transaction is designed to create or modify the special cells named ConfigCellXxxx. All the global configuration of DAS runtime is modified by this transaction, and any off-chain services should also listen to this transaction to retrieve the latest status of DAS runtime in time.

**Action structure**

```
table ActionData {
  action: "config",
  params: [],
}
```

**Transaction structure**

```
CellDeps:
  config-cell-type
Inputs:
  [ConfigCell]
  FeeCell
Outputs:
  ConfigCell
  [ChangeCell]
```
