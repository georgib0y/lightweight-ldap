use crate::{commands::AddEntryCommand, service::EntryService};

use anyhow::Result;
use rasn_ldap::{
    AddRequest, AddResponse, BindRequest, BindResponse, LdapResult, ProtocolOp, ResultCode,
    SearchRequest,
};

pub trait LdapController {
    fn handle_add_request(&self, req: &AddRequest) -> Result<ProtocolOp>;
    fn handle_bind_request(&self, req: &BindRequest) -> Result<ProtocolOp>;
    fn handle_search_request(&self, req: &SearchRequest) -> Result<ProtocolOp>;
}

pub struct LdapControllerImpl<'a, E: EntryService> {
    entry_service: &'a E,
}

impl<'a, E: EntryService> LdapControllerImpl<'a, E> {
    pub fn new(entry_service: &'a E) -> Self {
        Self { entry_service }
    }
}

impl<'a, E: EntryService> LdapController for LdapControllerImpl<'a, E> {
    fn handle_bind_request(&self, req: &BindRequest) -> Result<ProtocolOp> {
        Ok(ProtocolOp::BindResponse(BindResponse::new(
            ResultCode::Success,
            req.name.to_owned(),
            "not checking passwords, binding largely unimplemented".into(),
            None,
            None,
        )))
    }

    fn handle_add_request(&self, req: &AddRequest) -> Result<ProtocolOp> {
        let command = match AddEntryCommand::try_from(req) {
            Ok(cmd) => cmd,
            Err(err) => {
                return Ok(ProtocolOp::AddResponse(AddResponse(LdapResult::try_from(
                    err,
                )?)))
            }
        };

        match self.entry_service.add_entry(command) {
            Ok(entry) => Ok(ProtocolOp::AddResponse(AddResponse(LdapResult::new(
                ResultCode::Success,
                req.entry.to_owned(),
                "Created new entry".into(),
            )))),
            Err(err) => Ok(ProtocolOp::AddResponse(AddResponse(LdapResult::try_from(
                err,
            )?))),
        }
    }

    fn handle_search_request(&self, req: &SearchRequest) -> Result<ProtocolOp> {
        todo!();
    }
}
