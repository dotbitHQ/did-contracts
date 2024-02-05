# Formulas


## Total Amount of PreRegister Transaction

Users are required to pay both **storage fee** and **registration fee** when executing a [PreRegister](Transaction-Structure.md#PreRegister) transaction. Whatever crypto currency the user is paying, it must be exchanged to CKB before this transaction. The detail formula is:

```
// All numbers in the following pseudo-code are of type uint64
storage_fee = (AccountCell_basic_capacity + bytes_of_account + 4) * 100_000_000 + prepared_fee
amount = storage_fee + registration_fee

// Among them, the registration_fee must meet the following conditions
if annual_price_in_USD < exchange_rate_of_CKB {
  annual_price_in_CKB = annual_price_in_USD * 100_000_000 / exchange_rate_of_CKB
} else {
  annual_price_in_CKB = annual_price_in_USD / exchange_rate_of_CKB * 100_000_000
}
annual_price_in_CKB = annual_price_in_CKB - (annual_price_in_CKB * invited_discount_rate / 10000)

assert(registration_fee >= annual_price_in_CKB)
```

- **AccountCell_basic_capacity** can be retrieved from `ConfigCellAccount.basic_capacity`.
- **prepared_fee** can be retrieved from `ConfigCellAccount.prepared_fee_capacity`.
- **annual_price_in_USD** can be retrieved from `ConfigCellPrice.prices` , its unit is **USDT**.
- **exchange_rate_of_CKB** can be retrieved from [QuoteCell](./Cell-Structure.md#QuoteCell) , its unit is **USDT/CKB**.
- **invited_discount_rate** can be retrieved from `ConfigCellAccount.discount`.
- The total amount is stored in `PreAccountCell.capacity` during the entire registration process.
- The registration fee must be greater than or equal to the annual fee for one year, i.e. a minimum of one year must be registered.


## Duration Calculation After Registerd/Renewed

After a successful registration or renewal of an account, the duration which the account finally received will be calculated according to the following formula:

```
// All numbers in the following pseudo-code are of type uint64
if annual_price_in_USD < exchange_rate_of_CKB {
  annual_price_in_CKB = annual_price_in_USD * 100_000_000 / exchange_rate_of_CKB
} else {
  annual_price_in_CKB = annual_price_in_USD / exchange_rate_of_CKB * 100_000_000
}
annual_price_in_CKB = annual_price_in_CKB - (annual_price_in_CKB * discount_rate / 10000)

duration_received = registration_fee * 365 / annual_price_in_CKB * 86400
```


## Profit Distribution upon Successful Registration

When an account is successfully registered, the registration fees carried in the PreAccountCell will be distributed to the various participants in a specific percentage, and the percentage of profit for each role can be found in ConfigCellProfitRate：

```
// All numbers in the following pseudo-code are of type uint64
profit = registration_fee

if is_inviter_exist {
  profit_of_inviter = profit * inviter_profit_rate
}
if is_channel_exist {
  profit_of_channel = profit * channel_profit_rate
}
profit_of_proposal_creator = profit * proposal_creator_profit_rate
profit_of_proposal_confirmer = profit * proposal_confirmer_profit_rate

profit_of_DAS = profit - profit_of_inviter - profit_of_channel - profit_of_proposal_creator - profit_of_proposal_confirmer
```
