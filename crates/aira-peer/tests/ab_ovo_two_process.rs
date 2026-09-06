//! QUEUE #249 — two external OS processes ab ovo harness (independent roots).

use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use tempfile::tempdir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_aira-ab-ovo-harness")
}

fn wait_exit(mut child: std::process::Child, timeout: Duration, label: &str) -> String {
    let start = Instant::now();
    loop {
        match child.try_wait().expect("try_wait") {
            Some(status) => {
                let mut stdout = String::new();
                let mut stderr = String::new();
                if let Some(mut s) = child.stdout.take() {
                    let mut buf = Vec::new();
                    s.read_to_end(&mut buf).ok();
                    stdout = String::from_utf8_lossy(&buf).into_owned();
                }
                if let Some(mut e) = child.stderr.take() {
                    let mut buf = Vec::new();
                    e.read_to_end(&mut buf).ok();
                    stderr = String::from_utf8_lossy(&buf).into_owned();
                }
                assert!(
                    status.success(),
                    "{label} failed status={status:?}\nstdout={stdout}\nstderr={stderr}"
                );
                return stdout;
            }
            None => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("{label} exceeded {timeout:?}");
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

#[test]
fn two_process_ab_ovo_publish_discover_trust_dial() {
    let base = tempdir().unwrap();
    let root_a = base.path().join("a");
    let root_b = base.path().join("b");
    let ledger = base.path().join("ledger");
    let ready = base.path().join("ready.txt");
    let peer_trust = base.path().join("peer_trust.txt");
    std::fs::create_dir_all(&root_a).unwrap();
    std::fs::create_dir_all(&root_b).unwrap();
    std::fs::create_dir_all(&ledger).unwrap();

    // Prove independent roots / empty book B before spawn.
    assert!(!root_a.join("peers").join("address_book.json").is_file());
    assert!(!root_b.join("peers").join("address_book.json").is_file());

    let publisher = Command::new(bin())
        .args([
            "publisher",
            "--root",
            root_a.to_str().unwrap(),
            "--ledger",
            ledger.to_str().unwrap(),
            "--ready",
            ready.to_str().unwrap(),
            "--peer-trust",
            peer_trust.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn publisher");

    // Discoverer starts after a brief pause so publisher can bind; ready-file gates the rest.
    std::thread::sleep(Duration::from_millis(100));

    let discoverer = Command::new(bin())
        .args([
            "discoverer",
            "--root",
            root_b.to_str().unwrap(),
            "--ledger",
            ledger.to_str().unwrap(),
            "--ready",
            ready.to_str().unwrap(),
            "--peer-trust",
            peer_trust.to_str().unwrap(),
            "--as-of",
            "2026-09-06T12:00:00Z",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn discoverer");

    let out_b = wait_exit(discoverer, Duration::from_secs(45), "discoverer");
    let out_a = wait_exit(publisher, Duration::from_secs(45), "publisher");

    assert!(
        out_a.contains("ab-ovo-harness: accepted"),
        "publisher stdout={out_a}"
    );
    assert!(
        out_b.contains("ab-ovo-harness: dialed"),
        "discoverer stdout={out_b}"
    );

    // Shared ledger persisted by publisher process; discoverer never wrote AddressBook for itself as peer.
    assert!(ledger
        .join("peers")
        .join("rendezvous_ledger.json")
        .is_file());
    let book_b = std::fs::read_to_string(root_b.join("peers").join("address_book.json")).unwrap();
    assert!(book_b.contains("aira:identity:harness-a"));
    // Distinct PIDs already implied by two Command::spawn; write marker files for evidence.
    let mut evidence = std::fs::File::create(base.path().join("harness_ok.txt")).unwrap();
    writeln!(evidence, "two-process ab ovo ok").unwrap();
}
