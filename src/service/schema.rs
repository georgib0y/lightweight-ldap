use std::collections::{HashMap, HashSet};

use crate::{
    commands::AddEntryAttributes,
    db::SchemaRepo,
    entity::{
        dn::{Rdn, DN},
        entry::{Entry, EntryId},
        schema::{Attribute, ObjectClass, Oid},
    },
    errors::LdapError,
};

pub trait SchemaService {
    fn create_normalised_dn(&self, dn: &str) -> Result<DN, LdapError>;
    fn get_normalised_obj_classes(
        &self,
        attributes: &AddEntryAttributes,
    ) -> Result<HashSet<Oid>, LdapError>;
    fn get_normalised_attributes(
        &self,
        attributes: &AddEntryAttributes,
    ) -> Result<HashMap<Oid, HashSet<String>>, LdapError>;
    fn validate_entry(&self, entry: &Entry<impl EntryId>) -> Result<(), LdapError>;
}

pub struct SchemaServiceImpl<'a, R: SchemaRepo> {
    schema_repo: &'a R,
}

impl<'a, R: SchemaRepo> SchemaServiceImpl<'a, R> {
    pub fn new(schema_repo: &'a R) -> Self {
        Self { schema_repo }
    }

    fn count_stuctural_obj_classes(
        &self,
        entry: &Entry<impl EntryId>,
        e_obj_classes: &[&ObjectClass],
    ) -> Result<(), LdapError> {
        // check that the entry has exactly one structural object class
        let structural_count = e_obj_classes.iter().filter(|o| o.is_structural()).count();
        if structural_count != 1 {
            Err(LdapError::InvalidEntry {
                id: entry.get_id_str(),
                msg: format!("Expected 1 structural obj class, got {}", structural_count),
            })?
        }
        Ok(())
    }

    fn validate_entry_attributes(
        &self,
        entry: &Entry<impl EntryId>,
        obj_classes: &[&ObjectClass],
    ) -> Result<(), LdapError> {
        let mut e_attrs = entry.get_attributes().clone();

        let mut must_attrs = HashSet::new();
        must_attrs.extend(obj_classes.iter().flat_map(|oc| oc.get_must_attrs().iter()));

        for must in must_attrs {
            let values = e_attrs.get(must).ok_or(LdapError::InvalidEntry {
                id: entry.get_id_str(),
                msg: format!("Entry contains is missing a must attr: {}", must),
            })?;

            let attribute = self
                .schema_repo
                .get_attribute(must)
                .ok_or(LdapError::InvalidSchema(must.to_string()))?;

            self.validate_entry_attribute(entry, attribute, true, values)?;

            e_attrs.remove(must);
        }

        let mut may_attrs = HashSet::new();
        may_attrs.extend(obj_classes.iter().flat_map(|oc| oc.get_may_attrs().iter()));

        for (attr, values) in e_attrs {
            if !may_attrs.contains(&attr) {
                return Err(LdapError::InvalidEntry {
                    id: entry.get_id_str(),
                    msg: format!("Entry contains an attr not in the schema: {}", attr),
                });
            }

            let attribute = self
                .schema_repo
                .get_attribute(&attr)
                .ok_or(LdapError::InvalidSchema(attr.to_string()))?;

            self.validate_entry_attribute(entry, attribute, true, &values)?;
        }

        Ok(())
    }

    fn validate_entry_attribute(
        &self,
        entry: &Entry<impl EntryId>,
        attribute: &Attribute,
        is_must: bool,
        values: &HashSet<String>,
    ) -> Result<(), LdapError> {
        let val_count = values.len();

        if is_must && val_count == 0 {
            return Err(LdapError::InvalidEntry {
                id: entry.get_id_str(),
                msg: format!(
                    "Entry missing attributes for must attr {}",
                    attribute.get_numericoid()
                ),
            });
        }

        if attribute.is_single() && val_count > 1 {
            return Err(LdapError::InvalidEntry {
                id: entry.get_id_str(),
                msg: format!(
                    "Entry has too many values for single attr {}: {}",
                    attribute.get_numericoid(),
                    val_count
                ),
            });
        }

        Ok(())
    }

    fn create_normalised_rdn(&self, rdn_str: &str) -> Result<Rdn, String> {
        let mut rdn = Vec::new();
        for att_val in rdn_str.split('+') {
            let (att, val) = att_val.split_once('=').ok_or(rdn_str)?;
            let oid = self
                .schema_repo
                .find_attribute_by_name(att)
                .map(|a| a.get_numericoid())
                .ok_or(format!("Could not find attribute {} for {}", att, rdn_str))?;
            rdn.push((oid.clone(), val.into()))
        }

        Ok(Rdn::from(rdn))
    }

    fn get_entry_object_classes(
        &self,
        entry: &Entry<impl EntryId>,
    ) -> Result<Vec<&ObjectClass>, LdapError> {
        self.schema_repo
            .get_entry_object_classes(entry)
            .ok_or(LdapError::InvalidEntry {
                id: entry.get_id_str(),
                msg: "Could not get all object classes for entry".into(),
            })
    }
}

