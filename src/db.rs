#![allow(unused)]
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    rc::Rc,
    sync::{Arc, Mutex},
};

use anyhow::anyhow;
use bytes::Bytes;
use rand::prelude::*;
use rasn::types::ObjectIdentifier;

use crate::entity::{Attribute, Entry, EntryId, ObjectClass, Oid, DN};

const ROOT_ID_U64: u64 = 0;

impl EntryId for u64 {
    fn new_random() -> Self {
        rand::random()
    }

    fn root_identifier() -> Self {
        ROOT_ID_U64
    }
}

impl EntryId for String {
    fn new_random() -> Self {
        (0..10).fold(String::new(), |acc, _| {
            format!("{}{}", acc, rand::random::<char>())
        })
    }

    fn root_identifier() -> Self {
        "root".into()
    }
}

pub trait EntryRepository<ID: EntryId> {
    fn get_root_entry(&self) -> Entry<ID>;
    fn get_by_id(&self, id: &ID) -> Option<Entry<ID>>;
    fn save(&mut self, entry: Entry<ID>) -> Entry<ID>;
}

pub struct InMemLdapDb<ID: EntryId> {
    entries: HashMap<ID, Entry<ID>>,
}

impl<ID: EntryId> InMemLdapDb<ID> {
    pub fn new() -> Arc<Mutex<InMemLdapDb<ID>>> {
        Arc::new(Mutex::new(InMemLdapDb {
            entries: HashMap::new(),
        }))
    }
}

impl<ID: EntryId> EntryRepository<ID> for Arc<Mutex<InMemLdapDb<ID>>> {
    fn save(&mut self, mut entry: Entry<ID>) -> Entry<ID> {
        let id = entry.get_id().unwrap_or_else(|| {
            let id = ID::new_random();
            entry.set_id(&id);
            id
        });

        self.lock().unwrap().entries.insert(id, entry).unwrap()
    }

    fn get_by_id(&self, id: &ID) -> Option<Entry<ID>> {
        self.lock().unwrap().entries.get(id).map(|e| e.clone())
    }

    fn get_root_entry(&self) -> Entry<ID> {
        self.lock()
            .unwrap()
            .entries
            .entry(ID::root_identifier())
            .or_insert(Entry::default())
            .clone()
    }
}

pub trait SchemaRepo {
    fn get_object_class(&self, oid: &Oid) -> Option<&ObjectClass>;
    fn get_entry_object_classes(&self, entry: &Entry<impl EntryId>) -> Option<Vec<&ObjectClass>>;

    fn get_attribute(&self, oid: &Oid) -> Option<&Attribute>;
    fn get_attributes<'a>(&self, oids: impl Iterator<Item = &'a Oid>) -> Option<Vec<&Attribute>>;

    fn find_object_class_by_name(&self, name: &str) -> Option<&ObjectClass>;
    fn find_attribute_by_name(&self, name: &str) -> Option<&Attribute>;
    fn find_object_class_attrs(
        &self,
        obj_class: &ObjectClass,
    ) -> Option<(Vec<&Attribute>, Vec<&Attribute>)>;
    fn find_all_attributes_by_name<'a>(
        &self,
        names: impl Iterator<Item = &'a str>,
    ) -> Option<Vec<&Attribute>>;
}

#[derive(Debug, Default)]
pub struct InMemSchemaDb {
    object_classes: HashMap<Oid, ObjectClass>,
    attributes: HashMap<Oid, Attribute>,
}

impl InMemSchemaDb {
    pub fn new(
        object_classes: HashMap<Oid, ObjectClass>,
        attributes: HashMap<Oid, Attribute>,
    ) -> InMemSchemaDb {
        InMemSchemaDb {
            object_classes,
            attributes,
        }
    }
}

impl SchemaRepo for InMemSchemaDb {
    fn find_object_class_by_name(&self, name: &str) -> Option<&ObjectClass> {
        self.object_classes.values().find(|o| o.has_name(name))
    }

    fn find_attribute_by_name(&self, name: &str) -> Option<&Attribute> {
        self.attributes.values().find(|a| a.has_name(name))
    }

    fn find_object_class_attrs(
        &self,
        obj_class: &ObjectClass,
    ) -> Option<(Vec<&Attribute>, Vec<&Attribute>)> {
        let must_attrs = obj_class
            .get_must_attrs()
            .iter()
            .map(|a| self.get_attribute(a))
            .collect::<Option<Vec<_>>>()?;

        let may_attrs = obj_class
            .get_may_attrs()
            .iter()
            .map(|a| self.get_attribute(a))
            .collect::<Option<Vec<_>>>()?;

        Some((must_attrs, may_attrs))
    }

    fn find_all_attributes_by_name<'a>(
        &self,
        names: impl Iterator<Item = &'a str>,
    ) -> Option<Vec<&Attribute>> {
        names.map(|a| self.find_attribute_by_name(a)).collect()
    }

    fn get_object_class(&self, oid: &Oid) -> Option<&ObjectClass> {
        self.object_classes.get(oid)
    }

    fn get_attribute(&self, oid: &Oid) -> Option<&Attribute> {
        self.attributes.get(oid)
    }

    fn get_entry_object_classes(&self, entry: &Entry<impl EntryId>) -> Option<Vec<&ObjectClass>> {
        entry
            .get_object_classes()
            .iter()
            .map(|oid| self.get_object_class(oid))
            .collect()
    }

    fn get_attributes<'a>(&self, oids: impl Iterator<Item = &'a Oid>) -> Option<Vec<&Attribute>> {
        oids.map(|oid| self.get_attribute(oid)).collect()
    }
}
