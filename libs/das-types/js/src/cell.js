function dataLengthError(actual, required) {
    throw new Error(`Invalid data length! Required: ${required}, actual: ${actual}`);
}

function assertDataLength(actual, required) {
  if (actual !== required) {
    dataLengthError(actual, required);
  }
}

function assertArrayBuffer(reader) {
  if (reader instanceof Object && reader.toArrayBuffer instanceof Function) {
    reader = reader.toArrayBuffer();
  }
  if (!(reader instanceof ArrayBuffer)) {
    throw new Error("Provided value must be an ArrayBuffer or can be transformed into ArrayBuffer!");
  }
  return reader;
}

function verifyAndExtractOffsets(view, expectedFieldCount, compatible) {
  if (view.byteLength < 4) {
    dataLengthError(view.byteLength, ">4");
  }
  const requiredByteLength = view.getUint32(0, true);
  assertDataLength(view.byteLength, requiredByteLength);
  if (requiredByteLength === 4) {
    return [requiredByteLength];
  }
  if (requiredByteLength < 8) {
    dataLengthError(view.byteLength, ">8");
  }
  const firstOffset = view.getUint32(4, true);
  if (firstOffset % 4 !== 0 || firstOffset < 8) {
    throw new Error(`Invalid first offset: ${firstOffset}`);
  }
  const itemCount = firstOffset / 4 - 1;
  if (itemCount < expectedFieldCount) {
    throw new Error(`Item count not enough! Required: ${expectedFieldCount}, actual: ${itemCount}`);
  } else if ((!compatible) && itemCount > expectedFieldCount) {
    throw new Error(`Item count is more than required! Required: ${expectedFieldCount}, actual: ${itemCount}`);
  }
  if (requiredByteLength < firstOffset) {
    throw new Error(`First offset is larger than byte length: ${firstOffset}`);
  }
  const offsets = [];
  for (let i = 0; i < itemCount; i++) {
    const start = 4 + i * 4;
    offsets.push(view.getUint32(start, true));
  }
  offsets.push(requiredByteLength);
  for (let i = 0; i < offsets.length - 1; i++) {
    if (offsets[i] > offsets[i + 1]) {
      throw new Error(`Offset index ${i}: ${offsets[i]} is larger than offset index ${i + 1}: ${offsets[i + 1]}`);
    }
  }
  return offsets;
}

function serializeTable(buffers) {
  const itemCount = buffers.length;
  let totalSize = 4 * (itemCount + 1);
  const offsets = [];

  for (let i = 0; i < itemCount; i++) {
    offsets.push(totalSize);
    totalSize += buffers[i].byteLength;
  }

  const buffer = new ArrayBuffer(totalSize);
  const array = new Uint8Array(buffer);
  const view = new DataView(buffer);

  view.setUint32(0, totalSize, true);
  for (let i = 0; i < itemCount; i++) {
    view.setUint32(4 + i * 4, offsets[i], true);
    array.set(new Uint8Array(buffers[i]), offsets[i]);
  }
  return buffer;
}

export class ActionData {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Bytes(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
  }

