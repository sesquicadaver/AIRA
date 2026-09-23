//! Host `ollama list` probe for Desktop Settings (process backend bind).
//!
//! Listing models is **observe-only** on the Desktop host. It does not activate
//! Phase D weights, does not grant VERIFIED, and does not imply the running
//! `aira-node` already has `AIRA_LLM_BACKEND=process` until restart applies
//! settings.
//!
//! `#368`: HTTP `GET /api/tags` metadata parse — missing JSON fields stay
//! [`OllamaJsonField::Unknown`]. Never invent boolean `false` / empty-known
//! for locality or capabilities when the key is absent.
//!
//! `#369`: `POST /api/show` when any fitness field is still Unknown after tags
//! (capabilities **or** locality) — not only when capabilities are missing.
//!
//! `#370`: in-memory cache keyed by endpoint/name/digest lives in
//! [`crate::ollama_meta_cache`] (generation + 2–4 parallel show).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use serde_json::Value;

/// Default program when Settings leave `llm_process_bin` empty.
pub const DEFAULT_OLLAMA_BIN: &str = "ollama";

/// Default Ollama HTTP endpoint for discovery and `ollama run` (`#366`).
///
/// Must match `execution-llm` ProcessBackend default. Ambient `OLLAMA_HOST` is
/// never the source of truth for Desktop list or node generate.
pub const DEFAULT_OLLAMA_HOST: &str = "http://127.0.0.1:11434";

/// Child / probe env key Ollama reads for its server address.
pub const OLLAMA_HOST_ENV: &str = "OLLAMA_HOST";

/// Effective endpoint for both `ollama list` and process `ollama run` (`#366`).
///
/// `configured` comes from Settings when present. Ambient process `OLLAMA_HOST`
/// is **ignored** so discovery cannot silently target a different server than run.
pub fn effective_ollama_host(configured: Option<&str>) -> String {
    configured
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_OLLAMA_HOST)
        .to_string()
}
/// One row from `ollama list` (name column only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OllamaListEntry {
    pub name: String,
}

/// Freshness of the host `ollama list` after refresh (`#365`).
///
/// Distinguishes a successful empty list («відсутня») from a failed refresh that
/// must keep the prior names («невідомо»).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OllamaHostListFreshness {
    /// Last successful list was empty, or no successful list yet.
    #[default]
    Absent,
    /// Last successful list returned one or more names.
    Present,
    /// Last refresh failed; [`OllamaHostListSnapshot::names`] may still hold the prior snapshot.
    Unknown,
}

/// Host list names plus freshness for Settings / Use Ollama (`#365`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OllamaHostListSnapshot {
    pub names: Vec<String>,
    pub freshness: OllamaHostListFreshness,
}

/// Apply one refresh outcome without inventing or wiping names on failure.
///
/// - `Ok([])` → Absent (names cleared).
/// - `Ok([…])` → Present.
/// - `Err(_)` → Unknown, previous names preserved.
pub fn apply_ollama_list_refresh(
    previous: &OllamaHostListSnapshot,
    outcome: Result<Vec<String>, String>,
) -> OllamaHostListSnapshot {
    match outcome {
        Ok(names) if names.is_empty() => OllamaHostListSnapshot {
            names: Vec::new(),
            freshness: OllamaHostListFreshness::Absent,
        },
        Ok(names) => OllamaHostListSnapshot {
            names,
            freshness: OllamaHostListFreshness::Present,
        },
        Err(_) => OllamaHostListSnapshot {
            names: previous.names.clone(),
            freshness: OllamaHostListFreshness::Unknown,
        },
    }
}

/// Parse tabular `ollama list` stdout (header + whitespace columns).
///
/// First column is the model name (`name:tag` or `org/name:tag`). Empty /
/// header-only output → empty vec (not an error).
pub fn parse_ollama_list_stdout(stdout: &str) -> Vec<OllamaListEntry> {
    let mut out = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut cols = line.split_whitespace();
        let Some(name) = cols.next() else {
            continue;
        };
        if name.eq_ignore_ascii_case("NAME") {
            continue;
        }
        out.push(OllamaListEntry {
            name: name.to_string(),
        });
    }
    out
}

