# Deprecated transaction structure

This document contains all obsolete transactions, which can be ignored unless parsing old historical transactions is required.

## Reverse analysis of related transactions v1

### Declare reverse resolution (DeclareReverseRecord)

> **Deprecated**! This transaction has been deprecated and this document is only used for transaction parsing.

This transaction can mark an account/sub-account as the resolution record of a certain address.

> The creation of ReverseRecordCell is not limited in quantity. When repeated creation occurs, the only one will be selected following [specific deduplication rules] (Cell-structure protocol.md#ReverseRecordCell).

**action structure**

```
table ActionData {
   action: "declare_reverse_record",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   balance-cell-type
   reverse-record-cell-type
   ConfigCellMain
   ConfigCellReverseResolution
Inputs:
   BalanceCell {1,}
Outputs:
   ReverseRecordCell
   [BalanceCell]
```

**agreement**

- The lock of ReverseRecordCell must be consistent with the BalanceCell of inputs[0], so as to ensure that reverse parsing can only be declared if you have the private key of the corresponding address;
- ReverseRecordCell must be das-lock;

### Change reverse resolution (RedeclareReverseRecord)

> **Deprecated**! This transaction has been deprecated and this document is only used for transaction parsing.

This transaction can modify an account corresponding to an existing reverse resolution.

**action structure**

```
table ActionData {
   action: "redeclare_reverse_record",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   balance-cell-type
   reverse-record-cell-type
   ConfigCellMain
   ConfigCellReverseResolution
Inputs:
   ReverseRecordCell
Outputs:
   ReverseRecordCell
```

**agreement**

- Only ReverseRecordCell.data.account can be modified;

### Undo reverse resolution (RetractReverseRecord)

> **Deprecated**! This transaction will be completely deprecated soon and this document is for transaction parsing purposes only.

This transaction can revoke one or more reverse resolution claims.

**action structure**

```
table ActionData {
   action: "retract_reverse_record",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   balance-cell-type
   reverse-record-cell-type
   ConfigCellMain
   ConfigCellReverseResolution
Inputs:
   ReverseRecordCell {1,}
Outputs:
   BalanceCell {1,}
```

**agreement**

- Outputs must contain a refund equal to the ReverseRecordCell storage fee in inputs;

## Sub-account related transactions v1

### Create sub-account (CreateSubAccount)

After opening a sub-account, the user can create a sub-account through this transaction.

**action structure**

```
table ActionData {
   action: "create_sub_account",
   params: [0x00/0x01], // Either owner or manager can create sub-accounts
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
   [QuoteCell] // If the user sets a custom script, then QuoteCell needs to be placed in cell_deps
   ConfigCellAccount
   ConfigCellSubAccount
   AccountCell
   [CustomScriptCell] // If SubAccountCell defines a custom script, you need to reference the script
Inputs:
   SubAccountCell
   BalanceCell {1,}
Outputs:
   SubAccountCell // The Merkle root of the subaccount must be updated to the final state
   [BalanceCell]
```

**agreement**

- Both owner or manager have permission to create sub-accounts;
- The AccountCell must not be in the grace period or beyond;
- When no custom script is set:
    - The registration fee for each sub-account is equal to `ConfigCellSubAccount.new_sub_account_price`;
- When a custom script is set:
    - The registration fee of each sub-account is governed by a custom script. The registration fee needs to be stored in `SubAccountCell.capacity` and according to `ConfigCellSubAccount.new_sub_account_custom_price_das_profit_rate`
      Record the accumulated profit distribution amount in `SubAccountCell.data.das_profit` and `SubAccountCell.data.owner_profit` respectively;
    - All input and output BalanceCells can only use the same lock;

### Edit sub-account (EditSubAccount)

The holder of the sub-account can edit the sub-account through this transaction to transfer, modify the administrator, modify the resolution record, etc.

**action structure**

```
table ActionData {
   action: "edit_sub_account",
   params: [],
}
```

**Transaction Structure**

```
CellDeps:
   das-lock
   sub-account-cell-type
   TimeCell
   HeightCell
   ConfigCellSubAccount
   AccountCell
Inputs:
   SubAccountCell
Outputs:
   SubAccountCell // The Merkle root of the subaccount must be updated to the final state
```

**agreement**

- AccountCell must not be in **grace period** or later;
- The handling fee that can be deducted from SubAccountCell for this transaction shall not be higher than the value configured in `ConfigCellSubAccount.edit_fee`;