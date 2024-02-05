# 基础数据结构


主账户、子账户都使用了相同的基本结构存储授权相关的信息，且字段名都为 approval：


```
# molecule
table AccountApproval {
    action: Bytes,
    params: Bytes,
}
```

- `action` 就是授权的类型，为 utf8 编码的字符串的 bytes ；
- `params` 就是下述的各种授权的详细 molecule 结构，为了支持可变类型这里使用了 `Bytes` 类型作为容器；


## 授权的类型

授权包含了以下类型：

- `Transfer` ，可以授权将账户所有权的转移给其他用户，详见 [Transfer Approval](transfer-approval.md)；


## 验签

这里对于验签的 digest 生成方式进行说明。
* 主账户的授权交易，基本上和普通交易一样构造 digest 即可；
* 子账户由于数据结构的特殊性，采用以下算法生成 digest ；

按序拼接以下数据：
- from did:  字符串的 utf8 bytes；
- 一个按照 ckb-hash 生成的 hash ，创建方法是按顺序拼接以下字段后进行 hash：
  - action 字段，对于主账户来说是交易的 action ，对于子账户来说是子账户 witness 自身的 action 字段；
  - approval 字段的 molecule 编码 bytes ;
  - 一个防止重放的字段：
    - 子账户依序拼接 sign_expired_at 和 nonce 字段；
