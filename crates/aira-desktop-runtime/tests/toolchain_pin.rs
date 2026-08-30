//! Toolchain pin contract (#107).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn rust_toolchain_pinned_version() {
    let path = repo_root().join("rust-toolchain.toml");
    let text = std::fs::read_to_string(&path).expect("read rust-toolchain.toml");
    assert!(
        text.contains("1.94.0"),
        "rust-toolchain.toml must pin explicit stable version"
    );
    assert!(
        !text.contains("channel = \"stable\""),
        "use explicit version channel, not floating stable"
    );
    assert!(text.contains("rustfmt"), "rustfmt component required");
    assert!(text.contains("clippy"), "clippy component required");
}

#[test]
fn ci_toolchain_matches_pin() {
    let toolchain = repo_root().join("rust-toolchain.toml");
    let toolchain_text = std::fs::read_to_string(&toolchain).expect("read rust-toolchain.toml");
    let ci_path = repo_root().join(".github/workflows/ci.yml");
    let ci_text = std::fs::read_to_string(&ci_path).expect("read ci.yml");
    let pin = "1.94.0";
    assert!(
        toolchain_text.contains(pin),
        "rust-toolchain.toml must pin {pin}"
    );
    assert!(
        ci_text.contains(pin),
        "ci.yml must reference pinned toolchain {pin}"
    );
}

#[test]
fn workspace_rust_version_matches_ci_pin() {
    let cargo = std::fs::read_to_string(repo_root().join("Cargo.toml")).expect("Cargo.toml");
    assert!(
        cargo.contains("rust-version = \"1.94\""),
        "workspace rust-version must match CI 1.94.0 (QUEUE #197)"
    );
    assert!(
        !cargo.contains("rust-version = \"1.75\""),
        "stale MSRV 1.75 must not remain after #197"
    );
}

#[test]
fn ci_actions_are_sha_pinned() {
    let ci = std::fs::read_to_string(repo_root().join(".github/workflows/ci.yml")).expect("ci.yml");
    let mut found = 0usize;
    for (i, line) in ci.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("- uses:") {
            found += 1;
            let spec = rest.split('#').next().unwrap().trim();
            let hash = spec.rsplit('@').next().unwrap_or("");
            assert_eq!(
                hash.len(),
                40,
                "ci.yml line {} action must be SHA-pinned (40 hex): {spec}",
                i + 1
            );
            assert!(
                hash.chars().all(|c| c.is_ascii_hexdigit()),
                "ci.yml line {} action SHA must be hex: {spec}",
                i + 1
            );
        }
        if let Some(rest) = trimmed.strip_prefix("uses:") {
            found += 1;
            let spec = rest.split('#').next().unwrap().trim();
            let hash = spec.rsplit('@').next().unwrap_or("");
            assert_eq!(
                hash.len(),
                40,
                "ci.yml line {} action must be SHA-pinned (40 hex): {spec}",
                i + 1
            );
            assert!(
                hash.chars().all(|c| c.is_ascii_hexdigit()),
                "ci.yml line {} action SHA must be hex: {spec}",
                i + 1
            );
        }
    }
    assert!(found >= 8, "expected SHA-pinned uses: lines, found {found}");
}
