use bytes::Bytes;
use rasn_ldap::{LdapResult, ResultCode};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum LdapError {
    #[error("Invalid Add Request: {msg:?}")]
    InvalidAddRequest { name: Bytes, msg: String },
    #[error("Cannot convert this error to protocol op")]
    CantConvertoToProtocolOp,
    #[error("Invalid DN: {dn}")]
    InvalidDN { dn: String },
    #[error("Entry already exists at {dn}")]
    EntryAlreadyExists { dn: String },
    #[error("Entry does not exist at {dn}")]
    EntryDoesNotExists { dn: String },
    #[error("Entry {id} has invalid state: {msg}")]
    InvalidEntry { id: String, msg: String },
}

impl TryFrom<LdapError> for LdapResult {
    type Error = anyhow::Error;

    fn try_from(value: LdapError) -> Result<Self, Self::Error> {
        match value {
            LdapError::InvalidAddRequest { name, msg } => {
                Ok(LdapResult::new(ResultCode::ProtocolError, name, msg.into()))
            }

            LdapError::InvalidDN { dn } => Ok(LdapResult::new(
                ResultCode::InvalidDnSyntax,
                dn.into(),
                value.to_string().into(),
            )),
            LdapError::EntryAlreadyExists { dn } => Ok(LdapResult::new(
                ResultCode::EntryAlreadyExists,
                dn.into(),
                value.to_string().into(),
            )),
            LdapError::EntryDoesNotExists { dn } => Ok(LdapResult::new(
                ResultCode::NoSuchObject,
                dn.into(),
                value.to_string().into(),
            )),
            _ => Err(value)?,
        }
    }
}
