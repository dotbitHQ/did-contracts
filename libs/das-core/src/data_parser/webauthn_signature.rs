use alloc::boxed::Box;
use core::ops::Index;

use crate::error::{ErrorCode, ScriptError};

pub struct WebAuthnSignature<'a> {
    inner: &'a [u8],
    pubkey_index_start: usize,
    signature_start: usize,
    pubkey_start: usize,
    authenticator_data_start: usize,
    client_data_json_start: usize,
}

impl<'a> WebAuthnSignature<'a> {
    pub fn pubkey_index(&self) -> &[u8] {
        self.inner.index(self.pubkey_index_start..self.signature_start)
    }

    pub fn signature(&self) -> &[u8] {
        self.inner.index(self.signature_start..self.pubkey_start)
    }

    pub fn pubkey(&self) -> &[u8] {
        self.inner.index(self.pubkey_start..self.authenticator_data_start)
    }

    pub fn authenticator_data(&self) -> &[u8] {
        self.inner
            .index(self.authenticator_data_start..self.client_data_json_start)
    }

    pub fn client_data_json(&self) -> &[u8] {
        self.inner.index(self.client_data_json_start..)
    }
}

impl<'a> TryFrom<&'a [u8]> for WebAuthnSignature<'a> {
    type Error = Box<dyn ScriptError>;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;
        let pubkey_index_bytes = value
            .get(cursor)
            .ok_or(code_to_error!(ErrorCode::WitnessDataDecodingError))?;
        let pubkey_index_start = cursor + 1;
        
        cursor = pubkey_index_start + *pubkey_index_bytes as usize;
        let signature_bytes = value
            .get(cursor)
            .ok_or(code_to_error!(ErrorCode::WitnessDataDecodingError))?;
        let signature_start = cursor + 1;
        
        cursor = signature_start + *signature_bytes as usize;
        let pubkey_bytes = value
            .get(cursor)
            .ok_or(code_to_error!(ErrorCode::WitnessDataDecodingError))?;
        let pubkey_start = cursor + 1;
        
        cursor = pubkey_index_start + *pubkey_bytes as usize;
        let authenticator_data_bytes = value
            .get(cursor)
            .ok_or(code_to_error!(ErrorCode::WitnessDataDecodingError))?;
        let authenticator_data_start = cursor + 1;
        
        cursor = authenticator_data_start + *authenticator_data_bytes as usize;
        let client_data_json_bytes = value
            .get(cursor)
            .ok_or(code_to_error!(ErrorCode::WitnessDataDecodingError))?;
        let client_data_json_start = cursor + 1;
        
        cursor = client_data_json_start + *client_data_json_bytes as usize;
        das_assert!(
            value.get(cursor).is_none() && value.get(cursor - 1).is_some(),
            ErrorCode::WitnessDataDecodingError,
            "There's residue after parsing WebAuthnSignature"
        );

        Ok(Self {
            inner: value,
            pubkey_index_start,
            signature_start,
            pubkey_start,
            authenticator_data_start,
            client_data_json_start,
        })
    }
}
