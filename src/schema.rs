#![allow(unused)]

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, bail, Result};

use crate::{entity::Entry, errors::LdapError};

#[derive(Debug, Clone)]
pub struct Schema {
    object_classes: Vec<ObjectClass>,
    attributes: Vec<Attribute>,
}

impl Schema {
    pub fn new() -> Schema {
        Schema {
            object_classes: Vec::new(),
            attributes: Vec::new(),
        }
    }

    pub fn find_object_class_by_name(&self, name: &str) -> Option<&ObjectClass> {
        self.object_classes
            .iter()
            .find(|o| o.names.iter().any(|n| n == name))
    }

    pub fn find_attribute_by_name(&self, name: &str) -> Option<&Attribute> {
        self.attributes
            .iter()
            .find(|a| a.names.iter().any(|n| n == name))
    }

    pub fn validate_entry(&self, entry: &Entry) -> Result<(), LdapError> {
        // find object classes, ensure only one structural one
        let obj_class_set =
            entry
                .get_attributes()
                .get("objectClass")
                .ok_or(LdapError::InvalidEntry {
                    id: entry.get_id().into(),
                    msg: "Entry has no object classes".into(),
                })?;

        let object_classes = obj_class_set
            .iter()
            .map(|o| self.find_object_class_by_name(o))
            .collect::<Option<Vec<_>>>()
            .ok_or(LdapError::InvalidEntry {
                id: entry.get_id().into(),
                msg: "Entry has an object class not specified in schema".into(),
            })?;

        dbg!(&object_classes);

        let structural_count = object_classes
            .iter()
            .filter(|c| matches!(c.kind, Kind::Structural))
            .count();

        if structural_count != 1 {
            Err(LdapError::InvalidEntry {
                id: entry.get_id().into(),
                msg: format!("Expected 1 structural obj class, got {}", structural_count),
            })?
        }

        // find all attrs for all classes (must/may)
        let must_attrs = object_classes
            .iter()
            .flat_map(|o| o.must_attrs.iter())
            .map(|a| self.find_attribute_by_name(a))
            .collect::<Option<Vec<_>>>()
            .ok_or(LdapError::InvalidEntry {
                id: entry.get_id().into(),
                msg: "Entry is missing a MUST attr specified in schema".into(),
            })?;

        dbg!(&must_attrs);

        let may_attrs = object_classes
            .iter()
            .flat_map(|o| o.may_attrs.iter())
            .map(|a| self.find_attribute_by_name(a))
            .collect::<Option<Vec<_>>>()
            .ok_or(LdapError::InvalidEntry {
                id: entry.get_id().into(),
                msg: "Entry has an MAY attr not specified in schema".into(),
            })?;

        dbg!(&may_attrs);

        // check all must attrs exist and are valid (single/multi)
        for must_attr in must_attrs {
            // get the entry by any of the attr names
            let e_attr_values: Vec<&str> = must_attr
                .names
                .iter()
                .filter_map(|n| entry.get_attributes().get(n))
                .flatten()
                .map(|s| s.as_ref())
                .collect();

            // validate each
            must_attr.validate_values(true, e_attr_values)?;
        }

        for may_attr in may_attrs {
            // get the entry by any of the attr names
            let e_attr_values: Vec<&str> = may_attr
                .names
                .iter()
                .filter_map(|n| entry.get_attributes().get(n))
                .flatten()
                .map(|s| s.as_ref())
                .collect();

            // validate each
            may_attr.validate_values(false, e_attr_values)?;
        }
        // check any may attrs are valid (single multi)

        // TODO obviously more rigorous validation is needed
        Ok(())
    }
}

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

#[derive(Debug, Copy, Clone)]
pub enum Kind {
    Abstract,
    Structural,
    Auxiliary,
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
    pub fn validate_values(&self, required: bool, values: Vec<&str>) -> Result<()> {
        if required && values.is_empty() {
            bail!("values is empty")
        }

        if self.single_value && values.len() > 1 {
            bail!("multiple values set for a single value attribute")
        }

        Ok(())
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

#[test]
fn test_validate_schema() {
    let mut person_class = ObjectClass::default();
    person_class.names.push("person".into());
    person_class.kind = Kind::Structural;
    person_class.must_attrs.push("cn".into());
    person_class.must_attrs.push("sn".into());
    person_class.may_attrs.push("userPassword".into());

    let mut cn_attr = Attribute::default();
    cn_attr.names.push("cn".into());

    let mut sn_attr = Attribute::default();
    sn_attr.names.push("sn".into());

    let mut user_pw_attr = Attribute::default();
    user_pw_attr.names.push("userPassword".into());

    let schema = Schema {
        object_classes: vec![person_class],
        attributes: vec![cn_attr, sn_attr, user_pw_attr],
    };

    let entry_all_attrs = LdapEntry::new(
        HashSet::from(["person".into()]),
        HashMap::from([
            ("cn".into(), vec!["My Name".into()]),
            ("sn".into(), vec!["Name".into()]),
            ("userPassword".into(), vec!["password123".into()]),
        ]),
    );
    assert!(
        schema.validate_entry(&entry_all_attrs).is_ok(),
        "failed to validate all attrs"
    );

    let no_entry = LdapEntry::new(HashSet::new(), HashMap::new());
    assert!(
        schema.validate_entry(&no_entry).is_err(),
        "failed to validate no attrs"
    );

    let entry_must_attrs = LdapEntry::new(
        HashSet::from(["person".into()]),
        HashMap::from([
            ("cn".into(), vec!["My Name".into()]),
            ("sn".into(), vec!["Name".into()]),
        ]),
    );
    assert!(
        schema.validate_entry(&entry_must_attrs).is_ok(),
        "failed to validate only must attrs"
    );

    let entry_missing_must_attr = LdapEntry::new(
        HashSet::from(["person".into()]),
        HashMap::from([
            ("cn".into(), vec!["My Name".into()]),
            ("userPassword".into(), vec!["password123".into()]),
        ]),
    );

    assert!(
        schema.validate_entry(&entry_missing_must_attr).is_err(),
        "failed to validate missing must"
    )
}
