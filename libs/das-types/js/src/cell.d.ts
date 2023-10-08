export interface CastToArrayBuffer {
  toArrayBuffer(): ArrayBuffer;
}

export type CanCastToArrayBuffer = ArrayBuffer | CastToArrayBuffer;

export interface CreateOptions {
  validate?: boolean;
}

export interface UnionType {
  type: string;
  value: any;
}

export interface ActionDataType {
  action: BytesType;
  params: BytesType;
}

export interface ConfigCellMainType {
  status: Uint8Type;
  type_id_table: TypeIdTableType;
  das_lock_out_point_table: DasLockOutPointTableType;
  das_lock_type_id_table: DasLockTypeIdTableType;
}

export interface TypeIdTableType {
  account_cell: HashType;
  apply_register_cell: HashType;
  balance_cell: HashType;
  income_cell: HashType;
  pre_account_cell: HashType;
  proposal_cell: HashType;
  account_sale_cell: HashType;
  account_auction_cell: HashType;
  offer_cell: HashType;
  reverse_record_cell: HashType;
  sub_account_cell: HashType;
  eip712_lib: HashType;
  reverse_record_root_cell: HashType;
  key_list_config_cell: HashType;
}

export interface DasLockOutPointTableType {
  ckb_signall: OutPointType;
  ckb_multisign: OutPointType;
  ckb_anyone_can_pay: OutPointType;
  eth: OutPointType;
  tron: OutPointType;
  ed25519: OutPointType;
  web_authn: OutPointType;
}

export interface DasLockTypeIdTableType {
  ckb_signhash: HashType;
  ckb_multisig: HashType;
  ed25519: HashType;
  eth: HashType;
  tron: HashType;
  doge: HashType;
  web_authn: HashType;
}

export interface ConfigCellAccountType {
  max_length: Uint32Type;
  basic_capacity: Uint64Type;
  prepared_fee_capacity: Uint64Type;
  expiration_grace_period: Uint32Type;
  record_min_ttl: Uint32Type;
  record_size_limit: Uint32Type;
  transfer_account_fee: Uint64Type;
  edit_manager_fee: Uint64Type;
  edit_records_fee: Uint64Type;
  common_fee: Uint64Type;
  transfer_account_throttle: Uint32Type;
  edit_manager_throttle: Uint32Type;
  edit_records_throttle: Uint32Type;
  common_throttle: Uint32Type;
}

export interface ConfigCellApplyType {
  apply_min_waiting_block_number: Uint32Type;
  apply_max_waiting_block_number: Uint32Type;
}

export type CharsType = BytesType[];

export interface ConfigCellPriceType {
  discount: DiscountConfigType;
  prices: PriceConfigListType;
}

export interface DiscountConfigType {
  invited_discount: Uint32Type;
}

export type PriceConfigListType = PriceConfigType[];

export interface PriceConfigType {
  length: Uint8Type;
  new: Uint64Type;
  renew: Uint64Type;
}

export interface ConfigCellProposalType {
  proposal_min_confirm_interval: Uint8Type;
  proposal_min_extend_interval: Uint8Type;
  proposal_min_recycle_interval: Uint8Type;
  proposal_max_account_affect: Uint32Type;
  proposal_max_pre_account_contain: Uint32Type;
}

export interface ConfigCellProfitRateType {
  inviter: Uint32Type;
  channel: Uint32Type;
  proposal_create: Uint32Type;
  proposal_confirm: Uint32Type;
  income_consolidate: Uint32Type;
  sale_buyer_inviter: Uint32Type;
  sale_buyer_channel: Uint32Type;
  sale_das: Uint32Type;
  auction_bidder_inviter: Uint32Type;
  auction_bidder_channel: Uint32Type;
  auction_das: Uint32Type;
  auction_prev_bidder: Uint32Type;
}

export interface ConfigCellIncomeType {
  basic_capacity: Uint64Type;
  max_records: Uint32Type;
  min_transfer_capacity: Uint64Type;
}

export interface ConfigCellReleaseType {
  lucky_number: Uint32Type;
}

