use super::*;
use rusqlite::Connection;

pub const fn base_schema() -> DatabaseMigration {
    DatabaseMigration {
        version: 2,
        forward: create_saves_table,
        backwards: delete_saves_table,
    }
}

fn create_saves_table(con: &mut Connection) -> Result<(), rusqlite::Error> {
    con.execute(
        r#"
CREATE TABLE saves(
    name TEXT NOT NULL, 
    rom TEXT NOT NULL,
    emulator TEXT, 
    created TEXT NOT NULL, 
    updated TEXT NOT NULL, 
    md5 BLOB NOT NULL, 
    size INTEGER NOT NULL,
    UNIQUE (name, rom, emulator)
);"#,
        (),
    )?;
    Ok(())
}

fn delete_saves_table(con: &mut Connection) -> Result<(), rusqlite::Error> {
    con.execute("DROP TABLE saves;", ())?;
    Ok(())
}

/*

    pub rom: String,
    pub name: String,
    pub emulator: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub md5: Md5Hash,

*/
