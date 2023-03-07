use std::path::PathBuf;

use bonsaidb::core::schema::SerializedCollection;
use bonsaidb::local::{
    config::{Builder, StorageConfiguration},
    Database,
};
use miette::{IntoDiagnostic, Result};
use vpc::VPC;

#[test]
fn simple_save_and_load() -> Result<()> {
    let td: PathBuf = testdir::testdir!();
    let db_dir = td.join("vpc.db");

    let db = Database::open::<VPC>(StorageConfiguration::new(&db_dir)).into_diagnostic()?;

    let doc1 = vpc::Builder::new(uuid::Builder::nil().as_uuid(), "testnet")
        .into_vpc()
        .push_into(&db)
        .into_diagnostic()?;

    let doc2 = VPC::get(doc1.header.id, &db)
        .into_diagnostic()?
        .expect("no vpc retrieved");

    assert_eq!(doc1.contents.name, doc2.contents.name);

    Ok(())
}
