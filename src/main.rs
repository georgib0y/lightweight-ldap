use std::{
    net::TcpListener,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use controller::LdapControllerImpl;
use db::{InMemLdapDb, InMemSchemaDb};
use entity::{
    entry::{Entry, EntryBuilder, EntryId},
    schema::{AttributeBuilder, Kind, ObjectClassBuilder},
};
use infrastructure::LdapTcpConnection;
use service::{entry::EntryServiceImpl, schema::SchemaServiceImpl};

mod commands;
mod controller;
mod db;
mod entity;
mod errors;
mod infrastructure;
mod service;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000")?;

    let entry_repo = populated_entry_repo();
    let schema_repo = populated_schema_repo();

    let schema_service = SchemaServiceImpl::new(&schema_repo);
    let entry_service = EntryServiceImpl::new(&schema_service, &entry_repo);
    let controller = LdapControllerImpl::new(&entry_service);

    loop {
        let (stream, _) = listener.accept()?;
        let mut conn = LdapTcpConnection::new(stream, &controller)?;
        conn.run()?;
    }
}

pub fn populated_entry_repo() -> Arc<Mutex<InMemLdapDb<u64>>> {
    let dc_georgiboy: Entry<u64> = EntryBuilder::new()
        .set_id(u64::new_random_id())
        .add_attr_val("dc-oid", "georgiboy")
        .build();

    let dc_dev: Entry<u64> = EntryBuilder::new()
        .set_id(u64::root_identifier())
        .add_attr_val("dc-oid", "dev")
        .add_child(dc_georgiboy.get_id().unwrap())
        .build();

    InMemLdapDb::<u64>::with_entries([dc_georgiboy, dc_dev].into_iter())
}

pub fn populated_schema_repo() -> InMemSchemaDb {
    InMemSchemaDb::new(
        [ObjectClassBuilder::new()
            .set_numericoid("person-oid")
            .add_name("person")
            .add_sup_oid("top")
            .set_kind(Kind::Structural)
            .add_must_attr("cn-oid")
            .add_may_attr("sn-oid")
            .build()]
        .into_iter(),
        [
            AttributeBuilder::new()
                .set_numericoid("cn-oid")
                .add_name("cn")
                .add_name("commonName")
                .build(),
            AttributeBuilder::new()
                .set_numericoid("sn-oid")
                .add_name("sn")
                .build(),
            AttributeBuilder::new()
                .set_numericoid("dc-oid")
                .add_name("dc")
                .build(),
        ]
        .into_iter(),
    )
}
