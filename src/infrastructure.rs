use std::{io::Read, io::Write, net::TcpStream};

use anyhow::{anyhow, Result};
use rasn::{
    der::{de, enc},
    Decode, Encode,
};
use rasn_ldap::{LdapMessage, LdapResult, ProtocolOp, ResultCode};

use crate::controller::LdapController;

pub struct LdapTcpConnection<'a, C: LdapController> {
    stream: TcpStream,
    ldap_controller: &'a C,
}

impl<'a, C: LdapController> LdapTcpConnection<'a, C> {
    pub fn new(stream: TcpStream, ldap_controller: &C) -> Self {
        LdapTcpConnection {
            stream,
            ldap_controller,
        }
    }

    pub fn read_msg(&mut self) -> Result<LdapMessage> {
        let mut buf = [0; 1024];
        let n = self.stream.read(&mut buf)?;

        let mut ber_decoder = de::Decoder::new(&buf, de::DecoderOptions::ber());
        LdapMessage::decode(&mut ber_decoder).map_err(|e| anyhow!(e.to_string()))
    }

    pub fn send_msg(&mut self, msg: impl Encode) -> Result<()> {
        let mut ber_encoder = enc::Encoder::new(enc::EncoderOptions::ber());
        msg.encode(&mut ber_encoder)
            .map_err(|e| anyhow!(e.to_string()));
        self.stream.write_all(ber_encoder.output().as_slice())?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let msg = self.read_msg()?;

            let protocol_op_res = match msg.protocol_op {
                ProtocolOp::AddRequest(ref req) => self.ldap_controller.handle_add_request(req),
                ProtocolOp::BindRequest(ref req) => self.ldap_controller.handle_bind_request(req),
                ProtocolOp::SearchRequest(ref req) => {
                    self.ldap_controller.handle_search_request(req)
                }
                ProtocolOp::UnbindRequest(_) => break,
                _ => {
                    dbg!(msg.protocol_op);
                    eprintln!("Prococol Request type not implemented");

                    self.send_msg(LdapResult::new(
                        ResultCode::UnwillingToPerform,
                        "".into(),
                        "Request type not implemented".into(),
                    ))?;

                    continue;
                }
            }?;

            let res_msg = LdapMessage::new(msg.message_id, protocol_op_res);

            self.send_msg(res_msg)?;
        }

        Ok(())
    }
}
