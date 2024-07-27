use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use crate::{
    commands::{AddEntryAttributes, AddEntryCommand},
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

        if self.find_by_dn(&dn.parent_dn())?.is_none() {
            return Err(LdapError::EntryDoesNotExists { dn: command.dn });
        }

        let entry_object_classes = self
            .schema_service
            .get_normalised_obj_classes(&command.attributes)?;

        let entry_attributes = self
            .schema_service
            .get_normalised_attributes(&command.attributes)?;

        let entry = Entry::new(entry_object_classes, entry_attributes);
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

        todo!();
        Ok(curr_entry)
    }
}

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
                id: entry.get_id_str(),
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

            self.validate_entry_attribute(entry, &attribute, true, values)?;

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
            .map_err(|_| LdapError::InvalidDN { dn: dn_str.into() })
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
    use std::collections::{HashMap, HashSet};

    use crate::{
        db::InMemSchemaDb,
        entity::{AttributeBuilder, Entry, Kind, ObjectClassBuilder, Oid},
        service::{SchemaService, SchemaServiceImpl},
    };

    #[test]
    fn test_validate_schema() {
        let p_oid: Oid = "person-oid".into();
        let person_class = ObjectClassBuilder::new()
            .set_numericoid(&p_oid)
            .add_name("person")
            .set_kind(Kind::Structural)
            .add_must_attr("cn-oid".into())
            .add_must_attr("sn-oid".into())
            .add_may_attr("upw-oid".into())
            .build();

        let cn_oid: Oid = "cn-oid".into();
        let cn_attr = AttributeBuilder::new()
            .set_numericoid(&cn_oid)
            .add_name("cn")
            .build();
        let sn_oid: Oid = "sn-oid".into();
        let sn_attr = AttributeBuilder::new()
            .set_numericoid(&sn_oid)
            .add_name("sn")
            .build();
        let upw_oid: Oid = "upw-oid".into();
        let user_pw_attr = AttributeBuilder::new()
            .set_numericoid(&upw_oid)
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

        let entry_all_attrs: Entry<String> = Entry::new(
            HashSet::from([p_oid.clone()]),
            HashMap::from([
                ("cn-oid".into(), HashSet::from(["My Name".into()])),
                ("sn-oid".into(), HashSet::from(["Name".into()])),
                ("upw-oid".into(), HashSet::from(["password123".into()])),
            ]),
        );

        schema_service.validate_entry(&entry_all_attrs).unwrap();

        let no_entry: Entry<String> = Entry::new(HashSet::new(), HashMap::new());
        schema_service.validate_entry(&no_entry).unwrap_err();

        let entry_must_attrs: Entry<String> = Entry::new(
            HashSet::from([p_oid.clone()]),
            HashMap::from([
                ("cn-oid".into(), HashSet::from(["My Name".into()])),
                ("sn-oid".into(), HashSet::from(["Name".into()])),
            ]),
        );
        schema_service.validate_entry(&entry_must_attrs).unwrap();

        let entry_missing_must_attr: Entry<String> = Entry::new(
            HashSet::from([p_oid.clone()]),
            HashMap::from([
                ("cn-oid".into(), HashSet::from(["My Name".into()])),
                ("upw-oid".into(), HashSet::from(["password123".into()])),
            ]),
        );

        schema_service
            .validate_entry(&entry_missing_must_attr)
            .unwrap_err();
    }
}
