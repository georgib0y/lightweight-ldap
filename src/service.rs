use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    marker::PhantomData,
};

use crate::{
    commands::AddEntryCommand,
    db::{EntryRepository, SchemaRepo},
    entity::{Attribute, Entry, EntryId, ObjectClass, Oid, Rdn, DN},
    errors::LdapError,
};

pub trait EntryService {
    fn add_entry(&self, command: AddEntryCommand) -> Result<Entry<impl EntryId>, LdapError>;
    fn find_by_dn(&self, dn: &DN) -> Result<Option<Entry<impl EntryId>>, LdapError>;
}

pub struct EntryServiceImpl<'a, ID, S, R>
where
    ID: EntryId,
    S: SchemaService,
    R: EntryRepository<ID>,
{
    schema_service: &'a S,
    entry_repo: &'a R,
    _entry_id_type: PhantomData<ID>,
}

impl<'a, ID, S, R> EntryServiceImpl<'a, ID, S, R>
where
    ID: EntryId,
    S: SchemaService,
    R: EntryRepository<ID>,
{
    pub fn new(schema_service: &'a S, entry_repo: &'a R) -> Self {
        Self {
            schema_service,
            entry_repo,
            _entry_id_type: PhantomData,
        }
    }
}

impl<'a, ID, S, R> EntryService for EntryServiceImpl<'a, ID, S, R>
where
    ID: EntryId,
    S: SchemaService,
    R: EntryRepository<ID>,
{
    fn add_entry(&self, command: AddEntryCommand) -> Result<Entry<ID>, LdapError> {
        let dn = self.schema_service.create_normalised_dn(&command.dn)?;

        if self.find_by_dn(&dn)?.is_some() {
            return Err(LdapError::EntryAlreadyExists { dn: command.dn });
        }

        if !self.find_by_dn(&dn.parent_dn())?.is_none() {
            return Err(LdapError::EntryDoesNotExists { dn: command.dn });
        }

        let normalised_attributes = self
            .schema_service
            .normalise_attributes(command.attributes)?;

        let entry = Entry::new(normalised_attributes);
        self.schema_service.validate_entry(&entry)?;
        Ok(entry)
    }

    fn find_by_dn(&self, dn: &DN) -> Result<Option<Entry<ID>>, LdapError> {
        let mut curr_entry = Some(self.entry_repo.get_root_entry());

        // TODO this assumes that the root entry and the end of the DN is the same
        // (might not be true)

        for rdn in dn.into_iter().rev().skip(1) {
            // if the entry could not be found
            if curr_entry.is_none() {
                return Ok(None);
            }
        }

        Ok(curr_entry)
    }
}

