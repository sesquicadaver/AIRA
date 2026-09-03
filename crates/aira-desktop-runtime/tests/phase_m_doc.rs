//! Phase M wiring contract (#224) + Landlock FS (#225) + seccomp (#226) + netns (#227) + sandbox-required (#228) + OS vs AIRA-mediated (#229). Per-atom tests land with #230.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rfc_0117_hits() -> Vec<String> {
    let rfc_dir = repo_root().join("specs/rfc");
    std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0117") || n.contains("rfc-0117"))
        .collect()
}

#[test]
fn phase_m_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-m-plan.md")).unwrap();
    for needle in [
        "Phase M",
        "#224",
        "#225",
        "#230",
        "M0 govern",
        "Landlock",
        "seccomp",
        "network namespace",
        "AIRA-RFC-0117",
        "confirmed free",
        "GPU marketplace",
        "LLM runtime (Core як inference host)",
        "Calculate 2 + 2",
        "AIRA-mediated",
    ] {
        assert!(text.contains(needle), "phase-m-plan missing: {needle}");
    }
    assert!(
        !text.contains("не в QUEUE"),
        "phase-m-plan must not stay 'не в QUEUE' after wiring"
    );
}

#[test]
fn phase_m_queue_wiring_224_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(
        text.contains("phase-m-plan.md"),
        "QUEUE missing phase-m-plan"
    );
    assert!(
        text.contains("| 224 | **DONE**"),
        "QUEUE #224 must be DONE after wiring"
    );
    assert!(
        !text.contains("| 224 | **OPEN**"),
        "QUEUE #224 must not stay OPEN after wiring"
    );
    assert!(
        text.contains("| 225 | **DONE**"),
        "QUEUE #225 must be DONE after Landlock FS"
    );
    assert!(
        !text.contains("| 225 | **OPEN**"),
        "QUEUE #225 must not stay OPEN after Landlock FS"
    );
    assert!(
        text.contains("| 226 | **DONE**"),
        "QUEUE #226 must be DONE after seccomp"
    );
    assert!(
        !text.contains("| 226 | **OPEN**"),
        "QUEUE #226 must not stay OPEN after seccomp"
    );
    assert!(
        text.contains("| 227 | **DONE**"),
        "QUEUE #227 must be DONE after netns"
    );
    assert!(
        !text.contains("| 227 | **OPEN**"),
        "QUEUE #227 must not stay OPEN after netns"
    );
    assert!(
        text.contains("| 228 | **DONE**"),
        "QUEUE #228 must be DONE after sandbox-required"
    );
    assert!(
        !text.contains("| 228 | **OPEN**"),
        "QUEUE #228 must not stay OPEN after sandbox-required"
    );
    assert!(
        text.contains("| 229 | **DONE**"),
        "QUEUE #229 must be DONE after OS vs AIRA-mediated"
    );
    assert!(
        !text.contains("| 229 | **OPEN**"),
        "QUEUE #229 must not stay OPEN after OS vs AIRA-mediated"
    );
    assert!(text.contains("| 230 | **OPEN**"), "QUEUE #230 must be OPEN after #229");
    assert!(
        !text.contains("| 230 | **DONE**"),
        "QUEUE #230 must not be DONE at #229"
    );
    for needle in [
        "M0 govern",
        "Analyze-259",
        "RFC-0117",
        "Landlock",
        "QUEUE L closed",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_m_rfc_0117_id_free() {
    let hits = rfc_0117_hits();
    assert!(
        hits.is_empty(),
        "RFC-0117 must stay file-free until #230, found {hits:?}"
    );
}

#[test]
fn phase_m_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-m-plan.md"));
    assert!(readme.contains("#224"));
    assert!(readme.contains("#230"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-m-plan.md"));
    assert!(docs.contains("#224"));
    assert!(docs.contains("#230"));
}

#[test]
fn phase_l_points_to_phase_m() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-l-plan.md")).unwrap();
    assert!(text.contains("phase-m-plan.md"));
    assert!(text.contains("#224"));
}

#[test]
fn phase_m_status_row_224() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("Phase M gates"));
    assert!(status.contains("phase_m_doc.rs"));
    assert!(status.contains("| #224 | Phase M wiring + contract"));
    assert!(status.contains("phase-m-plan.md"));
}