export interface ConfigCellSecondaryMarketType {
  common_fee: Uint64Type;
  sale_min_price: Uint64Type;
  sale_expiration_limit: Uint32Type;
  sale_description_bytes_limit: Uint32Type;
  sale_cell_basic_capacity: Uint64Type;
  sale_cell_prepared_fee_capacity: Uint64Type;
  auction_max_extendable_duration: Uint32Type;
  auction_duration_increment_each_bid: Uint32Type;
  auction_min_opening_price: Uint64Type;
  auction_min_increment_rate_each_bid: Uint32Type;
  auction_description_bytes_limit: Uint32Type;
  auction_cell_basic_capacity: Uint64Type;
  auction_cell_prepared_fee_capacity: Uint64Type;
  offer_min_price: Uint64Type;
  offer_cell_basic_capacity: Uint64Type;
  offer_cell_prepared_fee_capacity: Uint64Type;
  offer_message_bytes_limit: Uint32Type;
}

export interface ConfigCellReverseResolutionType {
  record_basic_capacity: Uint64Type;
  record_prepared_fee_capacity: Uint64Type;
  common_fee: Uint64Type;
}

export interface ConfigCellSubAccountType {
  basic_capacity: Uint64Type;
  prepared_fee_capacity: Uint64Type;
  new_sub_account_price: Uint64Type;
  renew_sub_account_price: Uint64Type;
  common_fee: Uint64Type;
  create_fee: Uint64Type;
  edit_fee: Uint64Type;
  renew_fee: Uint64Type;
  recycle_fee: Uint64Type;
  new_sub_account_custom_price_das_profit_rate: Uint32Type;
  renew_sub_account_custom_price_das_profit_rate: Uint32Type;
}

export interface ConfigCellSystemStatusType {
  apply_register_cell_type: ContractStatusType;
  pre_account_cell_type: ContractStatusType;
  proposal_cell_type: ContractStatusType;
  config_cell_type: ContractStatusType;
  account_cell_type: ContractStatusType;
  account_sale_cell_type: ContractStatusType;
  sub_account_cell_type: ContractStatusType;
  offer_cell_type: ContractStatusType;
  balance_cell_type: ContractStatusType;
  income_cell_type: ContractStatusType;
  reverse_record_cell_type: ContractStatusType;
  reverse_record_root_cell_type: ContractStatusType;
  eip712_lib: ContractStatusType;
}

export interface ContractStatusType {
  status: CanCastToArrayBuffer;
  version: BytesType;
}

export interface ProposalCellDataType {
  proposer_lock: ScriptType;
  created_at_height: Uint64Type;
  slices: SliceListType;
}

export type SliceListType = SLType[];

export type SLType = ProposalItemType[];

export interface ProposalItemType {
  account_id: AccountIdType;
  item_type: Uint8Type;
  next: AccountIdType;
}

export interface IncomeCellDataType {
  creator: ScriptType;
  records: IncomeRecordsType;
}

export type IncomeRecordsType = IncomeRecordType[];

export interface IncomeRecordType {
  belong_to: ScriptType;
  capacity: Uint64Type;
}

export interface AccountCellDataV2Type {
  id: AccountIdType;
  account: AccountCharsType;
  registered_at: Uint64Type;
  last_transfer_account_at: Uint64Type;
  last_edit_manager_at: Uint64Type;
  last_edit_records_at: Uint64Type;
  status: Uint8Type;
  records: RecordsType;
}

export interface AccountCellDataType {
  id: AccountIdType;
  account: AccountCharsType;
  registered_at: Uint64Type;
  last_transfer_account_at: Uint64Type;
  last_edit_manager_at: Uint64Type;
  last_edit_records_at: Uint64Type;
  status: Uint8Type;
  records: RecordsType;
  enable_sub_account: Uint8Type;
  renew_sub_account_price: Uint64Type;
}

export type AccountIdType = CanCastToArrayBuffer;

export interface RecordType {
  record_type: BytesType;
  record_key: BytesType;
  record_label: BytesType;
  record_value: BytesType;
  record_ttl: Uint32Type;
}

export type RecordsType = RecordType[];

export interface AccountSaleCellDataV1Type {
  account_id: AccountIdType;
  account: BytesType;
  price: Uint64Type;
  description: BytesType;
  started_at: Uint64Type;
}

