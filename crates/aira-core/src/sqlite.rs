//! SQLite adapter for local Object Store (Issue #26).

use std::path::Path;

use aira_object::{AiraRef, Handle, ObjectDescriptor};
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::CoreError;
use crate::store::ObjectStore;

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
                "#,
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
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
            Ok(_) => Ok(Handle::new(descriptor.object_id, token)),
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
                params![handle.storage_token() as i64],
                |r| r.get(0),
            )
            .map_err(|_| CoreError::NotFound(handle.object_ref().clone()))?;
        serde_json::from_str(&json).map_err(|e| CoreError::Storage(e.to_string()))
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
            Some(j) => Ok(Some(
                serde_json::from_str(&j).map_err(|e| CoreError::Storage(e.to_string()))?,
            )),
        }
    }
}
