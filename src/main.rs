use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use anyhow::Result;
use db::{DistinguishedName, EntryRepository, InMemLdapDb};
use rasn::ber::{de, enc};
use rasn::error::DecodeError;
use rasn::prelude::*;
use rasn_ldap::{
    AddRequest, AddResponse, Attribute, BindRequest, BindResponse, LdapMessage, LdapResult,
    ProtocolOp, ResultCode, SearchRequest, UnbindRequest,
};
use schema::Schema;

use crate::db::LdapEntry;

mod commands;
mod controller;
mod db;
mod entity;
mod errors;
mod infrastructure;
mod schema;
mod service;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000")?;

    let schema = Schema::new();
    let db = InMemLdapDb::new();

    let (stream, _) = listener.accept()?;
    let mut conn = LdapController::new(schema, db, stream);

    loop {
        let msg = conn.read_msg().unwrap();
        conn.handle_ldap_message(msg).unwrap();
    }
}

impl<R: EntryRepository> LdapController<R> {
    fn handle_add_request(&mut self, req: &AddRequest) -> ProtocolOp {
        // get dn, check parent exists
        let dn = match DistinguishedName::try_from(req.entry.to_owned()) {
            Ok(dn) => dn,
            Err(_) => {
                return ProtocolOp::AddResponse(AddResponse(LdapResult::new(
                    ResultCode::InvalidDnSyntax,
                    req.entry.to_owned(),
                    "Could not parse dn".into(),
                )))
            }
        };

        if !self.repo.dn_parent_exists(&dn) {
            return ProtocolOp::AddResponse(AddResponse(LdapResult::new(
                ResultCode::NoSuchObject,
                req.entry.to_owned(),
                "Parent does not exist".into(),
            )));
        }

        // check that an entry doesnt already exist for this dn
        if self.repo.find_by_dn(&dn).is_some() {
            return ProtocolOp::AddResponse(AddResponse(LdapResult::new(
                ResultCode::EntryAlreadyExists,
                req.entry.to_owned(),
                "Entry with that dn already exists".into(),
            )));
        }

        let (obj_classes, attrs) = match get_obj_classes_and_attrs(&req.attributes) {
            Ok(res) => res,
            Err(_) => {
                return ProtocolOp::AddResponse(AddResponse(LdapResult::new(
                    ResultCode::ProtocolError,
                    req.entry.to_owned(),
                    "Could not parse octet stream as utf8".into(),
                )))
            }
        };

        // create the ldap entry
        let entry = LdapEntry::new(obj_classes, attrs);

        // validate against the schema
        if self.schema.validate_entry(&entry).is_err() {
            return ProtocolOp::AddResponse(AddResponse(LdapResult::new(
                ResultCode::ObjectClassViolation,
                req.entry.to_owned(),
                "Entry does not fit the schema".into(),
            )));
        }

        // save it to the db
        self.repo.save(&dn, entry);

        // return ldap res
        ProtocolOp::AddResponse(AddResponse(LdapResult::new(
            ResultCode::Success,
            req.entry.to_owned(),
            "Add request successful".into(),
        )))
    }

    fn handle_search_request(&mut self, req: &SearchRequest) -> ProtocolOp {
        ProtocolOp::SearchResDone(rasn_ldap::SearchResultDone(LdapResult::new(
            ResultCode::Success,
            req.base_object.to_owned(),
            "Search is not implemented so no results found".into(),
        )))
    }
}

type ObjectClassSet = HashSet<String>;
type AttributeMap = HashMap<String, Vec<String>>;

fn get_obj_classes_and_attrs(attrlist: &Vec<Attribute>) -> Result<(ObjectClassSet, AttributeMap)> {
    let mut object_classes = HashSet::new();
    let mut attrs: HashMap<String, Vec<String>> = HashMap::new();

    for a in attrlist {
        let key = String::from_utf8(a.r#type.to_owned().into())?;
        let mut values = a
            .vals
            .iter()
            .map(|v| String::from_utf8(v.to_owned().into()))
            .collect::<Result<Vec<String>, std::string::FromUtf8Error>>()?;

        if key == "objectClass" {
            for v in values.into_iter() {
                object_classes.insert(v);
            }
            continue;
        }

        let attr_entry = attrs.entry(key).or_default();
        attr_entry.append(&mut values)
    }

    Ok((object_classes, attrs))
}