export interface AccountSaleCellDataType {
  account_id: AccountIdType;
  account: BytesType;
  price: Uint64Type;
  description: BytesType;
  started_at: Uint64Type;
  buyer_inviter_profit_rate: Uint32Type;
}

export interface AccountAuctionCellDataType {
  account_id: AccountIdType;
  account: BytesType;
  description: BytesType;
  opening_price: Uint64Type;
  increment_rate_each_bid: Uint32Type;
  started_at: Uint64Type;
  ended_at: Uint64Type;
  current_bidder_lock: ScriptType;
  current_bid_price: Uint64Type;
  prev_bidder_profit_rate: Uint32Type;
}

export interface PreAccountCellDataV1Type {
  account: AccountCharsType;
  refund_lock: ScriptType;
  owner_lock_args: BytesType;
  inviter_id: BytesType;
  inviter_lock?: ScriptType;
  channel_lock?: ScriptType;
  price: PriceConfigType;
  quote: Uint64Type;
  invited_discount: Uint32Type;
  created_at: Uint64Type;
}

export interface PreAccountCellDataV2Type {
  account: AccountCharsType;
  refund_lock: ScriptType;
  owner_lock_args: BytesType;
  inviter_id: BytesType;
  inviter_lock?: ScriptType;
  channel_lock?: ScriptType;
  price: PriceConfigType;
  quote: Uint64Type;
  invited_discount: Uint32Type;
  created_at: Uint64Type;
  initial_records: RecordsType;
}

export interface PreAccountCellDataType {
  account: AccountCharsType;
  refund_lock: ScriptType;
  owner_lock_args: BytesType;
  inviter_id: BytesType;
  inviter_lock?: ScriptType;
  channel_lock?: ScriptType;
  price: PriceConfigType;
  quote: Uint64Type;
  invited_discount: Uint32Type;
  created_at: Uint64Type;
  initial_records: RecordsType;
  initial_cross_chain: ChainIdType;
}

export interface ChainIdType {
  checked: Uint8Type;
  coin_type: Uint64Type;
  chain_id: Uint64Type;
}

export type AccountCharsType = AccountCharType[];

export interface AccountCharType {
  char_set_name: Uint32Type;
  bytes: BytesType;
}

export interface OfferCellDataType {
  account: BytesType;
  price: Uint64Type;
  message: BytesType;
  inviter_lock: ScriptType;
  channel_lock: ScriptType;
}

export interface SubAccountType {
  lock: ScriptType;
  id: AccountIdType;
  account: AccountCharsType;
  suffix: BytesType;
  registered_at: Uint64Type;
  expired_at: Uint64Type;
  status: Uint8Type;
  records: RecordsType;
  nonce: Uint64Type;
  enable_sub_account: Uint8Type;
  renew_sub_account_price: Uint64Type;
}

export interface SubAccountRuleType {
  index: Uint32Type;
  name: BytesType;
  note: BytesType;
  price: Uint64Type;
  ast: ASTExpressionType;
  status: Uint8Type;
}

export type SubAccountRulesType = SubAccountRuleType[];

export interface ASTExpressionType {
  expression_type: CanCastToArrayBuffer;
  expression: BytesType;
}

export type ASTExpressionsType = ASTExpressionType[];

export interface ASTOperatorType {
  symbol: CanCastToArrayBuffer;
  expressions: ASTExpressionsType;
}

export interface ASTFunctionType {
  name: CanCastToArrayBuffer;
  arguments: ASTExpressionsType;
}

export interface ASTVariableType {
  name: CanCastToArrayBuffer;
}

export interface ASTValueType {
  value_type: CanCastToArrayBuffer;
  value: BytesType;
}

export interface DeviceKeyType {
  main_alg_id: Uint8Type;
  sub_alg_id: Uint8Type;
  cid: Byte10Type;
  pubkey: Byte10Type;
}

export type DeviceKeyListType = DeviceKeyType[];

export interface DeviceKeyListCellDataType {
  keys: DeviceKeyListType;
  refund_lock: ScriptType;
}

export type Uint8Type = CanCastToArrayBuffer;

export type Uint32Type = CanCastToArrayBuffer;

export type Uint64Type = CanCastToArrayBuffer;

export type Byte10Type = CanCastToArrayBuffer;

export type BytesType = CanCastToArrayBuffer;

