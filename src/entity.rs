use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::errors::LdapError;

#[derive(Debug, Default, Clone)]
pub struct ObjectClass {
    numericoid: String,
    names: Vec<String>,
    desc: String,
    obsolete: bool,
    sup_oids: Vec<String>,
    kind: Kind,
    must_attrs: Vec<String>,
    may_attrs: Vec<String>,
}

impl ObjectClass {
    pub fn get_names(&self) -> &Vec<String> {
        &self.names
    }

    pub fn add_name(&mut self, name: &str) {
        self.names.push(name.into())
    }

    pub fn has_name(&self, name: &str) -> bool {
        self.names.iter().any(|n| n == name)
    }

    pub fn is_structural(&self) -> bool {
        self.kind.is_structural()
    }

    pub fn get_must_attrs(&self) -> &Vec<String> {
        &self.must_attrs
    }

    pub fn get_may_attrs(&self) -> &Vec<String> {
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

    pub fn set_numericoid(mut self, numericoid: &str) -> Self {
        self.obj_class.numericoid = numericoid.into();
        self
    }

    pub fn add_name(mut self, name: &str) -> Self {
        self.obj_class.names.push(name.into());
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

    pub fn add_sup_oid(mut self, sup_oid: &str) -> Self {
        self.obj_class.sup_oids.push(sup_oid.into());
        self
    }

    pub fn set_kind(mut self, kind: Kind) -> Self {
        self.obj_class.kind = kind;
        self
    }

    pub fn add_must_attr(mut self, must_attr: &str) -> Self {
        self.obj_class.must_attrs.push(must_attr.into());
        self
    }

    pub fn add_may_attr(mut self, may_attr: &str) -> Self {
        self.obj_class.may_attrs.push(may_attr.into());
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
    numericoid: String,
    names: Vec<String>,
    desc: String,
    obsolete: bool,
    sup_oids: Vec<String>,
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

impl Attribute {
    pub fn get_names(&self) -> &Vec<String> {
        &self.names
    }

    pub fn has_name(&self, name: &str) -> bool {
        self.names.iter().any(|n| n == name)
    }

    pub fn validate_values(&self, required: bool, values: Vec<&str>) -> Result<()> {
        todo!()
        // if required && values.is_empty() {
        //     bail!("values is empty")
        // }

        // if self.single_value && values.len() > 1 {
        //     bail!("multiple values set for a single value attribute")
        // }

        // Ok(())
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

    pub fn set_numericoid(mut self, numericoid: &str) -> Self {
        self.attribute.numericoid = numericoid.into();
        self
    }

    pub fn add_name(mut self, name: &str) -> Self {
        self.attribute.names.push(name.into());
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

    pub fn add_sup_oid(mut self, sup_oid: &str) -> Self {
        self.attribute.sup_oids.push(sup_oid.into());
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

    pub fn get_attribute(&self, attr_name: &str) -> Option<&HashSet<String>> {
        self.attributes.get(attr_name)
    }
}
