use std::path::PathBuf;

use rusqlite::params;

use crate::{
    constant::CRDT_TABLE,
    db::{CrSqliteDB, CrsqlChangesRowData},
};

pub struct FileSync {
    db: CrSqliteDB,
}

impl FileSync {
    pub fn new(db_path: PathBuf) -> FileSync {
        FileSync {
            db: CrSqliteDB::new(
                db_path,
                PathBuf::from("/Users/zingerbee/Desktop/crsqlite.dylib"),
                &CRDT_TABLE,
            )
            .expect("Failed to create CrSqliteDB"),
        }
    }

    fn batch_pack(&self, data: Vec<String>) -> Vec<Vec<u8>> {
        data.iter()
            .map(|f| self.db.pack(f).unwrap_or_default())
            .collect()
    }

    /// (AssetObjectId, FilePathId, MediaDataId)
    pub fn pull_asset_object_changes(
        &self,
        db_version: usize,
        asset_object_id: String,
        file_path_id: String,
        media_data_id: String,
    ) -> rusqlite::Result<String> {
        let packed_data = self.batch_pack(vec![asset_object_id, file_path_id, media_data_id]);
        let packed_asset_object_id = &packed_data[0];
        let packed_file_path_id = &packed_data[1];
        let packed_media_data_id = &packed_data[2];

        tracing::debug!(
            "db_version: {}, packed_asset_object_id: {:?}, packed_file_path_id: {:?}, packed_media_data_id: {:?}",
            db_version, packed_asset_object_id, packed_file_path_id, packed_media_data_id);

        let mut stmt = self.db.conn.prepare(
            r#"SELECT * FROM crsql_changes
            WHERE db_version > ?1 AND site_id = crsql_site_id() AND
                  (("table" = 'AssetObject' AND pk = ?2) OR
                   ("table" = 'FilePath' AND pk = ?3) OR
                   ("table" = 'MediaData' AND pk = ?4));"#,
        )?;

        let rows = stmt.query_map(
            params![
                db_version,
                packed_asset_object_id,
                packed_file_path_id,
                packed_media_data_id,
            ],
            |row| CrsqlChangesRowData::try_from(row),
        )?;

        let changes: Vec<CrsqlChangesRowData> = rows.map(|r| r.expect("")).collect();
        Ok(serde_json::to_string(&changes)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?)
    }

    pub fn apple_changes(&mut self, changes: String) -> rusqlite::Result<()> {
        self.db.apple_changes(changes)
    }
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, str::FromStr};

    use super::FileSync;

    #[test]
    fn test_file_sync() {
        let file_sync = FileSync::new(PathBuf::from_str("/Users/zingerbee/Library/Application Support/ai.gendam.desktop/libraries/48425991-2f1a-4d3b-ae6c-f7afec9fdc5b/databases/library.db").unwrap());

        let str = file_sync.pull_asset_object_changes(
            0,
            "clvz3mnfz0001tkh5iwrs1j6j".to_string(),
            "clvz3mng20004tkh5ix7dupem".to_string(),
            "clvz3mng20004tkh5ix7d234234".to_string(),
        );

        println!("{:?}", str);
    }
}