export type BytesVecType = BytesType[];

export type HashType = CanCastToArrayBuffer;

export interface ScriptType {
  code_hash: HashType;
  hash_type: CanCastToArrayBuffer;
  args: BytesType;
}

export type ScriptOptType = ScriptType | undefined;

export interface OutPointType {
  tx_hash: HashType;
  index: Uint32Type;
}

export interface DataType {
  dep?: DataEntityType;
  old?: DataEntityType;
  new?: DataEntityType;
}

export interface DataEntityType {
  index: Uint32Type;
  version: Uint32Type;
  entity: BytesType;
}

export type DataEntityOptType = DataEntityType | undefined;

export function SerializeActionData(value: ActionDataType): ArrayBuffer;
export class ActionData {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getAction(): Bytes;
  getParams(): Bytes;
}

export function SerializeConfigCellMain(value: ConfigCellMainType): ArrayBuffer;
export class ConfigCellMain {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getStatus(): Uint8;
  getTypeIdTable(): TypeIdTable;
  getDasLockOutPointTable(): DasLockOutPointTable;
  getDasLockTypeIdTable(): DasLockTypeIdTable;
}

export function SerializeTypeIdTable(value: TypeIdTableType): ArrayBuffer;
export class TypeIdTable {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getAccountCell(): Hash;
  getApplyRegisterCell(): Hash;
  getBalanceCell(): Hash;
  getIncomeCell(): Hash;
  getPreAccountCell(): Hash;
  getProposalCell(): Hash;
  getAccountSaleCell(): Hash;
  getAccountAuctionCell(): Hash;
  getOfferCell(): Hash;
  getReverseRecordCell(): Hash;
  getSubAccountCell(): Hash;
  getEip712Lib(): Hash;
  getReverseRecordRootCell(): Hash;
  getKeyListConfigCell(): Hash;
}

export function SerializeDasLockOutPointTable(value: DasLockOutPointTableType): ArrayBuffer;
export class DasLockOutPointTable {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getCkbSignall(): OutPoint;
  getCkbMultisign(): OutPoint;
  getCkbAnyoneCanPay(): OutPoint;
  getEth(): OutPoint;
  getTron(): OutPoint;
  getEd25519(): OutPoint;
  getWebAuthn(): OutPoint;
}

export function SerializeDasLockTypeIdTable(value: DasLockTypeIdTableType): ArrayBuffer;
export class DasLockTypeIdTable {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getCkbSignhash(): Hash;
  getCkbMultisig(): Hash;
  getEd25519(): Hash;
  getEth(): Hash;
  getTron(): Hash;
  getDoge(): Hash;
  getWebAuthn(): Hash;
}

export function SerializeConfigCellAccount(value: ConfigCellAccountType): ArrayBuffer;
export class ConfigCellAccount {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getMaxLength(): Uint32;
  getBasicCapacity(): Uint64;
  getPreparedFeeCapacity(): Uint64;
  getExpirationGracePeriod(): Uint32;
  getRecordMinTtl(): Uint32;
  getRecordSizeLimit(): Uint32;
  getTransferAccountFee(): Uint64;
  getEditManagerFee(): Uint64;
  getEditRecordsFee(): Uint64;
  getCommonFee(): Uint64;
  getTransferAccountThrottle(): Uint32;
  getEditManagerThrottle(): Uint32;
  getEditRecordsThrottle(): Uint32;
  getCommonThrottle(): Uint32;
}

export function SerializeConfigCellApply(value: ConfigCellApplyType): ArrayBuffer;
export class ConfigCellApply {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getApplyMinWaitingBlockNumber(): Uint32;
  getApplyMaxWaitingBlockNumber(): Uint32;
}

export function SerializeChars(value: Array<BytesType>): ArrayBuffer;
export class Chars {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): Bytes;
  length(): number;
}

export function SerializeConfigCellPrice(value: ConfigCellPriceType): ArrayBuffer;
export class ConfigCellPrice {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getDiscount(): DiscountConfig;
  getPrices(): PriceConfigList;
}

export function SerializeDiscountConfig(value: DiscountConfigType): ArrayBuffer;
export class DiscountConfig {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getInvitedDiscount(): Uint32;
}