#[test]
fn phase_m_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        text.contains("RESOLVED") || text.contains("provenance"),
        "NEXT_PROBLEM must stay provenance"
    );
    assert!(
        !text.contains("перший OPEN = `#229`"),
        "NEXT_PROBLEM must not keep #229 as first-OPEN after OS vs AIRA-mediated"
    );
    assert!(
        text.contains("перший OPEN = `#230`") || text.contains("first OPEN `#230`"),
        "NEXT_PROBLEM must point at first OPEN #230"
    );
    assert!(
        text.contains("QUEUE L closed"),
        "NEXT_PROBLEM must keep QUEUE L closed"
    );
    assert!(
        text.contains("phase-m-plan.md") || text.contains("QUEUE.md"),
        "NEXT_PROBLEM must point at QUEUE / Phase M"
    );
}

#[test]
fn phase_m_landlock_225() {
    let process =
        std::fs::read_to_string(repo_root().join("csu/execution-llm/src/process.rs")).unwrap();
    let landlock =
        std::fs::read_to_string(repo_root().join("csu/execution-llm/src/landlock.rs")).unwrap();
    let rfc = std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0118-landlock-fs.md"))
        .unwrap();
    for needle in [
        "LANDLOCK_FAILED",
        "with_landlock",
        "ENV_LLM_LANDLOCK",
        "fn landlock_denies_read_outside_allowlist",
        "pre_exec",
    ] {
        assert!(
            process.contains(needle),
            "process.rs missing Landlock needle: {needle}"
        );
    }
    assert!(
        landlock.contains("PR_SET_NO_NEW_PRIVS"),
        "landlock.rs must set no-new-privs before restrict"
    );
    assert!(
        landlock.contains("LANDLOCK_FAILED"),
        "landlock.rs must fail-closed with LANDLOCK_FAILED"
    );
    for needle in [
        "AIRA-RFC-0118",
        "Landlock",
        "fail-closed",
        "seccomp",
        "GPU marketplace",
        "RFC-0117",
    ] {
        assert!(rfc.contains(needle), "RFC-0118 missing: {needle}");
    }
    assert!(
        !rfc.contains("AIRA-RFC-0117 —"),
        "RFC-0118 must not be filed as RFC-0117"
    );
}

#[test]
fn phase_m_seccomp_226() {
    let process =
        std::fs::read_to_string(repo_root().join("csu/execution-llm/src/process.rs")).unwrap();
    let seccomp =
        std::fs::read_to_string(repo_root().join("csu/execution-llm/src/seccomp.rs")).unwrap();
    let rfc =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0119-seccomp.md")).unwrap();
    for needle in [
        "SECCOMP_VIOLATION",
        "with_seccomp",
        "ENV_LLM_SECCOMP",
        "fn seccomp_forbidden_syscall_is_fail_closed",
        "pre_exec",
    ] {
        assert!(
            process.contains(needle),
            "process.rs missing seccomp needle: {needle}"
        );
    }
    assert!(
        seccomp.contains("SECCOMP_SET_MODE_FILTER"),
        "seccomp.rs must install SECCOMP_SET_MODE_FILTER"
    );
    assert!(
        seccomp.contains("PR_SET_NO_NEW_PRIVS"),
        "seccomp.rs must set no-new-privs before filter"
    );
    for needle in [
        "AIRA-RFC-0119",
        "seccomp",
        "fail-closed",
        "netns",
        "GPU marketplace",
        "RFC-0117",
    ] {
        assert!(rfc.contains(needle), "RFC-0119 missing: {needle}");
    }
    assert!(
        !rfc.contains("AIRA-RFC-0117 —"),
        "RFC-0119 must not be filed as RFC-0117"
    );
}

