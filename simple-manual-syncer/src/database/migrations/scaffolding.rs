use super::DatabaseMigration;
use rusqlite::Connection;

pub const fn metadata_migration() -> DatabaseMigration {
    DatabaseMigration {
        version: 1,
        forward: build_metadata,
        backwards: unbuild_metadata,
    }
}

fn build_metadata(con: &mut Connection) -> Result<(), rusqlite::Error> {
    con.execute("CREATE TABLE metadata (version INTEGER NOT NULL);", ())?;
    con.execute("INSERT INTO metadata (version) VALUES (0);", ())?;
    Ok(())
}

fn unbuild_metadata(con: &mut Connection) -> Result<(), rusqlite::Error> {
    con.execute("DROP TABLE metadata;", ())?;
    Ok(())
}