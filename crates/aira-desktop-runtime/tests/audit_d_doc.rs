//! Audit block D documentation contracts (`#383`…).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `#383`: installed local file is visible; not Work-ready generate via Ollama.
#[test]
fn audit_d_383_installed_file_visible_not_ollama_work_ready() {
    let root = repo_root();
    let rfc = std::fs::read_to_string(
        root.join("specs/rfc/AIRA-RFC-0240-two-model-installed-acceptance.md"),
    )
    .expect("RFC-0240");
    assert!(rfc.contains("#383"), "RFC-0240 §8b must cite #383 honesty");
    assert!(
        rfc.contains("not** Work-ready for generate")
            || rfc.contains("**not** Work-ready for generate"),
        "RFC-0240 must say file is not Work-ready for generate on Ollama"
    );
    assert!(
        !rfc.contains("Prepare → Work ready"),
        "RFC-0240 must not claim Prepare → Work ready for file path"
    );
    assert!(
        rfc.contains("file-executor") || rfc.contains("No new file-executor"),
        "RFC-0240 must forbid adding a file-executor for this gate"
    );

    let installed =
        std::fs::read_to_string(root.join("docs/installed-product-llm-acceptance.md")).unwrap();
    assert!(
        installed.contains("#383") || installed.contains("not** Work ready"),
        "installed-product gate must document #383 / not Work ready"
    );
    assert!(
        !installed.contains("Prepare → Work ready"),
        "installed-product must not claim Prepare → Work ready for files"
    );

    let local = std::fs::read_to_string(root.join("docs/local-node.md")).unwrap();
    assert!(
        local.contains("#383") && local.contains("not** Work ready for generate"),
        "local-node Desktop row must state file ≠ Work ready on Ollama"
    );

    for rel in [
        "docs/help/en/model.select.md",
        "docs/help/uk/model.select.md",
    ] {
        let text = std::fs::read_to_string(root.join(rel)).unwrap();
        assert!(
            text.contains("not** Work ready")
                || text.contains("**не** Work ready")
                || text.contains("не** Work ready"),
            "{rel} must say prepared file is not Work ready on Ollama"
        );
    }

    // Runtime still fail-closes file weights on ollama (no new executor).
    let proc = std::fs::read_to_string(root.join("csu/execution-llm/src/process.rs")).unwrap();
    assert!(proc.contains("file_weight_binding_on_ollama_backend_is_fail_closed"));
    let wr =
        std::fs::read_to_string(root.join("crates/aira-desktop-runtime/src/work_readiness.rs"))
            .unwrap();
    assert!(wr.contains("cannot run it"));
}

/// `#384`: C1 Executed, VRA fixture, and OP-001 are three separate Living Spec statuses.
#[test]
fn audit_d_384_c1_executed_vra_op001_are_three_statuses() {
    let root = repo_root();
    let conf = std::fs::read_to_string(root.join("docs/conformance.md")).unwrap();
    assert!(
        conf.contains("## C1 three statuses (`#384`)"),
        "docs/conformance.md must define the three-status table"
    );
    for needle in [
        "**C1 Executed**",
        "**VRA fixture**",
        "**OP-001**",
        "c1.pipeline.process_executor_executed",
        "c1.result.verified_completeness",
        "calculator returned to GUI",
    ] {
        assert!(
            conf.contains(needle),
            "docs/conformance.md missing: {needle}"
        );
    }

    let status = std::fs::read_to_string(root.join("docs/implementation-status.md")).unwrap();
    assert!(
        status.contains("**C1 Executed** (process smoke)")
            && status.contains("**VRA schema fixture**")
            && status.contains("**OP-001** (legacy math)"),
        "implementation-status Living Spec must list three separate rows"
    );
    assert!(
        status.contains("Do not collapse Executed / VRA fixture / OP-001"),
        "matrix must forbid collapsing the three statuses"
    );

    let specs = std::fs::read_to_string(root.join("specs/conformance.md")).unwrap();
    assert!(
        specs.contains("#384") && specs.contains("legacy non-normative"),
        "specs OP-001 must carry #384 legacy banner"
    );

    let gov = std::fs::read_to_string(root.join("docs/ci-governance.md")).unwrap();
    assert!(
        gov.contains("#384") && gov.contains("calculator"),
        "ci-governance must separate smoke/VRA from OP-001 and GUI calculator"
    );

    let i18n = std::fs::read_to_string(root.join("crates/aira-desktop/src/app/i18n.rs")).unwrap();
    assert!(
        i18n.contains("assert!(!Labels::get(UiLang::En).work_tech_details.contains(\"OP-001\"))"),
        "desktop i18n must keep OP-001 out of Work tech details"
    );
}

