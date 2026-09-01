//! Local CLI generate backend (QUEUE #215 / Analyze-250).
//!
//! Spawns a **fixed argv** via [`std::process::Command::new`] (explicit program +
//! args). Never `sh -c`, never a user-controlled shell string.
//!
//! Network (RFC-0105 / RFC-0110): the payload still requires `network=none`.
//! This adapter opens **no sockets**. A child such as `ollama` may talk to a
//! loopback daemon; AIRA does not initiate WAN. llama.cpp-style argv is offline.
//!
//! Missing binary, spawn failure, non-zero exit, timeout, or empty stdout →
//! error string for [`EventType::CapsuleFailed`](aira_event::EventType::CapsuleFailed)
//! — never a fake VERIFIED result.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::{env, thread};

use serde_json::{json, Value};

use super::{GenerateBackend, GenerateLocalPayload, ACTION_GENERATE_LOCAL, MOCK_BACKEND_ID};

/// Backend id stamped on successful process output.
pub const PROCESS_BACKEND_ID: &str = "process";

/// Fail-closed when the CLI binary is not on PATH and not an existing file.
pub const MISSING_BINARY: &str = "generate process binary missing (fail-closed; not VERIFIED)";

/// Fail-closed spawn error (not NotFound).
pub const SPAWN_FAILED: &str = "generate process spawn failed (fail-closed; not VERIFIED)";

/// Fail-closed non-zero child exit.
pub const NONZERO_EXIT: &str = "generate process exited non-zero (fail-closed; not VERIFIED)";

/// Fail-closed wait timeout.
pub const TIMED_OUT: &str = "generate process timed out (fail-closed; not VERIFIED)";

/// Fail-closed empty stdout.
pub const EMPTY_STDOUT: &str = "generate process produced no stdout (fail-closed; not VERIFIED)";

/// `AIRA_LLM_BACKEND=mock|process`. Unset / anything else → mock (CI default).
pub const ENV_LLM_BACKEND: &str = "AIRA_LLM_BACKEND";

/// Program name or filesystem path. Not a marketplace id.
pub const ENV_PROCESS_BIN: &str = "AIRA_LLM_PROCESS_BIN";

/// Extra argv tokens, whitespace-split. Not passed to a shell.
pub const ENV_PROCESS_ARGS: &str = "AIRA_LLM_PROCESS_ARGS";

/// Child wait timeout in milliseconds (default 30000).
pub const ENV_PROCESS_TIMEOUT_MS: &str = "AIRA_LLM_PROCESS_TIMEOUT_MS";

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const STDERR_LIMIT: usize = 512;

/// Select mock vs process from an env value. Default (None / not `process`) is mock.
pub fn backend_kind_from(value: Option<&str>) -> &'static str {
    match value {
        Some(v) if v.eq_ignore_ascii_case("process") => PROCESS_BACKEND_ID,
        _ => MOCK_BACKEND_ID,
    }
}

/// [`backend_kind_from`] over [`ENV_LLM_BACKEND`].
pub fn backend_kind_from_env() -> &'static str {
    backend_kind_from(env::var(ENV_LLM_BACKEND).ok().as_deref())
}

/// Host-local CLI adapter (ollama and/or llama.cpp-style argv).
#[derive(Debug, Clone)]
pub struct ProcessBackend {
    program: PathBuf,
    args: Vec<String>,
    timeout: Duration,
}

impl ProcessBackend {
    /// Named program (PATH) or explicit filesystem path. No shell.
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// llama.cpp-style: `{program} -p {prompt}`.
    pub fn llama_cpp(program: impl Into<PathBuf>) -> Self {
        Self::new(program).with_args(["-p"])
    }

    /// ollama-style: `{program} run {model} {prompt}`.
    ///
    /// AIRA still does not open sockets. The child may use loopback only.
    pub fn ollama(program: impl Into<PathBuf>, model: impl Into<String>) -> Self {
        Self::new(program).with_args(["run", &model.into()])
    }

    /// Fixed extra argv (not including the prompt). Tokens are not shell-parsed.
    pub fn with_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    /// Child wait timeout. Elapsed wait → fail-closed.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Config from env. Missing binary is **not** resolved here; generate fails closed.
    pub fn from_env() -> Self {
        let program = env::var(ENV_PROCESS_BIN).unwrap_or_else(|_| "ollama".into());
        let mut backend = Self::new(program);
        if let Ok(raw) = env::var(ENV_PROCESS_ARGS) {
            backend = backend.with_args(raw.split_whitespace().map(str::to_string));
        }
        if let Ok(ms) = env::var(ENV_PROCESS_TIMEOUT_MS) {
            if let Ok(n) = ms.parse::<u64>() {
                backend = backend.with_timeout(Duration::from_millis(n));
            }
        }
        backend
    }