impl<'a, R: SchemaRepo> SchemaService for SchemaServiceImpl<'a, R> {
    fn validate_entry(&self, entry: &Entry<impl EntryId>) -> Result<(), LdapError> {
        let object_classes = self.get_entry_object_classes(entry)?;
        self.count_stuctural_obj_classes(entry, &object_classes)?;
        self.validate_entry_attributes(entry, &object_classes)?;

        Ok(())
    }

    fn create_normalised_dn(&self, dn_str: &str) -> Result<DN, LdapError> {
        dn_str
            .split(',')
            .map(|rdn| self.create_normalised_rdn(rdn))
            .collect::<Result<Vec<_>, String>>()
            .map(DN::new)
            .map_err(|msg| LdapError::InvalidDN {
                dn: dn_str.into(),
                msg,
            })
    }

    fn get_normalised_obj_classes(
        &self,
        attributes: &AddEntryAttributes,
    ) -> Result<HashSet<Oid>, LdapError> {
        let oids = attributes
            .get("objectClass")
            .ok_or(LdapError::UnknownObjectClass(
                "No object classes specified".into(),
            ))?
            .iter()
            .map(|name| {
                self.schema_repo
                    .find_object_class_by_name(name)
                    .ok_or(LdapError::UnknownObjectClass(name.into()))
            })
            .collect::<Result<Vec<_>, LdapError>>()?;

        let mut object_classes = HashSet::new();
        object_classes.extend(oids.iter().map(|oc| oc.get_numericoid().clone()));
        Ok(object_classes)
    }

    fn get_normalised_attributes(
        &self,
        attributes: &AddEntryAttributes,
    ) -> Result<HashMap<Oid, HashSet<String>>, LdapError> {
        let mut attrs = HashMap::new();

        for (a, v) in attributes {
            if a == "objectClass" {
                continue;
            }

            let attr = self
                .schema_repo
                .find_attribute_by_name(a)
                .ok_or(LdapError::UnknownAttribute(a.into()))?;

            attrs.insert(attr.get_numericoid().clone(), v.clone());
        }

        Ok(attrs)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        db::InMemSchemaDb,
        entity::entry::Entry,
        entity::{
            entry::EntryBuilder,
            schema::{AttributeBuilder, Kind, ObjectClassBuilder, Oid},
        },
        service::schema::{SchemaService, SchemaServiceImpl},
    };

    #[test]
    fn test_validate_schema() {
        let p_oid: Oid = "person-oid".into();
        let person_class = ObjectClassBuilder::new()
            .set_numericoid(p_oid.clone())
            .add_name("person")
            .set_kind(Kind::Structural)
            .add_must_attr("cn-oid")
            .add_must_attr("sn-oid")
            .add_may_attr("upw-oid")
            .build();

        let cn_oid: Oid = "cn-oid".into();
        let cn_attr = AttributeBuilder::new()
            .set_numericoid(cn_oid.clone())
            .add_name("cn")
            .build();
        let sn_oid: Oid = "sn-oid".into();
        let sn_attr = AttributeBuilder::new()
            .set_numericoid(sn_oid.clone())
            .add_name("sn")
            .build();
        let upw_oid: Oid = "upw-oid".into();
        let user_pw_attr = AttributeBuilder::new()
            .set_numericoid(upw_oid.clone())
            .add_name("userPassword")
            .build();

        let schema_db = InMemSchemaDb::new(
            HashMap::from([(p_oid.clone(), person_class)]),
            HashMap::from([
                (cn_oid, cn_attr),
                (sn_oid, sn_attr),
                (upw_oid, user_pw_attr),
            ]),
        );

        let schema_service = SchemaServiceImpl {
            schema_repo: &schema_db,
        };

        let entry_all_attrs: Entry<String> = EntryBuilder::new()
            .add_object_class(p_oid.clone())
            .add_attr_val("cn-oid", "My Name")
            .add_attr_val("sn-oid", "Name")
            .add_attr_val("upw-oid", "password123")
            .build();

        schema_service.validate_entry(&entry_all_attrs).unwrap();

        let no_entry: Entry<String> = EntryBuilder::new().build();

        schema_service.validate_entry(&no_entry).unwrap_err();

        let entry_must_attrs: Entry<String> = EntryBuilder::new()
            .add_object_class(p_oid.clone())
            .add_attr_val("cn-oid", "My Name")
            .add_attr_val("sn-oid", "Name")
            .build();

        schema_service.validate_entry(&entry_must_attrs).unwrap();

        let entry_missing_must_attr: Entry<String> = EntryBuilder::new()
            .add_object_class(p_oid)
            .add_attr_val("cn-oid", "My Name")
            .add_attr_val("upw-oid", "password123")
            .build();

        schema_service
            .validate_entry(&entry_missing_must_attr)
            .unwrap_err();
    }

    #[test]
    fn test_create_normalised_dn() {}
}
