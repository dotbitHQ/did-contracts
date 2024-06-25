# Cell structure protocol

## Protocol symbol convention

The following contents of this document will use a unified structure to describe a cell:

```
lock: ...
type: ...
data: ...
witness: ...
```

Among them, `lock`, `type`, `outputs_data` are information that each cell must contain, which can also be seen from the data structure returned by the RPC interface, and `data` is the `outputs_data' corresponding to the cell in this transaction. `. `witness` is special, it is the same as cell
There is no correlation between them, so the `witness` here specifically refers to the DAS witness, and only refers to the `entity` part of the DAS witness, because the DAS witness stores which cell it corresponds to.
Relevant information, so there is an association. For details, see Witness-Structure.

**All field names in data mean a piece of data parsed according to a specific offset**. Because the volume of data will affect the number of CKB that needs to be pledged, in addition to splitting the data according to the offset given in the document The outer itself does not have any data structures. **All field names in witness mean a molecule-encoded data structure**
, first you need to use the structure/class of the corresponding structure to parse the data, and then you can access the corresponding fields.

You may see the following symbols when describing the cell structure:

| Symbol                 | Description                                                                                                                                                                                                                                                 |
| ------------------- |-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| lock: <...>         | represents a specific script, and its code_hash, args, hash_type have simple conventions, so the details will not be listed                                                                                                                                 |
| type: <...>         | Same as above                                                                                                                                                                                                                                               |
| hash(...)           | refers to the hash value calculated by the data stored here                                                                                                                                                                                                 |
| ======              | In the code segment class describing the cell structure, this delimiter means that the following content is a detailed introduction to the specific molecule structure, but the latest schema please refer to the schemas/ directory in the das-types repo. |
| ConfigCellXXXX.yyyy | Refers to data that needs to be obtained from a specific field in the witness of a ConfigCell. For details, see [ConfigCell](#ConfigCell)                                                                                                                                                                                       |

## Data structure

### ApplyRegisterCell

To apply for a registered account, the user must first create this Cell to apply, and then wait for `ConfigCellApply.apply_min_waiting_block_number` before using this Cell.
To register. The purpose of this design is to prevent the transactions of users' registered accounts from being maliciously intercepted and registered during the process of uploading to the chain.

#### Structure

```
lock: <ckb_lock_script>
type: <apply-register-cell-type>
data:
  hash(lock_args + account) // account contains .bit suffix
  [height] // Deprecated
  [timestamp] // Deprecated
```

- hash, account name and owner's lock_args hash;
- height, the block height when the cell was created (little endian), obtained from heightcell;
- timestamp, the timestamp when the cell was created (little endian), obtained from timecell;

#### Volume

actual volume：142 Bytes

### PreAccountCell

When [ApplyRegisterCell](#ApplyRegisterCell) exists on the chain for more than `ConfigCellApply.apply_min_waiting_time`, the user can convert it into a PreAccountCell and wait for the Keeper
This is ultimately converted to [AccountCell](#AccountCell) by creating a [ProposalCell](#ProposalCell) proposal.

#### Structure

```
lock: <always_success>
type: <pre-account-cell-type>
data:
  hash(witness: PreAccountCellData)
  id // account ID, the generation algorithm is hash(account), and then take the first 20 Bytes. This one contains the .bit suffix

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
    account: AccountChars, // no .bit
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

- account, the account name actually registered by the user, does not contain the `.bit` suffix;
- refund_lock, if PreAccountCell ultimately fails to pass the proposal, the refund lock script, that is, the address;
- owner_lock_args, if PreAccountCell finally passes the proposal, the value of [AccountCell.lock.args](#AccountCell) is the args of das-lock;
- inviter_id: The main purpose is to facilitate the server to display the name of the inviter;
- inviter_lock, the inviter's lock script, the profit distribution will be transferred to IncomeCell and accounted for with this lock script;
- channel_lock, the lock script of the channel provider, the profit distribution will be transferred to IncomeCell and accounted for with this lock script;
- price, the selling price when registering the account;
- quote, the USD unit price of CKB at the time of account registration;
- created_at, the time of TimeCell when PreAccountCell was created;
- initial_records, the initial parsing record when the AccountCell is created successfully;
- initial_cross_chain, whether the AccountCell is directly locked into the cross-chain state when it is successfully created;


#### Calculation logic of profit and duration of registration

When creating a PreAccountCell, the user needs to pay the registration fee and the basic fees required to create various Cells. At this time, the calculation formula for the amount of CKB the user should pay is:
```
// This section is pseudocode, and there is a context for execution from top to bottom.
Storage fee = (Basic volume of AccountCell + Account length + 4) * 100_000_000 + Pre-deposit fee

Profit = PreAccountCell.capacity - storage fee

if USD annual fee < CKB exchange rate {
   CKB annual fee = USD annual fee * 100_000_000 / CKB exchange rate
} else {
   CKB annual fee = USD annual fee / CKB exchange rate * 100_000_000
}

CKB annual fee = CKB annual fee - (CKB annual fee * discount rate / 10000) // The discount rate is a percentage based on 10000

Registration length = profit * 365 / CKB annual fee * 86400
```

- The year needs to be greater than or equal to 1, and the actual calculation is based on 365 \* 86400 seconds as one year;
- **Annual account registration fee** is stored in [ConfigCellPrice.prices](#ConfigCell) in **USD**;
- **CKB exchange rate** is obtained from QuoteCell, and the unit is **USD/CKB**. We agreed in the **USD unit** section of [Data Storage Plan.md] (Data Storage Plan.md) 1 dollar is recorded as `1_000_000`, so if the record in QuoteCell is `1_000`
  Then it means that the CKB exchange rate is `0.001 USD/CKB`;
- **AccountCell basic cost** will only change when the Cell data structure is adjusted. It can be considered a fixed constant and can be obtained by checking the **volume** of the corresponding Cell;
- **Account byte length**, since AccountCell will save the complete account in the data field, for example, `das.bit` will save `0x6461732E626974`, so this part of the volume needs to be added;
- Anything with division is automatically rounded
- 
#### Volume

Base volume：126 Bytes

Actual volume: Depends on the length of the registered account, the year of registration, the CKB unit price at the time of registration, whether to register, etc.

### ProposalCell

After the user creates [PreAccountCell](#PreAccountCell), Keeper needs to collect them to initiate a proposal, that is, create a ProposalCell
, the proposal can only be passed after waiting for a certain period of time. The so-called passing means creating a transaction consumption proposal and converting the [PreAccountCell](#PreAccountCell) applied to it into the final [AccountCell](#AccountCell). This process will ensure the uniqueness of the account name on the chain.

#### Structure

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

====== For example it looks like this
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

- proposer_lock, the lock script of the proposal initiator; if the proposal is recycled, the recycled CKB should be transferred to this lock script; if the proposal is passed, the profits belonging to the proposal initiator should be transferred to IncomeCell and accounted for with this lock script ;
- created_at_height, the current height obtained from the time cell when the proposal was initiated;
- slices, the final state of the modified part of the AccountCell linked list after the current proposal is passed. See `TODO` for its explanation;
- item_type meaning description
    - exist, the value corresponds to 0x00, indicating that when this proposal was initiated, the account pointed to by account_id was already registered, and the AccountCell can be found on the chain;
    - proposed, the value corresponds to 0x01, indicating that when this proposal is initiated, the account pointed to by account_id is already in pre-registration status, and PreAccountCell can be found on the chain. When the pre-proposal of this proposal is passed, it will be converted into AccountCell;
    - new, the value corresponds to 0x02, indicating that when this proposal is initiated, the account pointed to by account_id is already in pre-registration status, and PreAccountCell can be found on the chain. When this proposal is passed, it will be converted into AccountCell;
#### Volume

Base volume：106 Bytes

Actual volume: 106 Bytes

### AccountCell

When the proposal is confirmed, that is, when [ProposalCell](#ProposalCell) is consumed, [PreAccountCell](#PreAccountCell) can be converted into AccountCell, which stores various account information.

#### Structure

```
lock:
   code_hash: <das-lock>
   type: type
   args: [ // This is the args structure of das-lock, which also contains owner and manager information.
     owner_algorithm_id,
     owner_pubkey_hash,
     manager_algorithm_id,
     manager_pubkey_hash,
   ]
type: <account-cell-type>

data:
   hash(witness: AccountCellData) // 32 bytes
   id // 20 bytes, your own ID, the generation algorithm is hash(account), and then take the first 20 Bytes
   next // 20 bytes, the ID of the next AccountCell
   expired_at // 8 bytes, little-endian encoded u64 timestamp
   account // All bytes after expired_at, utf-8 encoding, AccountCell. In order to avoid data loss and prevent users from being able to retrieve their own users, AccountCell additionally stores the clear text information of the account, including the .bit suffix.
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
    last_transfer_account_at: Uint64,
    last_edit_manager_at: Uint64,
    last_edit_records_at: Uint64,
    // The status of the account, 0x00 means normal, 0x01 means being sold, 0x02 means being auctioned.
    status: Uint8,
    records: Records,
    // The status of sub-account function, 0x00 means disabled, 0x01 means enabled.
    enable_sub_account: Uint8,
    // The price of renewing sub-account for one year.
    renew_sub_account_price: Uint64,
    // The approval that can be fulfilled in the future.
    approval: AccountApproval,
}
```

- id, account ID, after calculating the hash of the account name (**including suffix**), the first 20 bytes are the account ID, which is unique in the entire network;
- account, account name field;
- registered_at, registration time;
- status , status field:
    - 0, normal;
    - 1, for sale;
    - 2, in auction;
    - 3, expiring in auction;
- records, parsing record fields, **This field can only be edited by users with administrative rights**;
- enable_sub_account, status field:
    - 0, sub-account is not enabled;
    - 1, sub-account is enabled;
- approval, the authorization information of the account, see [Authorization Structure Related Documents](approval/transfer-approval.md) for details;
- 
das-lock is a special lock script designed for DAS. It will dynamically load different signature verification logic executions based on the xx_algorithm_id part in args. **xx_algorithm_id in args are all 1 byte, and pubkey_hash is taken first
20 bytes**.

Transactions involving signature verification need to indicate whether the authority used by the current transaction is owner or manager in ActionData.params in witnesses. **owner uses 0 and manager uses 1**.

#### Volume

Actual volume: `201 + n` Bytes, `n` depends on the length of account.

On-chain volume: depends on the configuration items in ConfigCellAccount

### IncomeCell

A Cell used to solve the problem of insufficient 61 CKB in a single account when batch arrival. This Cell and its related solutions mainly have the following advantages:

1. In the scenario of batch transfer, it can solve the problem of being unable to create an ordinary Cell due to insufficient single account balance of 61 CKB;
2. Solved the problem of being unable to create IncomeCell when the total ledger account is insufficient for 61 CKB;
3. By reusing IncomeCell, the above advantages 1 and 2 are achieved while reducing the probability of multiple transactions seizing the same IncomeCell;
4. 
#### Structure

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

The main fields in Witness are as follows:

- creator, records the creator of this IncomeCell. Anyone can freely create an IncomeCell. When there is only one record of the creator in the records, this IncomeCell can only be used to confirm proposal transactions;
- records, account records, record which lock scripts the capacity of IncomeCell belongs to, and how many CKB each lock script has;
- 
#### Volume

Actual volume: 106 Bytes

On-chain volume: depends on the configuration items in ConfigCellIncome

### AccountSaleCell

This is a Cell used to describe account sales information. Each account on sale has a corresponding AccountSaleCell.

#### Structure

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

The main fields in Witness are as follows:

- account_id, the associated account ID;
- account, account name;
- price, account selling price;
- description, user-defined profile information;
- started_at, the timestamp when the account started selling;

#### Volume

Actual volume: `148 ~ 170` Bytes, depending on the length of args of das-lock.

On-chain volume: depends on the configuration items in ConfigCellSecondaryMarket

### AccountAuctionCell

This is a Cell that describes account bidding information. Each account in the auction has a corresponding AccountAuctionCell.

#### Structure

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

- account_id, the account ID of the auction account;
- account, the account name of the auction account;
- description, description information of the auction;
- opening_price, starting price;
- increment_rate_each_bid, the price increase ratio for each bid;
- started_at, the starting time of the auction;
- ended_at, the end time of the auction;
- current_bidder_lock, the lock script of the current bidder;
- current_bid_price, the current bid;
- prev_bidder_profit_rate, the profit available to the previous auctioneer after each round of bidding;
- 
#### Volume

Actual volume: `148 ~ 170` Bytes, depending on the length of args of das-lock.

On-chain volume: depends on the configuration items in ConfigCellSecondaryMarket

### BalanceCell

This is the Cell used to act as the user's ckb balance. The data field has no data and no associated witness.

#### Structure

```
lock: <das-lock>
type: <balance-cell-type>
data: 0x
```


#### Volume

Actual volume: 116 Bytes.

### ~~ReverseRecordCell~~

> **Deprecated**! This Cell has been deprecated and this document is only used for transaction parsing.

There may be multiple Cells that store reverse resolution records at the same address, and they need to be deduplicated according to [Protocol](../design/Reverse Resolution Mechanism.md).

#### Structure
```
lock: <ckb_lock_script> | <das_lock_script>
type: <reverse-record-cell-type>
data:
  account // Reverse analysis of the corresponding account name
```
#### Volume

`116 + n` Bytes, `n` depends on the length of lock's args and the length of account.

### ReverseRecordRootCell

Cell that stores the reverse parsing record SMT Root.

#### Structure
```
lock: <always_success>
type: <reverse-record-root-cell-type>
data:
  smt_root
```
- smt_root, reversely parses the Root of SMT;

#### Volume

`116` Bytes

### OfferCell

Quote Cell. Users can use this Cell to give quotes for any account name, even account names that have not yet been registered.
#### Structure

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

#### Volume

`148 ~ 170` Bytes, depending on the length of das-lock’s args.
### SubAccountCell

#### Structure

```
lock: <always_success>
type:
  code_hash: <sub-account-cell-type>,
  type: type,
  args: [account_id], // Account ID, which is the same value as AccountCell.data.id

data: [ smt_root ][ das_profit ][ owner_profit ][ flag ]
// OR
data: [ smt_root ][ das_profit ][ owner_profit ][ flag ][ custom_script ][ script_args ]
// OR
data: [ smt_root ][ das_profit ][ owner_profit ][ flag ][ status_flag ][ price_rules_hash ][ preserved_rules_hash ]
```

- smt_root, 32 bytes, which is the merkle root corresponding to all sub-accounts under the main account;
- das_profit, 8 bytes, since SubAccountCell is also responsible for storing the profits belonging to the DAS official, this value indicates how much DAS official profits are in the capacity;
- owner_profit, 8 bytes, since SubAccountCell is also responsible for storing the profit of the owner belonging to the parent account AccountCell, this value indicates how many owners' profits there are in the capacity;
- flag, 1 byte, identifies how to parse the following fields:
    - When `flag == 0x00` or does not exist, it indicates that the user only uses manual distribution, omitting all subsequent fields, and only the parent account can create sub-accounts;
    - When `flag == 0x01`, it indicates that the user has used a custom price based on the dynamic library, omitting the two fields `price_rules_hash` and `preserved_rules_hash`, and `custom_script` is the `type.args` of the dynamic library, at this time` script_args` must be 10 bytes;
    - When `flag == 0xff`, indicates that the user has enabled the configuration-based automatic distribution feature, omitting the `custom_script` and `script_args` fields, the default `status_flag = 0x00` and `price_rules_hash` and `preserved_rules_hash` need to be filled in completely` 0x00`;
- custom_script, 32 bytes, is the `type.args` of the dynamic library;
- script_args, 10 bytes, stores the first 10 bytes of the witness hash used by the custom script;
- status_flag, 1 byte, identifies whether the configuration-based automatic distribution feature is turned on. It is 0x01 when turned on and 0x00 when turned off. The default is on;
- price_rules_hash, a hash of 10 bytes, first calculate the hash of the `rules` part bytes of all witness types of SubAccountPriceRule, and then splice these hashes in the order of generation, calculate the hash again for the spliced bytes and take the first 10 bytes;
- preserved_rules_hash, 10 bytes hash, first calculate the hash of all the `rules` bytes of the witness type SubAccountPreservedRule, and then splice these hashes in the order of generation, calculate the hash again for the spliced bytes and take the first 10 bytes;
- #### custom_script derives dynamic library Type ID

When `flag == 1`, `custom_script` needs to be used as `type.args` to form the following Script structure:

```
{
   code_hash: "0x0000000000000000000000000000000000000000000000000545950455f4944",
   hash_type: flag,
   args: custom_script
}
```

After molecule encoding the above structure, perform blake2b hash operation to obtain the type ID of the custom script.

> This Cell has no associated witness.

#### Volume

- `143` Bytes when `flag == 0`
- `143 + 42` Bytes when `flag == 1`
- `143 + 20` bytes when `flag == 255`

### ExpiredAccountAuctionCell

#### Structure

### DeviceKeylistCell

When the user selects "Enhanced Security", this cell is created to store the user's multi-device `WebAuthn` authorization information, including the user's `Credential ID` and `Public key`.

When the user adds more devices, the `Credential ID` and `Public Key` of the device will be added to the witness;

#### Structure：

```
lock: <das-lock>
type: <device-key-list-cell-type>
data:
  hash(witness: DeviceKeyListCellData)

witness:
  table Data {
    old: table DataEntityOpt {
        index: Uint32,
        version: Uint32,
        entity: DeviceKeyListCellData
    },
    new: table DataEntityOpt {
      index: Uint32,
      version: Uint32,
      entity: DeviceKeyListCellData
    },
  }

======
vector DeviceKeyList <DeviceKey>;

struct DeviceKey {
    main_alg_id : Uint8,  //main algorithm id
    sub_alg_id : Uini8, //sub algorithm id
    cid: Byte10, //credential id in sha256
    pubkey: Byte10,
}

table DeviceKeyListCellData {
    keys: DeviceKeyList, // Device keys
    refund_lock: Script, // On destroy-device-key-list, send the remaining capacity to refund_lock
}

```The main fields in DeviceKey are as follows:

* main_alg_id: main algorithm ID, 08 identifies the use of device management, and currently the main sub-algorithm is provided by WebAuthn;
* sub_alg_id: sub-algorithm ID, identifying which algorithm of WebAuthn is used for public key generation and verification;
* cid: After hashing (sha256) 5 times of WebAuthn's credential ID, take the first 10 bytes;
* pubkey: After hashing (sha256) 5 times of WebAuthn's public key, take the first 10 bytes;

```c
enum sub_alg_id {
     Secp256r1,
     ...
};
```
#### Volume: ToDo

### DPointCell

This is a Cell that describes the current Cell DID Point balance.
DID Point is launched by the .bit team and is anchored to points in US dollars. Users can purchase DID Point with US dollars or ckb, etc., and then use DID Point to purchase and renew .bit domain names.

#### Structure

```
lock: <das-lock>
type: <dpoint-cell-type>
data:
   value: Uint64
```
The data of DPointCell uses the LV (Length/Value) structure to store data, and there is no witness:
- value, u64 type, the total number of DPoints carried in the current Cell;

#### Volume

Actual volume: 128 Bytes.


## ConfigCell

This is a Cell that saves DAS configuration on the chain and is currently only updated manually via the DAS super private key. Because CKB VM has performance problems when loading data, the overhead increases sharply as the data becomes larger, so a saving method of dispersing different configurations into multiple ConfigCells is adopted.

#### Structure

All ConfigCells follow the following basic data. Different ConfigCells are mainly reflected through `type.args`:
```
lock: <super_lock> // An official ckb multi-signature lock
type:
     code_hash: <config-cell-type>,
     type: type,
     args: [DateType], // A uint32 DateType value is saved here. This value is mainly used to facilitate identification and query of ConfigCell.
data:
     hash(witness) // All data of ConfigCell is a Hash calculated from witness
```

> For details of the DataType of each ConfigCell, see [Type constant list](#Type constant list) below.

#### Volume

All ConfigCell have the same cell structure, so the volume is `130` Bytes.
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

The character set Cell can be used to register an account. The design of this Cell is that in addition to the two global character sets of Emoji and Digit, each other language can have its own independent character set. Different languages cannot be mixed. **Only the global character set Emoji and Digit can be mixed with any language**.

**witness:**

```
length|global|char|char|char ...
```

The witness of this cell stores pure binary data in its entity part without molecule encoding. The first 4 bytes are the total data length of uint32, **including these 4 bytes themselves**; the 5th bytes records whether the current character set is a global character set, 0x00
No, 0x01 is; after that are the bytes of available characters, all bytes corresponding to utf-8 encoding, and each character is separated by `0x00`.

The currently available character sets are:

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
    dpoint_cell: Hash,
}

table DasLockOutPointTable {
    ckb_signall: OutPoint,
    ckb_multisign: OutPoint,
    ckb_anyone_can_pay: OutPoint,
    eth: OutPoint,
    tron: OutPoint,
    ed25519: OutPoint,
    doge: OutPoint,
    webauthn: OutPoint,
}

table DasLockTypeIdTable {
    ckb_signhash: Hash,
    ckb_multisig: Hash,
    ed25519: Hash,
    eth: Hash,
    tron: Hash,
    doge: Hash,
    webauthn: Hash,
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

- discount, a list of discount amounts in various situations in DAS;
- prices, DAS price list of account names with different lengths;
- 
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

- proposal_min_confirm_interval, the minimum number of waiting blocks for proposal confirmation;
- proposal_min_extend_interval, the minimum number of waiting blocks for proposal extension;
- proposal_min_recycle_interval, the minimum number of waiting blocks for proposal recycling;
- proposal_max_account_affect, the maximum number of AccountCells that a single proposal can involve;
- proposal_max_pre_account_contain, the maximum number of PreAccountCells that a single proposal can involve;

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

- inviter, the profit rate of the inviter during the account registration process;
- channel, the profit margin of the channel during the account registration process;
- das, DAS official profit margin during the account registration process;
- proposal_create, the profit rate of the proposal created by the keeper during the account registration process;
- proposal_confirm, the keeper confirms the profit margin of the proposal during the account registration process;
- income_consolidate, the profit rate of the keeper in the IncomeCell consolidation process;
- 
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

- inviter, the profit rate of the inviter during the account registration process;

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

- length, the length of the account name, 0 means all lengths except the enumeration length;
- release_start, release start time, unit seconds;
- release_end, release end time, unit seconds;

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

- common_fee, in transactions involving the consumption of AccountSaleCell and AccountAuctionCell, the handling fee that can be taken from these two Cells;
- sale_min_price, the lowest selling price when selling the account at a fixed price;
- sale_expiration_limit, expiration time limit for fixed price orders;
- sale_description_bytes_limit, the byte limit of the description information when placing a fixed price order;
- sale_cell_basic_capacity, the basic storage fee of AccountSaleCell;
- sale_cell_prepared_fee_capacity, the handling fee that should be carried in AccountSaleCell;
- auction_max_duration, the maximum value that the **wait for bid time** can reach in the auction;
- auction_duration_increment, each bid can be the increment brought by **waiting bid time**;
- auction_min_opening_price, the minimum starting price of the auction;
- auction_min_increment_rate_each_bid, the minimum markup rate for each bid;
- auction_description_bytes_limit, the byte limit of the description information when placing an auction order;
- auction_cell_basic_capacity, the basic storage fee of AccountAuctionCell;
- auction_cell_prepared_fee_capacity, the handling fee that should be carried in AccountAuctionCell;
- 
#### ConfigCellReverseResolution

```
table ConfigCellReverseResolution {
    // The common fee for every transactions ReverseCell involved.
    common_fee: Uint64,
    // The basic capacity ReverseCell required, it is bigger than or equal to ReverseCell occupied capacity.
    basic_capacity: Uint64,
}
```

- common_fee, the handling fee that can be taken from itself in transactions involving the consumption of ReverseCell;
- basic_capacity, the basic storage fee of ReverseCell;

#### ConfigCellRecordKeyNamespace

Parse record key namespace.

**witness：**

```
length|key|key|key ...
```
The witness of this cell stores pure binary data in its entity part without molecule encoding. The first 4 bytes are the total data length of uint32, including the 4 bytes itself, and then the ASCII parsing of each key value string available in the record
Encoding, for example, if the original key is `address.eth`, then the stored key is `0x616464726573732E657468`, and each key is separated by `0x00`.

#### ConfigCellPreservedAccountXX

The account name filter is retained. There are currently 20 of them. After hashing the account name, the result of modulo 8 is distributed based on the u8 integer of the first byte.

**witness:**

```
length|hash|hash|hash ...
```

The witness of this cell stores pure binary data in its entity part without molecule encoding. The first 4 bytes are the total data length of uint32, including the 4 bytes itself, followed by the first 20 of the hash of each account name without the suffix.
Data spliced by bytes, because each piece of data is fixed at 20 bytes, so there are no delimiters and other bytes.

#### ConfigCellDPoint
Used to store DPointCell related configurations.
```
table ConfigCellDPoint {
     // The basic capacity DPointCell required, it is bigger than or equal to DPointCell occupied capacity.
     basic_capacity: Uint64,
     // The fees prepared for various transactions.
     prepared_fee_capacity: Uint64,
     // The addresses can transfer and receive DPointCells.
     transfer_whitelist: Scripts,
     // The addresses for recycling the CKB occupied by DPointCells.
     capacity_recycle_whitelist: Scripts,
}

```

### TimeCell, HeightCell, QuoteCell

This is the folk contract script [ckb-time-scripts](https://github.com/nervina-labs/ckb-time-scripts) developed by the Nervina team. It defines a series of Cells that implement oracle-like functions.

Since there is currently no oracle service provider in the Nervos ecosystem, we adopted this solution and added a QuoteCell category.

#### TimeCell

```
lock: <ckb_lock_script>
type:
  code_hash: <ckb-time-scripts>
  type: type
  args: "0x01"
data:
    [index] // 1-byte big-endian encoded u8 integer, which stores the current number in TimeCell
    [type] // 1-byte type, used to identify whether the current Cell is HeightCell
    [timestamp] // 4-byte big-endian encoded u32 integer to store the current UTC timestamp
```

> TimeCell Because the timestamp of TimeCell is actually generated based on the timestamp on the chain, there is an error of about 5 minutes from the real time.
> 
#### HeightCell

```
lock: <ckb_lock_script>
type:
  code_hash: <ckb-time-scripts>
  type: type
  args: "0x02"
data:
    [index] // 1-byte big-endian encoded u8 integer, storing the current number in Height
    [type] // 1-byte type, used to identify whether the current Cell is HeightCell
    [block_height] // 8-byte big-endian encoded u64 integer to store the current block height
```

#### QuoteCell

Since there is currently no stablecoin pegged to the U.S. dollar on the CKB chain, DAS officials will announce the current exchange rate of CKB against the U.S. dollar through a quote Cell. The unit is CKB/USD, because USD adopts a method of expanding `1_000_000` times to be accurate to 6 decimal places, so CKB must also be expanded proportionally, that is, the market price is 0.001
When CKB/USD is used, QuoteCell needs to be recorded as 1000 CKB/USD.

```
lock: <ckb_lock_script>
type:
  code_hash: <ckb-time-scripts>
  type: type
  args: "0x00"
data:
    [index] // 1-byte big-endian encoded u8 integer, storing the current number in Height
    [type] // 1-byte type, used to identify whether the current Cell is HeightCell
    [block_height] // 8-byte big-endian encoded u64 integer to store the current block height
```

#### Volume

`105` Bytes