/// `#385`: settlement receipt `$id` body in schema-pack matches the canonical schema file.
#[test]
fn audit_d_385_settlement_receipt_id_matches_schema_pack() {
    use serde_json::Value;
    let root = repo_root();
    let schema_path = root.join("schemas/settlement/receipt.schema.json");
    let file: Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_path).unwrap()).unwrap();
    let id = file
        .get("$id")
        .and_then(|v| v.as_str())
        .expect("schema $id");
    assert_eq!(id, "aira:schema:settlement:receipt:0.1");
    assert!(
        file.pointer("/properties/privacy_class").is_some(),
        "canonical schema must require privacy_class (PRIV-001)"
    );
    assert!(
        file.get("required")
            .and_then(|v| v.as_array())
            .is_some_and(|a| a.iter().any(|x| x.as_str() == Some("privacy_class"))),
        "privacy_class must be in required[]"
    );

    let pack = std::fs::read_to_string(root.join("specs/schema-pack.md")).unwrap();
    assert!(
        pack.contains("One `$id` ↔ one schema body (`#385`)"),
        "schema-pack must document one-id/one-body rule for settlement receipt"
    );
    // Extract the fenced JSON whose $id is the settlement receipt.
    let marker = "\"$id\": \"aira:schema:settlement:receipt:0.1\"";
    let start = pack
        .find(marker)
        .expect("schema-pack must embed settlement receipt $id");
    let before = pack[..start].rfind('{').expect("json object start");
    let after = pack[start..]
        .find("\n```")
        .expect("closing fence after settlement receipt JSON");
    let block = &pack[before..start + after];
    let pack_obj: Value = serde_json::from_str(block).unwrap_or_else(|e| {
        panic!(
            "schema-pack settlement JSON parse: {e}; block={}",
            &block[..block.len().min(120)]
        );
    });
    assert_eq!(
        pack_obj, file,
        "schema-pack settlement receipt JSON must equal schemas/settlement/receipt.schema.json"
    );

    // Fixtures and registry stay on the same $id (not a second schema identity).
    let manifest = std::fs::read_to_string(root.join("fixtures/manifest.json")).unwrap();
    assert!(manifest.contains("\"schema\": \"aira:schema:settlement:receipt:0.1\""));
    let status = std::fs::read_to_string(root.join("docs/implementation-status.md")).unwrap();
    assert!(
        status.contains("`aira:schema:settlement:receipt:0.1`") && status.contains("#385"),
        "implementation-status must cite the single $id and #385 pack sync"
    );
}

/// `#386`: specs/README is authority map (baseline + amendments + priority), not a second Manifesto.
#[test]
fn audit_d_386_specs_readme_baseline_amendments_priority() {
    let root = repo_root();
    let readme = std::fs::read_to_string(root.join("specs/README.md")).unwrap();
    assert!(readme.contains("#386"), "specs/README.md must cite #386");
    assert!(
        readme.contains("## Immutable baseline"),
        "specs/README must name immutable baseline"
    );
    assert!(
        readme.contains("Manifesto etc/"),
        "baseline must point at Manifesto etc/"
    );
    assert!(
        readme.contains("## Amendments (RFCs)"),
        "specs/README must name amendments (RFCs)"
    );
    assert!(
        readme.contains("specs/rfc/") && readme.contains("rfc-process.md"),
        "amendments must point at specs/rfc/ and rfc-process"
    );
    assert!(
        readme.contains("## Priority (conflict resolution)"),
        "specs/README must define conflict priority"
    );
    for needle in [
        "**`Manifesto etc/`**",
        "**`schemas/**/*.json`**",
        "**Accepted RFCs**",
        "**Working copies**",
        "second Manifesto",
        "Do not rewrite `Manifesto etc/`",
    ] {
        assert!(
            readme.contains(needle),
            "priority / non-second-canon missing: {needle}"
        );
    }
    assert!(
        readme.contains("## RFC metadata (`class` / `status`)")
            && readme.contains("`class`")
            && readme.contains("`status`")
            && readme.contains("draft")
            && readme.contains("accepted"),
        "RFC metadata class/status must be documented"
    );

    let process = std::fs::read_to_string(root.join("specs/rfc-process.md")).unwrap();
    assert!(
        process.contains("# 15. RFC Metadata Template"),
        "rfc-process §15 metadata template must remain the class/status authority"
    );
    assert!(
        process.contains("class: enum") && process.contains("status: enum"),
        "rfc-process §15 must declare class and status enums"
    );
}

