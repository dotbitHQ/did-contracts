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

#### Forced account restoration (ForceRecoverAccountStatus)

When the account is in a non-ordinary state, that is, when `witness.status != 0`, and the account reaches the **grace period** in the life cycle, then the Keeper can **forcibly restore the account's status** at this time, so that in
The expired account auction will be carried out after the account **fully expires**, and the account recovery will be carried out when the expired auction fails.

> For accounts in the auction, if someone has already bid, the Keeper cannot be forcibly restored. However, after AccountCell enters the **grace period**, bidders can no longer pass AccountAuctionCell.
> Make a bid and the account will be handed over to the last bidder after the bidding time ends.

**action structure**

```
table ActionData {
   action: "force_recover_account_status",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   account-cell-type
   [account-sale-cell-type]
   [account-auction-cell-type]
   balance-cell-type
   TimeCell
   HeightCell
   ConfigCellMain
   ConfigCellAccount
Inputs:
   AccountCell
   [AccountSaleCell] // If the account status == 1, then the corresponding AccountSaleCell must be carried
   [AccountAuctionCell] // If the account status == 2, then the corresponding AccountAuctionCell must be carried
Outputs:
   AccountCell
   ChangeCell // When AccountSaleCell, AccountAuctionCell is destroyed, etc., the capacity must be returned to the user
```

**agreement**

- AccountCell must be **On Sale** or **On Auction**, i.e. 1 or 2;
- When AccountSaleCell and AccountAuctionCell are destroyed, the capacity of these Cells must be returned to the user in the form of BalanceCell;
- Keeper can use 10_000 shannon from the refunded amount as transaction fee;

#### Recycle ExpiredAccount

**action structure**

```
table ActionData {
   action: "recycle_expired_account",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   account-cell-type
   TimeCell
   HeightCell
   ConfigCellMain
   ConfigCellAccount
Inputs:
   AccountCell (n - 1) // next pointer points to the previous account of the overdue account
   AccountCell (n) // Overdue account
   [SubAccountCell] // If the overdue account has opened a sub-account, you need to recycle the SubAccountCell here.
Outputs:
   AccountCell (n - 1)
   ChangeCell // Capacity returned to parent account owner
   [ChangeCell] // If the profit of DAS exceeds 61CKB, then this part of the profit should be returned to DAS
```

** Agreement **

- If the sub-account function is turned on, recycling of the sub-account is a must;
- The account must be in the Normal, LockedForCrossChain state. If it is in the Selling, Auction state, the account status should be restored through the `force_recover_account_status` transaction;
- When recycling, you need to modify the next pointer to point to the previous AccountCell(n - 1) of the current AccountCell(n), so that AccountCell(n - 1) inherits the current AccountCell(n).next pointer;
- After AccountCell(n) is recycled, its remaining capacity needs to be returned to the owner lock, and an amount less than or equal to `ConfigCellAccount.common_fee` can be withdrawn as transaction fee;
- The capacity of SubAccountCell includes four parts: basic storage fee, handling fee, DAS profit, and owner profit of the parent account. Therefore, when recycling SubAccountCell, follow the following rules to return the capacity:
    - If the profit of DAS is more than 61CKB, it needs to be returned to DAS. If the profit is less than 61CKB, it can be taken away by the transaction constructor;
    - All profits other than DAS will be returned to the owner lock of the parent account;



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

#### UpgradeDid

When the status of Account cell is set from  Normal(0x00) to Upgraded(0x99), a DidCell should be created. This action is only allowed for AccountCell with status Normal(0x00).

**Action structure**

```
table ActionData {
  action: "upgrade_did",
  params: []
}
```

**Transaction structure**

```
CellDeps:
  das-lock
  account-cell-type
  eip712-lib
  income-cell-type
  TimeCell
  HeightCell
  QuoteCell
  ConfigCellMain
  ConfigCellPrice
  ConfigCellAccount
Inputs:
  AccountCell
  FeeCell // Provides storage capacity for DidCell
Outputs:
  AccountCell
  DidCell
  [ChangeCell]
```


### Reverse analysis of related transactions V2

#### Create reverse parsing SMT tree (CreateReverseRecordRoot)

This transaction can create a Cell that stores the root of the reverse parsed SMT tree, that is, create a ReverseRecordRootCell.

**action structure**

```
table ActionData {
   action: "create_reverse_record_root",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   balance-cell-type
   reverse-record-root-cell-type
   ConfigCellMain
   ConfigCellReverseResolution
Inputs:
   BalanceCell {1,}
Outputs:
   ReverseRecordRootCell {1,}
   BalanceCell {1,}
```

**agreement**

- At least one BalanceCell in the inputs must use super lock.
- A ReverseRecordRootCell with an empty Root (0x000000000000000000000000000000000000000000000000000000000000000) must be created in outputs.
- ReverseRecordRootCell's lock must be `always-success`.

#### Update reverse resolution (UpdateReverseRecordRoot)

This transaction can create, edit, and delete a reverse parsing record corresponding to a public key.

**action structure**

```
table ActionData {
   action: "update_reverse_record_root",
   params: [],
}
```

**Transaction Structure**

```
HeaderDeps:
   block_hash(ReverseRecordRootCell)
CellDeps:
   das-lock
   balance-cell-type
   reverse-record-root-cell-type
   ConfigCellMain
   ConfigCellReverseResolution
   ConfigCellSMTNodeWhitelist
Inputs:
   ReverseRecordRootCell
   BalanceCell {1,}
Outputs:
   ReverseRecordRootCell
   BalanceCell {1,}
```

