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
fn ci_reads_rust_toolchain_file() {
    let path = repo_root().join(".github/workflows/ci.yml");
    let text = std::fs::read_to_string(&path).expect("read ci.yml");
    assert!(
        text.contains("toolchain: none"),
        "CI must install from rust-toolchain.toml (toolchain: none)"
    );
}
