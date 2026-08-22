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
