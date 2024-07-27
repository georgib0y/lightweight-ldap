use std::net::TcpListener;

use anyhow::Result;
use controller::LdapControllerImpl;
use db::{InMemLdapDb, InMemSchemaDb};
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

    let (stream, _) = listener.accept()?;

    let entry_repo = InMemLdapDb::<u64>::new();
    let schema_repo = InMemSchemaDb::default();
    let schema_service = SchemaServiceImpl::new(&schema_repo);
    let entry_service = EntryServiceImpl::new(&schema_service, &entry_repo);
    let controller = LdapControllerImpl::new(&entry_service);
    let mut conn = LdapTcpConnection::new(stream, &controller)?;

    conn.run()
}