/// Resolve program path or PATH name (existence not required until spawn).
pub fn resolve_ollama_bin(explicit: Option<&str>) -> PathBuf {
    match explicit.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => PathBuf::from(s),
        None => PathBuf::from(DEFAULT_OLLAMA_BIN),
    }
}

/// Run `ollama list` on the Desktop host and return model names.
///
/// `#366`: always sets `OLLAMA_HOST` to [`effective_ollama_host`] (Settings or
/// default). Ambient parent `OLLAMA_HOST` does not redirect discovery alone.
///
/// Fail-closed when the binary is missing, exits non-zero, or exceeds timeout
/// (Pack D / audit #3 — no unbounded UI hang). On timeout the child is killed
/// and waited so hang probes do not accumulate orphans.
pub fn list_ollama_models(bin: impl AsRef<Path>) -> Result<Vec<OllamaListEntry>> {
    list_ollama_models_at(bin, &effective_ollama_host(None), Duration::from_secs(15))
}

/// Same as [`list_ollama_models`] with an explicit endpoint and wait bound.
pub fn list_ollama_models_at(
    bin: impl AsRef<Path>,
    host: &str,
    timeout: Duration,
) -> Result<Vec<OllamaListEntry>> {
    let bin = bin.as_ref();
    let host = effective_ollama_host(Some(host));
    let mut child = Command::new(bin)
        .arg("list")
        .env(OLLAMA_HOST_ENV, &host)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("spawn `{} list` (is ollama installed?)", bin.display()))?;

    let start = Instant::now();
    loop {
        match child
            .try_wait()
            .with_context(|| format!("wait `{} list`", bin.display()))?
        {
            Some(status) => {
                let mut stdout = String::new();
                let mut stderr = String::new();
                if let Some(mut out) = child.stdout.take() {
                    let _ = out.read_to_string(&mut stdout);
                }
                if let Some(mut err) = child.stderr.take() {
                    let _ = err.read_to_string(&mut stderr);
                }
                if !status.success() {
                    bail!(
                        "`{} list` failed (status {:?}): {}",
                        bin.display(),
                        status.code(),
                        stderr.trim()
                    );
                }
                return Ok(parse_ollama_list_stdout(&stdout));
            }
            None => {
                if start.elapsed() >= timeout {
                    let child_pid = child.id();
                    let _ = child.kill();
                    let _ = child.wait();
                    bail!(
                        "`ollama list` timed out after {}ms (fail-closed; killed pid {child_pid}; UI not blocked forever)",
                        timeout.as_millis()
                    );
                }
                std::thread::sleep(Duration::from_millis(25));
            }
        }
    }
}

/// Same as [`list_ollama_models`] with an explicit wait bound (default host).
pub fn list_ollama_models_with_timeout(
    bin: impl AsRef<Path>,
    timeout: Duration,
) -> Result<Vec<OllamaListEntry>> {
    list_ollama_models_at(bin, &effective_ollama_host(None), timeout)
}

/// Optional field from Ollama JSON (`#368`).
///
/// Absent or JSON `null` → [`Unknown`](Self::Unknown). Never collapse that into
/// a known empty string, empty list, or boolean `false`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OllamaJsonField<T> {
    /// Key missing or explicitly null — not the same as a known empty value.
    Unknown,
    /// Key present with a typed value (may be empty string / empty vec when API said so).
    Known(T),
}

impl<T> Default for OllamaJsonField<T> {
    /// Default is Unknown — never invent Known empty / false.
    fn default() -> Self {
        Self::Unknown
    }
}

impl<T> OllamaJsonField<T> {
    /// True when the API did not supply this field.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

/// One model object from `GET /api/tags` (`#368`).
///
/// Locality (`remote_host` / `remote_model`) and `capabilities` are optional.
/// Missing keys stay [`OllamaJsonField::Unknown`] — not invented local/`false`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OllamaTagsModel {
    pub name: String,
    pub digest: OllamaJsonField<String>,
    pub size: OllamaJsonField<u64>,
    pub remote_host: OllamaJsonField<String>,
    pub remote_model: OllamaJsonField<String>,
    /// Classic `/api/tags` omits capabilities; untreated absence stays Unknown.
    pub capabilities: OllamaJsonField<Vec<String>>,
}

