# Witness 数据结构


## Mint 方式与 flag 标识符

子账户的 Mint 方式主要有 3 种：

- 由 Owner 或 Manager 签名的 **Sign Mint**；
- 由用户支付一定 USD 自助 Mint ，这种 Mint 方式又根据定价来源细分为以下两种：
  - 根据自定义脚本计算定价的 **Custom Script Mint** ；
  - 根据自定义规则计算定价的 **Custom Rule Mint** ；

这些不同的 Mint 方式和 SubAccountCell.data.flag 状态标识符有以下对应关系：

|    状态名    | 状态值 |       可选的 Mint 方式        |
|:------------:|:------:|:-----------------------------:|
|    Manual    |  0x00  |           Sign Mint           |
| CustomScript |  0x01  | Sign Mint, Custom Script Mint |
|  CustomRule  |  0xff  |  Sign Mint, Custom Rule Mint  |

> 当 **Sign Mint** 和其他 Mint 方式混合在一笔交易中时，会优先根据 `SubAccountMintSign` 去匹配新账户：
> - 如果匹配成功那么注册费就按照 `ConfigCellSubAccount.new_sub_account_price` 中给出的最低值计算；
> - 如果匹配失败，那么就继续根据 **Custom Script Mint** 或 **Custom Rule Mint** 的逻辑去计算注册费；


## witness 存储结构

当交易中涉及子账户的新增、修改、删除操作时，每个子账户需要有一条对应自己的 witness 记录，其基本结构和 DAS 的其他 witness 结构相同：

```
[
  lock 脚本需要的签名,
  lock 脚本需要的签名,
  lock 脚本需要的签名,
  ...
  [das, type, raw/entity/table],
  [das, type, raw/entity/table],
  [das, type, sub_account_mint_sign],
  [das, type, sub_account_price_rule],
  [das, type, sub_account_price_rule],
  ...
  [das, type, sub_account_preserved_rule],
  [das, type, sub_account_preserved_rule],
  ...
  [das, type, sub_account],
  ...
]
```

其中 [3:7] 4 个 bytes 为小端编码的 u32 整型，它标明了第 8 bytes 之后的数据类型是子账户类型，具体值详见 [Cell 结构协议.md/Type 常量列表/SubAccount](../%E7%B3%BB%E7%BB%9F%E6%9E%9A%E4%B8%BE%E5%80%BC.md)；

最后一段数据 `sub_account` 分为了三类：

- 为了批量 Mint 子账户而设计的 `SubAccountMintSign` ；
- 为了定义子账户定价而设计的 `SubAccountPriceRule` ；
- 为了定义子账户保留名单而设计的 `SubAccountPreservedRule` ；
- 为了创建、编辑子账户 SMT 而设计的 `SubAccount` ；

由于数据量较大，且在之前的实践中我们发现 molecule 编码在处理较长数据时在合约中性能不佳的问题存在，所以部分类型采用了以下**基于 LV 编码(Length-Value)的二进制**：

```
[ length ][ field_1 ][ length ][ field_2 ] ...

[ length ][ field_1 ]
[ length ][ field_2 ]
...
```

以上描述方法中，无论所有字段在同一行还是不同行，**在实际二进制数据中都是没有分割符/换行符的连续数据，这里只是为了可读性才进行了换行**。 `[ length ]` 固定为 4 Bytes 的小端编码的 u32 整型，其值为后面一段数据的长度。比如 `[ field_1 ]` 段数据长度为 65 Bytes，那么到 `[ field_1 ]` 为止的二进制数据就是以下形式(**过长的数据部分以 `...` 表示省略**)：

```
0x00000041FFFFFF...

上面的数据可以视为两个部分：
0x00000041 0xFFFFFF...

0x00000041 就是 length 部分
0xFFFFFF... 就是 field_1 部分
```


当某一个段数据的值为空时，其 `length` 需要为 `0x00000000`。比如 `field_2` 段数据为空时，那么这段二进制数据就是以下形式：
   ```
0x...FFF00000000

上面的数据可以视为 2 个部分
0x...FFF 0x00000000

0x...FFF 就是 field_2 的 length 之前的数据
0x00000000 就是 field_2 的 length ，其指明了 field_2 的值为空
```

### SubAccountMintSign 数据结构

当大批量地创建子账户时，因为交易体积的限制，一笔交易中无法携带过多的子账户信息，这时候就需要将交易拆分为多笔交易才能执行。因此，为了
保证交易拆分后仍然只需要用户进行一次签名，所以有了这种转为创建子账户而设计的签名数据结构。

```
[ length ][ version ]
[ length ][ signature ]
[ length ][ sign_role ]
[ length ][ sign_expired_at ]
[ length ][ account_list_smt_root ]
```

