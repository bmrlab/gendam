use std::path::PathBuf;

use rusqlite::{params, ToSql};

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
    pub fn pull_file_changes(
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

    pub fn pull_dir_changes(
        &self,
        db_version: usize,
        file_path_ids: Vec<String>,
        asset_object_ids: Vec<String>,
        media_data_ids: Vec<String>,
    ) -> rusqlite::Result<String> {
        let packed_file_path_ids = self.batch_pack(file_path_ids);
        let packed_asset_object_ids = self.batch_pack(asset_object_ids);
        let packed_media_data_ids = self.batch_pack(media_data_ids);

        let sql = format!(
            r#"SELECT * FROM crsql_changes
            WHERE db_version > ?1 AND site_id = crsql_site_id() AND cid != 'relativePath' AND
            (("table" = 'FilePath' AND pk IN ({})) OR
            ("table" = 'AssetObject' AND pk IN ({})) OR
            ("table" = 'MediaData' AND pk IN ({})));"#,
            packed_file_path_ids
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 1 + 1))
                .collect::<Vec<String>>()
                .join(", "),
            packed_asset_object_ids
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 1 + 1 + packed_file_path_ids.len()))
                .collect::<Vec<String>>()
                .join(", "),
            packed_media_data_ids
                .iter()
                .enumerate()
                .map(|(i, _)| format!(
                    "?{}",
                    i + 1 + 1 + packed_file_path_ids.len() + packed_asset_object_ids.len()
                ))
                .collect::<Vec<String>>()
                .join(", "),
        );

        let mut stmt = self.db.conn.prepare(&sql)?;

        let mut params: Vec<&dyn ToSql> = vec![&db_version];
        packed_file_path_ids.iter().for_each(|f| params.push(f));
        packed_asset_object_ids.iter().for_each(|f| params.push(f));
        packed_media_data_ids.iter().for_each(|f| params.push(f));

        let rows = stmt.query_map(params.as_slice(), |row| CrsqlChangesRowData::try_from(row))?;
        let changes: Vec<CrsqlChangesRowData> =
            rows.map(|r| r.expect("failed to get row")).collect();

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

        let str = file_sync.pull_file_changes(
            0,
            "clvz3mnfz0001tkh5iwrs1j6j".to_string(),
            "clvz3mng20004tkh5ix7dupem".to_string(),
            "clvz3mng20004tkh5ix7d234234".to_string(),
        );

        println!("{:?}", str);
    }

    #[test]
    fn test_get_dir_changes() {
        let file_sync = FileSync::new(PathBuf::from_str("/Users/zingerbee/Library/Application Support/ai.gendam.desktop/libraries/75fdd584-9716-4a41-a976-c16eb843cf64/databases/library.db").unwrap());

        let str = file_sync.pull_dir_changes(
            0,
            vec![
                "clw4p767b0004tkmjo6tsq9wg".to_string(),
                "clw4p7a3k000btkmjmt7g859e".to_string(),
                "clw4p7ezi000itkmjwgby90sr".to_string(),
                "clw4qf79e0000tk03ndn77pgb".to_string(),
                "clw4qfc030006tk03zcloo4tu".to_string(),
                "clw4qfffc000dtk030qa4w70m".to_string(),
                "clw4nm8cc0005tkbjd849x1lv".to_string(),
            ],
            vec![
                "clw4p767a0001tkmjik9tk5s7".to_string(),
                "clw4p7a3j0008tkmjntbodol0".to_string(),
                "clw4p7ezi000ftkmjq5q3ps57".to_string(),
                "clw4qfc020003tk03xi9e0ls6".to_string(),
                "clw4qfffb000atk037z7n6n0i".to_string(),
            ],
            vec![
                "clw4p76bk0006tkmj6h6xaadd".to_string(),
                "clw4p7a5q000dtkmj5bzmkiff".to_string(),
                "clw4p7f2o000ktkmjpgmgi775".to_string(),
                "clw4qfc5d0008tk03jw71nzjn".to_string(),
                "clw4qffhu000ftk036ajpwfgq".to_string(),
            ],
        );

        println!("{:?}", str);
    }
}
