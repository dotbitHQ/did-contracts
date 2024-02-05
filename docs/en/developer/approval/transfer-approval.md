# Transfer Approval

Transfer authorization is primarily used for account transactions in scenarios with a third-party regulatory platform. When this authorization is used, the basic data structure `AccountApproval` has the following values:

- `action` must be the UTF-8 bytes for "transfer", i.e., `0x7472616E73666572`;
- `params` contains the data structure for `AccountApprovalTransfer`

The specific data structure within `AccountApprovalTransfer` is:

``` 
table AccountApprovalTransfer {
    platform_lock: Script,
    protected_until: Uint64,
    sealed_until: Uint64,
    delay_count_remain: Uint8,
    to_lock: Script,
}
```
There are three roles in this authorization:

- **The Authorizer**, which is the current account's owner;
- `to_lock` represents **the Authorized**, which is the role that acquires the final account ownership;
- `platform_lock` represents **the Regulatory Platform**, which is the role supervising the execution of the authorization;

There are two key time nodes in the authorization:

- `protected_until` is the irrevocable time of the authorization, encoded as a little-endian `u64` integer;
- `sealed_until` is the opening time of the authorization, encoded as a little-endian `u64` integer;

Additionally, the authorizer can postpone the execution of the authorization once under special circumstances:

- `delay_count_remain` is the remaining number of times `sealed_until` can be postponed, currently only able to be `1`, encoded as a little-endian `u8` integer;

## Constraints

- The permission required to create this authorization is that of the owner;
- Before the authorization, the status field of the witness for the main account or sub-account must be `0x00`;
- After the authorization, the status field of the witness for the main account or sub-account must be updated to `0x04`, which means the current account is in a transfer approval state. In this state, the account is subject to the following constraints:
  - The main account cannot participate in any transactions that require owner permissions, such as `transfer_account`, `start_account_sale`, `start_account_auction`, `lock_account_for_cross_chain`;
  - Sub-accounts also cannot participate in any transactions that require owner permissions, such as transactions where `sub_account.action == edit` and `edit_key == owner` or `edit_key == manager`;
- The maximum period for `protected_until` cannot exceed 10 days from the current time;
- The maximum period for `sealed_until` cannot exceed 10 days from `protected_until`;
- After authorization, the account owner can execute the authorization at any time;
- Accounts with less than 30 days of validity are prohibited from creating transfer authorization;
- `platform_lock` is the regulatory platform's lock;
- `protected_until` is the irrevocable time of the authorization:
  - After `now > protected_until`, only the signature from the `platform_lock` address can revoke this approval;
- `sealed_until` is the opening time of the authorization, with `protected_until < sealed_until`:
  - The original account owner can delay `sealed_until` through a transaction, with the number of delays determined by `delay_count_remain`;
  - After `now > sealed_until`, it is considered the opening time for the authorization, after which anyone can execute this permission;
- `delay_count_remain` is the number of times the original account owner can delay `sealed_until`, with the initial value of `sealed_until` only able to be `1`. Each delay of `sealed_until` must decrease this value by 1, and when it reaches `0`, the original account owner can no longer delay `sealed_until`;
- `to_lock` is the beneficiary's lock, and the execution result of this approval can only change the account owner's address to this address;