**agreement**

- ReverseRecordRootCell must be consistent, only the SMT Root in outputs_data must be different;
- At least one lock in the BalanceCell of inputs must exist in ConfigCellSMTNodeWhitelist;
- The SMT operation records in the witness must be arranged in order from old to new. Each record is an update to the Root, and the updates must be performed in order;

### Secondary market related transactions

#### Fixed price transaction

##### Start Selling (StartAccountSale)

This transaction can mark the account as being sold. The transaction will create an AccountSaleCell, which stores the selling price and other related information, but the selling price must not be lower than `ConfigCellSecondaryMarket.min_sale_price`.

**action structure**

```
table ActionData {
   action: "start_account_sale",
   params: [0x00],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   account-cell-type
   eip712-lib
   account-sale-cell-type
   TimeCell
   HeightCell
   ConfigCellAccount
   ConfigCellSecondaryMarket
Inputs:
   AccountCell
   BalanceCell {1,}
Outputs:
   AccountCell
   AccountSaleCell
   [BalanceCell]
```

**agreement**

- The locks of AccountCell and AccountSaleCell must be consistent and be das-lock;
- The capacity of AccountSaleCell needs to be equal to `ConfigCellSecondaryMarket.sale_cell_basic_capacity + ConfigCellSecondaryMarket.sale_cell_prepared_fee_capacity`
- AccountSaleCell needs to comply with the restrictions of other `ConfigCellSecondaryMarket.sale_*` configuration items;

##### Modify product information (EditAccountSale)

This transaction can modify the selling price and other information stored in AccountSaleCell.

**action structure**

```
table ActionData {
   action: "edit_account_sale",
   params: [0x00],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   account-sale-cell-type
   TimeCell
   HeightCell
   ConfigCellSecondaryMarket
   AccountCell
Inputs:
   AccountSaleCell
Outputs:
   AccountSaleCell
```

**agreement**

- AccountCell needs to have the same account ID as AccountSaleCell;
- The handling fee can be deducted from AccountSaleCell in an amount equal to `ConfigCellSecondaryMarket.common_fee`;

##### Cancel Sale (CancelAccountSale)

This transaction allows you to cancel the fixed-price sale of an account as long as the account has not been sold.

**action structure**

```
table ActionData {
   action: "cancel_account_sale",
   params: [0x00],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   account-cell-type
   eip712-lib
   account-sale-cell-type
   TimeCell
   HeightCell
   ConfigCellAccount
Inputs:
   AccountCell
   AccountSaleCell
Outputs:
   AccountCell
   BalanceCell
```

**agreement**

- AccountCell needs to have the same account ID as AccountSaleCell;
- The transaction fee can be deducted from AccountSaleCell by an amount equal to `ConfigCellSecondaryMarket.common_fee`;
- There must be a ChangeCell containing the AccountSaleCell refund;

##### BuyAccount

Other users can purchase the account for sale through this transaction. If the purchase is successful, the account will be transferred to the new account name, and the original resolution records will be cleared.

**action structure**

```
table ActionData {
   action: "buy_account",
   params: [inviter_lock, channel_lock, 0x00],
}
```

- inviter_lock, if the user who purchased the account has an inviter, the inviter information can be passed through this parameter, which is a molecule-encoded Script structure. If there is no inviter, the default value of the Script structure needs to be passed in;
- channel_lock, the purchasing channel can fill in its own payment address through this parameter to collect the share. It must also be a molecule-encoded Script structure;

**Transaction Structure**

```
CellDeps:
   das-lock
   account-cell-type
   eip712-lib
   account-sale-cell-type
   TimeCell
   HeightCell
   ConfigCellAccount
   ConfigCellIncome
   ConfigCellSecondaryMarket
Inputs:
   AccountCell
   AccountSaleCell
   BalanceCell {1,}
Outputs:
   AccountCell
   IncomeCell // Stores the profits allocated to inviter_lock and channel_lock
   BalanceCell // The capacity of AccountSaleCell must be returned to the seller of the account in the form of NormalCell using das-lock
```

**agreement**

- If inviter_lock and channel_lock are the default values of the Script structure, it will be deemed that there is no inviter and no channel provider;
- AccountCell needs to have the same account ID as AccountSaleCell;
- The transaction fee can be deducted from AccountSaleCell by an amount equal to `ConfigCellSecondaryMarket.common_fee`;
- The profits of the three roles of inviter, channel, and DAS need to be stored in the IncomeCell, and the profits of the seller need to be stored in a NormalCell;
- IncomeCell can be created directly in this transaction. IncomeCell needs to meet the following constraints:
    - The total amount recorded must be equal to IncomeCell.capacity;
    - If the lock scripts of the invitor, chanenl, and DAS are the same, their profit-related records must be merged;
    - Other records cannot be merged with profit-related records;
    - The total number of records must be less than or equal to `ConfigCellIncome.max_records`;

#### Quote Trading

##### Create an offer (MakeOffer)

Any user can make active quotes for any DAS account.

**action structure**

```
table ActionData {
   action: "make_offer",
   params: [0x00],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   offer-cell-type
   ConfigCellSecondaryMarket
Inputs:
   BalanceCell {1,}
Outputs:
   OfferCell
   [BalanceCell] {1,}
```

**agreement**

