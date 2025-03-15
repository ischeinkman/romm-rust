use std::{path::Path, sync::Mutex};

use migrations::{apply_migrations, MigrationError};
use rusqlite::{params_from_iter, Connection};
use thiserror::Error;

use crate::{md5hash::Md5Hash, SaveMeta};

mod migrations;

/// A database containing metadata around previously seen save versions.
///
/// Used for detecting when a save can be safely synced to/from the device and
/// when there is a conflict; see this crate's `README` for more details as to
/// the exact process used for deciding when & how a save is synced.
pub struct SaveMetaDatabase {
    con: Mutex<Connection>,
}

impl SaveMetaDatabase {
    pub fn open(path: &Path) -> Result<Self, MigrationError> {
        let mut con = Connection::open(path).map_err(MigrationError::from_raw)?;
        apply_migrations(&mut con)?;
        let con = Mutex::new(con);
        Ok(Self { con })
    }

    /// Opens a temporary database in memory.
    ///
    /// Use only for tests.
    #[cfg_attr(not(test), expect(unused))]
    fn new_in_memory() -> Result<Self, MigrationError> {
        let mut con = Connection::open_in_memory().map_err(MigrationError::from_raw)?;
        apply_migrations(&mut con)?;
        let con = Mutex::new(con);
        Ok(Self { con })
    }

    /// Pulls the latest metadata seen for a given save file from the database.
    pub fn query_metadata(
        &self,
        rom: &str,
        name: &str,
        emulator: Option<&str>,
    ) -> Result<SaveMeta, DatabaseError> {
        let mut sql = "SELECT * FROM saves WHERE rom = ?1 AND name = ?2".to_owned();
        if emulator.is_some() {
            sql.push_str(" AND emulator = ?3");
        } else {
            sql.push_str(" AND emulator IS NULL");
        }
        let params: &[&str] = if let Some(emulator) = emulator {
            &[rom, name, emulator]
        } else {
            &[rom, name]
        };
        let con = self.con.lock().unwrap();
        let mut stmt = con.prepare(&sql)?;
        let mut rows = stmt.query_map(params_from_iter(params), |row| {
            let name = row.get("name")?;
            let rom = row.get("rom")?;
            let ext = row.get("ext")?;
            let emulator = row.get("emulator")?;
            let created = row.get("created")?;
            let updated = row.get("updated")?;
            let hash = Md5Hash::from_raw(row.get("md5")?);
            let size = row.get("size")?;
            let res = SaveMeta {
                rom,
                name,
                ext,
                emulator,
                created,
                updated,
                hash,
                size,
            };
            Ok(res)
        })?;
        let ret = rows.next().transpose()?;
        if rows.next().transpose()?.is_some() {
            return Err(DatabaseError::TooManyRows {
                count: 2 + rows.count(),
            });
        }
        let ret = ret.unwrap_or_else(|| {
            SaveMeta::new_empty(
                rom.to_owned(),
                name.to_owned(),
                String::default(),
                emulator.map(|s| s.to_owned()),
            )
        });
        Ok(ret)
    }

    /// Pushes new metadata into the database after a sync.
    pub fn upsert_metadata(&self, metadata: &SaveMeta) -> Result<(), DatabaseError> {
        const QUERY: &str = r#"
INSERT INTO saves(
    name, rom, ext, emulator, created, updated, md5, size
) VALUES 
    (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) 
ON CONFLICT DO UPDATE SET
    name = ?1,
    rom = ?2,
    ext = ?3,
    emulator = ?4,
    created = ?5,
    updated = ?6, 
    md5 = ?7,
    size = ?8"#;
        let con = self.con.lock().unwrap();
        let modified = con.execute(
            QUERY,
            (
                &metadata.name,
                &metadata.rom(),
                &metadata.ext,
                metadata.emulator.as_deref(),
                metadata.created,
                metadata.updated,
                metadata.hash.as_bytes(),
                metadata.size,
            ),
        )?;
        if modified != 1 {
            return Err(DatabaseError::TooManyRows { count: modified });
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
    #[error("Too many rows found for filter! Expected 1, found {count}.")]
    TooManyRows { count: usize },
}

#[cfg(test)]
mod tests {
    use crate::utils::timestamp_now;

    use super::*;

    #[test]
    fn test_db_basic_ops() {
        let db = SaveMetaDatabase::new_in_memory().unwrap();
        let test_rom = SaveMeta {
            rom: Some("TEST_ROM".to_owned()),
            name: "TEST_ROM_SAVE".to_owned(),
            ext: "sav".to_owned(),
            emulator: Some("TEST_EMULATOR".to_owned()),
            created: timestamp_now(),
            updated: timestamp_now(),
            hash: Md5Hash::from_raw(std::array::from_fn(|n| (n + 0xA) as u8)),
            size: 9,
        };
        assert!(db
            .query_metadata(
                &test_rom.rom(),
                &test_rom.name,
                test_rom.emulator.as_deref()
            )
            .unwrap()
            .is_empty());
        db.upsert_metadata(&test_rom).unwrap();
        assert_eq!(
            db.query_metadata(
                &test_rom.rom(),
                &test_rom.name,
                test_rom.emulator.as_deref()
            )
            .unwrap(),
            test_rom
        );
        assert!(db
            .query_metadata(&test_rom.rom(), &test_rom.name, None)
            .unwrap()
            .is_empty());

        let mut updated_rom = test_rom.clone();
        updated_rom.updated = timestamp_now();
        updated_rom.hash = Md5Hash::from_raw(std::array::from_fn(|n| (n + 0xB) as u8));
        updated_rom.size = 15;

        db.upsert_metadata(&updated_rom).unwrap();
        assert_eq!(
            db.query_metadata(
                &test_rom.rom(),
                &test_rom.name,
                test_rom.emulator.as_deref()
            )
            .unwrap(),
            updated_rom
        );

        let new_rom = SaveMeta {
            rom: Some("new_rom".to_owned()),
            name: "new_rom_SAVE".to_owned(),
            ext: "sav".to_owned(),
            emulator: None,
            created: timestamp_now(),
            updated: timestamp_now(),
            hash: Md5Hash::from_raw(std::array::from_fn(|n| (n + 0xA) as u8)),
            size: 9,
        };
        assert!(db
            .query_metadata(&new_rom.rom(), &new_rom.name, new_rom.emulator.as_deref())
            .unwrap()
            .is_empty());
        db.upsert_metadata(&new_rom).unwrap();
        assert_eq!(
            db.query_metadata(&new_rom.rom(), &new_rom.name, new_rom.emulator.as_deref())
                .unwrap(),
            new_rom
        );

        assert_eq!(
            db.query_metadata(
                &test_rom.rom(),
                &test_rom.name,
                test_rom.emulator.as_deref()
            )
            .unwrap(),
            updated_rom
        );
    }
}