export function SerializePriceConfigList(value: Array<PriceConfigType>): ArrayBuffer;
export class PriceConfigList {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): PriceConfig;
  length(): number;
}

export function SerializePriceConfig(value: PriceConfigType): ArrayBuffer;
export class PriceConfig {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getLength(): Uint8;
  getNew(): Uint64;
  getRenew(): Uint64;
}

export function SerializeConfigCellProposal(value: ConfigCellProposalType): ArrayBuffer;
export class ConfigCellProposal {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getProposalMinConfirmInterval(): Uint8;
  getProposalMinExtendInterval(): Uint8;
  getProposalMinRecycleInterval(): Uint8;
  getProposalMaxAccountAffect(): Uint32;
  getProposalMaxPreAccountContain(): Uint32;
}

export function SerializeConfigCellProfitRate(value: ConfigCellProfitRateType): ArrayBuffer;
export class ConfigCellProfitRate {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getInviter(): Uint32;
  getChannel(): Uint32;
  getProposalCreate(): Uint32;
  getProposalConfirm(): Uint32;
  getIncomeConsolidate(): Uint32;
  getSaleBuyerInviter(): Uint32;
  getSaleBuyerChannel(): Uint32;
  getSaleDas(): Uint32;
  getAuctionBidderInviter(): Uint32;
  getAuctionBidderChannel(): Uint32;
  getAuctionDas(): Uint32;
  getAuctionPrevBidder(): Uint32;
}

export function SerializeConfigCellIncome(value: ConfigCellIncomeType): ArrayBuffer;
export class ConfigCellIncome {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getBasicCapacity(): Uint64;
  getMaxRecords(): Uint32;
  getMinTransferCapacity(): Uint64;
}

export function SerializeConfigCellRelease(value: ConfigCellReleaseType): ArrayBuffer;
export class ConfigCellRelease {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getLuckyNumber(): Uint32;
}

export function SerializeConfigCellSecondaryMarket(value: ConfigCellSecondaryMarketType): ArrayBuffer;
export class ConfigCellSecondaryMarket {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getCommonFee(): Uint64;
  getSaleMinPrice(): Uint64;
  getSaleExpirationLimit(): Uint32;
  getSaleDescriptionBytesLimit(): Uint32;
  getSaleCellBasicCapacity(): Uint64;
  getSaleCellPreparedFeeCapacity(): Uint64;
  getAuctionMaxExtendableDuration(): Uint32;
  getAuctionDurationIncrementEachBid(): Uint32;
  getAuctionMinOpeningPrice(): Uint64;
  getAuctionMinIncrementRateEachBid(): Uint32;
  getAuctionDescriptionBytesLimit(): Uint32;
  getAuctionCellBasicCapacity(): Uint64;
  getAuctionCellPreparedFeeCapacity(): Uint64;
  getOfferMinPrice(): Uint64;
  getOfferCellBasicCapacity(): Uint64;
  getOfferCellPreparedFeeCapacity(): Uint64;
  getOfferMessageBytesLimit(): Uint32;
}

export function SerializeConfigCellReverseResolution(value: ConfigCellReverseResolutionType): ArrayBuffer;
export class ConfigCellReverseResolution {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getRecordBasicCapacity(): Uint64;
  getRecordPreparedFeeCapacity(): Uint64;
  getCommonFee(): Uint64;
}

export function SerializeConfigCellSubAccount(value: ConfigCellSubAccountType): ArrayBuffer;
export class ConfigCellSubAccount {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getBasicCapacity(): Uint64;
  getPreparedFeeCapacity(): Uint64;
  getNewSubAccountPrice(): Uint64;
  getRenewSubAccountPrice(): Uint64;
  getCommonFee(): Uint64;
  getCreateFee(): Uint64;
  getEditFee(): Uint64;
  getRenewFee(): Uint64;
  getRecycleFee(): Uint64;
  getNewSubAccountCustomPriceDasProfitRate(): Uint32;
  getRenewSubAccountCustomPriceDasProfitRate(): Uint32;
}