- The locks of all BalanceCells in the input must be consistent and das-lock;
- The lock of OfferCell must be consistent with the BalanceCell in the input and be das-lock;
- The capacity of OfferCell needs to be greater than `ConfigCellSecondaryMarket.offer_cell_basic_capacity + ConfigCellSecondaryMarket.offer_cell_prepared_fee_capacity`;
- And the capacity of OfferCell needs to be greater than or equal to `OfferCell.price` and less than or equal to `OfferCell.price + ConfigCellSecondaryMarket.offer_cell_prepared_fee_capacity`;
- OfferCell needs to comply with the restrictions of other `ConfigCellSecondaryMarket.offer_*` configuration items;
- The inviter information is directly stored in the inviter_lock and channel_lock fields of OfferCell. If these fields are the default values of the Script structure, it will be deemed that there is no inviter and no channel provider;

##### Modify offer (EditOffer)

**action structure**

```
table ActionData {
   action: "edit_offer",
   params: [0x00],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   offer-cell-type
   ConfigCellSecondaryMarket
Inputs:
   OfferCell
   [BalanceCell] {1,}
Outputs:
   OfferCell
   [BalanceCell] {1,}
```

**agreement**

- Only the price and message fields of OfferCell can be modified;
- The transaction fee can be deducted from OfferCell in an amount equal to `ConfigCellSecondaryMarket.common_fee`;
- When the price changes, the capacity can be filled/extracted on demand, while taking into account the minimum change amount of BalanceCell and other restrictions;

##### Cancel offer (CancelOffer)

The user's quotation can be canceled at any time as long as it is not accepted.

**action structure**

```
table ActionData {
   action: "cancel_offer",
   params: [0x00],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   offer-cell-type
   TimeCell
   ConfigCellSecondaryMarket
Inputs:
   OfferCell {1,}
Outputs:
   BalanceCell {1,}
```

**agreement**

- Users can revoke one or more OfferCells at one time;
- The transaction fee can be deducted from OfferCell in an amount equal to `ConfigCellSecondaryMarket.common_fee`;
- The total amount of BalanceCell in the output should be greater than or equal to the total amount of OfferCell in the input minus `ConfigCellSecondaryMarket.common_fee`;

#####AcceptOffer

Users holding the DAS account corresponding to the quotation can accept the quotation before the account expires.

**action structure**

```
table ActionData {
   action: "accept_offer",
   params: [0x00],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   account-cell-type
   eip712-lib
   TimeCell
   ConfigCellAccount
   ConfigCellIncome
   ConfigCellProfitRate
   ConfigCellSecondaryMarket
Inputs:
   OfferCell
   AccountCell
Outputs:
   AccountCell
   IncomeCell // Stores the profits allocated to inviter_lock and channel_lock
   BalanceCell {1,} // Profit allocated to Seller
```

**agreement**

- AccountCell needs to have exactly the same account as OfferCell;
- The transaction fee can be deducted from OfferCell in an amount equal to `ConfigCellSecondaryMarket.common_fee`;
- The remaining transaction fees in OfferCell do not need to be returned to the buyer;
- The profits of the three roles of inviter, channel, and DAS need to be stored in the IncomeCell, and the profits of the seller need to be stored in a NormalCell;
- IncomeCell can be created directly in this transaction. IncomeCell needs to meet the following constraints:
    - The total amount recorded must be equal to IncomeCell.capacity;
    - If the lock scripts of the invitor, chanenl, and DAS are the same, their profit-related records must be merged;
    - Other records cannot be merged with profit-related records;
    - The total number of records must be less than or equal to `ConfigCellIncome.max_records`;

### Expired account auction related transactions

##### Dutch Auction (BidExpiredAccountDutchAuction)

When a user participates in a Dutch auction, the user bids && ships the transaction.

> A Dutch auction is a special type of auction that is characterized by a price that gradually decreases from high to low until someone bids. In this type of auction, the seller sets a maximum price and then gradually lowers the price until someone bids.
> After the grace period, the user account enters the auction period.

**action structure**

```
table ActionData {
     action: "bid_expired_account_dutch_auction",
     params: [],
}
```

**Transaction Structure**

```
CellDeps:
   TimeCell
   das-lock
   account-cell-type
   dpoint-cell-type

Inputs:
   AccountCell{1}   # old owner
   DPointCell{1,}   # bidder
   NormalCell{0,1}  # DidSvr

Outputs:
   AccountCell{1}   # bidder
   DPCell{0,}       # bidder, change
   DPointCell{1,}   # DidSvr, receipt
   NormalCell{0,1}  # DidSvr
   BalanceCell      # old owner
   
```
**Explanation of Role**

In this transaction, the following roles are involved:
- old owner: refers to the owner of the account being auctioned. After the account is successfully auctioned, the storage fee in AccountCell will be returned to the user;
- bidder: refers to the person who bids in this auction;
- DidSvr: refers to the service address owned by the .bit team, used for advance payment, collection, etc. in transactions. Depending on the scenario, the contract may have a whitelist to verify the address;

**agreement**

- AccountCell must be first among Inputs and Outputs;
- The amount of DPointCell paid should be greater than or equal to the auction price. For price calculation rules, please refer to [Formulas](Formulas.md);
- The transaction should occur within the auction period, and the current time satisfies the following constraints:
   ```yaml
   expired_time + grace_period <= current time <= expired_time + grace_period + auction_period
   ```
    - expired_time refers to the account expiration time;
    - grace_period refers to the grace period after the account expires, usually 90 days;
    - auction_period refers to the auction period after the grace period, usually 30 days;