pub trait SchemaService {
    fn create_normalised_dn(&self, dn: &str) -> Result<DN, LdapError>;
    fn normalise_attributes(
        &self,
        attributes: HashMap<String, HashSet<String>>,
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

    fn get_obj_classes_for_entry(
        &self,
        entry: &Entry<impl EntryId>,
    ) -> Result<Vec<&ObjectClass>, LdapError> {
        entry
            .get_attribute("objectClass")
            .ok_or(LdapError::InvalidEntry {
                id: entry
                    .get_id()
                    .map(|e| e.to_string())
                    .unwrap_or("None".to_string()),
                msg: "Entry has no object classes".into(),
            })?
            .iter()
            .map(|o| self.schema_repo.find_object_class_by_name(o))
            .collect::<Option<Vec<_>>>()
            .ok_or(LdapError::InvalidEntry {
                id: entry
                    .get_id()
                    .map(|e| e.to_string())
                    .unwrap_or("None".to_string()),
                msg: "Entry has an object class not specified in schema".into(),
            })
    }

    fn get_obj_classes_attrs(
        &self,
        obj_classes: &[&ObjectClass],
    ) -> Result<(Vec<&Attribute>, Vec<&Attribute>), LdapError> {
        let mut must_attrs = Vec::new();
        let mut may_attrs = Vec::new();
        for o in obj_classes {
            let mut must = self
                .schema_repo
                .get_attributes(o.get_must_attrs().iter())
                .ok_or(LdapError::InvalidSchema(format!(
                    "Could not find a must attribute for {} ",
                    o.get_numericoid()
                )))?;
            let mut may = self
                .schema_repo
                .get_attributes(o.get_may_attrs().iter())
                .ok_or(LdapError::InvalidSchema(format!(
                    "Could not find a mayattribute for {} ",
                    o.get_numericoid()
                )))?;
            must_attrs.append(&mut must);
            may_attrs.append(&mut may);
        }

        Ok((must_attrs, may_attrs))
    }

    fn validate_structural_count(
        &self,
        entry: &Entry<impl EntryId>,
        e_obj_classes: &[&ObjectClass],
    ) -> Result<(), LdapError> {
        // check that the entry has exactly one structural object class
        let structural_count = e_obj_classes.iter().filter(|o| o.is_structural()).count();
        if structural_count != 1 {
            Err(LdapError::InvalidEntry {
                id: entry
                    .get_id()
                    .map(|e| e.to_string())
                    .unwrap_or("None".into()),
                msg: format!("Expected 1 structural obj class, got {}", structural_count),
            })?
        }
        Ok(())
    }

    fn validate_entry_missing_attrs(
        &self,
        entry: &Entry<impl EntryId>,
        must_attrs: &[&Attribute],
    ) -> Result<(), LdapError> {
        // check all entry contains all must attrs
        let missing_must = must_attrs
            .iter()
            .find(|a| entry.get_attribute(a.get_numericoid()).is_none());

        if let Some(missing) = missing_must {
            return Err(LdapError::InvalidEntry {
                id: entry
                    .get_id()
                    .map(|e| e.to_string())
                    .unwrap_or("None".into()),
                msg: format!(
                    "Entry does not contain '{}' must attr",
                    missing.get_numericoid()
                ),
            });
        }

        Ok(())
    }

    fn validate_entry_attributes(
        &self,
        entry: &Entry<impl EntryId>,
        must_attrs: &[&Attribute],
        may_attrs: &[&Attribute],
    ) -> Result<(), LdapError> {
        self.validate_entry_missing_attrs(entry, must_attrs)?;

        'outter: for (e_attr_oid, e_attr_v) in entry.get_attributes() {
            if e_attr_k == "objectClass" {
                continue;
            }

            for must_attr in must_attrs.iter() {
                if !must_attr.contains_name(e_attr_k) {
                    continue;
                }

                self.validate_entry_attribute(entry.get_id(), must_attr, true, e_attr_v)?;
                continue 'outter;
            }

            for may_attr in may_attrs.iter() {
                if !may_attr.contains_name(e_attr_k) {
                    continue;
                }

                self.validate_entry_attribute(entry.get_id(), may_attr, false, e_attr_v)?;
                continue 'outter;
            }

            return Err(LdapError::InvalidEntry {
                id: entry
                    .get_id()
                    .map(|e| e.to_string())
                    .unwrap_or("None".into()),
                msg: format!("Entry contains an unexpected attr: {}", e_attr_k),
            });
        }

        Ok(())
    }

    fn validate_entry_attribute(
        &self,
        attribute: &Attribute,
        is_must_attr: bool,
        values: &HashSet<String>,
    ) -> Result<(), LdapError> {
        let val_count = values.len();

        // if is_must_attr && val_count == 0 {
        //     return Err(LdapError::InvalidEntry {
        //         id: entry_id.into(),
        //         msg: format!(
        //             "Entry has no attributes for must attr {}",
        //             attribute.get_numericoid()
        //         ),
        //     });
        // }

        // if attribute.is_single() && val_count > 1 {
        //     return Err(LdapError::InvalidEntry {
        //         id: entry_id.into(),
        //         msg: format!(
        //             "Entry has too many values for single attr {}: {}",
        //             attribute.get_numericoid(),
        //             val_count
        //         ),
        //     });
        // }

        todo!()
        // Ok(())
    }

