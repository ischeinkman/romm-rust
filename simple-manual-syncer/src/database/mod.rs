use std::path::Path;

use migrations::{apply_migrations, MigrationError};
use rusqlite::{params_from_iter, Connection};
use thiserror::Error;

use crate::{md5hash::Md5Hash, SaveMeta};

mod migrations;

pub struct SaveMetaDatabase {
    con: Connection,
}

impl SaveMetaDatabase {
    #[expect(unused)]
    pub fn open(path: &Path) -> Result<Self, MigrationError> {
        let mut con = Connection::open(path).map_err(MigrationError::from_raw)?;
        apply_migrations(&mut con)?;
        Ok(Self { con })
    }

    #[expect(unused)]
    fn new_in_memory() -> Result<Self, MigrationError> {
        let mut con = Connection::open_in_memory().map_err(MigrationError::from_raw)?;
        apply_migrations(&mut con)?;
        Ok(Self { con })
    }

    #[expect(unused)]
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
        let mut stmt = self.con.prepare(&sql)?;
        let mut rows = stmt.query_map(params_from_iter(params), |row| {
            let name = row.get("name")?;
            let rom = row.get("rom")?;
            let emulator = row.get("emulator")?;
            let created = row.get("created")?;
            let updated = row.get("updated")?;
            let hash = Md5Hash::from_raw(row.get("md5")?);
            let size = row.get("size")?;
            let res = SaveMeta {
                name,
                rom,
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
                emulator.map(|s| s.to_owned()),
            )
        });
        Ok(ret)
    }

    #[expect(unused)]
    pub fn upsert_metadata(&self, metadata: &SaveMeta) -> Result<(), DatabaseError> {
        const QUERY: &str = r#"
INSERT INTO saves(
    name, rom, emulator, created, updated, md5, size
) VALUES 
    (?1, ?2, ?3, ?4, ?5, ?6, ?7) 
ON CONFLICT DO UPDATE SET
    name = ?1,
    rom = ?2,
    emulator = ?3,
    created = ?4,
    updated = ?5, 
    md5 = ?6,
    size = ?7"#;
        let modified = self.con.execute(
            QUERY,
            (
                &metadata.name,
                &metadata.rom,
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
            rom: "TEST_ROM".to_owned(),
            name: "TEST_ROM_SAVE".to_owned(),
            emulator: Some("TEST_EMULATOR".to_owned()),
            created: timestamp_now(),
            updated: timestamp_now(),
            hash: Md5Hash::from_raw(std::array::from_fn(|n| (n + 0xA) as u8)),
            size: 9,
        };
        assert!(db
            .query_metadata(&test_rom.rom, &test_rom.name, test_rom.emulator.as_deref())
            .unwrap()
            .is_empty());
        db.upsert_metadata(&test_rom).unwrap();
        assert_eq!(
            db.query_metadata(&test_rom.rom, &test_rom.name, test_rom.emulator.as_deref())
                .unwrap(),
            test_rom
        );
        assert!(db
            .query_metadata(&test_rom.rom, &test_rom.name, None)
            .unwrap()
            .is_empty());

        let mut updated_rom = test_rom.clone();
        updated_rom.updated = timestamp_now();
        updated_rom.hash = Md5Hash::from_raw(std::array::from_fn(|n| (n + 0xB) as u8));
        updated_rom.size = 15;

        db.upsert_metadata(&updated_rom).unwrap();
        assert_eq!(
            db.query_metadata(&test_rom.rom, &test_rom.name, test_rom.emulator.as_deref())
                .unwrap(),
            updated_rom
        );

        let new_rom = SaveMeta {
            rom: "new_rom".to_owned(),
            name: "new_rom_SAVE".to_owned(),
            emulator: None,
            created: timestamp_now(),
            updated: timestamp_now(),
            hash: Md5Hash::from_raw(std::array::from_fn(|n| (n + 0xA) as u8)),
            size: 9,
        };
        assert!(db
            .query_metadata(&new_rom.rom, &new_rom.name, new_rom.emulator.as_deref())
            .unwrap()
            .is_empty());
        db.upsert_metadata(&new_rom).unwrap();
        assert_eq!(
            db.query_metadata(&new_rom.rom, &new_rom.name, new_rom.emulator.as_deref())
                .unwrap(),
            new_rom
        );

        assert_eq!(
            db.query_metadata(&test_rom.rom, &test_rom.name, test_rom.emulator.as_deref())
                .unwrap(),
            updated_rom
        );
    }
}