    fn resolve_program(&self) -> Result<PathBuf, String> {
        resolve_program(&self.program)
    }
}

impl GenerateBackend for ProcessBackend {
    fn generate(&self, payload: &GenerateLocalPayload) -> Result<Value, String> {
        payload.validate()?;
        let program = self.resolve_program()?;
        let mut cmd = Command::new(&program);
        cmd.args(&self.args)
            .arg(&payload.prompt)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                MISSING_BINARY.to_string()
            } else {
                format!("{SPAWN_FAILED}: {e}")
            }
        })?;
        let mut stdout_pipe = child
            .stdout
            .take()
            .ok_or_else(|| format!("{SPAWN_FAILED}: stdout pipe missing"))?;
        let mut stderr_pipe = child
            .stderr
            .take()
            .ok_or_else(|| format!("{SPAWN_FAILED}: stderr pipe missing"))?;
        let out_h = thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stdout_pipe.read_to_end(&mut buf);
            buf
        });
        let err_h = thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stderr_pipe.read_to_end(&mut buf);
            buf
        });
        let status = wait_with_timeout(&mut child, self.timeout)?;
        let stdout = out_h.join().unwrap_or_default();
        let stderr = err_h.join().unwrap_or_default();
        if !status.success() {
            let err = truncate_utf8(&stderr);
            return Err(format!("{NONZERO_EXIT}: {err}"));
        }
        let text = String::from_utf8_lossy(&stdout).trim().to_string();
        if text.is_empty() {
            return Err(EMPTY_STDOUT.into());
        }
        Ok(json!({
            "result": text,
            "action": ACTION_GENERATE_LOCAL,
            "backend": PROCESS_BACKEND_ID,
        }))
    }
}

fn resolve_program(program: &Path) -> Result<PathBuf, String> {
    if program.as_os_str().is_empty() {
        return Err(MISSING_BINARY.into());
    }
    if program.components().count() > 1 || program.is_absolute() {
        if program.is_file() {
            return Ok(program.to_path_buf());
        }
        return Err(MISSING_BINARY.into());
    }
    let path_os = env::var_os("PATH").ok_or_else(|| MISSING_BINARY.to_string())?;
    for dir in env::split_paths(&path_os) {
        let candidate = dir.join(program);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(MISSING_BINARY.into())
}

fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<std::process::ExitStatus, String> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(TIMED_OUT.into());
                }
                thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                return Err(format!(
                    "generate process wait failed (fail-closed; not VERIFIED): {e}"
                ));
            }
        }
    }
}

fn truncate_utf8(bytes: &[u8]) -> String {
    let s = String::from_utf8_lossy(bytes);
    let t = s.trim();
    if t.len() <= STDERR_LIMIT {
        t.to_string()
    } else {
        let mut out = t.chars().take(STDERR_LIMIT).collect::<String>();
        out.push('…');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_kind_defaults_to_mock() {
        assert_eq!(backend_kind_from(None), MOCK_BACKEND_ID);
        assert_eq!(backend_kind_from(Some("")), MOCK_BACKEND_ID);
        assert_eq!(backend_kind_from(Some("mock")), MOCK_BACKEND_ID);
        assert_eq!(backend_kind_from(Some("PROCESS")), PROCESS_BACKEND_ID);
        assert_eq!(backend_kind_from(Some("process")), PROCESS_BACKEND_ID);
    }

    #[test]
    fn resolve_missing_bare_name_is_fail_closed() {
        let err = resolve_program(Path::new("aira-llm-process-missing-bin-215-do-not-install"))
            .unwrap_err();
        assert_eq!(err, MISSING_BINARY);
    }

    #[test]
    fn resolve_missing_absolute_path_is_fail_closed() {
        let err = resolve_program(Path::new(
            "/tmp/aira-llm-process-missing-bin-215-do-not-install",
        ))
        .unwrap_err();
        assert_eq!(err, MISSING_BINARY);
    }

    #[cfg(unix)]
    #[test]
    fn resolve_bin_false_exists() {
        let p = resolve_program(Path::new("/bin/false")).unwrap();
        assert_eq!(p, PathBuf::from("/bin/false"));
    }
}