- The properties of AccountCell in Inputs should meet the following conditions:
    - status can only be Normal or LockedForCrossChain;
- The properties of AccountCell in Outputs should meet the following conditions:
    - expired_at is the current time + 1 year;
    - records adds a default parsing record;
    - Status is Normal;
    - registered_at is the current time;
    - last_transfer_account_at, last_edit_manager_at, last_edit_records_at are 0;
- In Inputs, DPointCell can belong to anyone, including the old owner, but it can only belong to one user, and there cannot be multiple users' DPointCell to pay the auction price;
- In Inputs and Outputs, DidSvr's Normal Cell is used to pay or receive changes in Cell storage fees caused by the increase or decrease in the number of DPointCells;
- BalanceCell in Outputs is used to refund the basic storage fee of AccountCell;

### Sub-account related transactions v2

#### Enable sub-account (EnableSubAccount)

This transaction can enable sub-account functionality.

**action structure**

```
table ActionData {
   action: "enable_sub_account",
   params: [0x00],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   account-cell-type
   sub-account-cell-type
   TimeCell
   HeightCell
   ConfigCellAccount
   ConfigCellSubAccount
Inputs:
   AccountCell
   BalanceCell {1,}
Outputs:
   AccountCell // AccountCell.enable_sub_account needs to be set to 1
   SubAccountCell // Create a SubAccountCell to store the Merkle root of the sub-account
   [BalanceCell]
```

**agreement**