/// Soft cap so a runaway tags body cannot pin Desktop memory (`#368`; cache/generation is `#370`).
const OLLAMA_TAGS_MAX_BYTES: usize = 16 * 1024 * 1024;

/// Parse `GET /api/tags` JSON body into model rows (`#368`).
///
/// Requires a top-level `models` array. Each element needs a string `name`.
/// Optional fields use [`OllamaJsonField`]: absent/`null` → Unknown (no invented `false`).
pub fn parse_ollama_tags_json(raw: &str) -> Result<Vec<OllamaTagsModel>> {
    let v: Value = serde_json::from_str(raw).context("parse /api/tags JSON")?;
    let models = v
        .get("models")
        .and_then(|m| m.as_array())
        .ok_or_else(|| anyhow::anyhow!("/api/tags missing models array"))?;
    let mut out = Vec::with_capacity(models.len());
    for (i, item) in models.iter().enumerate() {
        let obj = item
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("/api/tags models[{i}] is not an object"))?;
        let name = obj
            .get("name")
            .and_then(|n| n.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow::anyhow!("/api/tags models[{i}] missing name"))?
            .to_string();
        out.push(OllamaTagsModel {
            name,
            digest: optional_string_field(obj.get("digest")),
            size: optional_u64_field(obj.get("size")),
            remote_host: optional_string_field(obj.get("remote_host")),
            remote_model: optional_string_field(obj.get("remote_model")),
            capabilities: optional_string_list_field(obj.get("capabilities")),
        });
    }
    Ok(out)
}

/// CLI names from a tags snapshot (order preserved).
pub fn ollama_tags_model_names(models: &[OllamaTagsModel]) -> Vec<String> {
    models.iter().map(|m| m.name.clone()).collect()
}

/// `GET {host}/api/tags` and parse (`#368`).
///
/// Uses [`effective_ollama_host`]. Only `http://` bases are supported here (no TLS).
/// Timeout applies to connect and read. Body larger than [`OLLAMA_TAGS_MAX_BYTES`] fails closed.
pub fn fetch_ollama_tags_at(host: &str, timeout: Duration) -> Result<Vec<OllamaTagsModel>> {
    let payload = ollama_http_json(host, "GET", "/api/tags", None, timeout)?;
    parse_ollama_tags_json(&payload)
}

/// Same as [`fetch_ollama_tags_at`] with the default Desktop endpoint.
pub fn fetch_ollama_tags(timeout: Duration) -> Result<Vec<OllamaTagsModel>> {
    fetch_ollama_tags_at(&effective_ollama_host(None), timeout)
}

/// Fields from `POST /api/show` used to fill Unknown fitness metadata (`#369`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OllamaShowFields {
    pub remote_host: OllamaJsonField<String>,
    pub remote_model: OllamaJsonField<String>,
    pub capabilities: OllamaJsonField<Vec<String>>,
}

/// True when any fitness field is still Unknown after `/api/tags` (`#369`).
///
/// Locality (`remote_host` / `remote_model`) and `capabilities` are both
/// evaluation inputs. Known capabilities with Unknown locality still need show.
pub fn ollama_model_needs_show(model: &OllamaTagsModel) -> bool {
    model.capabilities.is_unknown()
        || model.remote_host.is_unknown()
        || model.remote_model.is_unknown()
}

/// Parse `POST /api/show` JSON (`#369`). Missing fields stay Unknown.
pub fn parse_ollama_show_json(raw: &str) -> Result<OllamaShowFields> {
    let v: Value = serde_json::from_str(raw).context("parse /api/show JSON")?;
    let obj = v
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("/api/show root is not an object"))?;
    Ok(OllamaShowFields {
        remote_host: optional_string_field(obj.get("remote_host")),
        remote_model: optional_string_field(obj.get("remote_model")),
        capabilities: optional_string_list_field(obj.get("capabilities")),
    })
}

/// Fill only Unknown tags fields from show; never overwrite Known (`#369`).
pub fn merge_tags_with_show(tags: &OllamaTagsModel, show: &OllamaShowFields) -> OllamaTagsModel {
    OllamaTagsModel {
        name: tags.name.clone(),
        digest: tags.digest.clone(),
        size: tags.size.clone(),
        remote_host: fill_unknown(&tags.remote_host, &show.remote_host),
        remote_model: fill_unknown(&tags.remote_model, &show.remote_model),
        capabilities: fill_unknown(&tags.capabilities, &show.capabilities),
    }
}

