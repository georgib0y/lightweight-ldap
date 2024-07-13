use std::collections::{HashMap, HashSet};

use crate::{
    commands::AddEntryCommand,
    db::{EntryRepository, InMemSchemaDb, SchemaRepo},
    entity::{AttributeBuilder, Entry, Kind, ObjectClass, ObjectClassBuilder, DN},
    errors::LdapError,
};

pub trait EntryService {
    fn add_entry(&self, command: AddEntryCommand) -> Result<Entry, LdapError>;
}

pub struct EntryServiceImpl<'a, S: SchemaService, R: EntryRepository> {
    schema_service: &'a S,
    entry_repo: &'a R,
}

impl<'a, S: SchemaService, R: EntryRepository> EntryServiceImpl<'a, S, R> {
    pub fn new(schema_service: &'a S, entry_repo: &'a R) -> Self {
        Self {
            schema_service,
            entry_repo,
        }
    }
}

impl<'a, S: SchemaService, R: EntryRepository> EntryService for EntryServiceImpl<'a, S, R> {
    fn add_entry(&self, command: AddEntryCommand) -> Result<Entry, LdapError> {
        let dn = DN::try_from(command.dn.as_ref())?;

        if self.entry_repo.find_by_dn(&dn).is_some() {
            Err(LdapError::EntryAlreadyExists { dn: command.dn })?
        }

        if !self.entry_repo.dn_parent_exists(&dn) {
            Err(LdapError::EntryDoesNotExists { dn: command.dn })?
        }

        let entry = Entry::new(command.attributes);
        self.schema_service.validate_entry(&entry)?;
        Ok(entry)
    }
}

pub trait SchemaService {
    fn validate_entry(&self, entry: &Entry) -> Result<(), LdapError>;
}

pub struct SchemaServiceImpl<'a, R: SchemaRepo> {
    schema_repo: &'a R,
}

impl<'a, R: SchemaRepo> SchemaServiceImpl<'a, R> {
    pub fn new(schema_repo: &'a R) -> Self {
        Self { schema_repo }
    }

    fn get_obj_classes_for_entry(&self, entry: &Entry) -> Result<Vec<&ObjectClass>, LdapError> {
        entry
            .get_attribute("objectClass")
            .ok_or(LdapError::InvalidEntry {
                id: entry.get_id().into(),
                msg: "Entry has no object classes".into(),
            })?
            .iter()
            .map(|o| self.schema_repo.find_object_class_by_name(o))
            .collect::<Option<Vec<_>>>()
            .ok_or(LdapError::InvalidEntry {
                id: entry.get_id().into(),
                msg: "Entry has an object class not specified in schema".into(),
            })
    }
}

impl<'a, R: SchemaRepo> SchemaService for SchemaServiceImpl<'a, R> {
    fn validate_entry(&self, entry: &Entry) -> Result<(), LdapError> {
        let e_obj_class = self.get_obj_classes_for_entry(entry)?;

        // check that the entry has exactly one structural object class
        let structural_count = e_obj_class.iter().filter(|o| o.is_structural()).count();
        if structural_count != 1 {
            Err(LdapError::InvalidEntry {
                id: entry.get_id().into(),
                msg: format!("Expected 1 structural obj class, got {}", structural_count),
            })?
        }

        let must_attrs: Vec<_> = e_obj_class
            .iter()
            .flat_map(|o| o.get_must_attrs())
            .collect();
        let may_attrs: Vec<_> = e_obj_class.iter().flat_map(|o| o.get_may_attrs()).collect();

        let e_attr_keys: Vec<_> = entry.get_attributes().keys().collect();

        // checks that all must attributes exist for entry
        for must in must_attrs {
            if !e_attr_keys.contains(&must) {
                Err(LdapError::InvalidEntry {
                    id: entry.get_id().into(),
                    msg: format!("Entry is missing a MUST attr {}", must),
                })?;
            }
        }

        // check that no attrs have been specified in entry that are not in schema
        for e_attr in e_attr_keys {
            if !must_attrs.contains(&e_attr) && !may_attrs.contains(&e_attr) {
                Err(LdapError::InvalidEntry {
                    id: entry.get_id().into(),
                    msg: format!("Entry is has an attr not in schema {}", e_attr),
                })?;
            }
        }

        Ok(())
    }
}

#[test]
fn test_validate_schema() {
    let mut person_class = ObjectClassBuilder::new()
        .add_name("person")
        .set_kind(Kind::Structural)
        .add_must_attr("cn")
        .add_must_attr("sn")
        .add_may_attr("userPassword")
        .build();

    let mut cn_attr = AttributeBuilder::new().add_name("cn").build();
    let mut sn_attr = AttributeBuilder::new().add_name("sn").build();
    let mut user_pw_attr = AttributeBuilder::new().add_name("userPassword").build();

    let schema_db = InMemSchemaDb::new(vec![person_class], vec![cn_attr, sn_attr, user_pw_attr]);
    let schema_service = SchemaServiceImpl {
        schema_repo: &schema_db,
    };

    let entry_all_attrs = Entry::new(HashMap::from([
        ("objectClass".into(), HashSet::from(["person".into()])),
        ("cn".into(), HashSet::from(["My Name".into()])),
        ("sn".into(), HashSet::from(["Name".into()])),
        ("userPassword".into(), HashSet::from(["password123".into()])),
    ]));

    assert!(
        schema_service.validate_entry(&entry_all_attrs).is_ok(),
        "failed to validate all attrs"
    );

    let no_entry = Entry::new(HashMap::new());
    assert!(
        schema_service.validate_entry(&no_entry).is_err(),
        "failed to validate no attrs"
    );

    let entry_must_attrs = Entry::new(HashMap::from([
        ("objectClass".into(), HashSet::from(["person".into()])),
        ("cn".into(), HashSet::from(["My Name".into()])),
        ("sn".into(), HashSet::from(["Name".into()])),
    ]));
    assert!(
        schema_service.validate_entry(&entry_must_attrs).is_ok(),
        "failed to validate only must attrs"
    );

    let entry_missing_must_attr = Entry::new(HashMap::from([
        ("object_class".into(), HashSet::from(["person".into()])),
        ("cn".into(), HashSet::from(["My Name".into()])),
        ("userPassword".into(), HashSet::from(["password123".into()])),
    ]));

    assert!(
        schema_service
            .validate_entry(&entry_missing_must_attr)
            .is_err(),
        "failed to validate missing must"
    )
}
