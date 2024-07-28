use std::marker::PhantomData;

use crate::{
    commands::AddEntryCommand,
    db::EntryRepository,
    entity::{
        dn::{Rdn, DN},
        entry::{Entry, EntryBuilder, EntryId},
    },
    errors::LdapError,
};

use super::schema::SchemaService;

pub enum FindDnResult<ID: EntryId> {
    Found(Entry<ID>),
    NotFound(DN),
}

impl<ID: EntryId> FindDnResult<ID> {
    pub fn is_found(&self) -> bool {
        matches!(self, FindDnResult::Found(_))
    }
}

pub trait EntryService {
    fn add_entry(&self, command: AddEntryCommand) -> Result<Entry<impl EntryId>, LdapError>;
    fn find_by_dn(&self, dn: &DN) -> Result<FindDnResult<impl EntryId>, LdapError>;
    fn get_entry_dn(&self, entry: &Entry<impl EntryId>) -> Result<DN, LdapError>;
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

    pub fn find_dn_recursive(
        &self,
        curr_entry: Entry<ID>,
        rdns: &[Rdn],
    ) -> Result<FindDnResult<ID>, LdapError> {
        if rdns.len() == 1 {
            return if curr_entry.matches_rdn(rdns.first().unwrap()) {
                Ok(FindDnResult::Found(curr_entry))
            } else {
                Ok(FindDnResult::NotFound(DN::default()))
            };
        }

        // check if current entry matches the last rdn in the slice
        let curr_rdn = rdns.last().unwrap();
        if !curr_entry.matches_rdn(curr_rdn) {
            return Ok(FindDnResult::NotFound(DN::default()));
        }

        // if so, try to find the entry recursively for each child
        let next_slice = &rdns[..rdns.len() - 1];

        let mut res: Option<FindDnResult<ID>> = None;

        for child_id in curr_entry.get_children() {
            let child = self
                .entry_repo
                .get_by_id(child_id)
                .ok_or(LdapError::InvalidEntry {
                    id: curr_entry.get_id_str(),
                    msg: format!("Entry has supposed child {} but could not find", child_id),
                })?;

            res = Some(self.find_dn_recursive(child, next_slice)?);
            if matches!(res, Some(FindDnResult::Found(_))) {
                return Ok(res.unwrap());
            }
        }

        let FindDnResult::NotFound(mut dn) = res.unwrap_or(FindDnResult::NotFound(DN::default()))
        else {
            panic!("find dn result somehow found after search")
        };
        dn.append(rdns.last().unwrap().clone());

        Ok(FindDnResult::NotFound(dn))
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

        if self.find_by_dn(&dn)?.is_found() {
            return Err(LdapError::EntryAlreadyExists { dn: command.dn });
        }

        let mut parent = match self.find_by_dn(&dn.parent_dn())? {
            FindDnResult::Found(parent) => parent,
            FindDnResult::NotFound(not_found_dn) => {
                return Err(LdapError::EntryDoesNotExists {
                    dn: not_found_dn.to_string(),
                })
            }
        };

        let entry_object_classes = self
            .schema_service
            .get_normalised_obj_classes(&command.attributes)?;

        let entry_attributes = self
            .schema_service
            .get_normalised_attributes(&command.attributes)?;

        let mut builder = EntryBuilder::new();
        for oid in entry_object_classes {
            builder = builder.add_object_class(oid);
        }

        // add the attributes from the rdn to the entry as per the specs
        for (oid, val) in dn.first().unwrap() {
            builder = builder.add_attr_val(oid.clone(), val)
        }

        for (oid, val) in entry_attributes {
            builder = builder.add_attr_vals(oid, val.into_iter())
        }

        builder = builder.set_parent(parent.get_id().unwrap());

        let entry = builder.build();
        self.schema_service.validate_entry(&entry)?;

        let entry = self.entry_repo.save(entry)?;

        parent.add_child(entry.get_id().unwrap());
        self.entry_repo.save(parent)?;

        Ok(entry)
    }

    fn find_by_dn(&self, dn: &DN) -> Result<FindDnResult<ID>, LdapError> {
        self.find_dn_recursive(self.entry_repo.get_root_entry(), dn.as_slice())
    }

    fn get_entry_dn(&self, _entry: &Entry<impl EntryId>) -> Result<DN, LdapError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::InMemLdapDb,
        entity::{
            dn::{Rdn, DN},
            entry::{Entry, EntryBuilder, EntryId},
        },
        service::{
            entry::{EntryService, FindDnResult},
            schema::SchemaService,
        },
    };

    use super::EntryServiceImpl;

    struct MockSchemaService {}
    impl SchemaService for MockSchemaService {
        fn create_normalised_dn(
            &self,
            _dn: &str,
        ) -> Result<crate::entity::dn::DN, crate::errors::LdapError> {
            unimplemented!()
        }

        fn get_normalised_obj_classes(
            &self,
            _attributes: &crate::commands::AddEntryAttributes,
        ) -> Result<std::collections::HashSet<crate::entity::schema::Oid>, crate::errors::LdapError>
        {
            unimplemented!()
        }

        fn get_normalised_attributes(
            &self,
            _attributes: &crate::commands::AddEntryAttributes,
        ) -> Result<
            std::collections::HashMap<
                crate::entity::schema::Oid,
                std::collections::HashSet<String>,
            >,
            crate::errors::LdapError,
        > {
            unimplemented!()
        }

        fn validate_entry(
            &self,
            _entry: &crate::entity::entry::Entry<impl EntryId>,
        ) -> Result<(), crate::errors::LdapError> {
            unimplemented!()
        }
    }

    #[test]
    fn test_find_by_dn() {
        let grandchild: Entry<u64> = EntryBuilder::new()
            .set_id(u64::new_random_id())
            .add_attr_val("cn-oid", "Grandchild")
            .build();

        let grandsibling: Entry<u64> = EntryBuilder::new()
            .set_id(u64::new_random_id())
            .add_attr_val("cn-oid", "Grandchild's sibling")
            .build();

        let parent: Entry<u64> = EntryBuilder::new()
            .set_id(u64::new_random_id())
            .add_child(grandchild.get_id().unwrap())
            .add_attr_val("ou-oid", "parent")
            .build();

        let p_sibling: Entry<u64> = EntryBuilder::new()
            .set_id(u64::new_random_id())
            .add_child(grandsibling.get_id().unwrap())
            .add_attr_val("ou-oid", "p_sibling")
            .build();

        let root: Entry<u64> = EntryBuilder::new()
            .set_id(u64::root_identifier())
            .add_child(parent.get_id().unwrap())
            .add_child(p_sibling.get_id().unwrap())
            .add_attr_val("dc-oid", "com")
            .build();

        let entry_repo = InMemLdapDb::with_entries(
            vec![grandchild.clone(), grandsibling, parent, p_sibling, root].into_iter(),
        );

        let entry_service = EntryServiceImpl::new(&MockSchemaService {}, &entry_repo);

        let grandchild_dn = DN::new(vec![
            Rdn::from(("cn-oid", "Grandchild")),
            Rdn::from(("ou-oid", "parent")),
            Rdn::from(("dc-oid", "com")),
        ]);

        let res = entry_service.find_by_dn(&grandchild_dn).unwrap();
        assert!(res.is_found());
        let FindDnResult::Found(entry) = res else {
            panic!("entry was not found")
        };
        assert_eq!(entry.get_id(), grandchild.get_id())
    }
}