export function SerializeConfigCellSystemStatus(value: ConfigCellSystemStatusType): ArrayBuffer;
export class ConfigCellSystemStatus {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getApplyRegisterCellType(): ContractStatus;
  getPreAccountCellType(): ContractStatus;
  getProposalCellType(): ContractStatus;
  getConfigCellType(): ContractStatus;
  getAccountCellType(): ContractStatus;
  getAccountSaleCellType(): ContractStatus;
  getSubAccountCellType(): ContractStatus;
  getOfferCellType(): ContractStatus;
  getBalanceCellType(): ContractStatus;
  getIncomeCellType(): ContractStatus;
  getReverseRecordCellType(): ContractStatus;
  getReverseRecordRootCellType(): ContractStatus;
  getEip712Lib(): ContractStatus;
}

export function SerializeContractStatus(value: ContractStatusType): ArrayBuffer;
export class ContractStatus {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getStatus(): number;
  getVersion(): Bytes;
}

export function SerializeProposalCellData(value: ProposalCellDataType): ArrayBuffer;
export class ProposalCellData {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getProposerLock(): Script;
  getCreatedAtHeight(): Uint64;
  getSlices(): SliceList;
}

export function SerializeSliceList(value: Array<SLType>): ArrayBuffer;
export class SliceList {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): SL;
  length(): number;
}

export function SerializeSL(value: Array<ProposalItemType>): ArrayBuffer;
export class SL {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): ProposalItem;
  length(): number;
}

export function SerializeProposalItem(value: ProposalItemType): ArrayBuffer;
export class ProposalItem {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getAccountId(): AccountId;
  getItemType(): Uint8;
  getNext(): AccountId;
}

export function SerializeIncomeCellData(value: IncomeCellDataType): ArrayBuffer;
export class IncomeCellData {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getCreator(): Script;
  getRecords(): IncomeRecords;
}

export function SerializeIncomeRecords(value: Array<IncomeRecordType>): ArrayBuffer;
export class IncomeRecords {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): IncomeRecord;
  length(): number;
}

export function SerializeIncomeRecord(value: IncomeRecordType): ArrayBuffer;
export class IncomeRecord {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getBelongTo(): Script;
  getCapacity(): Uint64;
}

export function SerializeAccountCellDataV2(value: AccountCellDataV2Type): ArrayBuffer;
export class AccountCellDataV2 {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getId(): AccountId;
  getAccount(): AccountChars;
  getRegisteredAt(): Uint64;
  getLastTransferAccountAt(): Uint64;
  getLastEditManagerAt(): Uint64;
  getLastEditRecordsAt(): Uint64;
  getStatus(): Uint8;
  getRecords(): Records;
}

export function SerializeAccountCellData(value: AccountCellDataType): ArrayBuffer;
export class AccountCellData {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getId(): AccountId;
  getAccount(): AccountChars;
  getRegisteredAt(): Uint64;
  getLastTransferAccountAt(): Uint64;
  getLastEditManagerAt(): Uint64;
  getLastEditRecordsAt(): Uint64;
  getStatus(): Uint8;
  getRecords(): Records;
  getEnableSubAccount(): Uint8;
  getRenewSubAccountPrice(): Uint64;
}

export function SerializeAccountId(value: CanCastToArrayBuffer): ArrayBuffer;
export class AccountId {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): number;
  raw(): ArrayBuffer;
  static size(): Number;
}

export function SerializeRecord(value: RecordType): ArrayBuffer;
export class Record {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getRecordType(): Bytes;
  getRecordKey(): Bytes;
  getRecordLabel(): Bytes;
  getRecordValue(): Bytes;
  getRecordTtl(): Uint32;
}

export function SerializeRecords(value: Array<RecordType>): ArrayBuffer;
export class Records {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): Record;
  length(): number;
}

export function SerializeAccountSaleCellDataV1(value: AccountSaleCellDataV1Type): ArrayBuffer;
export class AccountSaleCellDataV1 {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getAccountId(): AccountId;
  getAccount(): Bytes;
  getPrice(): Uint64;
  getDescription(): Bytes;
  getStartedAt(): Uint64;
}

export function SerializeAccountSaleCellData(value: AccountSaleCellDataType): ArrayBuffer;
export class AccountSaleCellData {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getAccountId(): AccountId;
  getAccount(): Bytes;
  getPrice(): Uint64;
  getDescription(): Bytes;
  getStartedAt(): Uint64;
  getBuyerInviterProfitRate(): Uint32;
}

