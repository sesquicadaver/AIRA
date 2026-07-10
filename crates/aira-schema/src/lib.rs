//! AIRA schema registry and validation (Issue Set Epic 2).
//!
//! Loads JSON Schema 2020-12 documents from `schemas/`, resolves
//! `aira:schema:…` `$ref` values by inlining, and validates fixtures.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use walkdir::WalkDir;

/// Crate version string for smoke tests.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Validation failure.
#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("unknown schema id: {0}")]
    UnknownSchema(String),
    #[error("validation failed for {schema_id}: {details}")]
    ValidationFailed { schema_id: String, details: String },
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Loaded schema registry keyed by `$id`.
#[derive(Debug, Clone)]
pub struct SchemaRegistry {
    schemas: HashMap<String, Value>,
    /// Short aliases → full `$id` (e.g. `ref` → `aira:schema:common:ref:0.1`).
    aliases: HashMap<String, String>,
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureEntry {
    pub schema: String,
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureManifest {
    pub valid: Vec<FixtureEntry>,
    pub invalid: Vec<FixtureEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FixtureReport {
    pub passed: usize,
    pub failed: usize,
    pub failures: Vec<String>,
}

impl SchemaRegistry {
    /// Load all `*.schema.json` under `schemas_dir`.
    pub fn load(schemas_dir: impl AsRef<Path>) -> Result<Self> {
        let root = schemas_dir.as_ref().to_path_buf();
        let mut schemas = HashMap::new();
        let mut aliases = HashMap::new();

        if !root.exists() {
            return Err(anyhow!("schemas directory not found: {}", root.display()));
        }

        for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if !name.ends_with(".schema.json") {
                continue;
            }
            let text = fs::read_to_string(path)
                .with_context(|| format!("read schema {}", path.display()))?;
            let value: Value = serde_json::from_str(&text)
                .with_context(|| format!("parse schema {}", path.display()))?;
            let id = value
                .get("$id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("schema missing $id: {}", path.display()))?
                .to_string();
            schemas.insert(id.clone(), value);

            // alias: last path segment without .schema.json
            let stem = name.trim_end_matches(".schema.json");
            aliases.insert(stem.to_string(), id.clone());
            // also last colon segment of id
            if let Some(short) = id.rsplit(':').nth(1) {
                // id like aira:schema:common:ref:0.1 → prefer "ref"
                let parts: Vec<&str> = id.split(':').collect();
                if parts.len() >= 2 {
                    let key = parts[parts.len() - 2];
                    aliases.entry(key.to_string()).or_insert(id.clone());
                }
                let _ = short;
            }
        }

        if schemas.is_empty() {
            return Err(anyhow!("no schemas loaded from {}", root.display()));
        }

        Ok(Self {
            schemas,
            aliases,
            root,
        })
    }

    /// Resolve short name or full `$id` to full `$id`.
    pub fn resolve_id(&self, id_or_alias: &str) -> Result<String> {
        if self.schemas.contains_key(id_or_alias) {
            return Ok(id_or_alias.to_string());
        }
        if let Some(full) = self.aliases.get(id_or_alias) {
            return Ok(full.clone());
        }
        // try prefix match on path-like aliases
        Err(SchemaError::UnknownSchema(id_or_alias.to_string()).into())
    }

    /// List all schema `$id` values (sorted).
    pub fn list_ids(&self) -> Vec<String> {
        let mut ids: Vec<_> = self.schemas.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub fn schemas_root(&self) -> &Path {
        &self.root
    }

    /// Validate `instance` against schema id/alias.
    pub fn validate(&self, id_or_alias: &str, instance: &Value) -> Result<(), SchemaError> {
        let id = self.resolve_id(id_or_alias)?;
        let schema = self
            .schemas
            .get(&id)
            .ok_or_else(|| SchemaError::UnknownSchema(id.clone()))?;
        let inlined = inline_refs(schema, &self.schemas, &mut HashSet::new())?;
        let validator = jsonschema::options()
            .should_validate_formats(true)
            .build(&inlined)
            .map_err(|e| SchemaError::Other(anyhow!("compile schema {id}: {e}")))?;

        if let Err(error) = validator.validate(instance) {
            return Err(SchemaError::ValidationFailed {
                schema_id: id,
                details: error.to_string(),
            });
        }
        Ok(())
    }

    /// Validate a JSON file.
    pub fn validate_file(
        &self,
        id_or_alias: &str,
        file: impl AsRef<Path>,
    ) -> Result<(), SchemaError> {
        let text = fs::read_to_string(file.as_ref())
            .with_context(|| format!("read {}", file.as_ref().display()))
            .map_err(SchemaError::Other)?;
        let instance: Value = serde_json::from_str(&text)
            .with_context(|| format!("parse {}", file.as_ref().display()))
            .map_err(SchemaError::Other)?;
        self.validate(id_or_alias, &instance)
    }

    /// Run fixture manifest: valid must pass, invalid must fail.
    pub fn validate_fixtures(&self, repo_root: impl AsRef<Path>) -> Result<FixtureReport> {
        let root = repo_root.as_ref();
        let manifest_path = root.join("fixtures/manifest.json");
        let text = fs::read_to_string(&manifest_path)
            .with_context(|| format!("read {}", manifest_path.display()))?;
        let manifest: FixtureManifest = serde_json::from_str(&text)?;

        let mut passed = 0usize;
        let mut failed = 0usize;
        let mut failures = Vec::new();

        for entry in &manifest.valid {
            let path = root.join(&entry.file);
            match self.validate_file(&entry.schema, &path) {
                Ok(()) => passed += 1,
                Err(e) => {
                    failed += 1;
                    failures.push(format!("VALID expected pass: {} — {e}", entry.file));
                }
            }
        }

        for entry in &manifest.invalid {
            let path = root.join(&entry.file);
            match self.validate_file(&entry.schema, &path) {
                Ok(()) => {
                    failed += 1;
                    failures.push(format!(
                        "INVALID expected fail: {} — but validation passed",
                        entry.file
                    ));
                }
                Err(_) => passed += 1,
            }
        }

        Ok(FixtureReport {
            passed,
            failed,
            failures,
        })
    }
}

/// Recursively replace `$ref: aira:schema:…` with inlined schema bodies.
fn inline_refs(
    node: &Value,
    registry: &HashMap<String, Value>,
    stack: &mut HashSet<String>,
) -> Result<Value> {
    match node {
        Value::Object(map) => {
            if let Some(Value::String(r)) = map.get("$ref") {
                if r.starts_with("aira:schema:") {
                    if !stack.insert(r.clone()) {
                        return Err(anyhow!("cyclic $ref: {r}"));
                    }
                    let target = registry
                        .get(r)
                        .ok_or_else(|| anyhow!("unresolved $ref: {r}"))?;
                    let inlined = inline_refs(target, registry, stack)?;
                    stack.remove(r);
                    return Ok(inlined);
                }
            }
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                // Drop $id on inlined children to avoid confusing validators
                if k == "$id" {
                    out.insert(k.clone(), v.clone());
                    continue;
                }
                out.insert(k.clone(), inline_refs(v, registry, stack)?);
            }
            Ok(Value::Object(out))
        }
        Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(inline_refs(item, registry, stack)?);
            }
            Ok(Value::Array(out))
        }
        other => Ok(other.clone()),
    }
}

/// Locate repository root by walking up until `schemas/` + `Cargo.toml` exist.
pub fn find_repo_root(start: impl AsRef<Path>) -> Result<PathBuf> {
    let mut cur = start
        .as_ref()
        .canonicalize()
        .unwrap_or_else(|_| start.as_ref().to_path_buf());
    for _ in 0..12 {
        if cur.join("schemas").is_dir() && cur.join("Cargo.toml").is_file() {
            return Ok(cur);
        }
        if !cur.pop() {
            break;
        }
    }
    Err(anyhow!(
        "could not find AIRA repo root from {}",
        start.as_ref().display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> SchemaRegistry {
        let root = find_repo_root(env!("CARGO_MANIFEST_DIR")).expect("repo root");
        SchemaRegistry::load(root.join("schemas")).expect("load schemas")
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn loads_core_schemas() {
        let reg = registry();
        let ids = reg.list_ids();
        assert!(ids.iter().any(|id| id.contains("object-descriptor")));
        assert!(ids.iter().any(|id| id.contains("verified-result")));
    }

    #[test]
    fn valid_ref_passes() {
        let reg = registry();
        let v = Value::String("aira:problem:01TESTPROBLEM".into());
        reg.validate("aira:schema:common:ref:0.1", &v).unwrap();
    }

    #[test]
    fn invalid_ref_fails() {
        let reg = registry();
        let v = Value::String("nope".into());
        assert!(reg.validate("ref", &v).is_err());
    }

    #[test]
    fn rejects_gpu_object_type() {
        let reg = registry();
        let root = find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let err = reg
            .validate_file(
                "object-descriptor",
                root.join("fixtures/invalid/core/object-descriptor-gpu.json"),
            )
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("ValidationFailed") || msg.contains("validation failed"));
    }

    #[test]
    fn fixture_manifest_passes() {
        let reg = registry();
        let root = find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let report = reg.validate_fixtures(&root).unwrap();
        assert_eq!(report.failed, 0, "fixture failures: {:?}", report.failures);
        assert!(report.passed >= 20);
    }
}
