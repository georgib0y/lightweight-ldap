use crate::{
    commands::AddEntryCommand,
    db::EntryRepository,
    entity::{Entry, DN},
    errors::LdapError,
    schema::Schema,
};

pub trait EntryService {
    fn add_entry(&self, command: AddEntryCommand) -> Result<Entry, LdapError>;
}

pub struct EntryServiceImpl<R: EntryRepository> {
    schema: Schema,
    entry_repo: R,
}

impl<R: EntryRepository> EntryService for EntryServiceImpl<R> {
    fn add_entry(&self, command: AddEntryCommand) -> Result<Entry, LdapError> {
        let dn = DN::try_from(command.dn.as_ref())?;

        if self.entry_repo.find_by_dn(&dn).is_some() {
            Err(LdapError::EntryAlreadyExists { dn: command.dn })?
        }

        if !self.entry_repo.dn_parent_exists(&dn) {
            Err(LdapError::EntryDoesNotExists { dn: command.dn })?
        }

        let entry = Entry::new(command.attributes);
        todo!()
    }
}
