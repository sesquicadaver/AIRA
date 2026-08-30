//! SQLite adapter for local Object Store (Issue #26).

use std::path::Path;

use aira_object::object_store_access;
use aira_object::{AiraRef, Handle, ObjectDescriptor};
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::CoreError;
use crate::store::{bind_handle_open, verify_stored_descriptor, ObjectStore};

/// SQLite `objects` table schema token (migration smoke / doc anchor #143).
pub const OBJECTS_SCHEMA_VERSION: u32 = 1;

/// SQLite-backed immutable Object Store.
pub struct SqliteObjectStore {
    conn: Connection,
}

impl SqliteObjectStore {
    /// Open or create a SQLite database and ensure schema.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, CoreError> {
        let conn =
            Connection::open(path.as_ref()).map_err(|e| CoreError::Storage(e.to_string()))?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), CoreError> {
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS objects (
                    object_id TEXT PRIMARY KEY NOT NULL,
                    object_type TEXT NOT NULL,
                    schema_version TEXT NOT NULL,
                    content_hash TEXT NOT NULL,
                    descriptor_json TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    signature_json TEXT NOT NULL,
                    rowid_token INTEGER NOT NULL UNIQUE
                );
                CREATE TABLE IF NOT EXISTS schema_meta (
                    key TEXT PRIMARY KEY NOT NULL,
                    value TEXT NOT NULL
                );
                "#,
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        self.verify_objects_table_integrity()?;
        self.conn
            .execute(
                "INSERT OR IGNORE INTO schema_meta (key, value) VALUES ('objects_schema_version', ?1)",
                params![OBJECTS_SCHEMA_VERSION.to_string()],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Fail closed if migrate did not produce the expected `objects` table (#143).
    fn verify_objects_table_integrity(&self) -> Result<(), CoreError> {
        let table_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'objects'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if table_count != 1 {
            return Err(CoreError::Storage(
                "objects table missing after migrate".into(),
            ));
        }
        let column_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('objects')",
                [],
                |r| r.get(0),
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if column_count != 8 {
            return Err(CoreError::Storage(format!(
                "objects table has {column_count} columns, expected 8"
            )));
        }
        Ok(())
    }

    fn next_token(&self) -> Result<u64, CoreError> {
        let max: Option<Option<i64>> = self
            .conn
            .query_row("SELECT MAX(rowid_token) FROM objects", [], |r| r.get(0))
            .optional()
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        // Empty table → MAX returns SQL NULL → Option<i64>::None inside the row.
        let value = match max {
            None | Some(None) => 0,
            Some(Some(v)) => v,
        };
        Ok(value as u64 + 1)
    }
}

impl ObjectStore for SqliteObjectStore {
    fn create(&mut self, descriptor: ObjectDescriptor) -> Result<Handle, CoreError> {
        crate::store::admit_object(&descriptor)?;
        let token = self.next_token()?;
        let descriptor_json =
            serde_json::to_string(&descriptor).map_err(|e| CoreError::Storage(e.to_string()))?;
        let signature_json = serde_json::to_string(&descriptor.signature)
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let object_type = serde_json::to_value(descriptor.object_type)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| format!("{:?}", descriptor.object_type));

        let result = self.conn.execute(
            r#"
            INSERT INTO objects (
                object_id, object_type, schema_version, content_hash,
                descriptor_json, created_at, signature_json, rowid_token
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                descriptor.object_id.as_str(),
                object_type,
                descriptor.schema_version,
                descriptor.content_hash.as_str(),
                descriptor_json,
                descriptor.created_at.as_str(),
                signature_json,
                token as i64,
            ],
        );

        match result {
            Ok(_) => Ok(object_store_access::mint(descriptor.object_id, token)),
            Err(rusqlite::Error::SqliteFailure(e, _))
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(CoreError::DuplicateObject {
                    object_id: descriptor.object_id,
                })
            }
            Err(e) => Err(CoreError::Storage(e.to_string())),
        }
    }

    fn open(&self, handle: &Handle) -> Result<ObjectDescriptor, CoreError> {
        let json: String = self
            .conn
            .query_row(
                "SELECT descriptor_json FROM objects WHERE rowid_token = ?1",
                params![object_store_access::storage_token(handle) as i64],
                |r| r.get(0),
            )
            .map_err(|_| CoreError::NotFound(handle.object_ref().clone()))?;
        let descriptor =
            serde_json::from_str(&json).map_err(|e| CoreError::Storage(e.to_string()))?;
        bind_handle_open(handle, descriptor)
    }

    fn get_by_object_id(&self, object_id: &AiraRef) -> Result<Option<ObjectDescriptor>, CoreError> {
        let json: Option<String> = self
            .conn
            .query_row(
                "SELECT descriptor_json FROM objects WHERE object_id = ?1",
                params![object_id.as_str()],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match json {
            None => Ok(None),
            Some(j) => {
                let descriptor =
                    serde_json::from_str(&j).map_err(|e| CoreError::Storage(e.to_string()))?;
                Ok(Some(verify_stored_descriptor(descriptor)?))
            }
        }
    }
}