export function SerializeAccountAuctionCellData(value: AccountAuctionCellDataType): ArrayBuffer;
export class AccountAuctionCellData {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getAccountId(): AccountId;
  getAccount(): Bytes;
  getDescription(): Bytes;
  getOpeningPrice(): Uint64;
  getIncrementRateEachBid(): Uint32;
  getStartedAt(): Uint64;
  getEndedAt(): Uint64;
  getCurrentBidderLock(): Script;
  getCurrentBidPrice(): Uint64;
  getPrevBidderProfitRate(): Uint32;
}

export function SerializePreAccountCellDataV1(value: PreAccountCellDataV1Type): ArrayBuffer;
export class PreAccountCellDataV1 {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getAccount(): AccountChars;
  getRefundLock(): Script;
  getOwnerLockArgs(): Bytes;
  getInviterId(): Bytes;
  getInviterLock(): ScriptOpt;
  getChannelLock(): ScriptOpt;
  getPrice(): PriceConfig;
  getQuote(): Uint64;
  getInvitedDiscount(): Uint32;
  getCreatedAt(): Uint64;
}

export function SerializePreAccountCellDataV2(value: PreAccountCellDataV2Type): ArrayBuffer;
export class PreAccountCellDataV2 {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getAccount(): AccountChars;
  getRefundLock(): Script;
  getOwnerLockArgs(): Bytes;
  getInviterId(): Bytes;
  getInviterLock(): ScriptOpt;
  getChannelLock(): ScriptOpt;
  getPrice(): PriceConfig;
  getQuote(): Uint64;
  getInvitedDiscount(): Uint32;
  getCreatedAt(): Uint64;
  getInitialRecords(): Records;
}

export function SerializePreAccountCellData(value: PreAccountCellDataType): ArrayBuffer;
export class PreAccountCellData {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getAccount(): AccountChars;
  getRefundLock(): Script;
  getOwnerLockArgs(): Bytes;
  getInviterId(): Bytes;
  getInviterLock(): ScriptOpt;
  getChannelLock(): ScriptOpt;
  getPrice(): PriceConfig;
  getQuote(): Uint64;
  getInvitedDiscount(): Uint32;
  getCreatedAt(): Uint64;
  getInitialRecords(): Records;
  getInitialCrossChain(): ChainId;
}

export function SerializeChainId(value: ChainIdType): ArrayBuffer;
export class ChainId {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getChecked(): Uint8;
  getCoinType(): Uint64;
  getChainId(): Uint64;
}

export function SerializeAccountChars(value: Array<AccountCharType>): ArrayBuffer;
export class AccountChars {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): AccountChar;
  length(): number;
}

export function SerializeAccountChar(value: AccountCharType): ArrayBuffer;
export class AccountChar {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getCharSetName(): Uint32;
  getBytes(): Bytes;
}

export function SerializeOfferCellData(value: OfferCellDataType): ArrayBuffer;
export class OfferCellData {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getAccount(): Bytes;
  getPrice(): Uint64;
  getMessage(): Bytes;
  getInviterLock(): Script;
  getChannelLock(): Script;
}

export function SerializeSubAccount(value: SubAccountType): ArrayBuffer;
export class SubAccount {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getLock(): Script;
  getId(): AccountId;
  getAccount(): AccountChars;
  getSuffix(): Bytes;
  getRegisteredAt(): Uint64;
  getExpiredAt(): Uint64;
  getStatus(): Uint8;
  getRecords(): Records;
  getNonce(): Uint64;
  getEnableSubAccount(): Uint8;
  getRenewSubAccountPrice(): Uint64;
}

export function SerializeSubAccountRule(value: SubAccountRuleType): ArrayBuffer;
export class SubAccountRule {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getIndex(): Uint32;
  getName(): Bytes;
  getNote(): Bytes;
  getPrice(): Uint64;
  getAst(): ASTExpression;
  getStatus(): Uint8;
}

export function SerializeSubAccountRules(value: Array<SubAccountRuleType>): ArrayBuffer;
export class SubAccountRules {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): SubAccountRule;
  length(): number;
}

export function SerializeASTExpression(value: ASTExpressionType): ArrayBuffer;
export class ASTExpression {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getExpressionType(): number;
  getExpression(): Bytes;
}