#[test]
fn phase_m_netns_227() {
    let process =
        std::fs::read_to_string(repo_root().join("csu/execution-llm/src/process.rs")).unwrap();
    let netns =
        std::fs::read_to_string(repo_root().join("csu/execution-llm/src/netns.rs")).unwrap();
    let rfc =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0120-netns.md")).unwrap();
    for needle in [
        "with_netns",
        "ENV_LLM_NETNS",
        "NETNS_BLOCKS_LOOPBACK",
        "fn netns_isolates_host_loopback",
        "fn ollama_with_netns_is_fail_closed",
        "pre_exec",
    ] {
        assert!(
            process.contains(needle),
            "process.rs missing netns needle: {needle}"
        );
    }
    assert!(
        netns.contains("CLONE_NEWNET"),
        "netns.rs must unshare CLONE_NEWNET"
    );
    assert!(
        netns.contains("NETNS_FAILED"),
        "netns.rs must fail-closed with NETNS_FAILED"
    );
    for needle in [
        "AIRA-RFC-0120",
        "netns",
        "fail-closed",
        "CLONE_NEWNET",
        "GPU marketplace",
        "RFC-0117",
    ] {
        assert!(rfc.contains(needle), "RFC-0120 missing: {needle}");
    }
    assert!(
        !rfc.contains("AIRA-RFC-0117 —"),
        "RFC-0120 must not be filed as RFC-0117"
    );
}

#[test]
fn phase_m_sandbox_required_228() {
    let process =
        std::fs::read_to_string(repo_root().join("csu/execution-llm/src/process.rs")).unwrap();
    let sandbox =
        std::fs::read_to_string(repo_root().join("csu/execution-llm/src/sandbox.rs")).unwrap();
    let rfc =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0121-sandbox-required.md"))
            .unwrap();
    for needle in [
        "with_sandbox_required",
        "ENV_LLM_SANDBOX_REQUIRED",
        "SANDBOX_REQUIRED",
        "fn sandbox_required_missing_kernel_is_fail_closed",
        "with_unavailable_kernel_for_test",
    ] {
        assert!(
            process.contains(needle),
            "process.rs missing sandbox-required needle: {needle}"
        );
    }
    assert!(
        sandbox.contains("SANDBOX_REQUIRED"),
        "sandbox.rs must fail-closed with SANDBOX_REQUIRED"
    );
    assert!(
        sandbox.contains("fn enforce"),
        "sandbox.rs must expose enforce"
    );
    for needle in [
        "AIRA-RFC-0121",
        "sandbox required",
        "fail-closed",
        "SANDBOX_REQUIRED",
        "GPU marketplace",
        "RFC-0117",
    ] {
        assert!(rfc.contains(needle), "RFC-0121 missing: {needle}");
    }
    assert!(
        !rfc.contains("AIRA-RFC-0117 —"),
        "RFC-0121 must not be filed as RFC-0117"
    );
}

#[test]
fn phase_m_os_vs_aira_mediated_229() {
    let schema_text = std::fs::read_to_string(
        repo_root().join("schemas/execution/generate-local.schema.json"),
    )
    .unwrap();
    for needle in [
        "AIRA-mediated",
        "RFC-0116",
        "RFC-0122",
        "not encoded in this payload",
        "ProcessBackend",
        "Not an OS network-off sandbox",
    ] {
        assert!(
            schema_text.contains(needle),
            "generate-local schema missing: {needle}"
        );
    }
    let local = std::fs::read_to_string(repo_root().join("docs/local-node.md")).unwrap();
    for needle in [
        "OS isolation vs AIRA-mediated none",
        "AIRA-RFC-0122",
        "constraints.network=none",
        "Operator opt-in",
    ] {
        assert!(local.contains(needle), "local-node.md missing: {needle}");
    }
    let csu = std::fs::read_to_string(repo_root().join("docs/csu-development.md")).unwrap();
    assert!(
        csu.contains("RFC-0122"),
        "csu-development must cross-ref RFC-0122"
    );
    let rfc =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0122-os-vs-aira-mediated.md"))
            .unwrap();
    for needle in [
        "AIRA-RFC-0122",
        "OS isolation",
        "AIRA-mediated",
        "RFC-0116",
        "Landlock",
        "seccomp",
        "netns",
        "GPU marketplace",
        "RFC-0117",
    ] {
        assert!(rfc.contains(needle), "RFC-0122 missing: {needle}");
    }
    assert!(
        !rfc.contains("AIRA-RFC-0117 —"),
        "RFC-0122 must not be filed as RFC-0117"
    );
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #229 | OS vs AIRA-mediated docs"));
    assert!(status.contains("RFC-0122"));
}