/// `POST {host}/api/show` for one CLI name (`#369`).
pub fn fetch_ollama_show_at(
    host: &str,
    model_name: &str,
    timeout: Duration,
) -> Result<OllamaShowFields> {
    let name = model_name.trim();
    if name.is_empty() {
        bail!("/api/show requires a non-empty model name");
    }
    let body = serde_json::to_vec(&serde_json::json!({ "name": name }))
        .context("serialize /api/show body")?;
    let payload = ollama_http_json(host, "POST", "/api/show", Some(body.as_slice()), timeout)?;
    parse_ollama_show_json(&payload)
}

/// If fitness fields are incomplete, call `/api/show` and merge (`#369`).
///
/// When [`ollama_model_needs_show`] is false, returns `tags` unchanged (no HTTP).
pub fn enrich_ollama_model_with_show_at(
    host: &str,
    tags: &OllamaTagsModel,
    timeout: Duration,
) -> Result<OllamaTagsModel> {
    if !ollama_model_needs_show(tags) {
        return Ok(tags.clone());
    }
    let show = fetch_ollama_show_at(host, &tags.name, timeout)?;
    Ok(merge_tags_with_show(tags, &show))
}

fn fill_unknown<T: Clone>(
    current: &OllamaJsonField<T>,
    incoming: &OllamaJsonField<T>,
) -> OllamaJsonField<T> {
    match current {
        OllamaJsonField::Known(_) => current.clone(),
        OllamaJsonField::Unknown => incoming.clone(),
    }
}

fn optional_string_field(v: Option<&Value>) -> OllamaJsonField<String> {
    match v {
        None | Some(Value::Null) => OllamaJsonField::Unknown,
        Some(Value::String(s)) => OllamaJsonField::Known(s.clone()),
        Some(other) => {
            // Non-string present value: keep Unknown rather than inventing "" / false.
            let _ = other;
            OllamaJsonField::Unknown
        }
    }
}

fn optional_u64_field(v: Option<&Value>) -> OllamaJsonField<u64> {
    match v {
        None | Some(Value::Null) => OllamaJsonField::Unknown,
        Some(Value::Number(n)) => n
            .as_u64()
            .map(OllamaJsonField::Known)
            .unwrap_or(OllamaJsonField::Unknown),
        Some(_) => OllamaJsonField::Unknown,
    }
}

fn optional_string_list_field(v: Option<&Value>) -> OllamaJsonField<Vec<String>> {
    match v {
        None | Some(Value::Null) => OllamaJsonField::Unknown,
        Some(Value::Array(items)) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                match item.as_str() {
                    Some(s) => out.push(s.to_string()),
                    None => return OllamaJsonField::Unknown,
                }
            }
            OllamaJsonField::Known(out)
        }
        Some(_) => OllamaJsonField::Unknown,
    }
}

/// `http://host:port[/…]` → `(host:port, Host header)`. HTTPS is out of scope for `#368`/`#369`.
fn ollama_http_authority(base: &str) -> Result<(String, String)> {
    let trimmed = base.trim().trim_end_matches('/');
    let rest = trimmed.strip_prefix("http://").ok_or_else(|| {
        anyhow::anyhow!(
            "Ollama HTTP API requires http:// base (got {base}); TLS endpoints are out of #368/#369"
        )
    })?;
    let authority = rest
        .split('/')
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Ollama base missing host: {base}"))?;
    Ok((authority.to_string(), authority.to_string()))
}

