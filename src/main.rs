use std::collections::HashMap;
use std::fmt::Display;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use anyhow::Result;
use bytes::Bytes;
use db::{InMemLdapDb, LdapRepo};
use rasn::ber::{de, enc};
use rasn::error::DecodeError;
use rasn::prelude::*;
use rasn_ldap::{
    AddRequest, AddResponse, BindRequest, BindResponse, LdapMessage, LdapResult, ProtocolOp,
    ResultCode, SearchRequest, UnbindRequest,
};
use schema::Schema;

use crate::db::LdapEntry;

mod db;
mod schema;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000")?;

    loop {
        let schema = Schema::new();
        let db = InMemLdapDb::new();

        let (stream, _) = listener.accept()?;
        let mut conn = LdapController::new(schema, db, stream);

        loop {
            let msg = conn.read_msg().unwrap();
            conn.handle_ldap_message(msg).unwrap();
        }
    }
}

#[derive(Debug)]
enum LdapError {
    UnknownProtoOp(ProtocolOp),
    DecodingError(DecodeError),
}

impl Display for LdapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownProtoOp(op) => write!(f, "Unknown operation: {:?}", op),
            Self::DecodingError(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for LdapError {}

struct LdapController<R> {
    schema: Schema,
    repo: R,
    stream: TcpStream,
}

impl<R: LdapRepo> LdapController<R> {
    fn new(schema: Schema, repo: R, stream: TcpStream) -> LdapController<R> {
        LdapController {
            schema,
            repo,
            stream,
        }
    }

    fn read_msg(&mut self) -> Result<LdapMessage> {
        let mut buf = [0; 1024];
        self.stream.read(&mut buf)?;

        let mut ber_decoder = de::Decoder::new(&mut buf, de::DecoderOptions::ber());
        let msg = LdapMessage::decode(&mut ber_decoder).map_err(|e| LdapError::DecodingError(e))?;

        Ok(msg)
    }

    fn send_msg(&mut self, msg: impl Encode) -> Result<()> {
        let mut ber_encoder = enc::Encoder::new(enc::EncoderOptions::ber());
        msg.encode(&mut ber_encoder).unwrap();
        self.stream.write(ber_encoder.output().as_slice())?;
        Ok(())
    }

    fn handle_ldap_message(&mut self, msg: LdapMessage) -> Result<()> {
        dbg!(&msg);
        let resp = match msg.protocol_op {
            ProtocolOp::BindRequest(ref req) => self.handle_bind_request(req),
            ProtocolOp::UnbindRequest(ref req) => self.handle_unbind_request(req),
            ProtocolOp::AddRequest(ref req) => self.handle_add_request(req),
            ProtocolOp::SearchRequest(ref req) => self.handle_search_request(req),
            _ => Err(LdapError::UnknownProtoOp(msg.protocol_op))?,
        };

        self.send_msg(resp)
    }

    fn handle_bind_request(&mut self, req: &BindRequest) -> ProtocolOp {
        ProtocolOp::BindResponse(BindResponse::new(
            ResultCode::Success,
            req.name.to_owned(),
            "not checking passwords".into(),
            None,
            None,
        ))
    }

    fn handle_unbind_request(&mut self, _req: &UnbindRequest) -> ProtocolOp {
        todo!("figure out how to gracefully close the connection!")
    }

    fn handle_add_request(&mut self, req: &AddRequest) -> ProtocolOp {
        // get dn, check parent exists
        let dn = match parse_bytes_as_string(&req.entry, &req.entry) {
            Ok(dn) => dn,
            Err(err) => return ProtocolOp::AddResponse(AddResponse(err)),
        };

        if !self.repo.dn_parent_exists(&dn) {
            return ProtocolOp::AddResponse(AddResponse(LdapResult::new(
                ResultCode::NoSuchObject,
                req.entry.to_owned(),
                "Parent does not exist".into(),
            )));
        }

        // check that an entry doesnt already exist for this dn
        if self.repo.get(&dn).is_some() {
            return ProtocolOp::AddResponse(AddResponse(LdapResult::new(
                ResultCode::EntryAlreadyExists,
                req.entry.to_owned(),
                "Entry with that dn already exists".into(),
            )));
        }

        // get attrs and object classes
        let mut attrs: HashMap<String, Vec<String>> = HashMap::new();
        for a in req.attributes.iter() {
            let key = match parse_bytes_as_string(&a.r#type, &req.entry) {
                Ok(key) => key,
                Err(err) => return ProtocolOp::AddResponse(AddResponse(err)),
            };

            let mut values = vec![];
            for val in a.vals.iter() {
                let v = match parse_bytes_as_string(val, &req.entry) {
                    Ok(v) => v,
                    Err(err) => return ProtocolOp::AddResponse(AddResponse(err)),
                };
                values.push(v)
            }
            let entry = attrs.entry(key).or_default();
            entry.append(&mut values);
        }

        // validate against the schema
        if !self.schema.validate_attributes(attrs) {
            return ProtocolOp::AddResponse(AddResponse(LdapResult::new(
                ResultCode::ObjectClassViolation,
                req.entry.to_owned(),
                "Entry does not fit the schema".into(),
            )));
        }

        // determine rdn
        // safe to unwrap here as dn has already been checked
        let (rdn, _) = dn.split_once(",").unwrap();

        // create the ldap entry
        let entry = LdapEntry::new(rdn.into(), attrs);

        // save it to the db
        self.repo.save(entry);

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

fn parse_bytes_as_string(os: &Bytes, matched_dn: &Bytes) -> Result<String, LdapResult> {
    String::from_utf8(os.to_owned().into()).map_err(|e| {
        LdapResult::new(
            ResultCode::ProtocolError,
            matched_dn.to_owned(),
            "Could not parse octet stream as utf8".into(),
        )
    })
}