  getAction() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getParams() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeActionData(value) {
  const buffers = [];
  buffers.push(SerializeBytes(value.action));
  buffers.push(SerializeBytes(value.params));
  return serializeTable(buffers);
}

export class ConfigCellMain {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint8(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new TypeIdTable(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new DasLockOutPointTable(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new DasLockTypeIdTable(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
  }

  getStatus() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint8(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getTypeIdTable() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new TypeIdTable(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getDasLockOutPointTable() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new DasLockOutPointTable(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getDasLockTypeIdTable() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new DasLockTypeIdTable(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeConfigCellMain(value) {
  const buffers = [];
  buffers.push(SerializeUint8(value.status));
  buffers.push(SerializeTypeIdTable(value.type_id_table));
  buffers.push(SerializeDasLockOutPointTable(value.das_lock_out_point_table));
  buffers.push(SerializeDasLockTypeIdTable(value.das_lock_type_id_table));
  return serializeTable(buffers);
}

export class TypeIdTable {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Hash(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[6], offsets[7]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[7], offsets[8]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[8], offsets[9]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[9], offsets[10]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[10], offsets[11]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[11], offsets[12]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[12], offsets[13]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[13], offsets[14]), { validate: false }).validate();
  }

  getAccountCell() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getApplyRegisterCell() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getBalanceCell() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getIncomeCell() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getPreAccountCell() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getProposalCell() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAccountSaleCell() {
    const start = 28;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAccountAuctionCell() {
    const start = 32;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getOfferCell() {
    const start = 36;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getReverseRecordCell() {
    const start = 40;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getSubAccountCell() {
    const start = 44;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getEip712Lib() {
    const start = 48;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getReverseRecordRootCell() {
    const start = 52;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getKeyListConfigCell() {
    const start = 56;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeTypeIdTable(value) {
  const buffers = [];
  buffers.push(SerializeHash(value.account_cell));
  buffers.push(SerializeHash(value.apply_register_cell));
  buffers.push(SerializeHash(value.balance_cell));
  buffers.push(SerializeHash(value.income_cell));
  buffers.push(SerializeHash(value.pre_account_cell));
  buffers.push(SerializeHash(value.proposal_cell));
  buffers.push(SerializeHash(value.account_sale_cell));
  buffers.push(SerializeHash(value.account_auction_cell));
  buffers.push(SerializeHash(value.offer_cell));
  buffers.push(SerializeHash(value.reverse_record_cell));
  buffers.push(SerializeHash(value.sub_account_cell));
  buffers.push(SerializeHash(value.eip712_lib));
  buffers.push(SerializeHash(value.reverse_record_root_cell));
  buffers.push(SerializeHash(value.key_list_config_cell));
  return serializeTable(buffers);
}

export class DasLockOutPointTable {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new OutPoint(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new OutPoint(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new OutPoint(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new OutPoint(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new OutPoint(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new OutPoint(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
    new OutPoint(this.view.buffer.slice(offsets[6], offsets[7]), { validate: false }).validate();
  }

  getCkbSignall() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new OutPoint(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCkbMultisign() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new OutPoint(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCkbAnyoneCanPay() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new OutPoint(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getEth() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new OutPoint(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getTron() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new OutPoint(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getEd25519() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new OutPoint(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getWebAuthn() {
    const start = 28;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new OutPoint(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeDasLockOutPointTable(value) {
  const buffers = [];
  buffers.push(SerializeOutPoint(value.ckb_signall));
  buffers.push(SerializeOutPoint(value.ckb_multisign));
  buffers.push(SerializeOutPoint(value.ckb_anyone_can_pay));
  buffers.push(SerializeOutPoint(value.eth));
  buffers.push(SerializeOutPoint(value.tron));
  buffers.push(SerializeOutPoint(value.ed25519));
  buffers.push(SerializeOutPoint(value.web_authn));
  return serializeTable(buffers);
}

export class DasLockTypeIdTable {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Hash(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
    new Hash(this.view.buffer.slice(offsets[6], offsets[7]), { validate: false }).validate();
  }

  getCkbSignhash() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCkbMultisig() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getEd25519() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getEth() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getTron() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getDoge() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getWebAuthn() {
    const start = 28;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeDasLockTypeIdTable(value) {
  const buffers = [];
  buffers.push(SerializeHash(value.ckb_signhash));
  buffers.push(SerializeHash(value.ckb_multisig));
  buffers.push(SerializeHash(value.ed25519));
  buffers.push(SerializeHash(value.eth));
  buffers.push(SerializeHash(value.tron));
  buffers.push(SerializeHash(value.doge));
  buffers.push(SerializeHash(value.web_authn));
  return serializeTable(buffers);
}

export class ConfigCellAccount {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint32(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[6], offsets[7]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[7], offsets[8]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[8], offsets[9]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[9], offsets[10]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[10], offsets[11]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[11], offsets[12]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[12], offsets[13]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[13], offsets[14]), { validate: false }).validate();
  }

  getMaxLength() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getBasicCapacity() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getPreparedFeeCapacity() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getExpirationGracePeriod() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRecordMinTtl() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRecordSizeLimit() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getTransferAccountFee() {
    const start = 28;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getEditManagerFee() {
    const start = 32;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getEditRecordsFee() {
    const start = 36;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCommonFee() {
    const start = 40;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getTransferAccountThrottle() {
    const start = 44;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getEditManagerThrottle() {
    const start = 48;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getEditRecordsThrottle() {
    const start = 52;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCommonThrottle() {
    const start = 56;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeConfigCellAccount(value) {
  const buffers = [];
  buffers.push(SerializeUint32(value.max_length));
  buffers.push(SerializeUint64(value.basic_capacity));
  buffers.push(SerializeUint64(value.prepared_fee_capacity));
  buffers.push(SerializeUint32(value.expiration_grace_period));
  buffers.push(SerializeUint32(value.record_min_ttl));
  buffers.push(SerializeUint32(value.record_size_limit));
  buffers.push(SerializeUint64(value.transfer_account_fee));
  buffers.push(SerializeUint64(value.edit_manager_fee));
  buffers.push(SerializeUint64(value.edit_records_fee));
  buffers.push(SerializeUint64(value.common_fee));
  buffers.push(SerializeUint32(value.transfer_account_throttle));
  buffers.push(SerializeUint32(value.edit_manager_throttle));
  buffers.push(SerializeUint32(value.edit_records_throttle));
  buffers.push(SerializeUint32(value.common_throttle));
  return serializeTable(buffers);
}

export class ConfigCellApply {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint32(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
  }

  getApplyMinWaitingBlockNumber() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getApplyMaxWaitingBlockNumber() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeConfigCellApply(value) {
  const buffers = [];
  buffers.push(SerializeUint32(value.apply_min_waiting_block_number));
  buffers.push(SerializeUint32(value.apply_max_waiting_block_number));
  return serializeTable(buffers);
}

export class Chars {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    for (let i = 0; i < offsets.length - 1; i++) {
      new Bytes(this.view.buffer.slice(offsets[i], offsets[i + 1]), { validate: false }).validate();
    }
  }

  length() {
    if (this.view.byteLength < 8) {
      return 0;
    } else {
      return this.view.getUint32(4, true) / 4 - 1;
    }
  }

  indexAt(i) {
    const start = 4 + i * 4;
    const offset = this.view.getUint32(start, true);
    let offset_end = this.view.byteLength;
    if (i + 1 < this.length()) {
      offset_end = this.view.getUint32(start + 4, true);
    }
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeChars(value) {
  return serializeTable(value.map(item => SerializeBytes(item)));
}

export class ConfigCellPrice {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new DiscountConfig(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new PriceConfigList(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
  }

  getDiscount() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new DiscountConfig(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getPrices() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new PriceConfigList(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeConfigCellPrice(value) {
  const buffers = [];
  buffers.push(SerializeDiscountConfig(value.discount));
  buffers.push(SerializePriceConfigList(value.prices));
  return serializeTable(buffers);
}

export class DiscountConfig {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint32(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
  }

  getInvitedDiscount() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeDiscountConfig(value) {
  const buffers = [];
  buffers.push(SerializeUint32(value.invited_discount));
  return serializeTable(buffers);
}

export class PriceConfigList {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    for (let i = 0; i < offsets.length - 1; i++) {
      new PriceConfig(this.view.buffer.slice(offsets[i], offsets[i + 1]), { validate: false }).validate();
    }
  }

  length() {
    if (this.view.byteLength < 8) {
      return 0;
    } else {
      return this.view.getUint32(4, true) / 4 - 1;
    }
  }

  indexAt(i) {
    const start = 4 + i * 4;
    const offset = this.view.getUint32(start, true);
    let offset_end = this.view.byteLength;
    if (i + 1 < this.length()) {
      offset_end = this.view.getUint32(start + 4, true);
    }
    return new PriceConfig(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializePriceConfigList(value) {
  return serializeTable(value.map(item => SerializePriceConfig(item)));
}

export class PriceConfig {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint8(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
  }

  getLength() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint8(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getNew() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRenew() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializePriceConfig(value) {
  const buffers = [];
  buffers.push(SerializeUint8(value.length));
  buffers.push(SerializeUint64(value.new));
  buffers.push(SerializeUint64(value.renew));
  return serializeTable(buffers);
}

export class ConfigCellProposal {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint8(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Uint8(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Uint8(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
  }

  getProposalMinConfirmInterval() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint8(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getProposalMinExtendInterval() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint8(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getProposalMinRecycleInterval() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint8(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getProposalMaxAccountAffect() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getProposalMaxPreAccountContain() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeConfigCellProposal(value) {
  const buffers = [];
  buffers.push(SerializeUint8(value.proposal_min_confirm_interval));
  buffers.push(SerializeUint8(value.proposal_min_extend_interval));
  buffers.push(SerializeUint8(value.proposal_min_recycle_interval));
  buffers.push(SerializeUint32(value.proposal_max_account_affect));
  buffers.push(SerializeUint32(value.proposal_max_pre_account_contain));
  return serializeTable(buffers);
}

export class ConfigCellProfitRate {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint32(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[6], offsets[7]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[7], offsets[8]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[8], offsets[9]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[9], offsets[10]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[10], offsets[11]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[11], offsets[12]), { validate: false }).validate();
  }

  getInviter() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getChannel() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getProposalCreate() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getProposalConfirm() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getIncomeConsolidate() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getSaleBuyerInviter() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getSaleBuyerChannel() {
    const start = 28;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getSaleDas() {
    const start = 32;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAuctionBidderInviter() {
    const start = 36;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAuctionBidderChannel() {
    const start = 40;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAuctionDas() {
    const start = 44;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAuctionPrevBidder() {
    const start = 48;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeConfigCellProfitRate(value) {
  const buffers = [];
  buffers.push(SerializeUint32(value.inviter));
  buffers.push(SerializeUint32(value.channel));
  buffers.push(SerializeUint32(value.proposal_create));
  buffers.push(SerializeUint32(value.proposal_confirm));
  buffers.push(SerializeUint32(value.income_consolidate));
  buffers.push(SerializeUint32(value.sale_buyer_inviter));
  buffers.push(SerializeUint32(value.sale_buyer_channel));
  buffers.push(SerializeUint32(value.sale_das));
  buffers.push(SerializeUint32(value.auction_bidder_inviter));
  buffers.push(SerializeUint32(value.auction_bidder_channel));
  buffers.push(SerializeUint32(value.auction_das));
  buffers.push(SerializeUint32(value.auction_prev_bidder));
  return serializeTable(buffers);
}

export class ConfigCellIncome {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint64(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
  }

  getBasicCapacity() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getMaxRecords() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getMinTransferCapacity() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeConfigCellIncome(value) {
  const buffers = [];
  buffers.push(SerializeUint64(value.basic_capacity));
  buffers.push(SerializeUint32(value.max_records));
  buffers.push(SerializeUint64(value.min_transfer_capacity));
  return serializeTable(buffers);
}

export class ConfigCellRelease {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint32(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
  }

  getLuckyNumber() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeConfigCellRelease(value) {
  const buffers = [];
  buffers.push(SerializeUint32(value.lucky_number));
  return serializeTable(buffers);
}

export class ConfigCellSecondaryMarket {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint64(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[6], offsets[7]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[7], offsets[8]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[8], offsets[9]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[9], offsets[10]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[10], offsets[11]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[11], offsets[12]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[12], offsets[13]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[13], offsets[14]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[14], offsets[15]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[15], offsets[16]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[16], offsets[17]), { validate: false }).validate();
  }

  getCommonFee() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getSaleMinPrice() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getSaleExpirationLimit() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getSaleDescriptionBytesLimit() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getSaleCellBasicCapacity() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getSaleCellPreparedFeeCapacity() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAuctionMaxExtendableDuration() {
    const start = 28;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAuctionDurationIncrementEachBid() {
    const start = 32;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAuctionMinOpeningPrice() {
    const start = 36;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAuctionMinIncrementRateEachBid() {
    const start = 40;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAuctionDescriptionBytesLimit() {
    const start = 44;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAuctionCellBasicCapacity() {
    const start = 48;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAuctionCellPreparedFeeCapacity() {
    const start = 52;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getOfferMinPrice() {
    const start = 56;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getOfferCellBasicCapacity() {
    const start = 60;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getOfferCellPreparedFeeCapacity() {
    const start = 64;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getOfferMessageBytesLimit() {
    const start = 68;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeConfigCellSecondaryMarket(value) {
  const buffers = [];
  buffers.push(SerializeUint64(value.common_fee));
  buffers.push(SerializeUint64(value.sale_min_price));
  buffers.push(SerializeUint32(value.sale_expiration_limit));
  buffers.push(SerializeUint32(value.sale_description_bytes_limit));
  buffers.push(SerializeUint64(value.sale_cell_basic_capacity));
  buffers.push(SerializeUint64(value.sale_cell_prepared_fee_capacity));
  buffers.push(SerializeUint32(value.auction_max_extendable_duration));
  buffers.push(SerializeUint32(value.auction_duration_increment_each_bid));
  buffers.push(SerializeUint64(value.auction_min_opening_price));
  buffers.push(SerializeUint32(value.auction_min_increment_rate_each_bid));
  buffers.push(SerializeUint32(value.auction_description_bytes_limit));
  buffers.push(SerializeUint64(value.auction_cell_basic_capacity));
  buffers.push(SerializeUint64(value.auction_cell_prepared_fee_capacity));
  buffers.push(SerializeUint64(value.offer_min_price));
  buffers.push(SerializeUint64(value.offer_cell_basic_capacity));
  buffers.push(SerializeUint64(value.offer_cell_prepared_fee_capacity));
  buffers.push(SerializeUint32(value.offer_message_bytes_limit));
  return serializeTable(buffers);
}

export class ConfigCellReverseResolution {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint64(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
  }

  getRecordBasicCapacity() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRecordPreparedFeeCapacity() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCommonFee() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeConfigCellReverseResolution(value) {
  const buffers = [];
  buffers.push(SerializeUint64(value.record_basic_capacity));
  buffers.push(SerializeUint64(value.record_prepared_fee_capacity));
  buffers.push(SerializeUint64(value.common_fee));
  return serializeTable(buffers);
}

export class ConfigCellSubAccount {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint64(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[6], offsets[7]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[7], offsets[8]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[8], offsets[9]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[9], offsets[10]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[10], offsets[11]), { validate: false }).validate();
  }

  getBasicCapacity() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getPreparedFeeCapacity() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getNewSubAccountPrice() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRenewSubAccountPrice() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCommonFee() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCreateFee() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getEditFee() {
    const start = 28;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRenewFee() {
    const start = 32;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRecycleFee() {
    const start = 36;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getNewSubAccountCustomPriceDasProfitRate() {
    const start = 40;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRenewSubAccountCustomPriceDasProfitRate() {
    const start = 44;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeConfigCellSubAccount(value) {
  const buffers = [];
  buffers.push(SerializeUint64(value.basic_capacity));
  buffers.push(SerializeUint64(value.prepared_fee_capacity));
  buffers.push(SerializeUint64(value.new_sub_account_price));
  buffers.push(SerializeUint64(value.renew_sub_account_price));
  buffers.push(SerializeUint64(value.common_fee));
  buffers.push(SerializeUint64(value.create_fee));
  buffers.push(SerializeUint64(value.edit_fee));
  buffers.push(SerializeUint64(value.renew_fee));
  buffers.push(SerializeUint64(value.recycle_fee));
  buffers.push(SerializeUint32(value.new_sub_account_custom_price_das_profit_rate));
  buffers.push(SerializeUint32(value.renew_sub_account_custom_price_das_profit_rate));
  return serializeTable(buffers);
}

export class ConfigCellSystemStatus {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new ContractStatus(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new ContractStatus(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new ContractStatus(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new ContractStatus(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new ContractStatus(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new ContractStatus(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
    new ContractStatus(this.view.buffer.slice(offsets[6], offsets[7]), { validate: false }).validate();
    new ContractStatus(this.view.buffer.slice(offsets[7], offsets[8]), { validate: false }).validate();
    new ContractStatus(this.view.buffer.slice(offsets[8], offsets[9]), { validate: false }).validate();
    new ContractStatus(this.view.buffer.slice(offsets[9], offsets[10]), { validate: false }).validate();
    new ContractStatus(this.view.buffer.slice(offsets[10], offsets[11]), { validate: false }).validate();
    new ContractStatus(this.view.buffer.slice(offsets[11], offsets[12]), { validate: false }).validate();
    new ContractStatus(this.view.buffer.slice(offsets[12], offsets[13]), { validate: false }).validate();
  }

  getApplyRegisterCellType() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ContractStatus(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getPreAccountCellType() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ContractStatus(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getProposalCellType() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ContractStatus(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getConfigCellType() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ContractStatus(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAccountCellType() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ContractStatus(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAccountSaleCellType() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ContractStatus(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getSubAccountCellType() {
    const start = 28;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ContractStatus(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getOfferCellType() {
    const start = 32;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ContractStatus(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getBalanceCellType() {
    const start = 36;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ContractStatus(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getIncomeCellType() {
    const start = 40;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ContractStatus(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getReverseRecordCellType() {
    const start = 44;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ContractStatus(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getReverseRecordRootCellType() {
    const start = 48;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ContractStatus(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getEip712Lib() {
    const start = 52;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new ContractStatus(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeConfigCellSystemStatus(value) {
  const buffers = [];
  buffers.push(SerializeContractStatus(value.apply_register_cell_type));
  buffers.push(SerializeContractStatus(value.pre_account_cell_type));
  buffers.push(SerializeContractStatus(value.proposal_cell_type));
  buffers.push(SerializeContractStatus(value.config_cell_type));
  buffers.push(SerializeContractStatus(value.account_cell_type));
  buffers.push(SerializeContractStatus(value.account_sale_cell_type));
  buffers.push(SerializeContractStatus(value.sub_account_cell_type));
  buffers.push(SerializeContractStatus(value.offer_cell_type));
  buffers.push(SerializeContractStatus(value.balance_cell_type));
  buffers.push(SerializeContractStatus(value.income_cell_type));
  buffers.push(SerializeContractStatus(value.reverse_record_cell_type));
  buffers.push(SerializeContractStatus(value.reverse_record_root_cell_type));
  buffers.push(SerializeContractStatus(value.eip712_lib));
  return serializeTable(buffers);
}

export class ContractStatus {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    if (offsets[1] - offsets[0] !== 1) {
      throw new Error(`Invalid offset for status: ${offsets[0]} - ${offsets[1]}`)
    }
    new Bytes(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
  }

  getStatus() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new DataView(this.view.buffer.slice(offset, offset_end)).getUint8(0);
  }

  getVersion() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeContractStatus(value) {
  const buffers = [];
  const statusView = new DataView(new ArrayBuffer(1));
  statusView.setUint8(0, value.status);
  buffers.push(statusView.buffer)
  buffers.push(SerializeBytes(value.version));
  return serializeTable(buffers);
}

export class ProposalCellData {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Script(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new SliceList(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
  }

  getProposerLock() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Script(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCreatedAtHeight() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getSlices() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new SliceList(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeProposalCellData(value) {
  const buffers = [];
  buffers.push(SerializeScript(value.proposer_lock));
  buffers.push(SerializeUint64(value.created_at_height));
  buffers.push(SerializeSliceList(value.slices));
  return serializeTable(buffers);
}

export class SliceList {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    for (let i = 0; i < offsets.length - 1; i++) {
      new SL(this.view.buffer.slice(offsets[i], offsets[i + 1]), { validate: false }).validate();
    }
  }

  length() {
    if (this.view.byteLength < 8) {
      return 0;
    } else {
      return this.view.getUint32(4, true) / 4 - 1;
    }
  }

  indexAt(i) {
    const start = 4 + i * 4;
    const offset = this.view.getUint32(start, true);
    let offset_end = this.view.byteLength;
    if (i + 1 < this.length()) {
      offset_end = this.view.getUint32(start + 4, true);
    }
    return new SL(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeSliceList(value) {
  return serializeTable(value.map(item => SerializeSL(item)));
}

export class SL {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    for (let i = 0; i < offsets.length - 1; i++) {
      new ProposalItem(this.view.buffer.slice(offsets[i], offsets[i + 1]), { validate: false }).validate();
    }
  }

  length() {
    if (this.view.byteLength < 8) {
      return 0;
    } else {
      return this.view.getUint32(4, true) / 4 - 1;
    }
  }

  indexAt(i) {
    const start = 4 + i * 4;
    const offset = this.view.getUint32(start, true);
    let offset_end = this.view.byteLength;
    if (i + 1 < this.length()) {
      offset_end = this.view.getUint32(start + 4, true);
    }
    return new ProposalItem(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeSL(value) {
  return serializeTable(value.map(item => SerializeProposalItem(item)));
}

export class ProposalItem {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new AccountId(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Uint8(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new AccountId(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
  }

  getAccountId() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new AccountId(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getItemType() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint8(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getNext() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new AccountId(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeProposalItem(value) {
  const buffers = [];
  buffers.push(SerializeAccountId(value.account_id));
  buffers.push(SerializeUint8(value.item_type));
  buffers.push(SerializeAccountId(value.next));
  return serializeTable(buffers);
}

export class IncomeCellData {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Script(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new IncomeRecords(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
  }

  getCreator() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Script(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRecords() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new IncomeRecords(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeIncomeCellData(value) {
  const buffers = [];
  buffers.push(SerializeScript(value.creator));
  buffers.push(SerializeIncomeRecords(value.records));
  return serializeTable(buffers);
}

export class IncomeRecords {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    for (let i = 0; i < offsets.length - 1; i++) {
      new IncomeRecord(this.view.buffer.slice(offsets[i], offsets[i + 1]), { validate: false }).validate();
    }
  }

  length() {
    if (this.view.byteLength < 8) {
      return 0;
    } else {
      return this.view.getUint32(4, true) / 4 - 1;
    }
  }

  indexAt(i) {
    const start = 4 + i * 4;
    const offset = this.view.getUint32(start, true);
    let offset_end = this.view.byteLength;
    if (i + 1 < this.length()) {
      offset_end = this.view.getUint32(start + 4, true);
    }
    return new IncomeRecord(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeIncomeRecords(value) {
  return serializeTable(value.map(item => SerializeIncomeRecord(item)));
}

export class IncomeRecord {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Script(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
  }

  getBelongTo() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Script(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCapacity() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeIncomeRecord(value) {
  const buffers = [];
  buffers.push(SerializeScript(value.belong_to));
  buffers.push(SerializeUint64(value.capacity));
  return serializeTable(buffers);
}

export class AccountCellDataV2 {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new AccountId(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new AccountChars(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
    new Uint8(this.view.buffer.slice(offsets[6], offsets[7]), { validate: false }).validate();
    new Records(this.view.buffer.slice(offsets[7], offsets[8]), { validate: false }).validate();
  }

  getId() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new AccountId(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAccount() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new AccountChars(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRegisteredAt() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getLastTransferAccountAt() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getLastEditManagerAt() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getLastEditRecordsAt() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getStatus() {
    const start = 28;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint8(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRecords() {
    const start = 32;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Records(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeAccountCellDataV2(value) {
  const buffers = [];
  buffers.push(SerializeAccountId(value.id));
  buffers.push(SerializeAccountChars(value.account));
  buffers.push(SerializeUint64(value.registered_at));
  buffers.push(SerializeUint64(value.last_transfer_account_at));
  buffers.push(SerializeUint64(value.last_edit_manager_at));
  buffers.push(SerializeUint64(value.last_edit_records_at));
  buffers.push(SerializeUint8(value.status));
  buffers.push(SerializeRecords(value.records));
  return serializeTable(buffers);
}

export class AccountCellData {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new AccountId(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new AccountChars(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
    new Uint8(this.view.buffer.slice(offsets[6], offsets[7]), { validate: false }).validate();
    new Records(this.view.buffer.slice(offsets[7], offsets[8]), { validate: false }).validate();
    new Uint8(this.view.buffer.slice(offsets[8], offsets[9]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[9], offsets[10]), { validate: false }).validate();
  }

  getId() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new AccountId(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAccount() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new AccountChars(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRegisteredAt() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getLastTransferAccountAt() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getLastEditManagerAt() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getLastEditRecordsAt() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getStatus() {
    const start = 28;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint8(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRecords() {
    const start = 32;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Records(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getEnableSubAccount() {
    const start = 36;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint8(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRenewSubAccountPrice() {
    const start = 40;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeAccountCellData(value) {
  const buffers = [];
  buffers.push(SerializeAccountId(value.id));
  buffers.push(SerializeAccountChars(value.account));
  buffers.push(SerializeUint64(value.registered_at));
  buffers.push(SerializeUint64(value.last_transfer_account_at));
  buffers.push(SerializeUint64(value.last_edit_manager_at));
  buffers.push(SerializeUint64(value.last_edit_records_at));
  buffers.push(SerializeUint8(value.status));
  buffers.push(SerializeRecords(value.records));
  buffers.push(SerializeUint8(value.enable_sub_account));
  buffers.push(SerializeUint64(value.renew_sub_account_price));
  return serializeTable(buffers);
}

export class AccountId {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    assertDataLength(this.view.byteLength, 20);
  }

  indexAt(i) {
    return this.view.getUint8(i);
  }

  raw() {
    return this.view.buffer;
  }

  static size() {
    return 20;
  }
}

export function SerializeAccountId(value) {
  const buffer = assertArrayBuffer(value);
  assertDataLength(buffer.byteLength, 20);
  return buffer;
}

export class Record {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Bytes(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
  }

  getRecordType() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRecordKey() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRecordLabel() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRecordValue() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRecordTtl() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeRecord(value) {
  const buffers = [];
  buffers.push(SerializeBytes(value.record_type));
  buffers.push(SerializeBytes(value.record_key));
  buffers.push(SerializeBytes(value.record_label));
  buffers.push(SerializeBytes(value.record_value));
  buffers.push(SerializeUint32(value.record_ttl));
  return serializeTable(buffers);
}

export class Records {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    for (let i = 0; i < offsets.length - 1; i++) {
      new Record(this.view.buffer.slice(offsets[i], offsets[i + 1]), { validate: false }).validate();
    }
  }

  length() {
    if (this.view.byteLength < 8) {
      return 0;
    } else {
      return this.view.getUint32(4, true) / 4 - 1;
    }
  }

  indexAt(i) {
    const start = 4 + i * 4;
    const offset = this.view.getUint32(start, true);
    let offset_end = this.view.byteLength;
    if (i + 1 < this.length()) {
      offset_end = this.view.getUint32(start + 4, true);
    }
    return new Record(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeRecords(value) {
  return serializeTable(value.map(item => SerializeRecord(item)));
}

export class AccountSaleCellDataV1 {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new AccountId(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
  }

  getAccountId() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new AccountId(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAccount() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getPrice() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getDescription() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getStartedAt() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeAccountSaleCellDataV1(value) {
  const buffers = [];
  buffers.push(SerializeAccountId(value.account_id));
  buffers.push(SerializeBytes(value.account));
  buffers.push(SerializeUint64(value.price));
  buffers.push(SerializeBytes(value.description));
  buffers.push(SerializeUint64(value.started_at));
  return serializeTable(buffers);
}

export class AccountSaleCellData {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new AccountId(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
  }

  getAccountId() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new AccountId(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAccount() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getPrice() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getDescription() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getStartedAt() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getBuyerInviterProfitRate() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeAccountSaleCellData(value) {
  const buffers = [];
  buffers.push(SerializeAccountId(value.account_id));
  buffers.push(SerializeBytes(value.account));
  buffers.push(SerializeUint64(value.price));
  buffers.push(SerializeBytes(value.description));
  buffers.push(SerializeUint64(value.started_at));
  buffers.push(SerializeUint32(value.buyer_inviter_profit_rate));
  return serializeTable(buffers);
}

export class AccountAuctionCellData {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new AccountId(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[6], offsets[7]), { validate: false }).validate();
    new Script(this.view.buffer.slice(offsets[7], offsets[8]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[8], offsets[9]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[9], offsets[10]), { validate: false }).validate();
  }

  getAccountId() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new AccountId(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAccount() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getDescription() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getOpeningPrice() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getIncrementRateEachBid() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getStartedAt() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getEndedAt() {
    const start = 28;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCurrentBidderLock() {
    const start = 32;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Script(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCurrentBidPrice() {
    const start = 36;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getPrevBidderProfitRate() {
    const start = 40;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeAccountAuctionCellData(value) {
  const buffers = [];
  buffers.push(SerializeAccountId(value.account_id));
  buffers.push(SerializeBytes(value.account));
  buffers.push(SerializeBytes(value.description));
  buffers.push(SerializeUint64(value.opening_price));
  buffers.push(SerializeUint32(value.increment_rate_each_bid));
  buffers.push(SerializeUint64(value.started_at));
  buffers.push(SerializeUint64(value.ended_at));
  buffers.push(SerializeScript(value.current_bidder_lock));
  buffers.push(SerializeUint64(value.current_bid_price));
  buffers.push(SerializeUint32(value.prev_bidder_profit_rate));
  return serializeTable(buffers);
}

export class PreAccountCellDataV1 {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new AccountChars(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Script(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new ScriptOpt(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new ScriptOpt(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
    new PriceConfig(this.view.buffer.slice(offsets[6], offsets[7]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[7], offsets[8]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[8], offsets[9]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[9], offsets[10]), { validate: false }).validate();
  }

  getAccount() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new AccountChars(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRefundLock() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Script(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getOwnerLockArgs() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getInviterId() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getInviterLock() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ScriptOpt(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getChannelLock() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ScriptOpt(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getPrice() {
    const start = 28;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new PriceConfig(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getQuote() {
    const start = 32;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getInvitedDiscount() {
    const start = 36;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCreatedAt() {
    const start = 40;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializePreAccountCellDataV1(value) {
  const buffers = [];
  buffers.push(SerializeAccountChars(value.account));
  buffers.push(SerializeScript(value.refund_lock));
  buffers.push(SerializeBytes(value.owner_lock_args));
  buffers.push(SerializeBytes(value.inviter_id));
  buffers.push(SerializeScriptOpt(value.inviter_lock));
  buffers.push(SerializeScriptOpt(value.channel_lock));
  buffers.push(SerializePriceConfig(value.price));
  buffers.push(SerializeUint64(value.quote));
  buffers.push(SerializeUint32(value.invited_discount));
  buffers.push(SerializeUint64(value.created_at));
  return serializeTable(buffers);
}

export class PreAccountCellDataV2 {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new AccountChars(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Script(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new ScriptOpt(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new ScriptOpt(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
    new PriceConfig(this.view.buffer.slice(offsets[6], offsets[7]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[7], offsets[8]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[8], offsets[9]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[9], offsets[10]), { validate: false }).validate();
    new Records(this.view.buffer.slice(offsets[10], offsets[11]), { validate: false }).validate();
  }

  getAccount() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new AccountChars(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRefundLock() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Script(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getOwnerLockArgs() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getInviterId() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getInviterLock() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ScriptOpt(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getChannelLock() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ScriptOpt(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getPrice() {
    const start = 28;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new PriceConfig(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getQuote() {
    const start = 32;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getInvitedDiscount() {
    const start = 36;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCreatedAt() {
    const start = 40;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getInitialRecords() {
    const start = 44;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Records(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializePreAccountCellDataV2(value) {
  const buffers = [];
  buffers.push(SerializeAccountChars(value.account));
  buffers.push(SerializeScript(value.refund_lock));
  buffers.push(SerializeBytes(value.owner_lock_args));
  buffers.push(SerializeBytes(value.inviter_id));
  buffers.push(SerializeScriptOpt(value.inviter_lock));
  buffers.push(SerializeScriptOpt(value.channel_lock));
  buffers.push(SerializePriceConfig(value.price));
  buffers.push(SerializeUint64(value.quote));
  buffers.push(SerializeUint32(value.invited_discount));
  buffers.push(SerializeUint64(value.created_at));
  buffers.push(SerializeRecords(value.initial_records));
  return serializeTable(buffers);
}

export class PreAccountCellData {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new AccountChars(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Script(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new ScriptOpt(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new ScriptOpt(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
    new PriceConfig(this.view.buffer.slice(offsets[6], offsets[7]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[7], offsets[8]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[8], offsets[9]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[9], offsets[10]), { validate: false }).validate();
    new Records(this.view.buffer.slice(offsets[10], offsets[11]), { validate: false }).validate();
    new ChainId(this.view.buffer.slice(offsets[11], offsets[12]), { validate: false }).validate();
  }

  getAccount() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new AccountChars(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRefundLock() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Script(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getOwnerLockArgs() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getInviterId() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getInviterLock() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ScriptOpt(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getChannelLock() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ScriptOpt(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getPrice() {
    const start = 28;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new PriceConfig(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getQuote() {
    const start = 32;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getInvitedDiscount() {
    const start = 36;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCreatedAt() {
    const start = 40;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getInitialRecords() {
    const start = 44;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Records(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getInitialCrossChain() {
    const start = 48;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new ChainId(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializePreAccountCellData(value) {
  const buffers = [];
  buffers.push(SerializeAccountChars(value.account));
  buffers.push(SerializeScript(value.refund_lock));
  buffers.push(SerializeBytes(value.owner_lock_args));
  buffers.push(SerializeBytes(value.inviter_id));
  buffers.push(SerializeScriptOpt(value.inviter_lock));
  buffers.push(SerializeScriptOpt(value.channel_lock));
  buffers.push(SerializePriceConfig(value.price));
  buffers.push(SerializeUint64(value.quote));
  buffers.push(SerializeUint32(value.invited_discount));
  buffers.push(SerializeUint64(value.created_at));
  buffers.push(SerializeRecords(value.initial_records));
  buffers.push(SerializeChainId(value.initial_cross_chain));
  return serializeTable(buffers);
}

export class ChainId {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint8(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
  }

  getChecked() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint8(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getCoinType() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getChainId() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeChainId(value) {
  const buffers = [];
  buffers.push(SerializeUint8(value.checked));
  buffers.push(SerializeUint64(value.coin_type));
  buffers.push(SerializeUint64(value.chain_id));
  return serializeTable(buffers);
}

export class AccountChars {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    for (let i = 0; i < offsets.length - 1; i++) {
      new AccountChar(this.view.buffer.slice(offsets[i], offsets[i + 1]), { validate: false }).validate();
    }
  }

  length() {
    if (this.view.byteLength < 8) {
      return 0;
    } else {
      return this.view.getUint32(4, true) / 4 - 1;
    }
  }

  indexAt(i) {
    const start = 4 + i * 4;
    const offset = this.view.getUint32(start, true);
    let offset_end = this.view.byteLength;
    if (i + 1 < this.length()) {
      offset_end = this.view.getUint32(start + 4, true);
    }
    return new AccountChar(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeAccountChars(value) {
  return serializeTable(value.map(item => SerializeAccountChar(item)));
}

export class AccountChar {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint32(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
  }

  getCharSetName() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getBytes() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeAccountChar(value) {
  const buffers = [];
  buffers.push(SerializeUint32(value.char_set_name));
  buffers.push(SerializeBytes(value.bytes));
  return serializeTable(buffers);
}

export class OfferCellData {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Bytes(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Script(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new Script(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
  }

  getAccount() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getPrice() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getMessage() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getInviterLock() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Script(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getChannelLock() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Script(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeOfferCellData(value) {
  const buffers = [];
  buffers.push(SerializeBytes(value.account));
  buffers.push(SerializeUint64(value.price));
  buffers.push(SerializeBytes(value.message));
  buffers.push(SerializeScript(value.inviter_lock));
  buffers.push(SerializeScript(value.channel_lock));
  return serializeTable(buffers);
}

export class SubAccount {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Script(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new AccountId(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new AccountChars(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
    new Uint8(this.view.buffer.slice(offsets[6], offsets[7]), { validate: false }).validate();
    new Records(this.view.buffer.slice(offsets[7], offsets[8]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[8], offsets[9]), { validate: false }).validate();
    new Uint8(this.view.buffer.slice(offsets[9], offsets[10]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[10], offsets[11]), { validate: false }).validate();
  }

  getLock() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Script(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getId() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new AccountId(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAccount() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new AccountChars(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getSuffix() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRegisteredAt() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getExpiredAt() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getStatus() {
    const start = 28;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint8(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRecords() {
    const start = 32;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Records(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getNonce() {
    const start = 36;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getEnableSubAccount() {
    const start = 40;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint8(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRenewSubAccountPrice() {
    const start = 44;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeSubAccount(value) {
  const buffers = [];
  buffers.push(SerializeScript(value.lock));
  buffers.push(SerializeAccountId(value.id));
  buffers.push(SerializeAccountChars(value.account));
  buffers.push(SerializeBytes(value.suffix));
  buffers.push(SerializeUint64(value.registered_at));
  buffers.push(SerializeUint64(value.expired_at));
  buffers.push(SerializeUint8(value.status));
  buffers.push(SerializeRecords(value.records));
  buffers.push(SerializeUint64(value.nonce));
  buffers.push(SerializeUint8(value.enable_sub_account));
  buffers.push(SerializeUint64(value.renew_sub_account_price));
  return serializeTable(buffers);
}

export class SubAccountRule {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint32(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
    new Uint64(this.view.buffer.slice(offsets[3], offsets[4]), { validate: false }).validate();
    new ASTExpression(this.view.buffer.slice(offsets[4], offsets[5]), { validate: false }).validate();
    new Uint8(this.view.buffer.slice(offsets[5], offsets[6]), { validate: false }).validate();
  }

  getIndex() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getName() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getNote() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getPrice() {
    const start = 16;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint64(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getAst() {
    const start = 20;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new ASTExpression(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getStatus() {
    const start = 24;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Uint8(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeSubAccountRule(value) {
  const buffers = [];
  buffers.push(SerializeUint32(value.index));
  buffers.push(SerializeBytes(value.name));
  buffers.push(SerializeBytes(value.note));
  buffers.push(SerializeUint64(value.price));
  buffers.push(SerializeASTExpression(value.ast));
  buffers.push(SerializeUint8(value.status));
  return serializeTable(buffers);
}

export class SubAccountRules {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    for (let i = 0; i < offsets.length - 1; i++) {
      new SubAccountRule(this.view.buffer.slice(offsets[i], offsets[i + 1]), { validate: false }).validate();
    }
  }

  length() {
    if (this.view.byteLength < 8) {
      return 0;
    } else {
      return this.view.getUint32(4, true) / 4 - 1;
    }
  }

  indexAt(i) {
    const start = 4 + i * 4;
    const offset = this.view.getUint32(start, true);
    let offset_end = this.view.byteLength;
    if (i + 1 < this.length()) {
      offset_end = this.view.getUint32(start + 4, true);
    }
    return new SubAccountRule(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeSubAccountRules(value) {
  return serializeTable(value.map(item => SerializeSubAccountRule(item)));
}

export class ASTExpression {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    if (offsets[1] - offsets[0] !== 1) {
      throw new Error(`Invalid offset for expression_type: ${offsets[0]} - ${offsets[1]}`)
    }
    new Bytes(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
  }

  getExpressionType() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new DataView(this.view.buffer.slice(offset, offset_end)).getUint8(0);
  }

  getExpression() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeASTExpression(value) {
  const buffers = [];
  const expressionTypeView = new DataView(new ArrayBuffer(1));
  expressionTypeView.setUint8(0, value.expression_type);
  buffers.push(expressionTypeView.buffer)
  buffers.push(SerializeBytes(value.expression));
  return serializeTable(buffers);
}

export class ASTExpressions {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    for (let i = 0; i < offsets.length - 1; i++) {
      new ASTExpression(this.view.buffer.slice(offsets[i], offsets[i + 1]), { validate: false }).validate();
    }
  }

  length() {
    if (this.view.byteLength < 8) {
      return 0;
    } else {
      return this.view.getUint32(4, true) / 4 - 1;
    }
  }

  indexAt(i) {
    const start = 4 + i * 4;
    const offset = this.view.getUint32(start, true);
    let offset_end = this.view.byteLength;
    if (i + 1 < this.length()) {
      offset_end = this.view.getUint32(start + 4, true);
    }
    return new ASTExpression(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeASTExpressions(value) {
  return serializeTable(value.map(item => SerializeASTExpression(item)));
}

export class ASTOperator {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    if (offsets[1] - offsets[0] !== 1) {
      throw new Error(`Invalid offset for symbol: ${offsets[0]} - ${offsets[1]}`)
    }
    new ASTExpressions(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
  }

  getSymbol() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new DataView(this.view.buffer.slice(offset, offset_end)).getUint8(0);
  }

  getExpressions() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new ASTExpressions(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeASTOperator(value) {
  const buffers = [];
  const symbolView = new DataView(new ArrayBuffer(1));
  symbolView.setUint8(0, value.symbol);
  buffers.push(symbolView.buffer)
  buffers.push(SerializeASTExpressions(value.expressions));
  return serializeTable(buffers);
}

export class ASTFunction {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    if (offsets[1] - offsets[0] !== 1) {
      throw new Error(`Invalid offset for name: ${offsets[0]} - ${offsets[1]}`)
    }
    new ASTExpressions(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
  }

  getName() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new DataView(this.view.buffer.slice(offset, offset_end)).getUint8(0);
  }

  getArguments() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new ASTExpressions(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeASTFunction(value) {
  const buffers = [];
  const nameView = new DataView(new ArrayBuffer(1));
  nameView.setUint8(0, value.name);
  buffers.push(nameView.buffer)
  buffers.push(SerializeASTExpressions(value.arguments));
  return serializeTable(buffers);
}

export class ASTVariable {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    if (offsets[1] - offsets[0] !== 1) {
      throw new Error(`Invalid offset for name: ${offsets[0]} - ${offsets[1]}`)
    }
  }

  getName() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new DataView(this.view.buffer.slice(offset, offset_end)).getUint8(0);
  }
}

export function SerializeASTVariable(value) {
  const buffers = [];
  const nameView = new DataView(new ArrayBuffer(1));
  nameView.setUint8(0, value.name);
  buffers.push(nameView.buffer)
  return serializeTable(buffers);
}

export class ASTValue {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    if (offsets[1] - offsets[0] !== 1) {
      throw new Error(`Invalid offset for value_type: ${offsets[0]} - ${offsets[1]}`)
    }
    new Bytes(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
  }

  getValueType() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new DataView(this.view.buffer.slice(offset, offset_end)).getUint8(0);
  }

  getValue() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeASTValue(value) {
  const buffers = [];
  const valueTypeView = new DataView(new ArrayBuffer(1));
  valueTypeView.setUint8(0, value.value_type);
  buffers.push(valueTypeView.buffer)
  buffers.push(SerializeBytes(value.value));
  return serializeTable(buffers);
}

export class DeviceKey {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  getMainAlgId() {
    return new Uint8(this.view.buffer.slice(0, 0 + Uint8.size()), { validate: false });
  }

  getSubAlgId() {
    return new Uint8(this.view.buffer.slice(0 + Uint8.size(), 0 + Uint8.size() + Uint8.size()), { validate: false });
  }

  getCid() {
    return new Byte10(this.view.buffer.slice(0 + Uint8.size() + Uint8.size(), 0 + Uint8.size() + Uint8.size() + Byte10.size()), { validate: false });
  }

  getPubkey() {
    return new Byte10(this.view.buffer.slice(0 + Uint8.size() + Uint8.size() + Byte10.size(), 0 + Uint8.size() + Uint8.size() + Byte10.size() + Byte10.size()), { validate: false });
  }

  validate(compatible = false) {
    assertDataLength(this.view.byteLength, DeviceKey.size());
    this.getMainAlgId().validate(compatible);
    this.getSubAlgId().validate(compatible);
    this.getCid().validate(compatible);
    this.getPubkey().validate(compatible);
  }
  static size() {
    return 0 + Uint8.size() + Uint8.size() + Byte10.size() + Byte10.size();
  }
}

export function SerializeDeviceKey(value) {
  const array = new Uint8Array(0 + Uint8.size() + Uint8.size() + Byte10.size() + Byte10.size());
  const view = new DataView(array.buffer);
  array.set(new Uint8Array(SerializeUint8(value.main_alg_id)), 0);
  array.set(new Uint8Array(SerializeUint8(value.sub_alg_id)), 0 + Uint8.size());
  array.set(new Uint8Array(SerializeByte10(value.cid)), 0 + Uint8.size() + Uint8.size());
  array.set(new Uint8Array(SerializeByte10(value.pubkey)), 0 + Uint8.size() + Uint8.size() + Byte10.size());
  return array.buffer;
}

export class DeviceKeyList {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    if (this.view.byteLength < 4) {
      dataLengthError(this.view.byteLength, ">4");
    }
    const requiredByteLength = this.length() * DeviceKey.size() + 4;
    assertDataLength(this.view.byteLength, requiredByteLength);
    for (let i = 0; i < 0; i++) {
      const item = this.indexAt(i);
      item.validate(compatible);
    }
  }

  indexAt(i) {
    return new DeviceKey(this.view.buffer.slice(4 + i * DeviceKey.size(), 4 + (i + 1) * DeviceKey.size()), { validate: false });
  }

  length() {
    return this.view.getUint32(0, true);
  }
}

export function SerializeDeviceKeyList(value) {
  const array = new Uint8Array(4 + DeviceKey.size() * value.length);
  (new DataView(array.buffer)).setUint32(0, value.length, true);
  for (let i = 0; i < value.length; i++) {
    const itemBuffer = SerializeDeviceKey(value[i]);
    array.set(new Uint8Array(itemBuffer), 4 + i * DeviceKey.size());
  }
  return array.buffer;
}

export class DeviceKeyListCellData {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new DeviceKeyList(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Script(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
  }

  getKeys() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new DeviceKeyList(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getRefundLock() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Script(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeDeviceKeyListCellData(value) {
  const buffers = [];
  buffers.push(SerializeDeviceKeyList(value.keys));
  buffers.push(SerializeScript(value.refund_lock));
  return serializeTable(buffers);
}

export class Uint8 {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    assertDataLength(this.view.byteLength, 1);
  }

  indexAt(i) {
    return this.view.getUint8(i);
  }

  raw() {
    return this.view.buffer;
  }

  static size() {
    return 1;
  }
}

export function SerializeUint8(value) {
  const buffer = assertArrayBuffer(value);
  assertDataLength(buffer.byteLength, 1);
  return buffer;
}

export class Uint32 {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    assertDataLength(this.view.byteLength, 4);
  }

  indexAt(i) {
    return this.view.getUint8(i);
  }

  raw() {
    return this.view.buffer;
  }

  toBigEndianUint32() {
    return this.view.getUint32(0, false);
  }

  toLittleEndianUint32() {
    return this.view.getUint32(0, true);
  }

  static size() {
    return 4;
  }
}

export function SerializeUint32(value) {
  const buffer = assertArrayBuffer(value);
  assertDataLength(buffer.byteLength, 4);
  return buffer;
}

export class Uint64 {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    assertDataLength(this.view.byteLength, 8);
  }

  indexAt(i) {
    return this.view.getUint8(i);
  }

  raw() {
    return this.view.buffer;
  }

  toBigEndianBigUint64() {
    return this.view.getBigUint64(0, false);
  }

  toLittleEndianBigUint64() {
    return this.view.getBigUint64(0, true);
  }

  static size() {
    return 8;
  }
}

export function SerializeUint64(value) {
  const buffer = assertArrayBuffer(value);
  assertDataLength(buffer.byteLength, 8);
  return buffer;
}

export class Byte10 {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    assertDataLength(this.view.byteLength, 10);
  }

  indexAt(i) {
    return this.view.getUint8(i);
  }

  raw() {
    return this.view.buffer;
  }

  static size() {
    return 10;
  }
}

export function SerializeByte10(value) {
  const buffer = assertArrayBuffer(value);
  assertDataLength(buffer.byteLength, 10);
  return buffer;
}

export class Bytes {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    if (this.view.byteLength < 4) {
      dataLengthError(this.view.byteLength, ">4")
    }
    const requiredByteLength = this.length() + 4;
    assertDataLength(this.view.byteLength, requiredByteLength);
  }

  raw() {
    return this.view.buffer.slice(4);
  }

  indexAt(i) {
    return this.view.getUint8(4 + i);
  }

  length() {
    return this.view.getUint32(0, true);
  }
}

export function SerializeBytes(value) {
  const item = assertArrayBuffer(value);
  const array = new Uint8Array(4 + item.byteLength);
  (new DataView(array.buffer)).setUint32(0, item.byteLength, true);
  array.set(new Uint8Array(item), 4);
  return array.buffer;
}

export class BytesVec {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    for (let i = 0; i < offsets.length - 1; i++) {
      new Bytes(this.view.buffer.slice(offsets[i], offsets[i + 1]), { validate: false }).validate();
    }
  }

  length() {
    if (this.view.byteLength < 8) {
      return 0;
    } else {
      return this.view.getUint32(4, true) / 4 - 1;
    }
  }

  indexAt(i) {
    const start = 4 + i * 4;
    const offset = this.view.getUint32(start, true);
    let offset_end = this.view.byteLength;
    if (i + 1 < this.length()) {
      offset_end = this.view.getUint32(start + 4, true);
    }
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeBytesVec(value) {
  return serializeTable(value.map(item => SerializeBytes(item)));
}

export class Hash {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    assertDataLength(this.view.byteLength, 32);
  }

  indexAt(i) {
    return this.view.getUint8(i);
  }

  raw() {
    return this.view.buffer;
  }

  static size() {
    return 32;
  }
}

export function SerializeHash(value) {
  const buffer = assertArrayBuffer(value);
  assertDataLength(buffer.byteLength, 32);
  return buffer;
}

export class Script {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Hash(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    if (offsets[2] - offsets[1] !== 1) {
      throw new Error(`Invalid offset for hash_type: ${offsets[1]} - ${offsets[2]}`)
    }
    new Bytes(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
  }

  getCodeHash() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Hash(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getHashType() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new DataView(this.view.buffer.slice(offset, offset_end)).getUint8(0);
  }

  getArgs() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeScript(value) {
  const buffers = [];
  buffers.push(SerializeHash(value.code_hash));
  const hashTypeView = new DataView(new ArrayBuffer(1));
  hashTypeView.setUint8(0, value.hash_type);
  buffers.push(hashTypeView.buffer)
  buffers.push(SerializeBytes(value.args));
  return serializeTable(buffers);
}

export class ScriptOpt {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    if (this.hasValue()) {
      this.value().validate(compatible);
    }
  }

  value() {
    return new Script(this.view.buffer, { validate: false });
  }

  hasValue() {
    return this.view.byteLength > 0;
  }
}

export function SerializeScriptOpt(value) {
  if (value) {
    return SerializeScript(value);
  } else {
    return new ArrayBuffer(0);
  }
}

export class OutPoint {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  getTxHash() {
    return new Hash(this.view.buffer.slice(0, 0 + Hash.size()), { validate: false });
  }

  getIndex() {
    return new Uint32(this.view.buffer.slice(0 + Hash.size(), 0 + Hash.size() + Uint32.size()), { validate: false });
  }

  validate(compatible = false) {
    assertDataLength(this.view.byteLength, OutPoint.size());
    this.getTxHash().validate(compatible);
    this.getIndex().validate(compatible);
  }
  static size() {
    return 0 + Hash.size() + Uint32.size();
  }
}

export function SerializeOutPoint(value) {
  const array = new Uint8Array(0 + Hash.size() + Uint32.size());
  const view = new DataView(array.buffer);
  array.set(new Uint8Array(SerializeHash(value.tx_hash)), 0);
  array.set(new Uint8Array(SerializeUint32(value.index)), 0 + Hash.size());
  return array.buffer;
}

export class Data {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new DataEntityOpt(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new DataEntityOpt(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new DataEntityOpt(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
  }

  getDep() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new DataEntityOpt(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getOld() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new DataEntityOpt(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getNew() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new DataEntityOpt(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeData(value) {
  const buffers = [];
  buffers.push(SerializeDataEntityOpt(value.dep));
  buffers.push(SerializeDataEntityOpt(value.old));
  buffers.push(SerializeDataEntityOpt(value.new));
  return serializeTable(buffers);
}

export class DataEntity {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    const offsets = verifyAndExtractOffsets(this.view, 0, true);
    new Uint32(this.view.buffer.slice(offsets[0], offsets[1]), { validate: false }).validate();
    new Uint32(this.view.buffer.slice(offsets[1], offsets[2]), { validate: false }).validate();
    new Bytes(this.view.buffer.slice(offsets[2], offsets[3]), { validate: false }).validate();
  }

  getIndex() {
    const start = 4;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getVersion() {
    const start = 8;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.getUint32(start + 4, true);
    return new Uint32(this.view.buffer.slice(offset, offset_end), { validate: false });
  }

  getEntity() {
    const start = 12;
    const offset = this.view.getUint32(start, true);
    const offset_end = this.view.byteLength;
    return new Bytes(this.view.buffer.slice(offset, offset_end), { validate: false });
  }
}

export function SerializeDataEntity(value) {
  const buffers = [];
  buffers.push(SerializeUint32(value.index));
  buffers.push(SerializeUint32(value.version));
  buffers.push(SerializeBytes(value.entity));
  return serializeTable(buffers);
}

export class DataEntityOpt {
  constructor(reader, { validate = true } = {}) {
    this.view = new DataView(assertArrayBuffer(reader));
    if (validate) {
      this.validate();
    }
  }

  validate(compatible = false) {
    if (this.hasValue()) {
      this.value().validate(compatible);
    }
  }

  value() {
    return new DataEntity(this.view.buffer, { validate: false });
  }

  hasValue() {
    return this.view.byteLength > 0;
  }
}

export function SerializeDataEntityOpt(value) {
  if (value) {
    return SerializeDataEntity(value);
  } else {
    return new ArrayBuffer(0);
  }
}