- AccountCell.enable_sub_account must be 0 for an unenabled account to initiate this transaction;
- The capacity of SubAccountCell needs to be equal to `ConfigCellSubAccount.basic_capacity + ConfigCellSubAccount.prepared_fee_capacity`;
- The `data` of SubAccountCell needs to be set to the `data.flag = 0xff` state. For other requirements of the corresponding state, see [SubAccountCell](Cell-structure protocol.md#SubAccountCell)`;

#### Set up the sub-account creation script (ConfigSubAccountCustomScript)

Set the type ID of the sub-account creation script. After setting it, you no longer need to put AccountCell into inputs when creating a sub-account. Instead, whether the script is passed or not is used as the criterion for transaction verification. If a sub-account creation script has been set up, the script type ID can be used to execute this transaction again.
After replacing or clearing the sub-account creation script, the creation process will be restored to the way of manually creating sub-accounts through the owner or manager.

**action structure**

```
table ActionData {
   action: "config_sub_account_custom_script",
   params: [0x00/0x01], // Either owner or manager can set the sub-account creation script
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   account-cell-type
   sub-account-cell-type
   TimeCell
   HeightCell
   ConfigCellAccount
   ConfigCellSubAccount
Inputs:
   AccountCell
   SubAccountCell
   BalanceCell {1,}
Outputs:
   AccountCell // AccountCell does not require any modification, it is only used for signature verification
   SubAccountCell //Add a custom creation script type ID, or reset all type IDs to 0
   [BalanceCell]
```

**agreement**

- Owner or manager both have permission to configure sub-account creation scripts;
- The AccountCell must not be in the grace period or beyond;
- When setting the sub-account creation script, if the type ID is not all 0, it is considered to be a valid type ID; if it is all 0, it is considered to be a cleared type ID;

#### Set up sub-account (ConfigSubAccount)

Set the type ID of the sub-account creation script. After setting it, you no longer need to put AccountCell into inputs when creating a sub-account. Instead, whether the script is passed or not is used as the criterion for transaction verification. If a sub-account creation script has been set up, the script type ID can be used to execute this transaction again.
After replacing or clearing the sub-account creation script, the creation process will be restored to the method of manually creating sub-accounts through the owner or manager.

**action structure**

```
table ActionData {
   action: "config_sub_account",
   params: [0x00/0x01],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   account-cell-type
   sub-account-cell-type
   TimeCell
   HeightCell
   ConfigCellAccount
   ConfigCellSubAccount
Inputs:
   AccountCell
   SubAccountCell
   BalanceCell {1,}
Outputs:
   AccountCell // AccountCell does not require any modification, it is only used for signature verification
   SubAccountCell
   [BalanceCell]
Witnesses:
   SubAccountPriceRule1
   SubAccountPriceRule2
   SubAccountPriceRule3
   ...
   SubAccountPreservedRule1
   SubAccountPreservedRule2
   SubAccountPreservedRule3
   ...
```

**agreement**

- Owner or manager both have permission to configure sub-account creation scripts;
- AccountCell cannot change before and after the transaction, and must not be in the **grace period** or later;
- SubAccountCell can only set `flag` to the following values. For other requirements of the corresponding status, see [SubAccountCell](Cell-Structure Protocol.md#SubAccountCell):
    - `0x00`, indicating that the user only uses manual distribution;
    - `0xff`, indicating that the user has enabled the configuration-based automatic distribution feature;
- `SubAccountPriceRule` must be sorted according to the order of its index field;
- `SubAccountPreservedRule` must be sorted according to the order of its index field;
- `SubAccountPriceRule` and `SubAccountPreservedRule` must be able to successfully pass type checking;

#### Update sub-account (UpdateSubAccount)

This transaction accommodates all operations related to sub-account creation, editing, renewal, recycling, etc. Therefore, in addition to `ActionData` which can be used to identify the current transaction, you also need to understand each sub-account witness, which is the meaning of SubAccount witness.

**action structure**

```
table ActionData {
   action: "update_sub_account",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   sub-account-cell-type
   QuoteCell
   TimeCell
   HeightCell
   ConfigCellAccount
   ConfigCellSubAccount
   AccountCell
Inputs:
   SubAccountCell
   [BalanceCell {1,}]
Outputs:
   SubAccountCell // The Merkle root of the subaccount must be updated to the final state
   [BalanceCell]
```

**Trading Agreement**

> The covenants here apply to this transaction.

- The AccountCell must not be in the grace period or beyond;
- The handling fee that can be deducted from SubAccountCell for this transaction shall not be higher than the value configured in `ConfigCellSubAccount.renew_fee`;

##### Create sub-account

**SubAccount witness structure**

```
version: 3
action:create
signature: null
sign_role: null
sign_expired_at: null
new_root: ...
proof: ...
sub_account: ...
edit_key: "manual" // custom_script, custom_rule
edit_value: ...
```

- When `edit_key == manual`, the `edit_value` value is a valid `proof` of `SubAccountMintSign.account_list_smt_root`, which must be able to prove that the currently created account name indeed exists in `SubAccountMintSign.account_list_smt_root`;
- When `edit_key == custom_script`, `edit_value` must be empty;
- When `edit_key == custom_rule`, the first 20 Bytes of `edit_value` are the identification ID of the channel provider, and the last 8 Bytes are the amount paid when registering this account;

> In order to support third-party channels to distribute sub-accounts through custom rules, the witness of each sub-account registered through third-party channels needs to contain the identification ID and registration amount of the channel provider. Subsequently, the dotbit team will use this as a basis and the third Profit distribution is carried out through three-party channels.
>
> When there is no third-party channel, the channel identifier is filled with 20 Bytes of `0x00`, and all profits belong to the dotbit team.

**agreement**

- Both owner or manager have permission to create sub-accounts;
- The registration period is at least 1 year;
- The `SubAccount.sub_account.expired_at` corresponding to the sub-account must be filled with the corresponding expiration time;
- When `SubAccountCell.data.flag == 0x00` indicates:
    - Users can specify the account list for manual Mint through `SubAccountMintSign`;
    - The registration fee for manual Mint sub-accounts is equal to `ConfigCellSubAccount.new_sub_account_price`;
    - `SubAccount.edit_key` stores the utf-8 encoded data of `manual`;
- When `SubAccountCell.data.flag == 0x01` indicates:
    - Users can still specify the account list for manual Mint through `SubAccountMintSign`;
    - The registration fee for manual Mint sub-accounts is equal to `ConfigCellSubAccount.new_sub_account_price`;
    - If it is a manual Mint sub-account, the following information must be carried:
        - `SubAccount.edit_key` stores the utf-8 encoded data of `manual`;
    - The registration fee of each sub-account is governed by a custom script. The registration fee needs to be stored in `SubAccountCell.capacity` and accumulated according to `ConfigCellSubAccount.new_sub_account_custom_price_das_profit_rate` in `SubAccountCell.data.das_profit` and `SubAccountCell.data.owner_profit` respectively. The final profit distribution amount;
    - If it is a sub-account registered according to custom rules, bring the following information:
        - `SubAccount.edit_key` stores the utf-8 encoded data of `custom_script`;
    - All input and output BalanceCells can only use the same lock;
- When `SubAccountCell.data.flag == 0xff` indicates:
    - Users can still specify the account list for manual Mint through `SubAccountMintSign`;
    - The registration fee for manual Mint sub-accounts is equal to `ConfigCellSubAccount.new_sub_account_price`;
    - If it is a manual Mint sub-account, the following information must be carried:
        - `SubAccount.edit_key` stores the utf-8 encoded data of `manual`;
    - Whether each sub-account can be registered is determined based on the execution result of `SubAccountPreservedRule`;
    - The pricing of registerable sub-accounts is determined based on the execution results of `SubAccountPriceRule`;
    - Accounts that are not successfully matched by `SubAccountPriceRule` cannot be registered;
    - If it is a sub-account registered according to custom rules, bring the following information:
        - `SubAccount.edit_key` stores the utf-8 encoded data of `custom_rule`;
        - `SubAccount.edit_value` stores the identification ID of the third-party channel and the CKB amount paid to register this account;
    - All registration fees are put into `SubAccountCell.data.das_profit` and then distributed after statistics;

##### Edit sub-account

**SubAccount witness structure**

```
version: 3
action: edit
signature: ...
sign_role: ...
sign_expired_at: ...
new_root: ...
proof: ...
sub_account: ...
edit_key: "owner" // manager, records
edit_value: ...
```

- `signature` requires the signature of owner or manager depending on the edited field:
    - `digest` is generated by concatenating the following data in order:
    - `from did: ` utf8 bytes of string;
    - A hash generated according to ckb-hash, created by splicing the following fields in order and then hashing:
        - `account_id`
        - `edit_key`
        - `edit_value`
        - `nonce`
        - `sign_expired_at`
- `sign_role` is used to indicate whether `signature` comes from owner or manager;
- `sign_expired_at` is a field designed to prevent `signature` from being replayed. Its timestamp must be less than or equal to the `expired_at` of the main account, that is, all sub-accounts in the transaction;
- When `edit_key == owner`, `edit_value` must be a valid args data of das-lock, and for security reasons, the records field of the sub-account should be cleared;
- When `edit_key == manager`, `edit_value` must be a valid args data of das-lock;
- When `edit_key == records`, `edit_value` must be a molecule encoded `Records` type data;

**agreement**

- If the current time has exceeded `SubAccount.expired_at`, the sub-account can no longer be edited;
- The edited field name and value must be clearly defined through `edit_key, edit_value`;

##### Renew sub-account

**SubAccount witness structure**

```
version: 3
action: renew
signature: null
sign_role: null
sign_expired_at: null
new_root: ...
proof: ...
sub_account: ...
edit_key: "manual" // custom_script, custom_rule
edit_value: ...
```

- In any case, the first 8 bytes of `edit_value` always store the new expiration time after renewal;
- When `edit_key == manual`:
    - If the owner/manager actively renews, in addition to the expiration time of 8 bytes, the value of `edit_value` must also store the valid `proof` account name of `SubAccountRenewSign.account_list_smt_root` that does exist in `SubAccountRenewSign.account_list_smt_root`;
    - If it is renewed by someone else, the value of `edit_value` only needs an expiration time of 8 bytes;
- When `edit_key == custom_rule`, in addition to the expiration time of 8 bytes, the value of `edit_value` also contains 20 bytes for the identification ID of the channel provider, and 8 bytes for the amount paid when registering this account;
- `edit_key == custom_script` is no longer supported and will be deleted in the future;

**agreement**

- The renewal period is at least 1 year;
- There is only one `SubAccountRenewSign` in a transaction, which can only belong to one of owner, manager or other;
- Regardless of any value of `SubAccountCell.data.flag`:
    - The owner or manager can always specify the account list for manual renewal through `SubAccountRenewSign`;
    - `SubAccountRenewSign.signature` must be called by type to `AccountCell.lock` in cell_deps for signature verification and pass;
    - You can use BalanceCell or NormalCell for payment. If you use BalanceCell for payment, you can only use BalanceCell with the same owner lock or manager lock;
    - During manual renewal, the renewal price of each sub-account is equal to `ConfigCellSubAccount.renew_sub_account_price`;
    - Sub-accounts for manual renewal must carry the following information:
        - `SubAccount.edit_key` stores the utf-8 encoded data of `manual`;
        - The first 8 bytes of `SubAccount.edit_value` store the new expired_at timestamp, and the subsequent bytes are filled with the proof corresponding to SubAccountRenewSign;
- When `SubAccountCell.data.flag == 0x00` indicates:
    - Sub-accounts can only be renewed manually;
    - Since the renewal fees are the same, sub-account users can also renew using the self-renewal method at the bottom;
- When `SubAccountCell.data.flag == 0xff` indicates:
    - Payment can only be made using NormalCell;
    - The renewal price is determined based on the execution results of `SubAccountPriceRule`;
    - Sub-accounts that are renewed according to custom rules must carry the following information:
        - `SubAccount.edit_key` stores the utf-8 encoded data of `custom_rule`;
        - The first 8 bytes of `SubAccount.edit_value` store the new expired_at timestamp, and the subsequent bytes store the identification ID of the third-party channel and the CKB amount paid for renewal;
    - All registration fees are put into `SubAccountCell.data.das_profit` and then distributed after statistics;
    - In order to prevent sub-account users from bypassing the custom price for renewal, only when the sub-account cannot match any price, the sub-account user can also renew by self-renewal at the bottom;
- When the above renewal rules cannot match the sub-account, the sub-account user can also renew himself:
    - Payment can only be made using NormalCell;
    - The renewal price is equal to `ConfigCellSubAccount.renew_sub_account_price`;
    - The sub-account to be renewed must carry the following information:
        - `SubAccount.edit_key` stores the utf-8 encoded data of `manual`;
        - The first 8 bytes of `SubAccount.edit_value` store the new expired_at timestamp;

##### Recycle sub-account

When a sub-account expires, anyone can reclaim the sub-account through this transaction.

**action**

```
version: 3
action: recycle
signature: null
sign_role: null
sign_expired_at: null
new_root: ...
proof: ...
sub_account: ...
edit_key: null
edit_value: null
```

**agreement**

- When the sub-account expires and passes the grace period of `ConfigCellAccount.expiration_grace_period`, it can be recycled;
- `witness.new_root` and `witness.proof` should be able to prove that the value of the current sub-account in SMT is empty, that is, 32 bytes of `0x00`;

#### Withdraw sub-account profit (CollectSubAccountProfit)

The profits of DAS and owner are stored in the capacity of the sub-account. When you need to withdraw profits, you need to initiate this transaction.

**action structure**

```
table ActionData {
   action: "collect_sub_account_profit",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   sub-account-cell-type
   balance-cell-type
   ConfigCellSubAccount
   AccountCell
Inputs:
   SubAccountCell
Outputs:
   SubAccountCell
   ChangeCell // Profit of DAS
   ChangeCell // owner's profit
```

**agreement**

- The account IDs of AccountCell and SubAccountCell in cell_deps must be consistent;
- No matter what state the AccountCell is in, profits can be withdrawn;
- Anyone can initiate this transaction to withdraw profits, but unless the input contains dotbit’s official specific lock, the owner’s profits can only be withdrawn to the owner’s address;
- When the profit of either DAS or owner in SubAccountCell is greater than or equal to 61CKB, this transaction can be initiated to withdraw profits;
- When withdrawing, all withdrawable CKB of the owner must be withdrawn, and the `owner_profit` record must be set to 0;
- The handling fee that can be deducted from SubAccountCell for this transaction shall not be higher than the value configured in `ConfigCellSubAccount.common_fee`;

#### Withdraw sub-account third-party channel profit (CollectSubAccountChannelProfit)

Different from the previous `collect_sub_account_profit` transaction, this transaction is a transaction that can only be initiated by dotbit to extract profits and distribute them to third-party channels:

**action structure**

```
table ActionData {
   action: "collect_sub_account_channel_profit",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   sub-account-cell-type
   balance-cell-type
   ConfigCellSubAccount
Inputs:
   SubAccountCell
   NormalCell
Outputs:
   SubAccountCell
   ChangeCell // Profit of channel provider 1
   ChangeCell // Profit of channel provider 2
```

**agreement**

- There must be a NormalCell containing dotbit official specific lock in the input for verification;

### Device management related transactions

#### Create DeviceKeyListCell (CreateDeviceKeyList)

Create a DeviceKeyListCell to store the DeviceKeyList used for signature verification of each device during device management.

**Action structure**

```C
table ActionData {
   action: "create_device_key_list",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   ConfigCellMain
   device-key-list-cell-type

Inputs:
   BalanceCell

Outputs:
   DeviceKeyListCell
   [BalanceCell]
```

**agreement**

- Anyone can create DeviceKeyListCell;
- In the args of das-lock of DeviceKeyListCell, owner and manager must be the same, equal to DeviceKey in DeviceKeyList;
- The locks of BalanceCell in input and BalanceCell in outputs need to be consistent;
- witness must have DeviceKeyList, and the array has only one DeviceKey;
- The contract does not verify the correctness of the DeviceKey in the transaction;

#### Update DeviceKeyListCell (UpdateDeviceKeyList)

Replace the old DeviceKeyListCell with the new DeviceKeyList in the transaction.

**Action structure**

```
table ActionData {
   action: "update_device_key_list",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   ConfigCellMain
   device-key-list-cell-type
Inputs:
   DeviceKeyListCell
Outputs:
   DeviceKeyListCell
```

**agreement**

- Update operations include add or delete operations;
- Only one device can be added or deleted at a time;
- In the UpdateDeviceKeyList operation, the lower limit of DeviceKeyList capacity is 1 and the upper limit of capacity is 10;

#### Delete DeviceKeyListCell (DestroyDeviceKeyList)

When the user chooses to give up device management, the list will be cleared, the cell will be released, and the ckb will be returned to DeviceKeyListCell.witness.refund_lock;

**action structure**

```
table ActionData {
   action: "destroy_device_key_list",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   device-key-list-cell-type
Inputs:
   DeviceKeyListCell {1,}
Outputs:
   BalanceCell
```

**agreement**

- When refunding, the ckb occupied by the cell will be refunded to the refund_lock specified in DeviceKeyListCell. This refund_lock is consistent with the lock of BalanceCell that provided ckb when creating DeviceKeyListCell.

### Authorize related transactions

All current authorization transactions use the same data structure:

```
CellDeps:
   das-lock
   account-cell-type
   eip712-lib
   TimeCell
   HeightCell
   ConfigCellMain
   ConfigCellAccount
Inputs:
   AccountCell
Outputs:
   AccountCell
```

#### Create authorization transaction (CreateApproval)

**action structure**

```
table ActionData {
   action: "create_approval",
   params: [0x00],
}
```

**Transaction Structure**

For the above public structure

**agreement**

- Transaction fees can be paid by AccountCell, but the upper limit of the payment amount cannot exceed the value of ConfigCellAccount.common_fee;
- Satisfy [specific constraints of each type of approval](approval/basic-structure.md);

#### Extend authorization transaction (DelayApproval)

**action structure**

```
table ActionData {
   action: "delay_approval",
   params: [0x00],
}
```

**Transaction Structure**

For the above public structure

**agreement**

- Transaction fees can be paid by AccountCell, but the upper limit of the payment amount cannot exceed the value of ConfigCellAccount.common_fee;
- Satisfy [specific constraints of each type of approval](approval/basic-structure.md);

#### Cancel authorization transaction (RevokeApproval)

**action structure**

```
table ActionData {
   action: "revoke_approval",
   params: [0x00],
}
```

**Transaction Structure**

For the above public structure

**agreement**

- Transaction fees can be paid by AccountCell, but the upper limit of the payment amount cannot exceed the value of ConfigCellAccount.common_fee;
- Satisfy [specific constraints of each type of approval](approval/basic-structure.md);

#### Execute authorized transaction (FulFillApproval)

**action structure**

```
table ActionData {
   action: "fulfill_approval",
   params: [null/0x00],
}
```

**Transaction Structure**

For the above public structure

**agreement**

- Transaction fees can be paid by AccountCell, but the upper limit of the payment amount cannot exceed the value of ConfigCellAccount.common_fee;
- Satisfy [specific constraints of each type of approval](approval/basic-structure.md);

### DIDPoint related transactions
DID Point is a dollar-anchored point launched by the .bit team. Users can obtain the corresponding amount of DID Points after recharging to .bit through some methods. You can then use it to purchase and renew .bit accounts.

Terminology convention
- The lock that exists in ConfigCellDPoint.transfer_whitelist is collectively referred to as `transfer lock` in the following;
- The locks that exist in ConfigCellDPoint.capacity_recycle_whitelist are collectively called `recycle lock`;
- Locks other than the above two are collectively called `user lock`;
#### Mint DIDPoint (MintDp)

**Action structure**

```
table ActionData {
   action: "mint_dp",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   dpoint-cell-type
   ConfigCellMain
   ConfigCellDPoint
Inputs:
   NormalCell {1, }
Outputs:
   DPointCell {1, }
   ChangeCell
```

**agreement**
- inputs must contain at least one cell using super lock;
- outputs must contain at least one DPointCell;
- DPointCell.capacity in outputs must be the sum of ConfigCellDPoint.basic_capacity + ConfigCellDPoint.prepared_fee_capacity;
- DPointCell.lock in outputs must all be `transfer lock`;
-
#### Transfer DIDPoint (TransferDp)


**Action structure**

```
table ActionData {
   action: "transfer_dp",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   eip712-lib
   dpoint-cell-type
   ConfigCellMain
   ConfigCellDPoint
Inputs:
   DPointCell {1, } [A]
Outputs:
   DPointCell {1, } [A]
```

**agreement**
- DPointCells.lock in inputs can only be the same lock, that is, a single `user lock` or a single `transfer lock`;
- There can only be `user lock` from the same user in inputs or outputs;
- There must be at least one `transfer lock` in inputs or outputs;
- The total number of DPoints in inputs is equal to the total number of DPoints in outputs;
- The handling fee for this transaction must be paid by other cells;
-
#### Destroy DIDPoint (BurnDp)


**Action structure**

```
table ActionData {
   action: "burn_dp",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   eip712-lib
   dpoint-cell-type
   ConfigCellMain
   ConfigCellDPoint
Inputs:
   DPointCell {1, } [A]
   NormalCell {1, }
Outputs:
   DPointCell {1, } [A]
   ChangeCell
```

**agreement**
- There must be at least one cell using `transfer lock` or `recycling lock` in inputs;
- All DPointCells in inputs and outputs must come from the same `user lock`;
- The total number of DPoints in inputs must be greater than the total number of DPoints in outputs;
- The number of DPointCells in outputs is allowed to be greater than or equal to the number of DPointCells in inputs, that is, the change DPointCells can be split in this transaction;
- The handling fee for this transaction must be paid by other cells;


## Cross-chain related transactions

### Cross-chain the account to other chains (LockAccountForCrossChain)

When the account needs to cross to other chains, it can modify its status through this transaction and lock itself. Subsequently, the cross-chain node will mint the corresponding NFT in other chains.

**action structure**

```
table ActionData {
   action: "lock_account_for_cross_chain",
   params: [coin_type, chain_id, role],
}
```

- coin_type, 8 bytes, little-endian encoded u64
- chain_id, 8 bytes, little-endian encoded u64
- role, 1 byte, this transaction requires the owner to sign, so it is a constant `0x00`

**Transaction Structure**

```
CellDeps:
   das-lock
   account-cell-type
   eip712-lib
   TimeCell
   ConfigCellAccount
Inputs:
   AccountCell
Outputs:
   AccountCell
```

**agreement**

- AccountCell must not be 90 days before the **grace period**;
- The AccountCell during input must be in **Normal** state, that is, `0x00`;
- The AccountCell in the output must be in the **LockedForCrossChain** state, that is, `0x03`;
- The `lock.args` of AccountCell in the output must be set to the black hole address `0x0300000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
- The parsing record of AccountCell in the output must be cleared;

> This is because if the ownership of the account is transferred after cross-chain, it should also be transferred on CKB. For safety reasons, the account status after the New Year's Eve will be based on the status of the ETH chain, so it needs to be transferred on the CKB chain. Clear the status.

### Return the account from other chains across chains (UnlockAccountForCrossChain)

When an account needs to be crossed back from other chains, if the cross-chain node detects that the account has been destroyed in other chains, it can unlock the account in the ckb chain through multi-signature.

**action structure**

```
table ActionData {
   action: "unlock_account_for_cross_chain",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   account-cell-type
Inputs:
   AccountCell
Outputs:
   AccountCell
```

**agreement**

- The AccountCell during input must be in the **LockedForCrossChain** state, that is, `0x03`;
- The AccountCell in the output must be in the **Normal** state, that is, `0x00`;

### Cross-chain sub-accounts to other chains (LockSubAccountForCrossChain)

It is the same as the cross-chain transaction of the main account, but the transaction structure is different.

**action structure**

```
table ActionData {
   action: "lock_sub_account_for_cross_chain",
   params: [coin_type, chain_id],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   sub-account-cell-type
   TimeCell
   ConfigCellSubAccount
   AccountCell
Inputs:
   SubAccountCell
Outputs:
   SubAccountCell
```

**agreement**

- The subaccount must not be within 90 days before the **grace period**;
- The input sub-account must be in **Normal** state, that is, `0x00`;
- The output neutron account must be in the **LockedForCrossChain** state, that is, `0x03`;
- The `lock.args` of the output neutron account must be set to the black hole address `0x030000000000000000000000000000000000000003000000000000000000000000000000000000000`;
- The parsing records of sub-accounts in the output must be cleared;

### Return the account from other chains across chains (UnlockSubAccountForCrossChain)

It is the same as the cross-chain transaction of the main account, but the transaction structure is different.

**action structure**

```
table ActionData {
   action: "unlock_sub_account_for_cross_chain",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   sub-account-cell-type
Inputs:
   SubAccountCell
Outputs:
   SubAccountCell
```

**agreement**

- The input sub-account must be in the **LockedForCrossChain** state, i.e. `0x03`;
- The output neutron account must be in **Normal** state, that is, `0x00`;

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