/// `#387`: entry-points share one current tip; stale Phase G/N tips and calculator GUI claims are gone.
#[test]
fn audit_d_387_entry_points_one_first_open() {
    let root = repo_root();
    // Live tip after Phase Z `#390` wiring (shared-tip rule from `#387` still applies).
    let tip = "перший OPEN `#391`";
    let paths = [
        "docs/demo.md",
        "docs/crypto.md",
        "docs/implementation-status.md",
        "NEXT_PROBLEM.md",
    ];
    for rel in paths {
        let text = std::fs::read_to_string(root.join(rel)).unwrap();
        assert!(
            text.contains("#387") && text.contains(tip),
            "{rel} must cite #387 and the shared tip {tip}"
        );
        assert!(
            !text.contains("Phase G OPEN"),
            "{rel} must not claim Phase G OPEN"
        );
        assert!(
            !text.contains("перший OPEN `#232`"),
            "{rel} must not keep stale Phase N tip #232"
        );
    }

    let demo = std::fs::read_to_string(root.join("docs/demo.md")).unwrap();
    assert!(
        demo.contains("legacy non-normative") || demo.contains("OP-001"),
        "demo must mark Calculate 2+2 / OP-001 as legacy"
    );
    assert!(
        demo.contains("does **not** promise a calculator")
            || demo.contains("does not promise a calculator"),
        "demo must not claim Desktop calculator 4.0 VERIFIED"
    );
    assert!(
        !demo.contains("shows **4.0** + **VERIFIED**"),
        "demo must not promise GUI 4.0 VERIFIED"
    );

    let next = std::fs::read_to_string(root.join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        next.contains("QUEUE W closed") && next.contains("QUEUE L closed"),
        "NEXT_PROBLEM must retain closed-phase provenance markers"
    );

    let status = std::fs::read_to_string(root.join("docs/implementation-status.md")).unwrap();
    assert!(
        status.contains("historical tip was `#344`") || status.contains("not the tip"),
        "implementation-status must not present #344 as the live tip"
    );
    assert!(
        !status.contains("; first OPEN `#344`."),
        "implementation-status must not end a live tip sentence with first OPEN #344"
    );
}

/// `#388`: installed acceptance evidence pinned to a concrete SHA (not Analyze-396 as HEAD).
#[test]
fn audit_d_388_installed_acceptance_sha_evidence() {
    let root = repo_root();
    let run = std::fs::read_to_string(root.join("analysis/Analyze-397/run.json")).unwrap();
    let doc: serde_json::Value = serde_json::from_str(&run).unwrap();
    let sha = doc
        .get("git_sha")
        .and_then(|v| v.as_str())
        .expect("Analyze-397 run.json must carry git_sha");
    assert!(
        sha.len() >= 7 && sha.chars().all(|c| c.is_ascii_hexdigit()),
        "git_sha must look like a hex commit: {sha}"
    );
    assert_eq!(doc.get("queue_atom").and_then(|v| v.as_str()), Some("#388"));
    assert_eq!(
        doc.get("not_analyze_396_head_proof")
            .and_then(|v| v.as_bool()),
        Some(true),
        "must refuse Analyze-396 as HEAD proof"
    );
    let digests = doc.get("ollama_digests").expect("ollama_digests");
    assert!(
        digests
            .get("a")
            .and_then(|v| v.as_str())
            .is_some_and(|s| !s.is_empty())
            && digests
                .get("b")
                .and_then(|v| v.as_str())
                .is_some_and(|s| !s.is_empty()),
        "both ollama digests required"
    );
    let executed = doc.get("executed").expect("executed");
    for needle in [
        "required_a_then_b_without_restart",
        "file_weight_fail_closed",
        "timeout_fail_closed",
        "m6_fixture_cli_http",
        "m6_fixture_gui",
    ] {
        assert_eq!(
            executed.get(needle).and_then(|v| v.as_bool()),
            Some(true),
            "executed.{needle} must be true"
        );
    }

    let brief = std::fs::read_to_string(root.join("analysis/Analyze-397/BRIEF.md")).unwrap();
    assert!(
        brief.contains("#388") && brief.contains(sha) && brief.contains("Analyze-396"),
        "BRIEF must cite #388, the SHA, and not-Analyze-396-as-HEAD"
    );

    let gate =
        std::fs::read_to_string(root.join("docs/installed-product-llm-acceptance.md")).unwrap();
    assert!(
        gate.contains("#388")
            && gate.contains("Analyze-397")
            && gate.contains(sha)
            && gate.contains("not Analyze-396 as HEAD proof"),
        "installed-product gate must point at Analyze-397 SHA evidence"
    );

    let script =
        std::fs::read_to_string(root.join("scripts/installed-product-llm-acceptance.sh")).unwrap();
    assert!(
        script.contains("AIRA_GIT_SHA")
            && script.contains("Analyze-397")
            && script.contains("phase_x_m6_acceptance"),
        "acceptance script must pin SHA and run M6 staff path"
    );

    let proc = std::fs::read_to_string(root.join("csu/execution-llm/src/process.rs")).unwrap();
    assert!(
        proc.contains("not_analyze_396_head_proof") && proc.contains("AIRA_GIT_SHA"),
        "process evidence writer must emit SHA / not-396 markers"
    );
}

