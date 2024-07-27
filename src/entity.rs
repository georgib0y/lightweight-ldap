#![allow(unused)]
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::Hash,
    vec::IntoIter,
};

use anyhow::Result;

use crate::errors::LdapError;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Oid(String);

impl Oid {
    pub fn new<S: Into<String>>(oid: S) -> Oid {
        Oid(oid.into())
    }
}

impl<T: Into<String>> From<T> for Oid {
    fn from(value: T) -> Self {
        Oid(value.into())
    }
}

impl Display for Oid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

#[derive(Debug, Default, Clone)]
pub struct ObjectClass {
    numericoid: Oid,
    names: HashSet<String>,
    desc: String,
    obsolete: bool,
    sup_oids: HashSet<Oid>,
    kind: Kind,
    must_attrs: HashSet<Oid>,
    may_attrs: HashSet<Oid>,
}

impl PartialEq for ObjectClass {
    fn eq(&self, other: &Self) -> bool {
        self.numericoid == other.numericoid
    }
}

impl ObjectClass {
    pub fn get_numericoid(&self) -> &Oid {
        &self.numericoid
    }

    pub fn get_names(&self) -> &HashSet<String> {
        &self.names
    }

    pub fn add_name(&mut self, name: &str) {
        self.names.insert(name.into());
    }

    pub fn has_name(&self, name: &str) -> bool {
        self.names.contains(name)
    }

    pub fn is_structural(&self) -> bool {
        self.kind.is_structural()
    }

    pub fn get_must_attrs(&self) -> &HashSet<Oid> {
        &self.must_attrs
    }

    pub fn get_may_attrs(&self) -> &HashSet<Oid> {
        &self.may_attrs
    }
}

pub struct ObjectClassBuilder {
    obj_class: ObjectClass,
}

impl ObjectClassBuilder {
    pub fn new() -> Self {
        Self {
            obj_class: ObjectClass::default(),
        }
    }

    pub fn set_numericoid(mut self, numericoid: &Oid) -> Self {
        self.obj_class.numericoid = numericoid.clone();
        self
    }

    pub fn add_name(mut self, name: &str) -> Self {
        self.obj_class.names.insert(name.into());
        self
    }

    pub fn set_desc(mut self, desc: &str) -> Self {
        self.obj_class.desc = desc.into();
        self
    }

    pub fn set_obsolete(mut self, obsolete: bool) -> Self {
        self.obj_class.obsolete = obsolete;
        self
    }

    pub fn add_sup_oid(mut self, sup_oid: Oid) -> Self {
        self.obj_class.sup_oids.insert(sup_oid);
        self
    }

    pub fn set_kind(mut self, kind: Kind) -> Self {
        self.obj_class.kind = kind;
        self
    }

    pub fn add_must_attr(mut self, must_attr: Oid) -> Self {
        self.obj_class.must_attrs.insert(must_attr);
        self
    }

    pub fn add_may_attr(mut self, may_attr: Oid) -> Self {
        self.obj_class.may_attrs.insert(may_attr);
        self
    }

