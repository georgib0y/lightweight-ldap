use std::collections::HashMap;

pub struct LdapEntry {
    id: Stirng,
    parent: String,
    children: Vec<String>,
    rdn: String,
    attributes: HashMap<String, Vec<String>>,
}

impl LdapEntry {
    pub fn new(rdn: String, attributes: HashMap<String, Vec<String>>) -> LdapEntry {
        LdapEntry { rdn, attributes }
    }
}

pub trait LdapRepo {
    fn get(&self, dn: &str) -> Option<&LdapEntry>;
    fn save(&mut self, entry: LdapEntry) -> Option<LdapEntry>;
    fn dn_parent_exists(&self, dn: &str) -> bool;
}

pub struct InMemLdapDb {
    entries: Vec<LdapEntry>,
}

impl InMemLdapDb {
    pub fn new() -> InMemLdapDb {
        InMemLdapDb {
            entries: Vec::new(),
        }
    }
}

impl LdapRepo for InMemLdapDb {
    fn get(&self, dn: &str) -> Option<&LdapEntry> {
        self.entries.get(dn)
    }

    fn save(&mut self, dn: String, entry: LdapEntry) -> Option<LdapEntry> {
        self.entries.insert(, entry)
    }

    fn dn_parent_exists(&self, dn: &str) -> bool {
        todo!()
    }
}
