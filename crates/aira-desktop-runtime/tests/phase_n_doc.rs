//! Phase N contract smoke (#231–#233). Per-atom tests land with #234–#247.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rfc_0123_hits() -> Vec<String> {
    let rfc_dir = repo_root().join("specs/rfc");
    std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0123") || n.contains("rfc-0123"))
        .collect()
}

#[test]
fn phase_n_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-n-plan.md")).unwrap();
    for needle in [
        "Phase N",
        "Global Node Rendezvous",
        "#231",
        "#232",
        "#233",
        "#247",
        "N0 govern",
        "Prime Port",
        "preferred_port",
        "P_AIRA",
        "1491",
        "49157",
        "65521",
        "aira:network:public:v1",
        "EvmRendezvousProvider",
        "Polygon Amoy",
        "DISCOVERED",
        "TRUSTED",
        "AIRA-RFC-0123",
        "confirmed free",
        "GPU marketplace",
        "LLM runtime (Core як inference host)",
        "Calculate 2 + 2",
        "aira-core",
    ] {
        assert!(text.contains(needle), "phase-n-plan missing: {needle}");
    }
    assert!(
        !text.contains("не в QUEUE"),
        "phase-n-plan must not stay 'не в QUEUE' after wiring"
    );
}

#[test]
fn phase_n_queue_233_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(
        text.contains("phase-n-plan.md"),
        "QUEUE missing phase-n-plan"
    );
    for n in 231..=233 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
        assert!(
            !text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must not stay OPEN"
        );
    }
    for n in 234..=247 {
        let open = format!("| {n} | **OPEN**");
        let done = format!("| {n} | **DONE**");
        assert!(text.contains(&open), "QUEUE #{n} must be OPEN after #233");
        assert!(!text.contains(&done), "QUEUE #{n} must not be DONE at #233");
    }
    for needle in [
        "N2 selection",
        "Analyze-268",
        "RFC-0125",
        "preferred_port",
        "перший OPEN `#234`",
        "QUEUE M closed",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_n_rfc_0123_id_free() {
    let hits = rfc_0123_hits();
    assert!(
        hits.is_empty(),
        "RFC-0123 must stay file-free until #247, found {hits:?}"
    );
}

#[test]
fn phase_n_rfc_0124_prime_port_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0124-prime-port.md");
    let text = std::fs::read_to_string(&path).unwrap();
    for needle in [
        "Prime Private Port",
        "P_AIRA",
        "1491",
        "49157",
        "65521",
        "#232",
        "#233",
        "fail closed",
        "Fail-closed",
        "HTTP",
        "STUN",
        "Polygon",
    ] {
        assert!(text.contains(needle), "RFC-0124 missing: {needle}");
    }
}

#[test]
fn phase_n_rfc_0125_preferred_port_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0125-preferred-port.md");
    let text = std::fs::read_to_string(&path).unwrap();
    for needle in [
        "preferred_port",
        "transport_class",
        "aira:port-select:v1",
        "1491",
        "#233",
        "#234",
        "wrap",
        "SHA-256",
        "tcp-peer",
        "udp-discv",
    ] {
        assert!(text.contains(needle), "RFC-0125 missing: {needle}");
    }
}

#[test]
fn phase_n_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-plan.md"));
    assert!(readme.contains("#231"));
    assert!(readme.contains("#247"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-n-plan.md"));
    assert!(docs.contains("#233"));
    assert!(docs.contains("#234"));
}

#[test]
fn phase_m_points_to_phase_n() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-m-plan.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
    assert!(text.contains("#231"));
}

#[test]
fn phase_n_status_row_233() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("Phase N gates"));
    assert!(status.contains("phase_n_doc.rs"));
    assert!(status.contains("| #233 | Deterministic port selection"));
    assert!(status.contains("RFC-0125"));
    assert!(status.contains("phase-n-plan.md"));
}

#[test]
fn phase_n_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        text.contains("RESOLVED") || text.contains("provenance"),
        "NEXT_PROBLEM must stay provenance"
    );
    assert!(
        !text.contains("перший OPEN = `#233`") && !text.contains("first OPEN `#233`"),
        "NEXT_PROBLEM must not keep #233 as first-OPEN after preferred port"
    );
    assert!(
        text.contains("перший OPEN = `#234`") || text.contains("first OPEN `#234`"),
        "NEXT_PROBLEM must point at first OPEN #234"
    );
    assert!(
        text.contains("QUEUE M closed"),
        "NEXT_PROBLEM must keep QUEUE M closed"
    );
    assert!(
        text.contains("phase-n-plan.md") || text.contains("QUEUE.md"),
        "NEXT_PROBLEM must point at QUEUE / Phase N"
    );
}

#[test]
fn phase_n_prime_port_module_contract() {
    assert_eq!(aira_peer::P_AIRA_COUNT, 1491);
    assert_eq!(aira_peer::P_AIRA_FIRST, 49157);
    assert_eq!(aira_peer::P_AIRA_LAST, 65521);
    assert!(aira_peer::is_valid_aira_port(49157));
    assert!(!aira_peer::is_valid_aira_port(9797));
    assert!(!aira_peer::is_valid_aira_port(0));
    assert!(aira_peer::validate_aira_bind("127.0.0.1:49157").is_ok());
    assert!(aira_peer::validate_aira_bind("127.0.0.1:9797").is_err());
}

#[test]
fn phase_n_preferred_port_contract() {
    let id = "aira:identity:phase-n-doc-preferred";
    let a = aira_peer::preferred_port(id, aira_peer::TransportClass::TcpPeer);
    let b = aira_peer::preferred_port(id, aira_peer::TransportClass::TcpPeer);
    assert_eq!(a, b);
    assert!(aira_peer::is_valid_aira_port(a));
    assert_eq!(aira_peer::PORT_SELECT_VERSION, "aira:port-select:v1");
    let err = aira_peer::select_available_port(id, aira_peer::TransportClass::TcpPeer, |_| false);
    assert!(err.is_err());
}
