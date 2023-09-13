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

