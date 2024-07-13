#![allow(unused)]
use std::collections::{HashMap, HashSet};

use anyhow::anyhow;
use bytes::Bytes;
use rasn::types::ObjectIdentifier;

use crate::entity::{Attribute, Entry, ObjectClass, DN};

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

pub trait SchemaRepo {
    fn find_object_class_by_name(&self, name: &str) -> Option<&ObjectClass>;
    fn find_attribute_by_name(&self, name: &str) -> Option<&Attribute>;
}

#[derive(Debug, Default)]
pub struct InMemSchemaDb {
    object_classes: Vec<ObjectClass>,
    attributes: Vec<Attribute>,
}

impl InMemSchemaDb {
    pub fn new(object_classes: Vec<ObjectClass>, attributes: Vec<Attribute>) -> InMemSchemaDb {
        InMemSchemaDb {
            object_classes,
            attributes,
        }
    }
}

impl SchemaRepo for InMemSchemaDb {
    fn find_object_class_by_name(&self, name: &str) -> Option<&ObjectClass> {
        self.object_classes.iter().find(|o| o.has_name(name))
    }

    fn find_attribute_by_name(&self, name: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|a| a.has_name(name))
    }
}
