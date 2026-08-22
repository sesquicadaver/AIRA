//! E4 network profiles doc contract (#106).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn network_profiles_doc_contract() {
    let path = repo_root().join("docs/desktop-network-profiles.md");
    let text = std::fs::read_to_string(&path).expect("read desktop-network-profiles.md");
    for needle in [
        "P0",
        "P6",
        "Mutex P3 | P4",
        "relay_ttl_days",
        "AIRA-RFC-0043",
        "desktop-ux.md",
        "desktop-peer.md",
        "peer-link.md",
        "RFC-0044",
        "RFC-0055",
        "#94",
        "#105",
        "fail-closed",
    ] {
        assert!(text.contains(needle), "doc missing: {needle}");
    }
}
