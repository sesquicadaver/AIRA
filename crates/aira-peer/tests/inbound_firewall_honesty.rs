//! QUEUE #252 — inbound block honesty: firewall DROP ≠ non-listening placeholder.

use aira_peer::{
    docker_daemon_usable, inbound_firewall_smoke_script, repo_root_from_peer_crate,
    run_inbound_firewall_smoke, INBOUND_BLOCK_KIND_FIREWALL_DROP,
    INBOUND_BLOCK_KIND_NON_LISTENING_PLACEHOLDER, INBOUND_FIREWALL_SMOKE_SCRIPT,
};

#[test]
fn inbound_block_kinds_are_honestly_named() {
    assert_ne!(
        INBOUND_BLOCK_KIND_FIREWALL_DROP,
        INBOUND_BLOCK_KIND_NON_LISTENING_PLACEHOLDER
    );
    assert!(INBOUND_BLOCK_KIND_NON_LISTENING_PLACEHOLDER.contains("non-listening"));
    assert!(!INBOUND_BLOCK_KIND_NON_LISTENING_PLACEHOLDER.contains("firewall"));
    assert!(INBOUND_BLOCK_KIND_FIREWALL_DROP.contains("firewall"));
}

#[test]
fn inbound_firewall_docker_or_documented_path() {
    let root = repo_root_from_peer_crate();
    let script = inbound_firewall_smoke_script(&root);
    assert!(
        script.is_file(),
        "documented path missing: {} ({INBOUND_FIREWALL_SMOKE_SCRIPT})",
        script.display()
    );

    if docker_daemon_usable() {
        let out = run_inbound_firewall_smoke(&root).expect("docker firewall smoke");
        assert!(out.contains("CONTROL_CONNECT_OK"));
        assert!(out.contains("FIREWALL_BLOCK_OK"));
        assert!(out.contains("STILL_LISTENING_OK"));
        assert!(out.contains(INBOUND_BLOCK_KIND_FIREWALL_DROP));
    } else {
        // Documented path (CI without Docker/NET_ADMIN): script + kinds still present.
        let text = std::fs::read_to_string(&script).unwrap();
        assert!(
            text.contains("iptables") && text.contains("NET_ADMIN"),
            "documented smoke must describe iptables/NET_ADMIN path"
        );
        assert!(text.contains(INBOUND_BLOCK_KIND_FIREWALL_DROP));
        eprintln!(
            "docker unavailable — took documented path {}",
            script.display()
        );
    }
}
