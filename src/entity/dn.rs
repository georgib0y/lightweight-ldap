use crate::errors::LdapError;

use super::schema::Oid;

#[derive(Debug, Default, Clone)]
pub struct Rdn(Vec<(Oid, String)>);

impl From<Vec<(Oid, String)>> for Rdn {
    fn from(value: Vec<(Oid, String)>) -> Self {
        Rdn(value)
    }
}

impl From<(Oid, String)> for Rdn {
    fn from(value: (Oid, String)) -> Self {
        Rdn(vec![value])
    }
}

impl<'a> TryFrom<&'a str> for Rdn {
    type Error = LdapError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut rdn = Vec::new();
        for val in value.split('+') {
            let Some((a, v)) = val.split_once('=') else {
                Err(LdapError::InvalidDN { dn: value.into() })?
            };

            rdn.push((a.into(), v.into()));
        }

        Ok(Rdn(rdn))
    }
}

impl<'a> IntoIterator for &'a Rdn {
    type Item = &'a (Oid, String);

    type IntoIter = std::slice::Iter<'a, (Oid, String)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

pub struct DN {
    rdns: Vec<Rdn>,
}

impl DN {
    pub fn new(rdns: Vec<Rdn>) -> DN {
        DN { rdns }
    }

    pub fn first(&self) -> Option<&Rdn> {
        self.rdns.first()
    }

    pub fn as_slice(&self) -> &[Rdn] {
        self.rdns.as_slice()
    }

    pub fn parent_dn(&self) -> DN {
        DN {
            rdns: self.rdns.iter().map(|rdn| rdn.to_owned()).skip(1).collect(),
        }
    }
}

impl<'a> IntoIterator for &'a DN {
    type Item = &'a Rdn;
    type IntoIter = std::slice::Iter<'a, Rdn>;

    fn into_iter(self) -> Self::IntoIter {
        self.rdns.iter()
    }
}