    pub fn build(self) -> ObjectClass {
        self.obj_class
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Kind {
    Abstract,
    Structural,
    Auxiliary,
}

impl Kind {
    pub fn is_structural(&self) -> bool {
        matches!(self, Kind::Structural)
    }
}

impl Default for Kind {
    fn default() -> Self {
        Self::Auxiliary
    }
}

#[derive(Debug, Default, Clone)]
pub struct Attribute {
    numericoid: Oid,
    names: HashSet<String>,
    desc: String,
    obsolete: bool,
    sup_oids: HashSet<Oid>,
    equality_rule: EqualityRule,
    ordering_rule: OrderingRule,
    substr_rule: SubstringRule,
    syntax: String,
    single_value: bool,
    collective: bool,
    no_user_modification: bool,
    usage: Usage,
    extensions: String,
}

impl PartialEq for Attribute {
    fn eq(&self, other: &Self) -> bool {
        self.numericoid == other.numericoid
    }
}

impl Attribute {
    pub fn get_numericoid(&self) -> &Oid {
        &self.numericoid
    }

    pub fn get_names(&self) -> &HashSet<String> {
        &self.names
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.names.contains(name)
    }

    pub fn has_name(&self, name: &str) -> bool {
        self.names.contains(name)
    }

    pub fn is_single(&self) -> bool {
        self.single_value
    }
}

pub struct AttributeBuilder {
    attribute: Attribute,
}

impl AttributeBuilder {
    pub fn new() -> Self {
        Self {
            attribute: Attribute::default(),
        }
    }

    pub fn set_numericoid(mut self, numericoid: &Oid) -> Self {
        self.attribute.numericoid = numericoid.clone();
        self
    }

    pub fn add_name(mut self, name: &str) -> Self {
        self.attribute.names.insert(name.into());
        self
    }

    pub fn set_desc(mut self, desc: &str) -> Self {
        self.attribute.desc = desc.into();
        self
    }

    pub fn set_obsolete(mut self, obsolete: bool) -> Self {
        self.attribute.obsolete = obsolete;
        self
    }

    pub fn add_sup_oid(mut self, sup_oid: Oid) -> Self {
        self.attribute.sup_oids.insert(sup_oid);
        self
    }

    pub fn set_equality_rule(mut self, eq_rule: EqualityRule) -> Self {
        self.attribute.equality_rule = eq_rule;
        self
    }

    pub fn set_ordering_rule(mut self, ord_rule: OrderingRule) -> Self {
        self.attribute.ordering_rule = ord_rule;
        self
    }

    pub fn set_syntax(mut self, syntax: &str) -> Self {
        self.attribute.syntax = syntax.into();
        self
    }

    pub fn set_single_value(mut self, single_value: bool) -> Self {
        self.attribute.single_value = single_value;
        self
    }

    pub fn set_collective(mut self, collective: bool) -> Self {
        self.attribute.collective = collective;
        self
    }

    pub fn set_no_user_modification(mut self, no_user_modification: bool) -> Self {
        self.attribute.no_user_modification = no_user_modification;
        self
    }

    pub fn set_usage_rule(mut self, usage: Usage) -> Self {
        self.attribute.usage = usage;
        self
    }

    pub fn set_extensions(mut self, extensions: &str) -> Self {
        self.attribute.extensions = extensions.into();
        self
    }

    pub fn build(self) -> Attribute {
        self.attribute
    }
}

#[derive(Debug, Copy, Clone)]
pub enum EqualityRule {
    None,
}

impl Default for EqualityRule {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Copy, Clone)]
pub enum OrderingRule {
    None,
}

impl Default for OrderingRule {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SubstringRule {
    None,
}

impl Default for SubstringRule {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Usage {
    UserApplications,
    DirectoryOperations,
    DistributedOperation,
    DSAOperatoin,
}

impl Default for Usage {
    fn default() -> Self {
        Self::UserApplications
    }
}

#[derive(Debug, Default, Clone)]
pub struct Rdn(Vec<(Oid, String)>);

impl From<Vec<(Oid, String)>> for Rdn {
    fn from(value: Vec<(Oid, String)>) -> Self {
        Rdn(value)
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

pub trait EntryId: Debug + Display + Default + Clone + Eq + Hash {
    fn new_random() -> Self;
    fn root_identifier() -> Self;
}

#[derive(Debug, Default, Clone)]
pub struct Entry<ID: EntryId> {
    _id: Option<ID>,
    parent: String,
    children: Vec<String>,
    object_classes: HashSet<Oid>,
    attributes: HashMap<Oid, HashSet<String>>,
}

impl<ID: EntryId> Entry<ID> {
    pub fn new(
        object_classes: HashSet<Oid>,
        attributes: HashMap<Oid, HashSet<String>>,
    ) -> Entry<ID> {
        Entry {
            object_classes,
            attributes,
            ..Default::default()
        }
    }

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

    pub fn matches_rdn(&self, rdn: &Rdn) -> bool {
        for (oid, val) in rdn {}

        true
    }
}
