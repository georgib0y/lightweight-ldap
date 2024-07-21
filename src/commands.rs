use std::collections::{HashMap, HashSet};

use rasn_ldap::AddRequest;

use crate::errors::LdapError;

pub struct AddEntryCommand {
    pub dn: String,
    pub attributes: HashMap<String, HashSet<String>>,
}

impl TryFrom<&AddRequest> for AddEntryCommand {
    type Error = LdapError;

    fn try_from(value: &AddRequest) -> Result<Self, Self::Error> {
        let dn = String::from_utf8(value.entry.to_owned().into()).map_err(|_| {
            LdapError::InvalidAddRequest {
                name: value.entry.clone(),
                msg: "DN entry not UTF-8 encoded".into(),
            }
        })?;

        let mut attributes: HashMap<String, HashSet<String>> = HashMap::new();

        for attr in value.attributes.iter() {
            let name = String::from_utf8(attr.r#type.to_owned().into()).map_err(|_| {
                LdapError::InvalidAddRequest {
                    name: value.entry.clone(),
                    msg: "Attribute key not UTF-8".into(),
                }
            })?;

            let values = attr
                .vals
                .iter()
                .map(|v| {
                    String::from_utf8(v.clone().into()).map_err(|_| LdapError::InvalidAddRequest {
                        name: value.entry.clone(),
                        msg: "Attribute value not UTF-8".into(),
                    })
                })
                .collect::<Result<HashSet<String>, LdapError>>()?;

            attributes.entry(name).or_default().extend(values);
        }

        Ok(AddEntryCommand { dn, attributes })
    }
}
