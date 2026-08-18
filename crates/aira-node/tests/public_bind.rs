//! Binary fail-closed checks for Analyze-69 public HTTP bind opt-in.

use std::io::Read;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use tempfile::tempdir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_aira-node")
}

fn output_deadline(cmd: &mut Command, timeout: Duration) -> Output {
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn aira-node");
    let start = Instant::now();
    loop {
        match child.try_wait().expect("try_wait") {
            Some(status) => {
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                if let Some(mut s) = child.stdout.take() {
                    s.read_to_end(&mut stdout).expect("stdout");
                }
                if let Some(mut e) = child.stderr.take() {
                    e.read_to_end(&mut stderr).expect("stderr");
                }
                return Output {
                    status,
                    stdout,
                    stderr,
                };
            }
            None => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("aira-node exceeded {timeout:?} (possible accidental bind)");
                }
                std::thread::sleep(Duration::from_millis(25));
            }
        }
    }
}

#[test]
fn http_listen_public_without_flag_exits() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::new(bin());
    cmd.args([
        "--root",
        dir.path().to_str().unwrap(),
        "--init",
        "--http",
        "--listen",
        "0.0.0.0:18787",
    ]);
    let out = output_deadline(&mut cmd, Duration::from_secs(8));
    assert!(
        !out.status.success(),
        "stdout={}",
        String::from_utf8_lossy(&out.stdout)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("--allow-public-bind"), "{err}");
    assert!(err.contains("fail-closed"), "{err}");
}

#[test]
fn allow_public_bind_without_http_exits() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::new(bin());
    cmd.args([
        "--root",
        dir.path().to_str().unwrap(),
        "--init",
        "--allow-public-bind",
    ]);
    let out = output_deadline(&mut cmd, Duration::from_secs(8));
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("require --http"), "{err}");
}