    fn create_normalised_rdn(&self, rdn_str: &str) -> Result<Rdn, String> {
        let mut rdn = Vec::new();
        for att_val in rdn_str.split('+') {
            let (att, val) = att_val.split_once('=').ok_or(rdn_str)?;
            let oid = self
                .schema_repo
                .find_attribute_by_name(att)
                .map(|a| a.get_numericoid())
                .ok_or(rdn_str)?;
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
                id: entry
                    .get_id()
                    .map(|e| e.to_string())
                    .unwrap_or("No ID".into()),
                msg: "Could not get all object classes for entry".into(),
            })
    }

    fn validate_entry_by_oc(
        &self,
        oc: &ObjectClass,
        entry: &Entry<impl EntryId>,
    ) -> Result<(), LdapError> {
        todo!()
    }
}

impl<'a, R: SchemaRepo> SchemaService for SchemaServiceImpl<'a, R> {
    fn validate_entry(&self, entry: &Entry<impl EntryId>) -> Result<(), LdapError> {
        let object_classes = self.get_entry_object_classes(entry)?;

        for oc in object_classes {
            self.validate_entry_by_oc(oc, entry)?;
        }

        self.validate_structural_count(entry, &object_classes)?;

        let (must_attrs, may_attrs) = self.get_obj_classes_attrs(&object_classes)?;

        self.validate_entry_attributes(entry, &must_attrs, &may_attrs)?;

        Ok(())
    }

    fn create_normalised_dn(&self, dn_str: &str) -> Result<DN, LdapError> {
        dn_str
            .split(',')
            .map(|rdn| self.create_normalised_rdn(rdn))
            .collect::<Result<Vec<_>, String>>()
            .map(DN::new)
            .map_err(|_| LdapError::InvalidDN { dn: dn_str.into() })
    }

    fn normalise_attributes(
        &self,
        attributes: HashMap<String, HashSet<String>>,
    ) -> Result<HashMap<Oid, HashSet<String>>, LdapError> {
        let mut normalised = HashMap::new();

        for (a, v) in attributes {
            let attr = self
                .schema_repo
                .find_attribute_by_name(&a)
                .ok_or(LdapError::UnknownAttribute(a))?;
            normalised.insert(attr.get_numericoid().into(), v);
        }

        Ok(normalised)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::{
        db::InMemSchemaDb,
        entity::{AttributeBuilder, Entry, Kind, ObjectClassBuilder},
        service::{SchemaService, SchemaServiceImpl},
    };

    #[test]
    fn test_validate_schema() {
        let person_class = ObjectClassBuilder::new()
            .add_name("person")
            .set_kind(Kind::Structural)
            .add_must_attr("cn-oid".into())
            .add_must_attr("sn-oid".into())
            .add_may_attr("userPassword-oid".into())
            .build();

        let cn_attr = AttributeBuilder::new().add_name("cn").build();
        let sn_attr = AttributeBuilder::new().add_name("sn").build();
        let user_pw_attr = AttributeBuilder::new().add_name("userPassword").build();

        let schema_db =
            InMemSchemaDb::new(vec![person_class], vec![cn_attr, sn_attr, user_pw_attr]);

        let schema_service = SchemaServiceImpl {
            schema_repo: &schema_db,
        };

        let entry_all_attrs: Entry<String> = Entry::new(HashMap::from([
            ("objectClass".into(), HashSet::from(["person".into()])),
            ("cn".into(), HashSet::from(["My Name".into()])),
            ("sn".into(), HashSet::from(["Name".into()])),
            ("userPassword".into(), HashSet::from(["password123".into()])),
        ]));

        schema_service.validate_entry(&entry_all_attrs).unwrap();

        let no_entry: Entry<String> = Entry::new(HashMap::new());
        schema_service.validate_entry(&no_entry).unwrap_err();

        let entry_must_attrs: Entry<String> = Entry::new(HashMap::from([
            ("objectClass".into(), HashSet::from(["person".into()])),
            ("cn".into(), HashSet::from(["My Name".into()])),
            ("sn".into(), HashSet::from(["Name".into()])),
        ]));
        schema_service.validate_entry(&entry_must_attrs).unwrap();

        let entry_missing_must_attr: Entry<String> = Entry::new(HashMap::from([
            ("object_class".into(), HashSet::from(["person".into()])),
            ("cn".into(), HashSet::from(["My Name".into()])),
            ("userPassword".into(), HashSet::from(["password123".into()])),
        ]));

        schema_service
            .validate_entry(&entry_missing_must_attr)
            .unwrap_err();
    }
}
