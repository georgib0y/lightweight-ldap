#![allow(unused)]

use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::hash::Hash;

use super::dn::Rdn;
use super::schema::Oid;

pub trait EntryId: Debug + Display + Default + Clone + Eq + Hash {
    fn new_random_id() -> Self;
    fn root_identifier() -> Self;
}

#[derive(Debug, Default, Clone)]
pub struct Entry<ID: EntryId> {
    _id: Option<ID>,
    parent: ID,
    children: HashSet<ID>,
    object_classes: HashSet<Oid>,
    attributes: HashMap<Oid, HashSet<String>>,
}

impl<ID: EntryId> Entry<ID> {
    pub fn get_id(&self) -> Option<ID> {
        self._id.to_owned()
    }

    pub fn get_id_str(&self) -> String {
        self._id
            .as_ref()
            .map(|id| id.to_string())
            .unwrap_or("No ID".to_string())
    }

    pub fn set_id(&mut self, id: &ID) {
        self._id = Some(id.to_owned());
    }

    pub fn get_object_classes(&self) -> &HashSet<Oid> {
        &self.object_classes
    }

    pub fn get_attributes(&self) -> &HashMap<Oid, HashSet<String>> {
        &self.attributes
    }

    pub fn get_attribute(&self, oid: &Oid) -> Option<&HashSet<String>> {
        self.attributes.get(oid)
    }

    pub fn get_children(&self) -> &HashSet<ID> {
        &self.children
    }

    pub fn matches_rdn(&self, rdn: &Rdn) -> bool {
        for (oid, val) in rdn {
            let Some(attr) = self.get_attribute(oid) else {
                continue;
            };

            if attr.contains(val) {
                return true;
            }
        }

        false
    }
}

pub struct EntryBuilder<ID: EntryId> {
    entry: Entry<ID>,
}

impl<ID: EntryId> EntryBuilder<ID> {
    pub fn new() -> Self {
        Self {
            entry: Default::default(),
        }
    }

    pub fn set_id(mut self, id: impl Into<ID>) -> Self {
        self.entry._id = Some(id.into());
        self
    }

    pub fn set_parent(mut self, parent: impl Into<ID>) -> Self {
        self.entry.parent = parent.into();
        self
    }

    pub fn add_child(mut self, child: impl Into<ID>) -> Self {
        self.entry.children.insert(child.into());
        self
    }

    pub fn add_object_class(mut self, obj_class_oid: impl Into<Oid>) -> Self {
        self.entry.object_classes.insert(obj_class_oid.into());
        self
    }

    pub fn add_attr_val(mut self, attr_oid: impl Into<Oid>, value: impl Into<String>) -> Self {
        self.entry
            .attributes
            .entry(attr_oid.into())
            .or_default()
            .insert(value.into());

        self
    }

    pub fn add_attr_vals(
        mut self,
        attr_oid: impl Into<Oid>,
        values: impl Iterator<Item = impl Into<String>>,
    ) -> Self {
        self.entry
            .attributes
            .entry(attr_oid.into())
            .or_default()
            .extend(values.map(|v| v.into()));

        self
    }

    pub fn build(self) -> Entry<ID> {
        self.entry
    }
}
