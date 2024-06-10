use std::io::{Read, Result, Write};
use std::net::TcpListener;

use rasn::ber::{de, enc};
use rasn::prelude::*;
use rasn_ldap::{BindRequest, BindResponse, LdapMessage, MessageId, ProtocolOp, ResultCode};

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    let (mut stream, _) = listener.accept()?;

    let mut buf = [0; 1024];
    stream.read(&mut buf)?;

    let mut ber_decoder = de::Decoder::new(&mut buf, de::DecoderOptions::ber());
    let msg: LdapMessage = LdapMessage::decode(&mut ber_decoder).unwrap();

    let res = handle_ldap_message(msg);

    let mut ber_encoder = enc::Encoder::new(enc::EncoderOptions::ber());
    res.encode(&mut ber_encoder).unwrap();

    stream.write(ber_encoder.output().as_slice())?;

    Ok(())
}

fn handle_ldap_message(msg: LdapMessage) -> LdapMessage {
    match msg.protocol_op {
        ProtocolOp::BindRequest(req) => handle_bind_request(msg.message_id, req),
        _ => unimplemented!("That message type is unimplemented, handling unimplemented errors is also unimplemented!")
    }
}

fn handle_bind_request(msg_id: MessageId, req: BindRequest) -> LdapMessage {
    LdapMessage::new(
        msg_id,
        ProtocolOp::BindResponse(BindResponse::new(
            ResultCode::Success,
            req.name,
            "not checking passwords".into(),
            None,
            None,
        )),
    )
}
