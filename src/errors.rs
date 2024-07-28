use bytes::Bytes;
use rasn_ldap::{LdapResult, ResultCode};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum LdapError {
    #[error("Invalid Add Request: {msg:?}")]
    InvalidAddRequest { name: Bytes, msg: String },
    #[error("An unknown error occured: {0}")]
    UnknownError(String),
    #[error("Invalid DN {dn}: {msg}")]
    InvalidDN { dn: String, msg: String },
    #[error("Entry already exists at {dn}")]
    EntryAlreadyExists { dn: String },
    #[error("Entry does not exist at {dn}")]
    EntryDoesNotExists { dn: String },
    #[error("Entry {id} has invalid state: {msg}")]
    InvalidEntry { id: String, msg: String },
    #[error("Schema is invalid: {0}")]
    InvalidSchema(String),
    #[error("Could not find attribute: {0}")]
    UnknownAttribute(String),
    #[error("Could not find object class: {0}")]
    UnknownObjectClass(String),
}

impl TryFrom<LdapError> for LdapResult {
    type Error = anyhow::Error;

    fn try_from(value: LdapError) -> Result<Self, Self::Error> {
        match &value {
            LdapError::InvalidAddRequest { name, msg } => Ok(LdapResult::new(
                ResultCode::ProtocolError,
                name.clone(),
                Bytes::from(msg.clone()),
            )),
            LdapError::InvalidDN { dn, .. } => Ok(LdapResult::new(
                ResultCode::InvalidDnSyntax,
                Bytes::from(dn.clone()),
                value.to_string().into(),
            )),
            LdapError::EntryAlreadyExists { dn } => Ok(LdapResult::new(
                ResultCode::EntryAlreadyExists,
                Bytes::from(dn.clone()),
                value.to_string().into(),
            )),
            LdapError::EntryDoesNotExists { dn } => Ok(LdapResult::new(
                ResultCode::NoSuchObject,
                Bytes::from(dn.clone()),
                value.to_string().into(),
            )),
            LdapError::UnknownAttribute(_) => Ok(LdapResult::new(
                ResultCode::UndefinedAttributeType,
                Bytes::new(),
                value.to_string().into(),
            )),
            LdapError::UnknownObjectClass(_) => Ok(LdapResult::new(
                ResultCode::UndefinedAttributeType,
                Bytes::new(),
                value.to_string().into(),
            )),
            _ => Err(value)?,
        }
    }
}
