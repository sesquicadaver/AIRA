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
    assert!(
        rfc.contains("#383"),
        "RFC-0240 §8b must cite #383 honesty"
    );
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
    let wr = std::fs::read_to_string(
        root.join("crates/aira-desktop-runtime/src/work_readiness.rs"),
    )
    .unwrap();
    assert!(wr.contains("cannot run it"));
}
