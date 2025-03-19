//! Tools for changing the sync database's schema over time in a non-destructive
//! manner.
//!
//! Each schema is associated with a version number and a migration. The
//! migration contains code for moving both from & to the previous version
//! without loss of the previous version's data.
//!
//! A non-existent database is considered version 0.

// This annotation will cause a compile error if we accidentally forget to put a
// new migration into the `MIGRATIONS` list.
#![deny(unused)]

use std::iter;

use rusqlite::Connection;
use thiserror::Error;
mod base;
mod scaffolding;

#[derive(Debug, Error)]
#[error("Error applying migration {version}: {error:?} (Revert error: {revert_error:?})")]
pub struct MigrationErrorInner {
    pub version: usize,
    pub error: Option<rusqlite::Error>,
    pub revert_error: Option<rusqlite::Error>,
}

impl MigrationErrorInner {
    pub fn from_raw(raw: rusqlite::Error) -> Self {
        Self {
            version: 0,
            error: Some(raw),
            revert_error: None,
        }
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct MigrationError(#[from] Box<MigrationErrorInner>);

impl From<MigrationErrorInner> for MigrationError {
    fn from(value: MigrationErrorInner) -> Self {
        Self(Box::new(value))
    }
}

impl MigrationError {
    pub fn from_raw(raw: rusqlite::Error) -> Self {
        Self::from(MigrationErrorInner::from_raw(raw))
    }
}

/// Attempts to update the database's schema to the latest version.
pub fn apply_migrations(con: &mut Connection) -> Result<(), MigrationError> {
    let version = database_version(con).map_err(|e| MigrationErrorInner {
        version: 0,
        error: Some(e),
        revert_error: None,
    })?;
    for migration in migrations().skip_while(|step| step.version <= version) {
        let Err(error) = migration.apply(con) else {
            continue;
        };
        let revert_error = migration.revert(con).err();
        return Err(MigrationErrorInner {
            version: migration.version,
            error: Some(error),
            revert_error,
        }
        .into());
    }
    Ok(())
}

#[cfg_attr(not(test), expect(unused))]
pub fn revert_migrations(con: &mut Connection) -> Result<(), MigrationError> {
    let version = database_version(con).map_err(|e| MigrationErrorInner {
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
            return Err(MigrationErrorInner {
                version: migration.version,
                error: None,
                revert_error: Some(e),
            }
            .into());
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

/// Retrieves the list of migrations in version order.
fn migrations() -> impl Iterator<Item = &'static DatabaseMigration> {
    let mut nxt = 1;
    iter::from_fn(move || {
        // Since we can't gurantee that `MIGRATIONS` is in order at compile time
        // we fix the order at runtime.
        //
        // While this is technically slow for a large number of versions bc of
        // the O(n^2) complexity we assume that a) we won't have a huge number
        // of versions, and b) this is only ever called at daemon start, so a
        // little delay at startup is not that bad.
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
    /// The version number of this database schema
    pub version: usize,
    /// The code to move FROM `version - 1` TO this `version.`
    pub forward: fn(&mut Connection) -> Result<(), rusqlite::Error>,
    /// The code to move TO `version - 1` FROM this `version.`
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

/// Compile time checks for sanity of [`MIGRATIONS`].
///
/// Specifically:
/// * No duplicate version numbers
/// * No skipped version numbers
#[expect(unused)]
const ASSERTIONS: () = {
    let mut flags: [bool; MIGRATIONS.len()] = [false; MIGRATIONS.len()];
    let mut idx = 0;
    while idx < MIGRATIONS.len() {
        let cur = MIGRATIONS[idx].version - 1;
        if cur >= flags.len() {
            panic!("Found a version higher than expected!");
        }
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
        let version = database_version(&mut con).unwrap();
        let highest_version = migrations().last().unwrap().version;
        assert_eq!(version, highest_version);
        revert_migrations(&mut con).unwrap();
        let version = database_version(&mut con).unwrap();
        assert_eq!(version, 0);
    }
}