/// `#389`: Analyze-396 (and similar old smoke evidence) carries explicit date/SHA as historical snapshot.
#[test]
fn audit_d_389_analyze_396_historical_snapshot_headers() {
    let root = repo_root();
    let sha = "7bb6716914b069b672a364b9a3990a7a9aed023a";
    let recorded = "2026-09-19T07:02:09Z";

    let run = std::fs::read_to_string(root.join("analysis/Analyze-396/run.json")).unwrap();
    let doc: serde_json::Value = serde_json::from_str(&run).unwrap();
    assert_eq!(
        doc.get("status").and_then(|v| v.as_str()),
        Some("historical_snapshot")
    );
    assert_eq!(doc.get("git_sha").and_then(|v| v.as_str()), Some(sha));
    assert_eq!(
        doc.get("recorded_at").and_then(|v| v.as_str()),
        Some(recorded)
    );
    assert_eq!(
        doc.get("not_head_proof").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        doc.get("queue_atom_historicize").and_then(|v| v.as_str()),
        Some("#389")
    );
    assert!(
        doc.get("superseded_by")
            .and_then(|v| v.as_str())
            .is_some_and(|s| s.contains("Analyze-397")),
        "historical 396 must point at Analyze-397 as living gate"
    );

    for rel in [
        "analysis/Analyze-396/BRIEF.md",
        "analysis/Analyze-396/EVIDENCE.md",
    ] {
        let text = std::fs::read_to_string(root.join(rel)).unwrap();
        assert!(
            text.contains("#389")
                && text.contains("HISTORICAL SNAPSHOT")
                && text.contains(sha)
                && text.contains(recorded),
            "{rel} must carry #389 historic header with date/SHA"
        );
        assert!(
            text.contains("not proof of current HEAD") || text.contains("not HEAD proof"),
            "{rel} must deny HEAD proof"
        );
    }

    let gate =
        std::fs::read_to_string(root.join("docs/installed-product-llm-acceptance.md")).unwrap();
    assert!(
        gate.contains("#389")
            && gate.contains("historical snapshot")
            && gate.contains("7bb6716")
            && gate.contains("Analyze-397"),
        "installed-product gate must label Analyze-396 historical under #389"
    );

    let queue = std::fs::read_to_string(root.join("QUEUE.md")).unwrap();
    assert!(
        queue.contains("| 389 | **DONE**"),
        "QUEUE #389 must be DONE after historicize"
    );
    assert!(
        !queue.contains("| 389 | OPEN"),
        "QUEUE must not keep #389 OPEN"
    );
    assert!(
        queue.contains("| 390 | **DONE**")
            && (queue.contains("**Перший OPEN:** `#391`") || queue.contains("first OPEN `#391`")),
        "QUEUE tip must advance to Phase Z first OPEN #391 after audit A closed"
    );
}
