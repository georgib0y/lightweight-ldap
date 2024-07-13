use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::errors::LdapError;

pub struct RDN {
    attr: String,
    value: String,
}

impl RDN {
    pub fn new(attr: &str, value: &str) -> RDN {
        RDN {
            attr: attr.into(),
            value: value.into(),
        }
    }
}

impl<'a> TryFrom<&'a str> for RDN {
    type Error = anyhow::Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let Some((attr, value)) = value.split_once('=') else {
            anyhow::bail!("could not get attr/val for rdn: {}", value)
        };

        Ok(RDN {
            attr: attr.into(),
            value: value.into(),
        })
    }
}

pub struct DN {
    rdns: Vec<Vec<RDN>>,
}

impl<'a> TryFrom<&'a str> for DN {
    type Error = LdapError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut rdns = Vec::new();

        for seg_str in value.split(',') {
            let mut seg = Vec::new();
            for rdn in seg_str.split('+') {
                let Some((a, v)) = rdn.split_once('=') else {
                    Err(LdapError::InvalidDN { dn: value.into() })?
                };

                seg.push(RDN::new(a, v));
            }
            rdns.push(seg);
        }

        Ok(DN { rdns })
    }
}

#[derive(Debug, Default, Clone)]
pub struct Entry {
    _id: String,
    parent: String,
    children: Vec<String>,
    attributes: HashMap<String, HashSet<String>>,
}

impl Entry {
    pub fn new(attributes: HashMap<String, HashSet<String>>) -> Entry {
        Entry {
            attributes,
            ..Default::default()
        }
    }

    pub fn get_id(&self) -> &str {
        self._id.as_ref()
    }

    pub fn get_attributes(&self) -> &HashMap<String, HashSet<String>> {
        &self.attributes
    }
}