export function SerializeASTExpressions(value: Array<ASTExpressionType>): ArrayBuffer;
export class ASTExpressions {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): ASTExpression;
  length(): number;
}

export function SerializeASTOperator(value: ASTOperatorType): ArrayBuffer;
export class ASTOperator {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getSymbol(): number;
  getExpressions(): ASTExpressions;
}

export function SerializeASTFunction(value: ASTFunctionType): ArrayBuffer;
export class ASTFunction {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getName(): number;
  getArguments(): ASTExpressions;
}

export function SerializeASTVariable(value: ASTVariableType): ArrayBuffer;
export class ASTVariable {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getName(): number;
}

export function SerializeASTValue(value: ASTValueType): ArrayBuffer;
export class ASTValue {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getValueType(): number;
  getValue(): Bytes;
}

export function SerializeDeviceKey(value: DeviceKeyType): ArrayBuffer;
export class DeviceKey {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  static size(): Number;
  getMainAlgId(): Uint8;
  getSubAlgId(): Uint8;
  getCid(): Byte10;
  getPubkey(): Byte10;
}

export function SerializeDeviceKeyList(value: Array<DeviceKeyType>): ArrayBuffer;
export class DeviceKeyList {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): DeviceKey;
  length(): number;
}

export function SerializeDeviceKeyListCellData(value: DeviceKeyListCellDataType): ArrayBuffer;
export class DeviceKeyListCellData {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getKeys(): DeviceKeyList;
  getRefundLock(): Script;
}

export function SerializeUint8(value: CanCastToArrayBuffer): ArrayBuffer;
export class Uint8 {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): number;
  raw(): ArrayBuffer;
  static size(): Number;
}

export function SerializeUint32(value: CanCastToArrayBuffer): ArrayBuffer;
export class Uint32 {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): number;
  raw(): ArrayBuffer;
  toBigEndianUint32(): number;
  toLittleEndianUint32(): number;
  static size(): Number;
}

export function SerializeUint64(value: CanCastToArrayBuffer): ArrayBuffer;
export class Uint64 {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): number;
  raw(): ArrayBuffer;
  toBigEndianBigUint64(): bigint;
  toLittleEndianBigUint64(): bigint;
  static size(): Number;
}

export function SerializeByte10(value: CanCastToArrayBuffer): ArrayBuffer;
export class Byte10 {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): number;
  raw(): ArrayBuffer;
  static size(): Number;
}

export function SerializeBytes(value: CanCastToArrayBuffer): ArrayBuffer;
export class Bytes {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): number;
  raw(): ArrayBuffer;
  length(): number;
}

export function SerializeBytesVec(value: Array<BytesType>): ArrayBuffer;
export class BytesVec {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): Bytes;
  length(): number;
}

export function SerializeHash(value: CanCastToArrayBuffer): ArrayBuffer;
export class Hash {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  indexAt(i: number): number;
  raw(): ArrayBuffer;
  static size(): Number;
}

export function SerializeScript(value: ScriptType): ArrayBuffer;
export class Script {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getCodeHash(): Hash;
  getHashType(): number;
  getArgs(): Bytes;
}

export function SerializeScriptOpt(value: ScriptType | null): ArrayBuffer;
export class ScriptOpt {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  value(): Script;
  hasValue(): boolean;
}

export function SerializeOutPoint(value: OutPointType): ArrayBuffer;
export class OutPoint {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  static size(): Number;
  getTxHash(): Hash;
  getIndex(): Uint32;
}

export function SerializeData(value: DataType): ArrayBuffer;
export class Data {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getDep(): DataEntityOpt;
  getOld(): DataEntityOpt;
  getNew(): DataEntityOpt;
}

export function SerializeDataEntity(value: DataEntityType): ArrayBuffer;
export class DataEntity {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  getIndex(): Uint32;
  getVersion(): Uint32;
  getEntity(): Bytes;
}

export function SerializeDataEntityOpt(value: DataEntityType | null): ArrayBuffer;
export class DataEntityOpt {
  constructor(reader: CanCastToArrayBuffer, options?: CreateOptions);
  validate(compatible?: boolean): void;
  value(): DataEntity;
  hasValue(): boolean;
}

