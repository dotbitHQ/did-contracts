// Generated by Molecule 0.7.3

use molecule::prelude::*;

use super::basic::*;
use super::cell::*;
#[derive(Clone)]
pub struct AccountCellDataV1(molecule::bytes::Bytes);
impl ::core::fmt::LowerHex for AccountCellDataV1 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        use molecule::hex_string;
        if f.alternate() {
            write!(f, "0x")?;
        }
        write!(f, "{}", hex_string(self.as_slice()))
    }
}
impl ::core::fmt::Debug for AccountCellDataV1 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{}({:#x})", Self::NAME, self)
    }
}
impl ::core::fmt::Display for AccountCellDataV1 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{} {{ ", Self::NAME)?;
        write!(f, "{}: {}", "id", self.id())?;
        write!(f, ", {}: {}", "account", self.account())?;
        write!(f, ", {}: {}", "registered_at", self.registered_at())?;
        write!(f, ", {}: {}", "updated_at", self.updated_at())?;
        write!(f, ", {}: {}", "status", self.status())?;
        write!(f, ", {}: {}", "records", self.records())?;
        let extra_count = self.count_extra_fields();
        if extra_count != 0 {
            write!(f, ", .. ({} fields)", extra_count)?;
        }
        write!(f, " }}")
    }
}
impl ::core::default::Default for AccountCellDataV1 {
    fn default() -> Self {
        let v: Vec<u8> = vec![
            73, 0, 0, 0, 28, 0, 0, 0, 48, 0, 0, 0, 52, 0, 0, 0, 60, 0, 0, 0, 68, 0, 0, 0, 69, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            4, 0, 0, 0,
        ];
        AccountCellDataV1::new_unchecked(v.into())
    }
}
impl AccountCellDataV1 {
    pub const FIELD_COUNT: usize = 6;
    pub fn total_size(&self) -> usize {
        molecule::unpack_number(self.as_slice()) as usize
    }
    pub fn field_count(&self) -> usize {
        if self.total_size() == molecule::NUMBER_SIZE {
            0
        } else {
            (molecule::unpack_number(&self.as_slice()[molecule::NUMBER_SIZE..]) as usize / 4) - 1
        }
    }
    pub fn count_extra_fields(&self) -> usize {
        self.field_count() - Self::FIELD_COUNT
    }
    pub fn has_extra_fields(&self) -> bool {
        Self::FIELD_COUNT != self.field_count()
    }
    pub fn id(&self) -> AccountId {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[4..]) as usize;
        let end = molecule::unpack_number(&slice[8..]) as usize;
        AccountId::new_unchecked(self.0.slice(start..end))
    }
    pub fn account(&self) -> AccountChars {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[8..]) as usize;
        let end = molecule::unpack_number(&slice[12..]) as usize;
        AccountChars::new_unchecked(self.0.slice(start..end))
    }
    pub fn registered_at(&self) -> Uint64 {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[12..]) as usize;
        let end = molecule::unpack_number(&slice[16..]) as usize;
        Uint64::new_unchecked(self.0.slice(start..end))
    }
    pub fn updated_at(&self) -> Uint64 {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[16..]) as usize;
        let end = molecule::unpack_number(&slice[20..]) as usize;
        Uint64::new_unchecked(self.0.slice(start..end))
    }
    pub fn status(&self) -> Uint8 {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[20..]) as usize;
        let end = molecule::unpack_number(&slice[24..]) as usize;
        Uint8::new_unchecked(self.0.slice(start..end))
    }
    pub fn records(&self) -> Records {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[24..]) as usize;
        if self.has_extra_fields() {
            let end = molecule::unpack_number(&slice[28..]) as usize;
            Records::new_unchecked(self.0.slice(start..end))
        } else {
            Records::new_unchecked(self.0.slice(start..))
        }
    }
    pub fn as_reader<'r>(&'r self) -> AccountCellDataV1Reader<'r> {
        AccountCellDataV1Reader::new_unchecked(self.as_slice())
    }
}
impl molecule::prelude::Entity for AccountCellDataV1 {
    type Builder = AccountCellDataV1Builder;
    const NAME: &'static str = "AccountCellDataV1";
    fn new_unchecked(data: molecule::bytes::Bytes) -> Self {
        AccountCellDataV1(data)
    }
    fn as_bytes(&self) -> molecule::bytes::Bytes {
        self.0.clone()
    }
    fn as_slice(&self) -> &[u8] {
        &self.0[..]
    }
    fn from_slice(slice: &[u8]) -> molecule::error::VerificationResult<Self> {
        AccountCellDataV1Reader::from_slice(slice).map(|reader| reader.to_entity())
    }
    fn from_compatible_slice(slice: &[u8]) -> molecule::error::VerificationResult<Self> {
        AccountCellDataV1Reader::from_compatible_slice(slice).map(|reader| reader.to_entity())
    }
    fn new_builder() -> Self::Builder {
        ::core::default::Default::default()
    }
    fn as_builder(self) -> Self::Builder {
        Self::new_builder()
            .id(self.id())
            .account(self.account())
            .registered_at(self.registered_at())
            .updated_at(self.updated_at())
            .status(self.status())
            .records(self.records())
    }
}
#[derive(Clone, Copy)]
pub struct AccountCellDataV1Reader<'r>(&'r [u8]);
impl<'r> ::core::fmt::LowerHex for AccountCellDataV1Reader<'r> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        use molecule::hex_string;
        if f.alternate() {
            write!(f, "0x")?;
        }
        write!(f, "{}", hex_string(self.as_slice()))
    }
}
impl<'r> ::core::fmt::Debug for AccountCellDataV1Reader<'r> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{}({:#x})", Self::NAME, self)
    }
}
impl<'r> ::core::fmt::Display for AccountCellDataV1Reader<'r> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{} {{ ", Self::NAME)?;
        write!(f, "{}: {}", "id", self.id())?;
        write!(f, ", {}: {}", "account", self.account())?;
        write!(f, ", {}: {}", "registered_at", self.registered_at())?;
        write!(f, ", {}: {}", "updated_at", self.updated_at())?;
        write!(f, ", {}: {}", "status", self.status())?;
        write!(f, ", {}: {}", "records", self.records())?;
        let extra_count = self.count_extra_fields();
        if extra_count != 0 {
            write!(f, ", .. ({} fields)", extra_count)?;
        }
        write!(f, " }}")
    }
}
impl<'r> AccountCellDataV1Reader<'r> {
    pub const FIELD_COUNT: usize = 6;
    pub fn total_size(&self) -> usize {
        molecule::unpack_number(self.as_slice()) as usize
    }
    pub fn field_count(&self) -> usize {
        if self.total_size() == molecule::NUMBER_SIZE {
            0
        } else {
            (molecule::unpack_number(&self.as_slice()[molecule::NUMBER_SIZE..]) as usize / 4) - 1
        }
    }
    pub fn count_extra_fields(&self) -> usize {
        self.field_count() - Self::FIELD_COUNT
    }
    pub fn has_extra_fields(&self) -> bool {
        Self::FIELD_COUNT != self.field_count()
    }
    pub fn id(&self) -> AccountIdReader<'r> {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[4..]) as usize;
        let end = molecule::unpack_number(&slice[8..]) as usize;
        AccountIdReader::new_unchecked(&self.as_slice()[start..end])
    }
    pub fn account(&self) -> AccountCharsReader<'r> {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[8..]) as usize;
        let end = molecule::unpack_number(&slice[12..]) as usize;
        AccountCharsReader::new_unchecked(&self.as_slice()[start..end])
    }
    pub fn registered_at(&self) -> Uint64Reader<'r> {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[12..]) as usize;
        let end = molecule::unpack_number(&slice[16..]) as usize;
        Uint64Reader::new_unchecked(&self.as_slice()[start..end])
    }
    pub fn updated_at(&self) -> Uint64Reader<'r> {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[16..]) as usize;
        let end = molecule::unpack_number(&slice[20..]) as usize;
        Uint64Reader::new_unchecked(&self.as_slice()[start..end])
    }
    pub fn status(&self) -> Uint8Reader<'r> {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[20..]) as usize;
        let end = molecule::unpack_number(&slice[24..]) as usize;
        Uint8Reader::new_unchecked(&self.as_slice()[start..end])
    }
    pub fn records(&self) -> RecordsReader<'r> {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[24..]) as usize;
        if self.has_extra_fields() {
            let end = molecule::unpack_number(&slice[28..]) as usize;
            RecordsReader::new_unchecked(&self.as_slice()[start..end])
        } else {
            RecordsReader::new_unchecked(&self.as_slice()[start..])
        }
    }
}
impl<'r> molecule::prelude::Reader<'r> for AccountCellDataV1Reader<'r> {
    type Entity = AccountCellDataV1;
    const NAME: &'static str = "AccountCellDataV1Reader";
    fn to_entity(&self) -> Self::Entity {
        Self::Entity::new_unchecked(self.as_slice().to_owned().into())
    }
    fn new_unchecked(slice: &'r [u8]) -> Self {
        AccountCellDataV1Reader(slice)
    }
    fn as_slice(&self) -> &'r [u8] {
        self.0
    }
    fn verify(slice: &[u8], compatible: bool) -> molecule::error::VerificationResult<()> {
        use molecule::verification_error as ve;
        let slice_len = slice.len();
        if slice_len < molecule::NUMBER_SIZE {
            return ve!(Self, HeaderIsBroken, molecule::NUMBER_SIZE, slice_len);
        }
        let total_size = molecule::unpack_number(slice) as usize;
        if slice_len != total_size {
            return ve!(Self, TotalSizeNotMatch, total_size, slice_len);
        }
        if slice_len == molecule::NUMBER_SIZE && Self::FIELD_COUNT == 0 {
            return Ok(());
        }
        if slice_len < molecule::NUMBER_SIZE * 2 {
            return ve!(Self, HeaderIsBroken, molecule::NUMBER_SIZE * 2, slice_len);
        }
        let offset_first = molecule::unpack_number(&slice[molecule::NUMBER_SIZE..]) as usize;
        if offset_first % molecule::NUMBER_SIZE != 0 || offset_first < molecule::NUMBER_SIZE * 2 {
            return ve!(Self, OffsetsNotMatch);
        }
        if slice_len < offset_first {
            return ve!(Self, HeaderIsBroken, offset_first, slice_len);
        }
        let field_count = offset_first / molecule::NUMBER_SIZE - 1;
        if field_count < Self::FIELD_COUNT {
            return ve!(Self, FieldCountNotMatch, Self::FIELD_COUNT, field_count);
        } else if !compatible && field_count > Self::FIELD_COUNT {
            return ve!(Self, FieldCountNotMatch, Self::FIELD_COUNT, field_count);
        };
        let mut offsets: Vec<usize> = slice[molecule::NUMBER_SIZE..offset_first]
            .chunks_exact(molecule::NUMBER_SIZE)
            .map(|x| molecule::unpack_number(x) as usize)
            .collect();
        offsets.push(total_size);
        if offsets.windows(2).any(|i| i[0] > i[1]) {
            return ve!(Self, OffsetsNotMatch);
        }
        AccountIdReader::verify(&slice[offsets[0]..offsets[1]], compatible)?;
        AccountCharsReader::verify(&slice[offsets[1]..offsets[2]], compatible)?;
        Uint64Reader::verify(&slice[offsets[2]..offsets[3]], compatible)?;
        Uint64Reader::verify(&slice[offsets[3]..offsets[4]], compatible)?;
        Uint8Reader::verify(&slice[offsets[4]..offsets[5]], compatible)?;
        RecordsReader::verify(&slice[offsets[5]..offsets[6]], compatible)?;
        Ok(())
    }
}
#[derive(Debug, Default)]
pub struct AccountCellDataV1Builder {
    pub(crate) id: AccountId,
    pub(crate) account: AccountChars,
    pub(crate) registered_at: Uint64,
    pub(crate) updated_at: Uint64,
    pub(crate) status: Uint8,
    pub(crate) records: Records,
}
impl AccountCellDataV1Builder {
    pub const FIELD_COUNT: usize = 6;
    pub fn id(mut self, v: AccountId) -> Self {
        self.id = v;
        self
    }
    pub fn account(mut self, v: AccountChars) -> Self {
        self.account = v;
        self
    }
    pub fn registered_at(mut self, v: Uint64) -> Self {
        self.registered_at = v;
        self
    }
    pub fn updated_at(mut self, v: Uint64) -> Self {
        self.updated_at = v;
        self
    }
    pub fn status(mut self, v: Uint8) -> Self {
        self.status = v;
        self
    }
    pub fn records(mut self, v: Records) -> Self {
        self.records = v;
        self
    }
}
impl molecule::prelude::Builder for AccountCellDataV1Builder {
    type Entity = AccountCellDataV1;
    const NAME: &'static str = "AccountCellDataV1Builder";
    fn expected_length(&self) -> usize {
        molecule::NUMBER_SIZE * (Self::FIELD_COUNT + 1)
            + self.id.as_slice().len()
            + self.account.as_slice().len()
            + self.registered_at.as_slice().len()
            + self.updated_at.as_slice().len()
            + self.status.as_slice().len()
            + self.records.as_slice().len()
    }
    fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
        let mut total_size = molecule::NUMBER_SIZE * (Self::FIELD_COUNT + 1);
        let mut offsets = Vec::with_capacity(Self::FIELD_COUNT);
        offsets.push(total_size);
        total_size += self.id.as_slice().len();
        offsets.push(total_size);
        total_size += self.account.as_slice().len();
        offsets.push(total_size);
        total_size += self.registered_at.as_slice().len();
        offsets.push(total_size);
        total_size += self.updated_at.as_slice().len();
        offsets.push(total_size);
        total_size += self.status.as_slice().len();
        offsets.push(total_size);
        total_size += self.records.as_slice().len();
        writer.write_all(&molecule::pack_number(total_size as molecule::Number))?;
        for offset in offsets.into_iter() {
            writer.write_all(&molecule::pack_number(offset as molecule::Number))?;
        }
        writer.write_all(self.id.as_slice())?;
        writer.write_all(self.account.as_slice())?;
        writer.write_all(self.registered_at.as_slice())?;
        writer.write_all(self.updated_at.as_slice())?;
        writer.write_all(self.status.as_slice())?;
        writer.write_all(self.records.as_slice())?;
        Ok(())
    }
    fn build(&self) -> Self::Entity {
        let mut inner = Vec::with_capacity(self.expected_length());
        self.write(&mut inner)
            .unwrap_or_else(|_| panic!("{} build should be ok", Self::NAME));
        AccountCellDataV1::new_unchecked(inner.into())
    }
}
