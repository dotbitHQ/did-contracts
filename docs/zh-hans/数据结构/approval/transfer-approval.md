# Transfer Approval

Transfer 授权主要用于在有第三方监管平台的场景下进行账户交易，当使用该授权时，基础数据结构 AccountApproval 为以下值：

- action 必须为 transfer 的 utf8 bytes ，即 0x7472616E73666572 ；
- params 内包含的数据结构为 AccountApprovalTransfer

AccountApprovalTransfer 中的具体数据结构为：

```
table AccountApprovalTransfer {
    platform_lock: Script,
    protected_until: Uint64,
    sealed_until: Uint64,
    delay_count_remain: Uint8,
    to_lock: Script,
}
```

该授权有三个角色：

- **授权方**，也就是当前账户的 owner ；
- `to_lock` 代表**被授权方**，也就获得最终账户所有权的角色；
- `platform_lock` 代表**监管平台**，监督授权执行情况的角色；

授权有两个关键的时间节点：

- `protected_until` 授权的不可撤销时间，为小端编码的 u64 整型；
- `sealed_until` ，授权的开放时间，为小端编码的 u64 整型；

最后授权方还可以在特殊情况下推迟一次授权的执行：

- `delay_count_remain` ，剩余的推迟 sealed_until 次数，当前仅能为 1，类型为小段编码的 u8 整型；


## 约束条件

- 创建此授权需提供的权限为 owner ；
- 授权之前主账户、子账户的 witness 的 status 字段必须为 0x00 ；
- 授权之后主账户、子账户的 witness 的 status 字段必须更新为 0x04，其含义为当前账户已经处于 transfer approval 状态，在此状态下账户会受到以下约束：
  - 主账户不能参与任何需要 owner 权限的交易，比如 transfer_account, start_account_sale, start_account_auction, lock_account_for_cross_chain；
  - 子账户也不能参与任何需要 owner 权限的交易，比如 sub_account.action == edit 时 edit_key == owner 或 edit_key == manager 的交易 ；
- protected_until 最大期限不能超过当前时间 10 天；
- sealed_until 最大期限不能超过 protected_until 10 天；
- 授权之后账户 owner 在任何时间都可以执行该授权；
- 账户有效期不足30天的，禁止创建 transfer 授权；
- platform_lock 监管平台 lock；
- protected_until 授权的不可撤销时间：
  - 当 now > protected_until之后，platform_lock 地址的签名才可以撤销该 approval，
- sealed_until 授权的开放时间，protected_until < sealed_until：
  - 账户原 owner 可以通过交易推迟 sealed_until ，可推迟次数由 delay_count_remain 决定；
  - 当 now > sealed_until 之后，就视为到达了开放授权的时间，此后任何人都可以执行此权限；
- delay_count_remain 账户原 owner 可以推迟 sealed_until 的次数，sealed_until 初始值只能为 1，每次推迟 sealed_until 该数值都必须同时 -1 (递减 1)，为 0 时就不再允许账户原 owner 推迟 sealed_until 了；
- to_lock 受益人 lock，该 approval 的执行结果只能是将账户 owner 地址改为此地址；