fn ollama_http_json(
    host: &str,
    method: &str,
    path: &str,
    body: Option<&[u8]>,
    timeout: Duration,
) -> Result<String> {
    let base = effective_ollama_host(Some(host));
    let (tcp_addr, host_header) = ollama_http_authority(&base)?;
    let addr = crate::health::resolve_listen(&tcp_addr)
        .with_context(|| format!("resolve Ollama {path} endpoint {tcp_addr}"))?;
    let mut stream = TcpStream::connect_timeout(&addr, timeout)
        .with_context(|| format!("connect Ollama {path} at {tcp_addr}"))?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    let mut req =
        format!("{method} {path} HTTP/1.1\r\nHost: {host_header}\r\nAccept: application/json\r\n");
    match body {
        Some(bytes) => {
            req.push_str("Content-Type: application/json\r\n");
            req.push_str(&format!("Content-Length: {}\r\n", bytes.len()));
            req.push_str("Connection: close\r\n\r\n");
            let mut msg = req.into_bytes();
            msg.extend_from_slice(bytes);
            stream.write_all(&msg)?;
        }
        None => {
            req.push_str("Connection: close\r\n\r\n");
            stream.write_all(req.as_bytes())?;
        }
    }
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                if buf.len().saturating_add(n) > OLLAMA_TAGS_MAX_BYTES {
                    bail!("Ollama {path} body exceeds {OLLAMA_TAGS_MAX_BYTES} bytes (fail-closed)");
                }
                buf.extend_from_slice(&chunk[..n]);
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                bail!("Ollama {path} timed out after {}ms", timeout.as_millis());
            }
            Err(e) => return Err(e).context(format!("read Ollama {path}")),
        }
    }
    let (status, payload) = parse_ollama_http_response(&buf)?;
    if !(200..300).contains(&status) {
        bail!(
            "Ollama {path} HTTP {status}: {}",
            payload.chars().take(200).collect::<String>()
        );
    }
    Ok(payload)
}

