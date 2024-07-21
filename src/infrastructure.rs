use std::{fmt::Debug, io::Read, io::Write, net::TcpStream};

use anyhow::{anyhow, Result};
use rasn::{
    ber::{de, enc},
    Decode, Encode,
};
use rasn_ldap::{LdapMessage, LdapResult, ProtocolOp, ResultCode};

use crate::controller::LdapController;

pub struct LdapTcpConnection<'a, C: LdapController> {
    stream: TcpStream,
    ldap_controller: &'a C,
}

impl<'a, C: LdapController> LdapTcpConnection<'a, C> {
    pub fn new(stream: TcpStream, ldap_controller: &'a C) -> Result<Self> {
        Ok(LdapTcpConnection {
            stream,
            ldap_controller,
        })
    }

    fn read_ber_len(&mut self) -> Result<(usize, Vec<u8>)> {
        let mut tag_len_buf = vec![0; 2];
        self.stream.read_exact(&mut tag_len_buf)?;

        let mut len = tag_len_buf[1] as usize;

        if tag_len_buf[1] & 0x80 == 0 {
            return Ok((len, tag_len_buf));
        }

        let mut long_len_buf = vec![0; len & 0x7F];
        self.stream.read_exact(&mut long_len_buf)?;

        len = long_len_buf
            .iter()
            .fold(0, |acc, b| (acc << 8) | *b as usize);

        tag_len_buf.extend(long_len_buf);
        Ok((len, tag_len_buf))
    }

    // TODO handle better message reading
    pub fn read_msg(&mut self) -> Result<LdapMessage> {
        let (len, mut buf) = self.read_ber_len()?;
        let mut rest_buf = vec![0; len];
        self.stream.read_exact(&mut rest_buf)?;

        buf.extend(rest_buf);

        let mut ber_decoder = de::Decoder::new(&buf, de::DecoderOptions::ber());
        LdapMessage::decode(&mut ber_decoder).map_err(|e| anyhow!(e.to_string()))
    }

    pub fn send_msg(&mut self, msg: impl Encode + Debug) -> Result<()> {
        let mut ber_encoder = enc::Encoder::new(enc::EncoderOptions::ber());
        msg.encode(&mut ber_encoder)
            .map_err(|e| anyhow!(e.to_string()))?;
        self.stream.write_all(ber_encoder.output().as_slice())?;
        self.stream.flush()?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let msg = self.read_msg()?;
            dbg!(&msg);

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

            dbg!(&res_msg);

            self.send_msg(res_msg)?;
        }

        Ok(())
    }
}