- `version` 即当前数据结构版本号，类型为小端编码的 u32 整形，后续的字段有任何改变时，此字段就会 `+1`；
- `signature` 也就是对当前 witness 部分数据的签名；
- `sign_role` 指明 `signature` 签名所属角色，`0x00` 表示以 owner 的身份验签，`0x01` 表示以 manager 的身份验签；
- `sign_expired_at` 指明 `signature` 签名过期期限，类型为小端编码的 u64 整形，由于拆分的多笔交易需要一段时间才能完成上链，因此此签名可以在一段时间内重复使用，**这个过期时间必须小于等于父账户 `expired_at` 和所有新创建的子账户的 `expired_at` 之间的最小值**；
- `account_list_smt_root` 为一颗 [SMT](https://github.com/nervosnetwork/sparse-merkle-tree) 的 root，其中的 key 为子账户的
account hash ，value 为子账户创建成功后的 `SubAccountData.lock.args` 字段中 owner 部分的地址；

#### sign_expired_at 安全性

由于该签名可以被重复使用，因此这里特别解释一下其防重放等安全性：

- 子账户创建出来以后过期时间至少为 1 年，即一年之内再次创建时会发现子账户已存在；
- `sign_expired_at` 只要小于 1 年，那么在新创建的子账户未过期前，其实就无法再次创建了；
- `sign_expired_at` 的对比对象为 SubAccountCell 的 block_header.timestamp ，因此除非 SubAccountCell 一年都未被使用否则时间的有效性毋庸置疑；

#### 签名与验签

`signature` 最终会使用和 `das-lock` 进行验签，因此签名的生成和验签就是 CKB、ETH、BTC 链的标准协议。唯一不同的是 `digest` 的生成，其组成为按顺序拼接以下字段：

- `from did: ` 字符串的二进制字节；
- 一个以 `ckb-default-hash` 为参数的 32 字节 blake2b hash ，创建方法是按顺序拼接以下字段后进行 hash：
  - `sign_expired_at`
  - `account_list_smt_root`

### SubAccountPriceRule 与 SubAccountPreservedRule 数据结构

这两类数据是一个有较为复杂层级关系的结构，因此主要采用了 molecule 编码来存放，只保留了一个 version 字段用来标识 molecule 结构体的版本号：

```
[ length ][ version ]
[ length ][ SubAccountRules ]
```

- `version` 即当前数据结构版本号，类型为小端编码的 u32 整形，后续的字段有任何改变时，此字段就会 `+1`；
- `SubAccountRules` SubAccountPriceRule 和 SubAccountPreservedRule 共用的 `SubAccountRules` 类型，区别在于 SubAccountPreservedRule 数据结构种的 `SubAccountRule.price` 会被忽略，关于这两种类型的定义及其对应的 JSON 描述详见 [自定义规则](./%E8%87%AA%E5%AE%9A%E4%B9%89%E8%A7%84%E5%88%99.md) 。

### SubAccount 数据结构

对子账户的所有增删改操作，最终都可以归纳为对 `SubAccountCell.data.smt_root` 的修改。因此这种 witness 的数据结构，每一条就可以理解为一条 `SubAccountCell.data.smt_root` 的修改记录。

当第一个字段的 `length != 4` 时，需要按照 `1` 版本的数据结构进行处理：

```
[ length ][ signature ]
[ length ][ sign_role ]
[ length ][ prev_root ]
[ length ][ current_root ]
[ length ][ proof ]
[ length ][ version ]
[ length ][ sub_account ]
[ length ][ edit_key ]
[ length ][ edit_value ]
```

当第一个字段的 `length == 4` 时，其后数据结构的版本就按照 `version` 所指明的数字进行处理，当前版本号为 `2`：

```
[ length ][ version ]
[ length ][ action ]
[ length ][ signature ]
[ length ][ sign_role ]
[ length ][ sign_expired_at ]
[ length ][ new_root ]
[ length ][ proof ]
[ length ][ sub_account ]
[ length ][ edit_key ]
[ length ][ edit_value ]
```

- `version` 指明之后字段的数据结构版本号，类型为小端编码的 u32 整形，后续的字段有任何改变时，此字段就会 `+1`；
- `action` 当前 witness 的意图，由于 SMT 结构在上链时必然存在严重的 Cell 抢占问题，因此在同一笔交易中支持了所有的子账户操作类型；
- `signature` 为子账户所有者对子账户进行编辑时的签名字段，其包含了对 `account_id, edit_key, edit_value, nonce` 信息的签名；
- `sign_role` 指明 `signature` 签名所属角色，`0x00` 表示以 owner 的身份验签，`0x01` 表示以 manager 的身份验签；
- `sign_expired_at` 指明 `signature` 签名的到期时间，**这个过期时间必须小于等于父账户 `expired_at` 和子账户的 `expired_at` 之间的最小值**；
- `new_root` 当前 witness 对 `SubAccountCell.data.smt_root` 修改后的新的 SMT root ；
- `proof` 即 SMT 的 proof，用于证明当前的 `SubAccountCell.data.smt_root` 和修改后的 `SubAccountCell.data.smt_root` 都是正确的；

剩余的 `action, sub_account, edit_key, edit_value` 等字段的详细信息详见后续章节的介绍。

#### action、edit_key 和 edit_value 字段数据结构

`action` 的值就是 utf-8 编码的字符串，根据当前 witness 的意图不同可以为 `create`, `edit`, `renew` 等等。

比较重要的一点是，当 `action` 不同时，`edit_key` 和 `edit_value` 的含义也有所不同：

- 如果 `action == create && (edit_key == manual || edit_key is empty)`，则 `edit_value` 必须是 `SubAccountMintSign.account_list_smt_root` 的有效 `proof` ，其要能够证明当前的创建的账户名确实存在于 `SubAccountMintSign.account_list_smt_root` 中；
- 如果 `action == create && edit_key == custom_rule`，`edit_value` 前 20 Bytes 为渠道商的识别 ID，后 8 Bytes 为此账号注册时所支付的金额；
- 如果 `action == edit`，那么 `edit_key` 就是 utf-8 编码的字符串，用于指明需要修改的字段，`edit_value` 就是具体修改后的值，根据字段的不同有以下类型：
  - `edit_key` 为 `expired_at`，那么 `edit_value` 必须为一个 molecule 编码的 `Uint64` 类型数据；
  - `edit_key` 为 `owner`，那么 `edit_value` 必须为一个合法的 das-lock 的 args 数据，并且出于安全考虑，新状态的子账户的 records 字段会被视为已清空；
  - `edit_key` 为 `manager`，那么 `edit_value` 必须为一个合法的 das-lock 的 args 数据；
  - `edit_key` 为 `records`，那么 `edit_value` 必须为一个 molecule 编码的 `Records` 类型数据；

> 为了支持第三方渠道通过自定义规则分发子账户，因此每个通过第三方渠道注册的子账户的 witness 中需要带上渠道商的识别 ID 和注册金额，后续在 dotbit 以此为依据和第三方渠道进行利润分配。

#### sub_account 字段数据结构

在整个子账户的 witness 中，`sub_account` 则是一个子账户的 molecule 编码的数据结构(**最新结构请以 [das-types](https://github.com/dotbitHQ/das-types) 中定义为准**)：

```
table SubAccountData {
    // The lock of owner and manager
    lock: Script,
    // The first 160 bits of the hash of account.
    id: AccountId,
    // Separate chars of account.
    account: AccountChars,
    // The suffix of this sub-account, it is always .bit currently.
    suffix: Bytes,
    // The sub-account register timestamp.
    registered_at: Uint64,
    // The sub-account expiration timestamp.
    expired_at: Uint64,
    // The status of the account, 0x00 means normal, 0x01 means being sold, 0x02 means being auctioned.
    status: Uint8,
    // Resolving records of this sub-account.
    records: Records,
    // This is a count field, it mainly used to prevent replay attacks.
    nonce: Uint64,
    // If sub-account of sub-account is enabled.
    enable_sub_account: Uint8,
    // The price of renew sub-account of this sub-account.
    renew_sub_account_price: Uint64,
}
```

> 目前 `lock` 字段仅支持 das-lock ，既其中的 `code_hash`, `hash_type` 字段必须和用于其他 Cell 上的 das-lock 完全一致。
>
> `nonce` 字段在每次发起需要子账户签名的交易时都需要自增 1 ，如此就可以防止重放攻击。 由于 witness.sub_account.nonce 的值总是**当前的 nonce 值**，
> 如果需要对子账户交易进行签名，那么使用**当前的 nonce 值**即可，如果需要计算交易上链后新的子账户信息，那么需要在**当前的 nonce 值上 +1** 。

#### 签名与验签

`signature` 最终会使用和 `das-lock` 进行验签，因此签名的生成和验签就是 CKB、ETH、BTC 链的标准协议。唯一不同的是 `digest` 的生成，其组成为按顺序拼接以下字段：

- `from did: ` 字符串的二进制字节；
- 一个以 `ckb-default-hash` 为参数的 32 字节 blake2b hash ，创建方法是按顺序拼接以下字段后进行 hash：
  - `account_id`
  - `edit_key`
  - `edit_value`
  - `nonce`
  - `sign_expired_at`
