#![deny(unused)]

use std::iter;

use rusqlite::Connection;
use thiserror::Error;
mod base;
mod scaffolding;

#[derive(Debug, Error)]
#[error("Error applying migration {version}: {error:?} (Revert error: {revert_error:?})")]
pub struct MigrationError {
    pub version: usize,
    pub error: Option<rusqlite::Error>,
    pub revert_error: Option<rusqlite::Error>,
}

impl MigrationError {
    pub fn from_raw(raw: rusqlite::Error) -> Self {
        Self {
            version: 0,
            error: Some(raw),
            revert_error: None,
        }
    }
}

pub fn apply_migrations(con: &mut Connection) -> Result<(), MigrationError> {
    let version = database_version(con).map_err(|e| MigrationError {
        version: 0,
        error: Some(e),
        revert_error: None,
    })?;
    for migration in migrations().skip_while(|step| step.version <= version) {
        let Err(error) = migration.apply(con) else {
            continue;
        };
        let revert_error = migration.revert(con).err();
        return Err(MigrationError {
            version: migration.version,
            error: Some(error),
            revert_error,
        });
    }
    Ok(())
}

#[expect(unused)]
pub fn revert_migrations(con: &mut Connection) -> Result<(), MigrationError> {
    let version = database_version(con).map_err(|e| MigrationError {
        version: 0,
        error: None,
        revert_error: Some(e),
    })?;
    let migrations = migrations().collect::<Vec<_>>();
    for migration in migrations
        .into_iter()
        .rev()
        .skip_while(|step| step.version > version)
    {
        if let Err(e) = migration.revert(con) {
            return Err(MigrationError {
                version: migration.version,
                error: None,
                revert_error: Some(e),
            });
        }
    }
    Ok(())
}

fn database_version(con: &mut Connection) -> Result<usize, rusqlite::Error> {
    match con.query_row("SELECT version FROM metadata", (), |row| row.get(0)) {
        Ok(n) => Ok(n),
        Err(rusqlite::Error::SqliteFailure(_, Some(msg)))
            if msg.eq_ignore_ascii_case("no such table: metadata") =>
        {
            Ok(0)
        }
        Err(e) => Err(e),
    }
}

fn migrations() -> impl Iterator<Item = &'static DatabaseMigration> {
    let mut nxt = 1;
    iter::from_fn(move || {
        for possible in MIGRATIONS {
            if possible.version == nxt {
                nxt += 1;
                return Some(possible);
            }
        }
        None
    })
}

struct DatabaseMigration {
    pub version: usize,
    pub forward: fn(&mut Connection) -> Result<(), rusqlite::Error>,
    pub backwards: fn(&mut Connection) -> Result<(), rusqlite::Error>,
}

impl DatabaseMigration {
    pub fn apply(&self, con: &mut Connection) -> Result<(), rusqlite::Error> {
        (self.forward)(con)?;
        con.execute("UPDATE metadata SET version = ?1", [&self.version])?;
        Ok(())
    }

    pub fn revert(&self, con: &mut Connection) -> Result<(), rusqlite::Error> {
        (self.backwards)(con)?;
        if self.version != 1 {
            con.execute("UPDATE metadata SET version = ?1", [&self.version - 1])?;
        }
        Ok(())
    }
}

const MIGRATIONS: &[DatabaseMigration] = &[scaffolding::metadata_migration(), base::base_schema()];

#[expect(unused)]
const ASSERTIONS: () = {
    let mut flags: [bool; MIGRATIONS.len()] = [false; MIGRATIONS.len()];
    let mut idx = 0;
    while idx < MIGRATIONS.len() {
        let cur = MIGRATIONS[idx].version - 1;
        if flags[cur] {
            panic!("Found duplicate version!");
        }
        flags[cur] = true;
        idx += 1;
    }

    idx = 0;
    while idx < flags.len() {
        if !flags[idx] {
            panic!("Missing database version!");
        }
        idx += 1;
    }
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations() {
        let mut con = Connection::open_in_memory().unwrap();
        apply_migrations(&mut con).unwrap();
        let version = database_version(&mut con).unwrap();
        let highest_version = migrations().last().unwrap().version;
        assert_eq!(version, highest_version);
        revert_migrations(&mut con).unwrap();
        let version = database_version(&mut con).unwrap();
        assert_eq!(version, 0);
    }
}
