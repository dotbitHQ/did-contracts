# Cell 结构协议

## 协议符号约定

本文档以下内容都会采用统一的结构描述一个 cell：

```
lock: ...
type: ...
data: ...
witness: ...
```

其中 `lock, type, outputs_data` 都是每个 cell 必定包含的信息，从 RPC 接口返回的数据结构中也可以看到，而 `data` 就是这笔交易中与 cell 对应的 `outputs_data` 。`witness` 比较特殊，它和 cell
之间是没有关联关系的，所以这里的 `witness` 特指 DAS witness ，而且仅仅指 DAS witness 中的 `entity` 部分，因为 DAS witness 中存放了它自己对应哪个 cell
的相关信息，所以才有了关联关系，详见 [数据存储方案.md](数据存储方案.md) 。

**data 中所有的字段名意味着一段按照特定偏移量解析的数据**，因为 data 的体积会影响需要质押的 CKB 数量，所以其中除了按照文档中给出的偏移量来切分数据外本身没有任何数据结构。**witness 中所有的字段名意味着一个 molecule 编码的数据结构**
，首先需要使用对应结构的结构体/类去解析数据，然后才能访问对应字段。

在描述 cell 结构时可能看到以下符号：

| 符号                 | 说明                                                                                                                                            |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| lock: <...>         | 代表一个特定的 script ，其 code_hash, args, hash_type 都有简单的约定，所以就不列举明细了                                                        |
| type: <...>         | 同上                                                                                                                                            |
| hash(...)           | 指代这里存放的数据是通过什么计算得出的 hash 值                                                                                                  |
| ======              | 在描述 cell 结构的代码段类，此分隔符意味着下面的内容是对特定 molecule 结构的详细介绍，但最新的 schema 请以 das-types 仓库中的 schemas/ 目录为准 |
| ConfigCellXXXX.yyyy | 指代数据需要去某个 ConfigCell 的 witness 中的特定字段获取，详见 [ConfigCell](#ConfigCell)                                                       |

## 数据结构

### ApplyRegisterCell

申请注册账户，用户在真正执行注册前必须先创建此 Cell 进行申请，然后等待 `ConfigCellApply.apply_min_waiting_block_number` 时间后才能使用此 Cell
进行注册。这样设计的目的是为了防止用户注册账户的交易在上链的过程中被恶意拦截并被抢注。

#### 结构

```
lock: <ckb_lock_script>
type: <apply-register-cell-type>
data:
  hash(lock_args + account) // account 包含 .bit 后缀
  [height] // Deprecated
  [timestamp] // Deprecated
```

- hash ，账户名和 owner 的 lock_args 的 hash；
- height ，值为 cell 创建时的区块高度(小端)，从 heightcell 里获取；
- timestamp ，值为 cell 创建时的时间戳(小端)，从 timecell 里获取；

#### 体积

实际体积：142 Bytes

### PreAccountCell

当 [ApplyRegisterCell](#ApplyRegisterCell) 在链上存在超过 `ConfigCellApply.apply_min_waiting_time` 时间之后，用户就可以将其转换为一个 PreAccountCell ，等待 Keeper
通过创建 [ProposalCell](#ProposalCell) 提案将其最终转换为 [AccountCell](#AccountCell) 。

#### 结构

```
lock: <always_success>
type: <pre-account-cell-type>
data:
  hash(witness: PreAccountCellData)
  id // account ID，生成算法为 hash(account)，然后取前 20 Bytes。这个包含 .bit 后缀

witness:
  table Data {
    old: table DataEntityOpt {
        index: Uint32,
        version: Uint32,
        entity: PreAccountCellData
    },
    new: table DataEntityOpt {
      index: Uint32,
      version: Uint32,
      entity: PreAccountCellData
    },
  }

======
table PreAccountCellData {
    // Separate chars of account.
    account: AccountChars, // 不带 .bit
    // If the PreAccountCell cannot be registered, this field specifies to whom the refund should be given.
    refund_lock: Script,
    // If the PreAccountCell is registered successfully, this field specifies to whom the account should be given.
    owner_lock_args: Bytes,
    // The lock script of inviter,
    inviter_id: Bytes,
    inviter_lock: ScriptOpt,
    // The lock script of channel,
    channel_lock: ScriptOpt,
    // Price of the account at the moment of registration.
    price: PriceConfig,
    // The exchange rate between CKB and USD.
    quote: Uint64,
    // The discount rate for invited user
    invited_discount: Uint32,
    // The created timestamp of the PreAccountCell.
    // Deprecated
    created_at: Uint64,
    // The initial records should be write into the AccountCell when it is created successfully.
    initial_records: Records,
    // Lock for cross chain when the AccountCell minted successfully.
    initial_cross_chain: ChainId,
}

table ChainId {
    // Indicate if this field should work. (0x00 means false, 0x01 mean true)
    checked: Uint8,
    coin_type: Uint64,
    chain_id: Uint64,
}

vector AccountChars <AccountChar>;

table AccountChar {
    // Name of the char set which the char belongs.
    char_set_name: Uint32,
    // Bytes of the char.
    bytes: Bytes,
}
```

- account，用户实际注册的账户名，不包含 `.bit` 后缀；
- refund_lock，假如 PreAccountCell 最终无法通过提案时，退款的 lock 脚本，即地址；
- owner_lock_args，假如 PreAccountCell 最终通过提案时，[AccountCell.lock.args](#AccountCell) 的值，即 das-lock 的 args；
- inviter_id: 主要目的方便服务端展示邀请者名称；
- inviter_lock，邀请者的 lock script，利润分配会被转入 IncomeCell 中并以此 lock script 记账；
- channel_lock，渠道商的 lock script，利润分配会被转入 IncomeCell 中并以此 lock script 记账；
- price，账户注册时的售价；
- quote，账户注册时的 CKB 的美元单价；
- created_at，PreAccountCell 创建时 TimeCell 的时间；
- initial_records，AccountCell 创建成功时的初始解析记录；
- initial_cross_chain，AccountCell 创建成功时是否直接锁定为跨链状态；

#### 利润以及注册所获时长的计算逻辑

创建 PreAccountCell 时，用户就需要支付注册费以及创建各种 Cell 所需的基础费用，此时用户应支付 CKB 数量的计算公式为：

```
// 这一段是伪代码，存在从上往下执行的上下文环境
存储费 =  (AccountCell 基础体积 + 账户长度 + 4) * 100_000_000 + 预存手续费

利润 = PreAccountCell.capacity - 存储费

if 美元年费 < CKB 汇率 {
  CKB 年费 = 美元年费 * 100_000_000 / CKB 汇率
} else {
  CKB 年费 = 美元年费 / CKB 汇率 * 100_000_000
}

CKB 年费 = CKB 年费 - (CKB 年费 * 折扣率 / 10000) // 折扣率是以 10000 为底的百分数

注册时长 = 利润 * 365 / CKB 年费 * 86400
```

- 年份需要大于等于 1 ，实际计算时按照 365 \* 86400 秒为一年来计算；
- **账户注册年费** 保存在 [ConfigCellPrice.prices](#ConfigCell) 中，单位为 **美元**；
- **CKB 汇率**从 QuoteCell 获取，单位为 **美元/CKB**，前面在 [数据存储方案.md](./数据存储方案.md) 的 **美元的单位** 一节我们约定了 1 美元记为 `1_000_000` ，因此如果 QuoteCell 中记录的是 `1_000`
  那么也就意味着 CKB 汇率就是 `0.001 美元/CKB`；
- **AccountCell 基础成本** 只在调整 Cell 数据结构时会发生变化，可以认为是固定的常量，查看对应 Cell 的 **体积** 即可获得；
- **Account 字节长度**，由于 AccountCell 会在 data 字段保存完整的账户，比如 `das.bit` 那么保存的就是 `0x6461732E626974` ，因此需要再加上这部分体积；
- 带除法的都是自动取整

#### 体积

基础体积：126 Bytes

实际体积：取决于注册的账户的长短、注册的年份、注册时刻的 CKB 单价、是否是请注册等

### ProposalCell

用户创建 [PreAccountCell](#PreAccountCell) 之后，就需要 Keeper 将它们收集起来发起提案，也就是创建 ProposalCell
，只有当提案等待一定时间后才能通过，所谓通过就是创建一笔交易消费提案，并将其应用的 [PreAccountCell ](#PreAccountCell) 转换为最终的 [AccountCell](#AccountCell)。这一过程会确保账户名在链上的唯一性。

#### 结构

```
lock: <always_success>
type: <proposal-cell-type>,

data:
  hash(witness: ProposalCellData)

witness:
  table Data {
    old: None,
    new: table DataEntityOpt {
      index: Uint32,
      version: Uint32,
      entity: ProposalCellData
    },
  }

======
table ProposalCellData {
    proposer_lock: Script,
    created_at_height: Uint64,
    slices: SliceList,
}

vector SliceList <SL>;

// SL is used here for "slice" because "slice" may be a keyword in some languages.
vector SL <ProposalItem>;

table ProposalItem {
  account_id: AccountId,
  item_type: Uint8,
  // When account is at the end of the linked list, its next pointer should be None.
  next: AccountId,
}

====== 举例来说看起来就像下面这样
table ProposalCellData {
  proposer_lock: Script,
  slices: [
    [
      { account_id: xxx, item_type: exist, next: xxx },
      { account_id: xxx, item_type: new, next: xxx },
    ],
    [
      { account_id: xxx, item_type: proposed, next: xxx },
      { account_id: xxx, item_type: new, next: xxx },
      { account_id: xxx, item_type: new, next: xxx },
    ],
    [
      { account_id: xxx, item_type: exist, next: xxx },
      { account_id: xxx, item_type: new, next: xxx },
      { account_id: xxx, item_type: new, next: xxx },
      { account_id: xxx, item_type: new, next: xxx },
    ],
    ...
  ]
}
```

- proposer_lock，提案发起者的 lock script；如果提案被回收，那么回收的 CKB 就应该转入此 lock script；如果提案通过，那么属于提案发起者的利润就应该转入 IncomeCell 并以此 lock script 记账；
- created_at_height ，提案发起时从时间 cell 获取到的当前高度；
- slices，当前提案通过后 AccountCell 链表被修改部分的最终状态，其解释详见 `TODO`；
- item_type 含义说明
    - exist ，值对应 0x00 ，表明此提案发起时，account_id 所指账户已经为注册状态，可以在链上找到 AccountCell；
    - proposed ，值对应 0x01，表明此提案发起时，account_id 所指账户已经为预注册状态，可以在链上找到 PreAccountCell，当此提案的前置提案通过时会将其转换为 AccountCell；
    - new ，值对应 0x02，表明此提案发起时，account_id 所指账户已经为预注册状态，可以在链上找到 PreAccountCell ，此提案通过时会将其转换为 AccountCell；

#### 体积

基础体积：106 Bytes

实际体积：106 Bytes

### AccountCell

当提案确认后，也就是 [ProposalCell](#ProposalCell) 被消费时，[PreAccountCell](#PreAccountCell) 才能被转换为 AccountCell ，它存放账户的各种信息。

#### 结构

```
lock:
  code_hash: <das-lock>
  type: type
  args: [ // 这是 das-lock 的 args 结构，同时包含了 owner 和 manager 信息
    owner_algorithm_id,
    owner_pubkey_hash,
    manager_algorithm_id,
    manager_pubkey_hash,
  ]
type: <account-cell-type>

data:
  hash(witness: AccountCellData) // 32 bytes
  id // 20 bytes，自己的 ID，生成算法为 hash(account)，然后取前 20 Bytes
  next // 20 bytes，下一个 AccountCell 的 ID
  expired_at // 8 bytes，小端编码的 u64 时间戳
  account // expired_at 之后的所有 bytes，utf-8 编码，AccountCell 为了避免数据丢失导致用户无法找回自己用户所以额外储存了 account 的明文信息, 包含 .bit 后缀

witness:
  table Data {
    old: table DataEntityOpt {
        index: Uint32,
        version: Uint32,
        entity: AccountCellData
    },
    new: table DataEntityOpt {
      index: Uint32,
      version: Uint32,
      entity: AccountCellData
    },
  }

======
table AccountCellData {
    // The first 160 bits of the hash of account.
    id: AccountId,
    // Separate chars of account.
    account: AccountChars,
    // AccountCell register timestamp.
    registered_at: Uint64,
    // AccountCell last action timestamp.
    last_transfer_account_at: Timestamp,
    last_edit_manager_at: Timestamp,
    last_edit_records_at: Timestamp,
    // The status of the account, 0x00 means normal, 0x01 means being sold, 0x02 means being auctioned.
    status: Uint8,
    records: Records,
    // The status of sub-account function, 0x00 means disabled, 0x01 means enabled.
    enable_sub_account: Uint8,
    // The price of renewing sub-account for one year.
    renew_sub_account_price: Uint64,
}

array AccountId [byte; 20];

table Record {
    record_type: Bytes,
    record_label: Bytes,
    record_key: Bytes,
    record_value: Bytes,
    record_ttl: Uint32,
}

vector Records <Record>;
```

- id ，账户 ID，对账户名(**含后缀**)计算 hash 之后，取前 20 bytes 就是账户 ID，全网唯一；
- account ，账户名字段；
- registered_at ，注册时间；
- status ，状态字段：
    - 0 ，正常；
    - 1 ，出售中；
    - 2 ，拍卖中；
    - 3 ，到期拍卖中；
- records ，解析记录字段，**此字段仅限有管理权的用户编辑**；
- enable_sub_account ，状态字段：
    - 0 ，未启用子账户；
    - 1 ，已启用子账户；

#### das-lock

das-lock 是为 DAS 设计的一个特殊 lock script ，它 **会根据 args 中的 xx_algorithm_id 部分去动态加载不同的验签逻辑执行**。args 中的 **xx_algorithm_id 都是 1 byte，pubkey_hash 都是取前
20 bytes** 。

涉及验签的交易需要在 witnesses 中的 ActionData.params 标明当前交易使用的权限是 owner 还是 manager ，**owner 使用 0，manager 使用 1**。

#### 体积

实际体积：`201 + n` Bytes，`n` 取决于 account 的长度。

链上体积：取决于 ConfigCellAccount 里的配置项

### IncomeCell

一种用来解决批量到账时单笔账目不足 61 CKB 无法独立存在的 Cell ，这个 Cell 以及其相关解决方案主要有以下优点：

1. 可以在批量转账的场景下，解决单笔账目不足 61 CKB 而无法创建普通 Cell 的问题；
2. 解决了总账目也不足 61 CKB 无法创建 IncomeCell 的问题；
3. 通过复用 IncomeCell 实现上面优点 1、2 的同时降低了多笔交易抢占同一个 IncomeCell 的概率；

#### 结构

```
lock: <always_success>
type: <income-cell-type>

data: hash(witness: IncomeCellData)

======
table IncomeCellData {
    creator: Script,
    records: IncomeRecords,
}

vector IncomeRecords <IncomeRecord>;

table IncomeRecord {
    belong_to: Script,
    capacity: Uint64,
}
```

Witness 中的主要字段如下：

- creator ，记录了这个 IncomeCell 的创建者，任何人都可以自由的创建 IncomeCell ，当 records 中只有创建者一条记录时，这个 IncomeCell 就只能用于确认提案交易；
- records ，账目记录，记录了 IncomeCell 的 capacity 分别属于哪些 lock script ，每个 lock script 拥有多少 CKB；

#### 体积

实际体积：106 Bytes

链上体积：取决于 ConfigCellIncome 里的配置项

### AccountSaleCell

这是一种用来描述账户出售信息的 Cell ，每一个出售中的账户都有一个对应的 AccountSaleCell。

#### 结构

```
lock: <das-lock>
type: <account-sale-cell-type>

data: hash(witness: AccountSaleCellData)

======
table AccountSaleCellData {
    // Account ID of associated account.
    account_id: AccountId,
    // Account name of associated account.
    account: Bytes,
    // The price user willing to sell the account.
    price: Uint64,
    // A customizable description for the account.
    description: Bytes,
    // timestamp of account sale start.
    started_at: Uint64,
}
```

Witness 中的主要字段如下：

- account_id ，关联的账户 ID；
- account ，账户名；
- price ，账户售价；
- description ，用户自定义的简介信息；
- started_at ，账户开始出售时的时间戳；

#### 体积

实际体积：`148 ~ 170` Bytes，具体取决于 das-lock 的 args 长度。

链上体积：取决于 ConfigCellSecondaryMarket 里的配置项

### AccountAuctionCell

这是一个描述账户竞拍信息的 Cell，每一个竞拍中的账户都有一个对应的 AccountAuctionCell。

#### 结构

```
lock: <das-lock>
type: <accont-sale-cell-type>

data: hash(witness: AccountAuctionCellData)

======
table AccountAuctionCellData {
    // The account ID of associated account.
    account_id: AccountId,
    // Account name of associated account.
    account: Bytes,
    // The description of the auction.
    description: Bytes,
    // The opening price of the auction in shannon.
    opening_price: Uint64,
    // The bid increment rate.
    increment_rate_each_bid: Uint32,
    // The start timestamp of auction, unit in seconds.
    started_at: Uint64,
    // The end timestamp of auction, unit in seconds.
    ended_at: Uint64,
    // The current bidder's lock script.
    current_bidder_lock: Script,
    // The current bidder's bid price.
    current_bid_price: Uint64,
    // The profit rate for previous bidder in each bid, the seller will be treated as the first bidder.
    prev_bidder_profit_rate: Uint32,
}
```

- account_id ，拍卖账户的账户 ID；
- account ，拍卖账户的账户名；
- description ，拍卖的描述信息；
- opening_price ，起拍价；
- increment_rate_each_bid ，每次出价的加价比例；
- started_at ，竞拍的起始时间；
- ended_at ，竞拍的结束时间；
- current_bidder_lock ，当前出价人的 lock script；
- current_bid_price ，当前的出价；
- prev_bidder_profit_rate ，每轮出价后，前一个拍卖者的可获得的利润；

#### 体积

实际体积：`148 ~ 170` Bytes，具体取决于 das-lock 的 args 长度。

链上体积：取决于 ConfigCellSecondaryMarket 里的配置项

### ~~ReverseRecordCell~~

> **Deprecated**！此 Cell 已经废弃，此文档仅供交易解析使用。

存放反向解析记录的 Cell ，同一个地址上可能有多个，需要按照[协议](../反向解析机制/反向解析机制.md)进行去重。

#### 结构

```
lock: <ckb_lock_script> | <das_lock_script>
type: <reverse-record-cell-type>
data:
  account // 反向解析对应的账户名
```

#### 体积

`116 + n` Bytes，`n` 具体取决于 lock 的 args 长度以及 account 的长度。

### ReverseRecordRootCell

存放反向解析记录 SMT Root 的 Cell 。

#### 结构

```
lock: <always_success>
type: <reverse-record-root-cell-type>
data:
  smt_root
```

- smt_root ，反向解析 SMT 的 Root ；

#### 体积

`116` Bytes

### OfferCell

报价 Cell ，用户可以通过此 Cell 给出任意账户名的报价，甚至尚未注册的账户名也可以。

#### 结构

```
lock: <das-lock>
type: <offer-cell-type>
data: hash(witness: OfferCellData)

======
table OfferCellData {
    // The account of the offer .
    account: Bytes,
    // The price of the offer.
    price: Uint64,
    // The message from the offer maker to the seller.
    message: Bytes,
    // The lock script of inviter.
    inviter_lock: Script,
    // The lock script of channel.
    channel_lock: Script,
}
```

#### 体积

`148 ~ 170` Bytes，具体取决于 das-lock 的 args 长度。

### SubAccountCell

#### 结构

```
lock: <always_success>
type:
  code_hash: <sub-account-cell-type>,
  type: type,
  args: [account_id], // 账户 ID ，也就是和 AccountCell.data.id 相同的值

data: [ smt_root ][ das_profit ][ owner_profit ][ flag ]
// OR
data: [ smt_root ][ das_profit ][ owner_profit ][ flag ][ custom_script ][ script_args ]
// OR
data: [ smt_root ][ das_profit ][ owner_profit ][ flag ][ status_flag ][ price_rules_hash ][ preserved_rules_hash ]
```

- smt_root ， 32 bytes ，也就是对应主账户下的所有子账户的 merkle root ；
- das_profit ，8 bytes ， 由于 SubAccountCell 也负责存放属于 DAS 官方的利润，这个值就是指明 capacity 当中有多少 DAS 官方的利润利润；
- owner_profit ，8 bytes ，由于 SubAccountCell 也负责存放属于父账户 AccountCell 的 owner 的利润，这个值就是指明 capacity 当中有多少 owner 的利润；
- flag ，1 byte，标识后面几个字段如何解析：
  - 当 `flag == 0x00` 或不存在时，指明用户仅使用手工分发，省略所有之后的字段，只有父账户可以创建子账户；
  - 当 `flag == 0x01`，指明用户使用了基于动态库的自定义价格，省略 `price_rules_hash` 和 `preserved_rules_hash` 两个字段，而 `custom_script` 就是动态库的 `type.args` ，此时 `script_args` 必须为 10 bytes；
  - 当 `flag == 0xff` ，指明用户启用了基于配置的自动分发特性，省略 `custom_script` 和 `script_args` 两个字段，默认 `status_flag = 0x00` 且 `price_rules_hash` 和 `preserved_rules_hash` 需要全部填充 `0x00`；
- custom_script ，32 bytes ，就是动态库的 `type.args` ；
- script_args ，10 bytes ，存放由自定义脚本使用的 witness 的 hash 的前 10 bytes；
- status_flag ，1 byte ，标识是否开启基于配置的自动分发特性，开启为 0x01 ，关闭为 0x00，默认为开启状态；
- price_rules_hash ， 10 bytes 的 hash ，将所有 witness 类型为 SubAccountPriceRule 的 `rules` 部分 bytes 直接拼接并计算 hash 后取前 10 bytes；
- preserved_rules_hash ， 10 bytes 的 hash ，将所有 witness 类型为 SubAccountPreservedRule 的 `rules` 部分 bytes 直接拼接并计算 hash 后取前 10 bytes；

#### custom_script 推导动态库 Type ID

当 `flag == 1` 时，需要将 `custom_script` 作为 `type.args` 来使用组成以下 Script 结构：

```
{
  code_hash: "0x00000000000000000000000000000000000000000000000000545950455f4944",
  hash_type: flag,
  args: custom_script
}
```

对上述结构体进行 molecule 编码后，进行 blake2b hash 运算即可获得自定义脚本的 type ID 。

> 该 Cell 没有关联的 witness 。

#### 体积

- 当 `flag == 0` 时为 `143` Bytes
- 当 `flag == 1` 时为 `143 + 42` Bytes
- 当 `flag == 255` 时为 `143 + 20` bytes

### ExpiredAccountAuctionCell

#### 结构

## ConfigCell

这是一个在链上保存 DAS 配置的 Cell，目前只通过 DAS 超级私钥手动更新。因为 CKB VM 在加载数据时存在性能存在数据越大开销急剧增大的问题，所以采用了将不同配置分散到多个 ConfigCell 中的保存方式。

#### 结构

所有的 ConfigCell 都遵循以下基本的数据，不同的 ConfigCell 主要是通过 `type.args` 来体现：

```
lock: <super_lock> // 一个官方的 ckb 多签 lock
type:
    code_hash: <config-cell-type>,
    type: type,
    args: [DateType], // 这里保存了一个 uint32 的 DateType 值，这个值主要是为了方便辨别和查询 ConfigCell
data:
    hash(witness) // 所有的 ConfigCell 的 data 都是一个计算自 witness 的 Hash
```

> 各个 ConfigCell 的 DataType 详见下面 [Type 常量列表](#Type 常量列表) 。

#### 体积

所有 ConfigCell 在 cell 结构上都一样，所以体积都是 `130` Bytes 。

#### ConfigCellAccount

**witness：**

```
table ConfigCellAccount {
    // The maximum length of accounts in characters.
    max_length: Uint32,
    // The basic capacity AccountCell required, it is bigger than or equal to AccountCell occupied capacity.
    basic_capacity: Uint64,
    // The fees prepared for various transactions for operating an account.
    prepared_fee_capacity: Uint64,
    // The grace period for account expiration in seconds
    expiration_grace_period: Uint32,
    // The minimum ttl of record in seconds
    record_min_ttl: Uint32,
    // The maximum size of all records in molecule encoding.
    record_size_limit: Uint32,
    // The transaction fee of each action.
    transfer_account_fee: Uint64,
    edit_manager_fee: Uint64,
    edit_records_fee: Uint64,
    common_fee: Uint64,
    // The action frequency limit for managing an account.
    transfer_account_throttle: Uint32,
    edit_manager_throttle: Uint32,
    edit_records_throttle: Uint32,
    common_throttle: Uint32,
}
```

#### ConfigCellApply

**witness：**

```
table ConfigCellApply {
    // The minimum number of waiting blocks before an ApplyRegisterCell can be converted into a PreAccountCell.
    apply_min_waiting_block_number: Uint32,
    // The maximum number of waiting blocks before an ApplyRegisterCell can be converted into a PreAccountCell.
    apply_max_waiting_block_number: Uint32,
}
```

#### ConfigCellCharSetXXXX

注册账户可用字符集 Cell ，这个 Cell 的设计是除了 Emoji, Digit 两个全局字符集外，其他各种语言都可以有一个自己独立的字符集，不同语言之间不能混用，**只有全局字符集 Emoji，Digit 能和任何语言混用**。

**witness：**

```
length|global|char|char|char ...
```

这个 cell 的 witness 在其 entity 部分**存储的是纯二进制数据**，未进行 molecule 编码。其中前 4 bytes 是 uint32 的数据总长度，**包括这 4 bytes 自身**；第 5 bytes 记录当前字符集是否为全局字符集，0x00
就不是，0x01 就是；之后就是可用字符的字节，全部为 utf-8 编码对应的字节，每个字符之间以 `0x00` 分割。

目前已经有的字符集为：

- ConfigCellCharSetEmoji
- ConfigCellCharSetDigit
- ConfigCellCharSetEn

#### ConfigCellIncome

**witness：**

```
table ConfigCellIncome {
    // The required basic capacity for an IncomeCell, which must be greater than or equal to the occupied capacity of the IncomeCell.
    basic_capacity: Uint64,
    // The maximum number of records an IncomeCell can hold.
    max_records: Uint32,
    // The minimum capacity required to determine if a record should be transferred.
    min_transfer_capacity: Uint64,
}
```

#### ConfigCellMain

**witness：**

```
table ConfigCellMain {
    // Global DAS system switch: 0x01 indicates system on, 0x00 indicates system off.
    status: Uint8,
    // The table of type ID for type scripts.
    type_id_table: TypeIdTable,
    // The table of code_hash for lock scripts.
    das_lock_out_point_table: DasLockOutPointTable,
    // The table of type ID for lock scripts.
    das_lock_type_id_table: DasLockTypeIdTable,
}

table TypeIdTable {
    account_cell: Hash,
    apply_register_cell: Hash,
    balance_cell: Hash,
    income_cell: Hash,
    pre_account_cell: Hash,
    proposal_cell: Hash,
    account_sale_cell: Hash,
    account_auction_cell: Hash,
    offer_cell: Hash,
    reverse_record_cell: Hash,
    sub_account_cell: Hash,
    eip712_lib: Hash,
    reverse_record_root_cell: Hash,
}

table DasLockOutPointTable {
    ckb_signall: OutPoint,
    ckb_multisign: OutPoint,
    ckb_anyone_can_pay: OutPoint,
    eth: OutPoint,
    tron: OutPoint,
    ed25519: OutPoint,
}

table DasLockTypeIdTable {
    ckb_signhash: Hash,
    ckb_multisig: Hash,
    ed25519: Hash,
    eth: Hash,
    tron: Hash,
    doge: Hash,
}
```

#### ConfigCellPrice

**witness：**

```
table ConfigCellPrice {
    // discount configurations
    discount: DiscountConfig,
    // Price list of different account length.
    prices: PriceConfigList,
}

table DiscountConfig {
    // The discount rate for invited user
    invited_discount: Uint32,
}

vector PriceConfigList <PriceConfig>;

table PriceConfig {
  // The length of the account, ".bit" suffix is not included.
  length: Uint8,
  // The price of registering an account. In USD, accurate to 6 decimal places.
  new: Uint64,
  // The price of renewing an account. In USD, accurate to 6 decimal places.
  renew: Uint64,
}
```

- discount ，DAS 中各种情况下的折扣额度列表；
- prices ，DAS 不同长度账户名的价格列表；

#### ConfigCellProposal

**witness：**

```
table ConfigCellProposal {
    // How many blocks required for every proposal to be confirmed.
    proposal_min_confirm_interval: Uint8,
    // How many blocks to wait before extending the proposal.
    proposal_min_extend_interval: Uint8,
    // How many blocks to wait before recycle the proposal.
    proposal_min_recycle_interval: Uint8,
    // How many account_cells every proposal can affect.
    proposal_max_account_affect: Uint32,
    // How many pre_account_cells be included in every proposal.
    proposal_max_pre_account_contain: Uint32,
}
```

- proposal_min_confirm_interval ，提案确认的最小等待区块数；
- proposal_min_extend_interval ，提案扩展的最小等待区块数；
- proposal_min_recycle_interval ，提案回收的最小等待区块数；
- proposal_max_account_affect ，单个提案可以涉及的最大 AccountCell 数；
- proposal_max_pre_account_contain ，单个提案可以涉及的最大 PreAccountCell 数；

> `proposal_max_account_affect `和 `proposal_max_pre_account_contain` 其中任何一个到达上限后就必须忽略 `proposal_min_extend_interval` 的限制创建新的提案，如此来控制每个提案的体积上限。

#### ConfigCellProfitRate

**witness：**

```
table ConfigCellProfitRate {
    // The profit rate of inviters who invite people to buy DAS accounts.
    inviter: Uint32,
    // The profit rate of channels who support people to create DAS accounts.
    channel: Uint32,
    // The profit rate for who created proposal
    proposal_create: Uint32,
    // The profit rate for who confirmed proposal
    proposal_confirm: Uint32,
    // The profit rate for consolidating IncomeCells
    income_consolidate: Uint32,
    // The profit rate for inviter in account sale.
    sale_inviter: Uint32,
    // The profit rate for channel in account sale.
    sale_channel: Uint32,
    // The profit rate for DAS in account sale.
    sale_das: Uint32,

}
```

- inviter ，账户注册流程中邀请人的利润率；
- channel ，账户注册流程中渠道的利润率；
- das ，账户注册流程中 DAS 官方的利润率；
- proposal_create ，账户注册流程中 keeper 创建提案的利润率；
- proposal_confirm ，账户注册流程中 keeper 确认提案的利润率；
- income_consolidate ，IncomeCell 合并流程中 keeper 的利润率；

#### ConfigCellSubAccount

**witness：**

```
table ConfigCellSubAccount {
    // The basic capacity SubAccountCell required, it is bigger than or equal to SubAccountCell occupied capacity.
    basic_capacity: Uint64,
    // The fees prepared for various transactions.
    prepared_fee_capacity: Uint64,
    // The price to register a new sub-account.
    new_sub_account_price: Uint64,
    // The price to register a renew sub-account.
    renew_sub_account_price: Uint64,
    // The common fee for every transactions SubAccountCell involved.
    common_fee: Uint64,
    // The fee for create_sub_account action.
    create_fee: Uint64,
    // The fee for edit_sub_account action.
    edit_fee: Uint64,
    // The fee for renew_sub_account action.
    renew_fee: Uint64,
    // The fee for recycle_sub_account action.
    recycle_fee: Uint64,
}
```

- inviter ，账户注册流程中邀请人的利润率；

#### ConfigCellRelease

**witness：**

```
table ConfigCellRelease {
    // Release datetime for accounts of different length.
    release_rules: ReleaseRules,
}

vector ReleaseRules <ReleaseRule>;

table ReleaseRule {
    length: Uint32,
    release_start: Timestamp,
    release_end: Timestamp,
}
```

- length ，账户名长度，0 表示除了列举长度以外的所有长度；
- release_start ，释放开始时间，单位 秒；
- release_end ，释放结束时间，单位 秒；

#### ConfigCellSecondaryMarket

```
table ConfigCellSecondaryMarket {
    // The common fee for every transactions AccountSaleCell and AccountAuctionCell involved.
    common_fee: Uint64,
    // SaleCell =======================================
    // The minimum price for selling an account.
    sale_min_price: Uint64,
    // Expiration time limit for selling accounts.
    sale_expiration_limit: Uint32,
    // Bytes size limitation of the description for account sale.
    sale_description_bytes_limit: Uint32,
    // The basic capacity AccountSaleCell required, it is bigger than or equal to AccountSaleCell occupied capacity.
    sale_cell_basic_capacity: Uint64,
    // The fees prepared for various transactions.
    sale_cell_prepared_fee_capacity: Uint64,
    // AuctionCell ====================================
    // The maximum extendable duration time for an auction, unit in seconds.
    auction_max_extendable_duration: Uint32,
    // The increment of duration brought by each bid in the auction, unit in seconds.
    auction_duration_increment_each_bid: Uint32,
    // The minimum opening price for an auction.
    auction_min_opening_price: Uint64,
    // The minimum bid increment rate of each bid.
    auction_min_increment_rate_each_bid: Uint32,
    // Bytes size limitation of the description for an auction.
    auction_description_bytes_limit: Uint32,
    // The basic capacity AccountAuctionCell required, it is bigger than or equal to AccountAuctionCell occupied capacity.
    auction_cell_basic_capacity: Uint64,
    // The fees prepared for various transactions.
    auction_cell_prepared_fee_capacity: Uint64,
    // The minimum price for making an offer.
    offer_min_price: Uint64,
    // The basic capacity OfferCell required, it is bigger than or equal to OfferCell occupied capacity.
    offer_cell_basic_capacity: Uint64,
    // The fees prepared for various transactions.
    offer_cell_prepared_fee_capacity: Uint64,
    // Bytes size limitation of the message for offer.
    offer_message_bytes_limit: Uint32,
}
```

- common_fee ，涉及消费 AccountSaleCell 和 AccountAuctionCell 的交易中，可从这两个 Cell 拿取的手续费；
- sale_min_price ，一口价出售账户时的最低售价；
- sale_expiration_limit ，一口价挂单的到期时间限制；
- sale_description_bytes_limit ，一口价挂单时的描述信息字节限制；
- sale_cell_basic_capacity ，AccountSaleCell 的基础存储费；
- sale_cell_prepared_fee_capacity ，AccountSaleCell 中应携带的手续费；
- auction_max_duration ，竞拍中**等待出价时间**可达到的最大值；
- auction_duration_increment ，每次出价可以为**等待出价时间**带来的增量；
- auction_min_opening_price ，竞拍的起拍价最小值；
- auction_min_increment_rate_each_bid ，每次出价的最小加价率；
- auction_description_bytes_limit ，竞拍挂单时的描述信息字节限制；
- auction_cell_basic_capacity ，AccountAuctionCell 的基础存储费；
- auction_cell_prepared_fee_capacity ，AccountAuctionCell 中应携带的手续费；

#### ConfigCellReverseResolution

```
table ConfigCellReverseResolution {
    // The common fee for every transactions ReverseCell involved.
    common_fee: Uint64,
    // The basic capacity ReverseCell required, it is bigger than or equal to ReverseCell occupied capacity.
    basic_capacity: Uint64,
}
```

- common_fee ，涉及消费 ReverseCell 的交易中，可从它自身拿取的手续费；
- basic_capacity ，ReverseCell 的基础存储费；

#### ConfigCellRecordKeyNamespace

解析记录 key 命名空间。

**witness：**

```
length|key|key|key ...
```

这个 cell 的 witness 在其 entity 部分**存储的是纯二进制数据**，未进行 molecule 编码。其中前 4 bytes 是 uint32 的数据总长度，**包括这 4 bytes 自身**，之后就是解析记录中可用的各个 key 值字符串的 ASCII
编码，比如原本的 key 是 `address.eth` 那么存储的就是 `0x616464726573732E657468` ，每个 key 之间以 `0x00` 分割。

#### ConfigCellPreservedAccountXX

保留账户名过滤器，目前有 20 个，将账户名 hash 后，根据第一个字节 的 u8 整形对 8 取模的结果进行分配。

**witness：**

```
length|hash|hash|hash ...
```

这个 cell 的 witness 在其 entity 部分**存储的是纯二进制数据**，未进行 molecule 编码。其中前 4 bytes 是 uint32 的数据总长度，**包括这 4 bytes 自身**，之后就是各个账户名不含后缀的部分 hash 后前 20
bytes 拼接而成的数据，因为每段数据固定为 20 bytes 所以**无分隔符等字节**。

### KeylistConfigCell

当用户选择“增强安全"后，创建这个 cell 用来存储用户的多设备的 `WebAuthn` 授权信息， 其中包括用户的 `Credential ID` 和 `Public key` 。

当用户添加更多的设备进来，会在 witness 里添加设备的 `Credential ID` 和 `Public Key` ；

#### 结构：

```
lock: <das-lock>
type: <key-list-config-cell-type>
data:
  hash(witness: WebAuthnKeyList)

witness:
  table Data {
    old: table DataEntityOpt {
        index: Uint32,
        version: Uint32,
        entity: WebAuthnKeyList
    },
    new: table DataEntityOpt {
      index: Uint32,
      version: Uint32,
      entity: WebAuthnKeyList
    },
  }
  
======
vector WebAuthnKeyList <WebAuthnKey>;

struct WebAuthnKey {
  
    main_alg_id : Uint8,  //main algorithm id
    sub_alg_id : Uini8, //sub algorithm id
    cid: Byte10, //credential id sha256
    pubkey: Byte10,
}
```

WebAuthnKey 中的主要字段如下：

* main_alg_id：主算法 ID，08标识使用WebAuthn；
* sub_alg_id：子算法 ID，表明使用 WebAuthn 的哪个算法进行公钥的生成以及验证；
* cid：WebAuthn 生成的 credential ID 进行 sha256 5次后，取前10字节；
* pubKey: WebAuthn 生成的 public key 进行 sha256 5次后，取前10字节；

```c
enum sub_alg_id {
    Secp256r1,
    ...
};
```
体积：



### TimeCell、HeightCell、QuoteCell

这是 folk 自 Nervina 团队开发的 [ckb-time-scripts](https://github.com/nervina-labs/ckb-time-scripts) 合约脚本，它定义了一系列实现类似预言机功能的 Cell。

由于现在 Nervos 生态中尚无预言机的服务提供方，因此我们采用了该方案，并增加了一个 QuoteCell 类别。

#### TimeCell

```
lock: <ckb_lock_script>
type:
  code_hash: <ckb-time-scripts>
  type: type
  args: "0x01"
data:
    [index] // 1 字节大端编码的 u8 整形，存放当前是 TimeCell 中的第几个
    [type] // 1 字节的类型，用于标识当前 Cell 是 HeightCell
    [timestamp] // 4 字节大端编码的 u32 整形，存放当前的 UTC 时间戳
```

> TimeCell 因为 TimeCell 的时间戳实际上还是基于链上的时间戳产生，所以与现实时间存在 5 分钟左右的误差。

#### HeightCell

```
lock: <ckb_lock_script>
type:
  code_hash: <ckb-time-scripts>
  type: type
  args: "0x02"
data:
    [index] // 1 字节大端编码的 u8 整形，存放当前是 Height 中的第几个
    [type] // 1 字节的类型，用于标识当前 Cell 是 HeightCell
    [block_height] // 8 字节大端编码的 u64 整形，存放当前的区块高度
```

#### QuoteCell

由于 CKB 链上目前暂无和美元挂钩的稳定币，DAS 官方会通过一个报价 Cell 公布当前采用的 CKB 对美元的汇率。单位为 CKB/USD ，因为 USD 采用了扩大 `1_000_000` 倍的方式精确到小数点后 6 位，所以 CKB 也要等比扩大，既市价为 0.001
CKB/USD 时，QuoteCell 需要记录为 1000 CKB/USD 。

```
lock: <ckb_lock_script>
type:
  code_hash: <ckb-time-scripts>
  type: type
  args: "0x00"
data:
    [index] // 1 字节大端编码的 u8 整形，存放当前是 Height 中的第几个
    [type] // 1 字节的类型，用于标识当前 Cell 是 HeightCell
    [block_height] // 8 字节大端编码的 u64 整形，存放当前的区块高度
```

#### 体积

`105` Bytes
