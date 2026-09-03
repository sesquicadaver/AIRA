//! Local CLI generate backend (QUEUE #215 / Analyze-250; env `#219`; pipes `#220`;
//! Landlock `#225`).
//!
//! Spawns a **fixed argv** via [`std::process::Command::new`] (explicit program +
//! args). Never `sh -c`, never a user-controlled shell string.
//!
//! Child environment (`#219`): [`Command::env_clear`] then only PATH / HOME / LANG.
//! Host secrets such as `AIRA_HTTP_TOKEN` MUST NOT be inherited.
//!
//! Pipes (`#220`): stdout/stderr are capped **during** read. Overflow → fail-closed,
//! never a truncated CapsuleCompleted / fake VERIFIED.
//!
//! Network (RFC-0105 / RFC-0110 / RFC-0116): `constraints.network = none` is
//! **AIRA-mediated**. This adapter opens **no sockets**. It is **not** an OS
//! network-off sandbox (no seccomp / netns in this atom). Opt-in Landlock FS
//! (`#225`) restricts the child filesystem when enabled. A child such as
//! `ollama` may talk to a loopback daemon — an explicit host-process exception,
//! not `network=none` OS enforcement. llama.cpp-style argv is offline.
//!
//! Missing binary, spawn failure, non-zero exit, timeout, empty stdout, or pipe
//! overflow → error string for
//! [`EventType::CapsuleFailed`](aira_event::EventType::CapsuleFailed)
//! — never a fake VERIFIED result.

use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{env, thread};

use serde_json::{json, Value};

use super::landlock::landlock_enabled_from;
#[cfg(target_os = "linux")]
use super::landlock::{default_allow_paths, restrict_fs_self};
use super::{GenerateBackend, GenerateLocalPayload, ACTION_GENERATE_LOCAL, MOCK_BACKEND_ID};

pub use super::landlock::{ENV_LLM_LANDLOCK, LANDLOCK_FAILED, LANDLOCK_UNSUPPORTED};

/// Backend id stamped on successful process output.
pub const PROCESS_BACKEND_ID: &str = "process";

/// Honest `constraints.network = none` (RFC-0116). Adapter-only; not OS isolation.
pub const NETWORK_NONE_CONTRACT: &str =
    "AIRA-mediated none (adapter opens no sockets; child is not OS-isolated)";

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

/// Fail-closed when stdout or stderr exceeds the bound **during** read.
pub const PIPE_OVERFLOW: &str =
    "generate process output exceeded bound (fail-closed; not VERIFIED)";

/// Max stdout bytes retained while reading. Overflow is not truncated success.
pub const PIPE_STDOUT_LIMIT: usize = 1024 * 1024;

/// Max stderr bytes retained while reading. Overflow is not truncated success.
pub const PIPE_STDERR_LIMIT: usize = 64 * 1024;

/// `AIRA_LLM_BACKEND=mock|process`. Unset / anything else → mock (CI default).
pub const ENV_LLM_BACKEND: &str = "AIRA_LLM_BACKEND";

/// Program name or filesystem path. Not a marketplace id.
pub const ENV_PROCESS_BIN: &str = "AIRA_LLM_PROCESS_BIN";

/// Extra argv tokens, whitespace-split. Not passed to a shell.
pub const ENV_PROCESS_ARGS: &str = "AIRA_LLM_PROCESS_ARGS";

/// Child wait timeout in milliseconds (default 30000).
pub const ENV_PROCESS_TIMEOUT_MS: &str = "AIRA_LLM_PROCESS_TIMEOUT_MS";

/// Keys copied into the child after [`Command::env_clear`]. Nothing else.
pub const CHILD_ENV_ALLOWLIST: &[&str] = &["PATH", "HOME", "LANG"];

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const STDERR_SNIPPET: usize = 512;

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
    landlock: bool,
}

