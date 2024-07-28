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

use crate::entity::schema::{Attribute, ObjectClass, Oid};
use crate::{
    entity::entry::{Entry, EntryId},
    errors::LdapError,
};

const ROOT_ID_U64: u64 = 0;

impl EntryId for u64 {
    fn new_random_id() -> Self {
        rand::random()
    }

    fn root_identifier() -> Self {
        ROOT_ID_U64
    }
}

impl EntryId for String {
    fn new_random_id() -> Self {
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
    fn save(&self, entry: Entry<ID>) -> Result<Entry<ID>, LdapError>;
}

#[derive(Debug)]
pub struct InMemLdapDb<ID: EntryId> {
    entries: HashMap<ID, Entry<ID>>,
}

impl<ID: EntryId> InMemLdapDb<ID> {
    pub fn new() -> Arc<Mutex<InMemLdapDb<ID>>> {
        Arc::new(Mutex::new(InMemLdapDb {
            entries: HashMap::new(),
        }))
    }

    pub fn with_entries(
        entries_iter: impl Iterator<Item = Entry<ID>>,
    ) -> Arc<Mutex<InMemLdapDb<ID>>> {
        let mut entries = HashMap::new();
        entries.extend(entries_iter.map(|e| (e.get_id().unwrap(), e)));
        Arc::new(Mutex::new(InMemLdapDb { entries }))
    }
}

impl<ID: EntryId> EntryRepository<ID> for Arc<Mutex<InMemLdapDb<ID>>> {
    fn save(&self, mut entry: Entry<ID>) -> Result<Entry<ID>, LdapError> {
        let id = entry.get_id().unwrap_or_else(|| {
            let id = ID::new_random_id();
            entry.set_id(&id);
            id
        });

        let res = self
            .lock()
            .ok()
            .ok_or(LdapError::UnknownError(
                "Could not insert entry into has table".to_string(),
            ))?
            .entries
            .insert(id, entry.clone());

        dbg!(self);

        Ok(res.unwrap_or(entry))
    }

    fn get_by_id(&self, id: &ID) -> Option<Entry<ID>> {
        self.lock().unwrap().entries.get(id).cloned()
    }

    fn get_root_entry(&self) -> Entry<ID> {
        self.lock()
            .unwrap()
            .entries
            .entry(ID::root_identifier())
            .or_default()
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
        object_classes: impl Iterator<Item = ObjectClass>,
        attributes: impl Iterator<Item = Attribute>,
    ) -> InMemSchemaDb {
        let mut oc_map = HashMap::new();
        let mut attr_map = HashMap::new();

        oc_map.extend(object_classes.map(|o| (o.get_numericoid().clone(), o)));
        attr_map.extend(attributes.map(|a| (a.get_numericoid().clone(), a)));

        InMemSchemaDb {
            object_classes: oc_map,
            attributes: attr_map,
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
