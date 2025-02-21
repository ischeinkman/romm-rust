use std::path::Path;

use rusqlite::Connection;
use migrations::apply_migrations;

mod migrations;

#[expect(unused)]
pub struct SaveMetaDatabase {
    con: Connection,
}

impl SaveMetaDatabase {
    #[expect(unused)]
    pub fn open(path: &Path) -> Result<Self, anyhow::Error> {
        let mut con = Connection::open(path)?;
        apply_migrations(&mut con)?;
        Ok(Self { con })
    }

    
}
