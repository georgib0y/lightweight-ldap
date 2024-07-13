#![allow(unused)]
use std::collections::{HashMap, HashSet};

use anyhow::anyhow;
use bytes::Bytes;

use crate::entity::{Entry, DN};

pub trait EntryRepository {
    fn find_by_dn(&self, dn: &DN) -> Option<Entry>;
    fn save(&mut self, dn: &DN, entry: Entry) -> Entry;
    fn dn_parent_exists(&self, dn: &DN) -> bool;
}

pub struct InMemLdapDb {
    entries: Vec<Entry>,
}

impl InMemLdapDb {
    pub fn new() -> InMemLdapDb {
        InMemLdapDb {
            entries: Vec::new(),
        }
    }
}

impl EntryRepository for InMemLdapDb {
    fn find_by_dn(&self, dn: &DN) -> Option<Entry> {
        // self.entries.get(dn).clone()
        todo!()
    }

    fn save(&mut self, dn: &DN, entry: Entry) -> Entry {
        // self.entries.insert(, entry)
        todo!()
    }

    fn dn_parent_exists(&self, dn: &DN) -> bool {
        todo!()
    }
}
