use std::collections::HashMap;

pub struct Attribute {
    key: String,
    val: String,
}

pub struct LdapEntry {
    dn: String,
    attributes: Vec<Attribute>,
}

pub trait LdapRepo {
    fn get(&self, dn: &str) -> Option<&LdapEntry>;
    fn save(&mut self, entry: LdapEntry) -> Option<LdapEntry>;
}

pub struct MemLdapDb {
    entries: HashMap<String, LdapEntry>,
}

impl MemLdapDb {
    pub fn new() -> MemLdapDb {
        MemLdapDb {
            entries: HashMap::new(),
        }
    }
}

impl LdapRepo for MemLdapDb {
    fn get(&self, dn: &str) -> Option<&LdapEntry> {
        self.entries.get(dn)
    }

    fn save(&mut self, entry: LdapEntry) -> Option<LdapEntry> {
        self.entries.insert(entry.dn.clone(), entry)
    }
}