fn parse_ollama_http_response(raw: &[u8]) -> Result<(u16, String)> {
    let text = std::str::from_utf8(raw).context("Ollama HTTP response not UTF-8")?;
    let (head, body) = text
        .split_once("\r\n\r\n")
        .or_else(|| text.split_once("\n\n"))
        .ok_or_else(|| anyhow::anyhow!("malformed Ollama HTTP response"))?;
    let status_line = head.lines().next().unwrap_or("");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("no HTTP status: {status_line}"))?;
    Ok((status, body.to_string()))
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn parse_skips_header_and_keeps_names() {
        let raw = "\
NAME                                                                 ID              SIZE      MODIFIED
llama3:latest                                                        365c0bd3c000    4.7 GB    7 hours ago
koill/sentence-transformers:paraphrase-multilingual-minilm-l12-v2    3ee258ffc9f2    476 MB    7 hours ago
";
        let rows = parse_ollama_list_stdout(raw);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "llama3:latest");
        assert_eq!(
            rows[1].name,
            "koill/sentence-transformers:paraphrase-multilingual-minilm-l12-v2"
        );
    }

    #[test]
    fn parse_empty_is_empty() {
        assert!(parse_ollama_list_stdout("").is_empty());
        assert!(parse_ollama_list_stdout("NAME ID SIZE MODIFIED\n").is_empty());
    }

    #[test]
    fn resolve_default_bin() {
        assert_eq!(resolve_ollama_bin(None), PathBuf::from(DEFAULT_OLLAMA_BIN));
        assert_eq!(
            resolve_ollama_bin(Some("  /usr/local/bin/ollama ")),
            PathBuf::from("/usr/local/bin/ollama")
        );
    }

    /// `#365`: failed refresh keeps names and marks Unknown.
    #[test]
    fn refresh_error_keeps_snapshot_as_unknown() {
        let prev = OllamaHostListSnapshot {
            names: vec!["phi:latest".into(), "llama3:latest".into()],
            freshness: OllamaHostListFreshness::Present,
        };
        let failed = apply_ollama_list_refresh(&prev, Err("spawn failed".into()));
        assert_eq!(failed.names, prev.names);
        assert_eq!(failed.freshness, OllamaHostListFreshness::Unknown);
    }

    /// `#365`: successful empty list is Absent (not Unknown).
    #[test]
    fn refresh_empty_ok_is_absent() {
        let prev = OllamaHostListSnapshot {
            names: vec!["phi:latest".into()],
            freshness: OllamaHostListFreshness::Present,
        };
        let empty_ok = apply_ollama_list_refresh(&prev, Ok(Vec::new()));
        assert!(empty_ok.names.is_empty());
        assert_eq!(empty_ok.freshness, OllamaHostListFreshness::Absent);

        let present = apply_ollama_list_refresh(&empty_ok, Ok(vec!["b:latest".into()]));
        assert_eq!(present.names, vec!["b:latest".to_string()]);
        assert_eq!(present.freshness, OllamaHostListFreshness::Present);
    }

    /// `#366`: ambient `OLLAMA_HOST` must not become the discovery/run endpoint.
    #[test]
    fn ambient_ollama_host_does_not_split_discovery_and_run() {
        let prev = std::env::var_os(OLLAMA_HOST_ENV);
        std::env::set_var(OLLAMA_HOST_ENV, "http://ambient-split.example:9999");
        let discovery = effective_ollama_host(None);
        let run = effective_ollama_host(None);
        match prev {
            Some(v) => std::env::set_var(OLLAMA_HOST_ENV, v),
            None => std::env::remove_var(OLLAMA_HOST_ENV),
        }
        assert_eq!(discovery, DEFAULT_OLLAMA_HOST);
        assert_eq!(run, discovery);
        assert_eq!(
            effective_ollama_host(Some("  http://configured.example:11434  ")),
            "http://configured.example:11434"
        );
    }

    /// `#368`: missing `remote_host` / capabilities stay Unknown — never invented `false`/local.
    #[test]
    fn tags_parser_missing_fields_are_unknown_not_false() {
        let raw = r#"{
  "models": [
    {
      "name": "llama3:latest",
      "digest": "abc",
      "size": 100
    },
    {
      "name": "kimi-k2.7-code:cloud",
      "remote_host": "https://ollama.com",
      "remote_model": "kimi-k2.7-code",
      "digest": "def",
      "size": 320,
      "capabilities": ["completion"]
    }
  ]
}"#;
        let rows = parse_ollama_tags_json(raw).unwrap();
        assert_eq!(rows.len(), 2);

        let local = &rows[0];
        assert_eq!(local.name, "llama3:latest");
        assert!(
            local.remote_host.is_unknown(),
            "absent remote_host must be Unknown"
        );
        assert!(local.remote_model.is_unknown());
        assert!(
            local.capabilities.is_unknown(),
            "absent capabilities must be Unknown, not Known([]) / false"
        );
        assert_eq!(local.digest, OllamaJsonField::Known("abc".into()));
        assert_eq!(local.size, OllamaJsonField::Known(100));
        // No invented boolean: callers must not treat Unknown as is_remote=false.
        let invented_remote = matches!(local.remote_host, OllamaJsonField::Known(_));
        assert!(!invented_remote);

        let cloud = &rows[1];
        assert_eq!(
            cloud.remote_host,
            OllamaJsonField::Known("https://ollama.com".into())
        );
        assert_eq!(
            cloud.remote_model,
            OllamaJsonField::Known("kimi-k2.7-code".into())
        );
        assert_eq!(
            cloud.capabilities,
            OllamaJsonField::Known(vec!["completion".into()])
        );

        assert_eq!(
            ollama_tags_model_names(&rows),
            vec![
                "llama3:latest".to_string(),
                "kimi-k2.7-code:cloud".to_string()
            ]
        );
    }

    /// `#368`: JSON null and empty-string presence are distinct from invented false.
    #[test]
    fn tags_parser_null_is_unknown_empty_string_is_known() {
        let raw = r#"{
  "models": [
    { "name": "a:latest", "remote_host": null },
    { "name": "b:latest", "remote_host": "" }
  ]
}"#;
        let rows = parse_ollama_tags_json(raw).unwrap();
        assert!(rows[0].remote_host.is_unknown());
        assert_eq!(rows[1].remote_host, OllamaJsonField::Known(String::new()));
    }

    /// `#368`: HTTP GET /api/tags against a local stub.
    #[test]
    fn fetch_ollama_tags_reads_http_json() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let body = r#"{"models":[{"name":"phi:latest"}]}"#;
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(resp.as_bytes());
        });
        let host = format!("http://{addr}");
        let rows = fetch_ollama_tags_at(&host, Duration::from_secs(2)).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "phi:latest");
        assert!(rows[0].remote_host.is_unknown());
        assert!(rows[0].capabilities.is_unknown());
    }

    /// `#369`: capabilities known but locality Unknown still requires /api/show.
    #[test]
    fn show_needed_when_capabilities_known_but_locality_unknown() {
        let with_caps = OllamaTagsModel {
            name: "phi:latest".into(),
            digest: OllamaJsonField::Known("d".into()),
            size: OllamaJsonField::Known(1),
            remote_host: OllamaJsonField::Unknown,
            remote_model: OllamaJsonField::Unknown,
            capabilities: OllamaJsonField::Known(vec!["completion".into()]),
        };
        assert!(
            ollama_model_needs_show(&with_caps),
            "missing locality must trigger show even when capabilities are known"
        );

        let complete = OllamaTagsModel {
            name: "phi:latest".into(),
            digest: OllamaJsonField::Known("d".into()),
            size: OllamaJsonField::Known(1),
            remote_host: OllamaJsonField::Known(String::new()),
            remote_model: OllamaJsonField::Known(String::new()),
            capabilities: OllamaJsonField::Known(vec!["completion".into()]),
        };
        assert!(!ollama_model_needs_show(&complete));
    }

    /// `#369`: enrich POSTs /api/show and fills Unknown locality without clobbering caps.
    #[test]
    fn enrich_calls_show_when_locality_missing() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::{Arc, Mutex};
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let seen = Arc::new(Mutex::new(Vec::<String>::new()));
        let seen_bg = Arc::clone(&seen);
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            seen_bg.lock().unwrap().push(req.clone());
            assert!(
                req.starts_with("POST /api/show"),
                "expected show, got {req}"
            );
            let body = r#"{"remote_host":"https://ollama.com","remote_model":"kimi","capabilities":["completion","vision"]}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(resp.as_bytes());
        });

        let tags = OllamaTagsModel {
            name: "kimi-k2.7-code:cloud".into(),
            digest: OllamaJsonField::Known("e".into()),
            size: OllamaJsonField::Known(320),
            remote_host: OllamaJsonField::Unknown,
            remote_model: OllamaJsonField::Unknown,
            capabilities: OllamaJsonField::Known(vec!["completion".into()]),
        };
        let host = format!("http://{addr}");
        let enriched =
            enrich_ollama_model_with_show_at(&host, &tags, Duration::from_secs(2)).unwrap();
        assert_eq!(
            enriched.remote_host,
            OllamaJsonField::Known("https://ollama.com".into())
        );
        assert_eq!(enriched.remote_model, OllamaJsonField::Known("kimi".into()));
        assert_eq!(
            enriched.capabilities,
            OllamaJsonField::Known(vec!["completion".into()]),
            "Known capabilities must not be overwritten by show"
        );
        assert!(!seen.lock().unwrap().is_empty());
    }

    /// `#369`: complete tags skip HTTP show.
    #[test]
    fn enrich_skips_show_when_fitness_fields_known() {
        let tags = OllamaTagsModel {
            name: "phi:latest".into(),
            digest: OllamaJsonField::Unknown,
            size: OllamaJsonField::Unknown,
            remote_host: OllamaJsonField::Known(String::new()),
            remote_model: OllamaJsonField::Known(String::new()),
            capabilities: OllamaJsonField::Known(vec!["completion".into()]),
        };
        // No server — would fail if HTTP were attempted.
        let out = enrich_ollama_model_with_show_at(
            "http://127.0.0.1:1",
            &tags,
            Duration::from_millis(50),
        )
        .unwrap();
        assert_eq!(out, tags);
    }

    /// Pack D / P2: hang script must fail-closed via kill+wait (no orphan sleep).
    #[cfg(unix)]
    #[test]
    fn list_ollama_models_timeout_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        let pidfile = dir.path().join("hang.pid");
        let script = dir.path().join("hang-ollama.sh");
        let pidfile_disp = pidfile.display().to_string();
        std::fs::write(
            &script,
            format!("#!/bin/sh\necho $$ > '{pidfile_disp}'\nexec sleep 30\n"),
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script, perms).unwrap();
        let err = list_ollama_models_with_timeout(&script, Duration::from_millis(200))
            .expect_err("hang must timeout");
        assert!(
            err.to_string().contains("timed out"),
            "expected timeout, got {err:#}"
        );
        // Child must be dead — no orphan accumulation across CI runs.
        let pid_text = std::fs::read_to_string(&pidfile).unwrap_or_default();
        let pid: u32 = pid_text.trim().parse().expect("hang script wrote pid");
        assert!(
            !crate::process::pid_alive(pid),
            "hang child pid {pid} must be killed+waited after list timeout"
        );
    }
}
