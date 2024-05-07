use std::path::PathBuf;

use rusqlite::{params, types::ValueRef, Connection, Result, ToSql};
use serde::{Deserialize, Serialize};

use crate::constant::{CRDT_TABLE, CR_SQLITE_ENDPOIONT};

#[derive(Serialize, Deserialize, Debug)]
struct CrsqlChangesRowData {
    table: String,
    pk: Vec<u8>,
    cid: String,
    val: serde_json::Value,
    col_version: i64,
    db_version: i64,
    site_id: Vec<u8>,
    cl: i64,
    seq: i64,
}

pub struct CrSqliteDB {
    conn: Connection,
}

impl CrSqliteDB {
    // TODO: 实装 extension_path
    pub fn new(path: PathBuf, extension_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(path).expect("failed to open sqlite db");

        unsafe {
            conn.load_extension_enable()?;
            conn.load_extension("./crsqlite", Some(CR_SQLITE_ENDPOIONT))?;
        }

        let select_sql = CRDT_TABLE
            .map(|table| format!("SELECT crsql_as_crr('{}');", table))
            .join("\n");

        conn.execute_batch(
            format!(
                "
            BEGIN;
            {}
            COMMIT;",
                select_sql
            )
            .as_str(),
        )?;
        Ok(Self { conn })
    }

    fn get_changes(&self) -> Result<Vec<CrsqlChangesRowData>> {
        let mut stmt = self.conn.prepare(r#"select "table", "pk", "cid", "val", "col_version", "db_version", COALESCE("site_id", crsql_site_id()), "cl", "seq" from crsql_changes;"#)?;

        let rows = stmt.query_map([], |row| {
            let table: String = row.get(0)?;
            let pk: Vec<u8> = row.get(1)?;
            let cid: String = row.get(2)?;

            let val = match row.get_ref(3)? {
                ValueRef::Text(text) => {
                    serde_json::Value::String(String::from_utf8(text.to_vec()).unwrap())
                }
                ValueRef::Blob(blob) => serde_json::Value::Array(
                    blob.iter()
                        .map(|&b| serde_json::Value::Number(b.into()))
                        .collect(),
                ),
                ValueRef::Integer(int) => serde_json::Value::Number(int.into()),
                ValueRef::Real(float) => {
                    serde_json::Value::Number(serde_json::Number::from_f64(float).unwrap())
                }
                ValueRef::Null => serde_json::Value::Null,
            };

            let col_version: i64 = row.get(4)?;
            let db_version: i64 = row.get(5)?;
            let site_id: Vec<u8> = row.get(6)?;

            let cl: i64 = row.get(7)?;
            let seq: i64 = row.get(8)?;

            Ok(CrsqlChangesRowData {
                table,
                pk,
                cid,
                val,
                col_version,
                db_version,
                site_id,
                cl,
                seq,
            })
        })?;

        Ok(rows.map(|r| r.unwrap()).collect())
    }

    pub fn get_changes_as_json(&self) -> Result<String> {
        let changes = self.get_changes()?;
        serde_json::to_string(&changes)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    }

    pub fn apple_changes(&mut self, json_string: String) -> Result<()> {
        let datas: Vec<CrsqlChangesRowData> =
            serde_json::from_str::<Vec<CrsqlChangesRowData>>(&json_string)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let tx = self.conn.transaction()?;

        for d in datas {
            let val: Box<dyn ToSql> = match d.val {
                serde_json::Value::Bool(b) => Box::new(b),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Box::new(i)
                    } else if let Some(u) = n.as_u64() {
                        Box::new(u)
                    } else if let Some(f) = n.as_f64() {
                        Box::new(f)
                    } else {
                        Box::new(None::<i32>)
                    }
                }
                serde_json::Value::String(s) => Box::new(s),
                _ => Box::new(None::<i32>),
            };

            let _ = tx.execute("insert into crsql_changes ('table', 'pk', 'cid', 'val', 'col_version', 'db_version', 'site_id', 'cl', 'seq')
            values
            (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);", params![
                d.table,
                d.pk,
                d.cid,
                val,
                d.col_version,
                d.db_version,
                d.site_id,
                d.cl,
                d.seq,
            ])?;
        }
        tx.commit()?;

        Ok(())
    }
}