impl ProcessBackend {
    /// Named program (PATH) or explicit filesystem path. No shell.
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            timeout: DEFAULT_TIMEOUT,
            landlock: false,
        }
    }

    /// llama.cpp-style: `{program} -p {prompt}`.
    pub fn llama_cpp(program: impl Into<PathBuf>) -> Self {
        Self::new(program).with_args(["-p"])
    }

    /// ollama-style: `{program} run {model} {prompt}`.
    ///
    /// AIRA still does not open sockets (RFC-0116). The child may use loopback
    /// only; that is not OS `network=none` enforcement.
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

    /// Opt-in Linux Landlock FS restrict in the child (`pre_exec`). Default off.
    ///
    /// Apply failure or non-Linux → fail-closed ([`LANDLOCK_FAILED`] /
    /// [`LANDLOCK_UNSUPPORTED`]), never unsandboxed success.
    pub fn with_landlock(mut self) -> Self {
        self.landlock = true;
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
        if landlock_enabled_from(env::var(ENV_LLM_LANDLOCK).ok().as_deref()) {
            backend = backend.with_landlock();
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
        apply_child_env(&mut cmd);
        if self.landlock {
            #[cfg(target_os = "linux")]
            {
                use std::os::unix::process::CommandExt;
                let allow = default_allow_paths(&program);
                // SAFETY: runs in the forked child before exec; Landlock applies to
                // this thread only. Failure returns Err so spawn fail-closes.
                unsafe {
                    cmd.pre_exec(move || restrict_fs_self(&allow));
                }
            }
            #[cfg(not(target_os = "linux"))]
            {
                return Err(LANDLOCK_UNSUPPORTED.to_string());
            }
        }
        let mut child = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                MISSING_BINARY.to_string()
            } else if self.landlock {
                let msg = e.to_string();
                if msg.contains(LANDLOCK_FAILED) {
                    msg
                } else {
                    format!("{LANDLOCK_FAILED}: {e}")
                }
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
        let overflow = Arc::new(AtomicBool::new(false));
        let out_flag = overflow.clone();
        let err_flag = overflow.clone();
        let out_h = thread::spawn(move || {
            let r = read_bounded(&mut stdout_pipe, PIPE_STDOUT_LIMIT);
            if matches!(r, BoundedRead::Overflow) {
                out_flag.store(true, Ordering::SeqCst);
            }
            r
        });
        let err_h = thread::spawn(move || {
            let r = read_bounded(&mut stderr_pipe, PIPE_STDERR_LIMIT);
            if matches!(r, BoundedRead::Overflow) {
                err_flag.store(true, Ordering::SeqCst);
            }
            r
        });
        let wait_result = wait_with_timeout(&mut child, self.timeout, &overflow);
        let stdout_r = out_h.join().unwrap_or(BoundedRead::Io);
        let stderr_r = err_h.join().unwrap_or(BoundedRead::Io);
        if matches!(stdout_r, BoundedRead::Overflow) || matches!(stderr_r, BoundedRead::Overflow) {
            return Err(PIPE_OVERFLOW.into());
        }
        if matches!(stdout_r, BoundedRead::Io) || matches!(stderr_r, BoundedRead::Io) {
            return Err(format!("{SPAWN_FAILED}: pipe read failed"));
        }
        let status = wait_result?;
        let BoundedRead::Complete(stdout) = stdout_r else {
            return Err(PIPE_OVERFLOW.into());
        };
        let BoundedRead::Complete(stderr) = stderr_r else {
            return Err(PIPE_OVERFLOW.into());
        };
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

/// PATH / HOME / LANG only. Host secrets (Bearer token, LLM env) stay in the parent.
fn child_env_pairs() -> Vec<(OsString, OsString)> {
    let mut out = Vec::new();
    match env::var_os("PATH") {
        Some(v) if !v.is_empty() => out.push((OsString::from("PATH"), v)),
        _ => out.push((OsString::from("PATH"), default_path())),
    }
    if let Some(v) = env::var_os("HOME") {
        if !v.is_empty() {
            out.push((OsString::from("HOME"), v));
        }
    }
    match env::var_os("LANG") {
        Some(v) if !v.is_empty() => out.push((OsString::from("LANG"), v)),
        _ => out.push((OsString::from("LANG"), OsString::from("C"))),
    }
    debug_assert!(out
        .iter()
        .all(|(k, _)| CHILD_ENV_ALLOWLIST.iter().any(|a| k == a)));
    out
}

fn default_path() -> OsString {
    #[cfg(windows)]
    {
        OsString::from(r"C:\Windows\System32;C:\Windows")
    }
    #[cfg(not(windows))]
    {
        OsString::from("/usr/bin:/bin")
    }
}

fn apply_child_env(cmd: &mut Command) {
    cmd.env_clear();
    for (key, value) in child_env_pairs() {
        cmd.env(key, value);
    }
}

#[derive(Debug)]
enum BoundedRead {
    Complete(Vec<u8>),
    Overflow,
    Io,
}

/// Cap the pipe **while** reading. Does not `read_to_end` then truncate.
fn read_bounded(reader: &mut impl Read, limit: usize) -> BoundedRead {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) => return BoundedRead::Complete(buf),
            Ok(n) => {
                let remaining = limit.saturating_sub(buf.len());
                if n > remaining {
                    return BoundedRead::Overflow;
                }
                buf.extend_from_slice(&chunk[..n]);
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return BoundedRead::Io,
        }
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
    overflow: &AtomicBool,
) -> Result<std::process::ExitStatus, String> {
    let start = Instant::now();
    loop {
        if overflow.load(Ordering::SeqCst) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(PIPE_OVERFLOW.into());
        }
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
    if t.len() <= STDERR_SNIPPET {
        t.to_string()
    } else {
        let mut out = t.chars().take(STDERR_SNIPPET).collect::<String>();
        out.push('…');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GenerateLocalConstraints, PAYLOAD_SCHEMA_ID};
    use aira_object::local_test_signature;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    fn dummy_payload(prompt: &str) -> GenerateLocalPayload {
        GenerateLocalPayload {
            payload_schema: PAYLOAD_SCHEMA_ID.into(),
            action: ACTION_GENERATE_LOCAL.into(),
            prompt: prompt.into(),
            problem_statement_ref: None,
            model_artifact_ref: None,
            constraints: GenerateLocalConstraints {
                network: "none".into(),
                shell: false,
            },
            provenance_refs: vec![],
            signature: local_test_signature(aira_object::LOCAL_TEST_DOMAIN_MSG),
        }
    }

    fn env_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn backend_kind_defaults_to_mock() {
        assert_eq!(backend_kind_from(None), MOCK_BACKEND_ID);
        assert_eq!(backend_kind_from(Some("")), MOCK_BACKEND_ID);
        assert_eq!(backend_kind_from(Some("mock")), MOCK_BACKEND_ID);
        assert_eq!(backend_kind_from(Some("PROCESS")), PROCESS_BACKEND_ID);
        assert_eq!(backend_kind_from(Some("process")), PROCESS_BACKEND_ID);
    }

    #[test]
    fn process_backend_adapter_does_not_open_sockets() {
        let src = include_str!("process.rs");
        let prod = src
            .split("#[cfg(test)]")
            .next()
            .expect("production ProcessBackend source");
        for needle in [
            "std::net::",
            "TcpStream",
            "TcpListener",
            "UdpSocket",
            "reqwest",
            "hyper::",
            "ureq",
        ] {
            assert!(
                !prod.contains(needle),
                "ProcessBackend must not open sockets; found {needle}"
            );
        }
        assert!(prod.contains(NETWORK_NONE_CONTRACT));
        assert!(prod.contains("OS-isolated"));
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

    #[test]
    fn child_env_pairs_never_include_http_token() {
        let _g = env_lock();
        let prev_token = env::var("AIRA_HTTP_TOKEN").ok();
        let prev_backend = env::var("AIRA_LLM_BACKEND").ok();
        env::set_var("AIRA_HTTP_TOKEN", "l219-secret-do-not-leak");
        env::set_var("AIRA_LLM_BACKEND", "process");
        let pairs = child_env_pairs();
        match prev_token {
            Some(v) => env::set_var("AIRA_HTTP_TOKEN", v),
            None => env::remove_var("AIRA_HTTP_TOKEN"),
        }
        match prev_backend {
            Some(v) => env::set_var("AIRA_LLM_BACKEND", v),
            None => env::remove_var("AIRA_LLM_BACKEND"),
        }
        assert!(!pairs.is_empty());
        for (k, v) in &pairs {
            let key = k.to_string_lossy();
            assert!(
                CHILD_ENV_ALLOWLIST.iter().any(|a| *a == key.as_ref()),
                "unexpected child env key {key}"
            );
            assert_ne!(key.as_ref(), "AIRA_HTTP_TOKEN");
            assert!(!v.to_string_lossy().contains("l219-secret-do-not-leak"));
        }
        assert!(pairs.iter().any(|(k, _)| k == "PATH"));
        assert!(pairs.iter().any(|(k, _)| k == "LANG"));
    }

    #[cfg(unix)]
    #[test]
    fn process_child_does_not_inherit_http_token() {
        let _g = env_lock();
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("dump-env");
        std::fs::write(
            &script,
            "#!/bin/sh\n\
             if [ -n \"$AIRA_HTTP_TOKEN\" ]; then echo LEAKED_HTTP_TOKEN; else echo HTTP_TOKEN_ABSENT; fi\n\
             if [ -n \"$AIRA_LLM_BACKEND\" ]; then echo LEAKED_LLM_BACKEND; else echo LLM_BACKEND_ABSENT; fi\n\
             echo PATH_OK=$PATH\n",
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(&script).unwrap().permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(&script, perm).unwrap();

        let prev_token = env::var("AIRA_HTTP_TOKEN").ok();
        let prev_backend = env::var("AIRA_LLM_BACKEND").ok();
        env::set_var("AIRA_HTTP_TOKEN", "l219-secret-do-not-leak");
        env::set_var("AIRA_LLM_BACKEND", "process");
        let out = ProcessBackend::new(&script)
            .generate(&dummy_payload("ignored-prompt"))
            .expect("dump-env script must complete");
        match prev_token {
            Some(v) => env::set_var("AIRA_HTTP_TOKEN", v),
            None => env::remove_var("AIRA_HTTP_TOKEN"),
        }
        match prev_backend {
            Some(v) => env::set_var("AIRA_LLM_BACKEND", v),
            None => env::remove_var("AIRA_LLM_BACKEND"),
        }
        let text = out["result"].as_str().expect("result string");
        assert!(
            text.contains("HTTP_TOKEN_ABSENT"),
            "child must not inherit AIRA_HTTP_TOKEN, got {text}"
        );
        assert!(!text.contains("LEAKED_HTTP_TOKEN"), "{text}");
        assert!(!text.contains("l219-secret-do-not-leak"), "{text}");
        assert!(
            text.contains("LLM_BACKEND_ABSENT"),
            "child must not inherit AIRA_LLM_BACKEND, got {text}"
        );
        assert!(text.contains("PATH_OK="), "{text}");
        assert_eq!(out["backend"], json!(PROCESS_BACKEND_ID));
    }

    #[test]
    fn read_bounded_ok_under_limit() {
        let data = b"hello".to_vec();
        match read_bounded(&mut std::io::Cursor::new(data), 50) {
            BoundedRead::Complete(buf) => assert_eq!(buf, b"hello"),
            other => panic!("expected complete, got {other:?}"),
        }
    }

    #[test]
    fn read_bounded_overflow_during_read() {
        let data = vec![b'x'; 100];
        assert!(matches!(
            read_bounded(&mut std::io::Cursor::new(data), 50),
            BoundedRead::Overflow
        ));
    }

    #[test]
    fn read_bounded_exact_limit_is_complete() {
        let data = vec![b'y'; 8];
        match read_bounded(&mut std::io::Cursor::new(data), 8) {
            BoundedRead::Complete(buf) => assert_eq!(buf.len(), 8),
            other => panic!("expected complete at limit, got {other:?}"),
        }
    }

    #[cfg(unix)]
    fn chmod_exec(path: &Path) {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(path).unwrap().permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(path, perm).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn stdout_overflow_during_read_is_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("overflow-stdout");
        let kb = (PIPE_STDOUT_LIMIT / 1024) + 512;
        std::fs::write(
            &script,
            format!("#!/bin/sh\ndd if=/dev/zero bs=1024 count={kb} 2>/dev/null\n"),
        )
        .unwrap();
        chmod_exec(&script);
        let err = ProcessBackend::new(&script)
            .with_timeout(Duration::from_secs(10))
            .generate(&dummy_payload("ignored"))
            .unwrap_err();
        assert!(
            err.contains(PIPE_OVERFLOW),
            "stdout overflow must fail-closed, got {err}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn stderr_overflow_during_read_is_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("overflow-stderr");
        let kb = (PIPE_STDERR_LIMIT / 1024) + 64;
        std::fs::write(
            &script,
            format!("#!/bin/sh\ndd if=/dev/zero bs=1024 count={kb} >&2\necho ok\n"),
        )
        .unwrap();
        chmod_exec(&script);
        let err = ProcessBackend::new(&script)
            .with_timeout(Duration::from_secs(10))
            .generate(&dummy_payload("ignored"))
            .unwrap_err();
        assert!(
            err.contains(PIPE_OVERFLOW),
            "stderr overflow must fail-closed, got {err}"
        );
    }

    #[test]
    fn from_env_landlock_opt_in() {
        let _g = env_lock();
        let prev = env::var(ENV_LLM_LANDLOCK).ok();
        env::remove_var(ENV_LLM_LANDLOCK);
        assert!(!ProcessBackend::from_env().landlock);
        env::set_var(ENV_LLM_LANDLOCK, "1");
        assert!(ProcessBackend::from_env().landlock);
        env::set_var(ENV_LLM_LANDLOCK, "true");
        assert!(ProcessBackend::from_env().landlock);
        env::set_var(ENV_LLM_LANDLOCK, "0");
        assert!(!ProcessBackend::from_env().landlock);
        match prev {
            Some(v) => env::set_var(ENV_LLM_LANDLOCK, v),
            None => env::remove_var(ENV_LLM_LANDLOCK),
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn landlock_denies_read_outside_allowlist() {
        let root = tempfile::tempdir().unwrap();
        let jail = root.path().join("jail");
        let secret_dir = root.path().join("secret");
        std::fs::create_dir(&jail).unwrap();
        std::fs::create_dir(&secret_dir).unwrap();
        let secret = secret_dir.join("secret.txt");
        std::fs::write(&secret, "LANDLOCK_SECRET_225\n").unwrap();
        let script = jail.join("read-secret");
        std::fs::write(
            &script,
            format!("#!/bin/sh\ncat '{}' && echo LEAKED\n", secret.display()),
        )
        .unwrap();
        chmod_exec(&script);

        let leaked = ProcessBackend::new(&script)
            .generate(&dummy_payload("ignored"))
            .expect("unsandboxed script must read sibling secret");
        let leaked_text = leaked["result"].as_str().expect("result string");
        assert!(
            leaked_text.contains("LANDLOCK_SECRET_225") || leaked_text.contains("LEAKED"),
            "control path must prove the leak, got {leaked_text}"
        );

        let err = ProcessBackend::new(&script)
            .with_landlock()
            .generate(&dummy_payload("ignored"))
            .unwrap_err();
        assert!(
            !err.contains("LANDLOCK_SECRET_225"),
            "sandboxed child must not leak secret, got {err}"
        );
        assert!(
            err.contains(NONZERO_EXIT) || err.contains(LANDLOCK_FAILED),
            "Landlock deny or apply fail must fail-closed, got {err}"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn landlock_echo_in_jail_succeeds() {
        let root = tempfile::tempdir().unwrap();
        let jail = root.path().join("jail");
        std::fs::create_dir(&jail).unwrap();
        let script = jail.join("echo-ok");
        std::fs::write(&script, "#!/bin/sh\necho LANDLOCK_OK\n").unwrap();
        chmod_exec(&script);
        let out = ProcessBackend::new(&script)
            .with_landlock()
            .generate(&dummy_payload("ignored"))
            .expect("echo-only jail script must complete under Landlock");
        let text = out["result"].as_str().expect("result string");
        assert!(text.contains("LANDLOCK_OK"), "got {text}");
        assert_eq!(out["backend"], json!(PROCESS_BACKEND_ID));
    }
}
