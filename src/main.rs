use std::fmt::Display;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use anyhow::Result;
use rasn::ber::{de, enc};
use rasn::error::DecodeError;
use rasn::prelude::*;
use rasn_ldap::{
    AddRequest, BindRequest, BindResponse, LdapMessage, LdapResult, ProtocolOp, ResultCode,
    SearchRequest, UnbindRequest,
};

mod db;
mod schema;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000")?;

    loop {
        let (stream, _) = listener.accept()?;
        let mut conn = LdapConn { stream };

        loop {
            let msg = conn.read_msg().unwrap();
            conn.handle_ldap_message(msg).unwrap();
        }
    }
}

#[derive(Debug)]
enum LdapError {
    DecodingError(DecodeError),
}

impl Display for LdapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DecodingError(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for LdapError {}

struct LdapConn {
    stream: TcpStream,
}

impl LdapConn {
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
        match msg.protocol_op {
            ProtocolOp::BindRequest(ref req) => self.handle_bind_request(&msg, req),
            ProtocolOp::UnbindRequest(ref req) => self.handle_unbind_request(&msg, req),
            ProtocolOp::AddRequest(ref req) => self.handle_add_request(&msg, req),
            ProtocolOp::SearchRequest(ref req) => self.handle_search_request(&msg, req),
            _ => {
                unimplemented!("That message type is unimplemented, handling unimplemented errors is also unimplemented!")
            }
        }
    }

    fn handle_bind_request(&mut self, msg: &LdapMessage, req: &BindRequest) -> Result<()> {
        self.send_msg(LdapMessage::new(
            msg.message_id,
            ProtocolOp::BindResponse(BindResponse::new(
                ResultCode::Success,
                req.name.to_owned(),
                "not checking passwords".into(),
                None,
                None,
            )),
        ))
    }

    fn handle_unbind_request(&mut self, _msg: &LdapMessage, _req: &UnbindRequest) -> Result<()> {
        todo!("figure out how to gracefully close the connection!")
    }

    fn handle_add_request(&mut self, msg: &LdapMessage, req: &AddRequest) -> Result<()> {
        dbg!(msg.message_id, req);
        todo!("finish handling add request")
    }

    fn handle_search_request(&mut self, msg: &LdapMessage, req: &SearchRequest) -> Result<()> {
        self.send_msg(LdapMessage::new(
            msg.message_id,
            ProtocolOp::SearchResDone(rasn_ldap::SearchResultDone(LdapResult::new(
                ResultCode::Success,
                req.base_object.to_owned(),
                "Search is not implemented so no results found".into(),
            ))),
        ))
    }
}
