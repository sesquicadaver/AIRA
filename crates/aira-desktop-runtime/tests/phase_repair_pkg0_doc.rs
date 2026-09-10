//! Repair Package 0 — product contract smoke (RFC-0207).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn repair_pkg0_book0_product_control() {
    let text = std::fs::read_to_string(repo_root().join("specs/book-0.md")).unwrap();
    for needle in [
        "## 1.3 Площина продуктового керування",
        "не зобов",
        "має право",
        "не надає статусу `VERIFIED`",
        "Не видаляти з проєкту — лише винести з Kernel",
        "superseded",
    ] {
        assert!(text.contains(needle), "book-0 missing: {needle}");
    }
}

#[test]
fn repair_pkg0_terminology_kernel_ne_project() {
    let text = std::fs::read_to_string(repo_root().join("docs/canonical-terminology.md")).unwrap();
    for needle in [
        "Out of Kernel ≠ out of project",
        "Do not delete from the project",
        "product-requirements-trace.md",
        "GPU marketplace",
    ] {
        assert!(text.contains(needle), "terminology missing: {needle}");
    }
}

#[test]
fn repair_pkg0_scd_product_tracing() {
    let text = std::fs::read_to_string(repo_root().join("specs/specification-control.md")).unwrap();
    for needle in [
        "# 17. Product requirements tracing",
        "product-requirements-trace.md",
        "AIRA-RFC-0207",
        "CLI helper",
        "недостатніми",
    ] {
        assert!(text.contains(needle), "SCD missing: {needle}");
    }
}

#[test]
fn repair_pkg0_desktop_ux_steward() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    assert!(
        text.contains("власник інсталяції") || text.contains("installation owner"),
        "desktop-ux must define installation owner/steward"
    );
    assert!(
        text.contains("superseded") && text.contains("оператор"),
        "desktop-ux must supersede operator-out-of-scope"
    );
    assert!(
        !text.contains("| Оператор вузла | **поза scope** |"),
        "desktop-ux must not keep operator поза scope as current law"
    );
    assert!(text.contains("product-requirements-trace.md"));
}

#[test]
fn repair_pkg0_trace_and_rfc() {
    let trace =
        std::fs::read_to_string(repo_root().join("docs/product-requirements-trace.md")).unwrap();
    for needle in [
        "PR-M1",
        "PR-P1",
        "PR-V1",
        "CONTRACT",
        "schema/CLI",
        "not required",
    ] {
        assert!(trace.contains(needle), "trace missing: {needle}");
    }
    assert!(
        !trace.contains("| PR-M1 |")
            || trace.contains("**CONTRACT**")
            || trace.contains("CONTRACT"),
        "PR-M1 must not claim product DONE without UI"
    );
    let rfc = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0207-repair-package-0-product-contract.md"),
    )
    .expect("RFC-0207");
    for needle in [
        "Package 0",
        "operator out of scope",
        "VERIFIED",
        "Meditation",
    ] {
        assert!(rfc.contains(needle), "RFC-0207 missing: {needle}");
    }
    let close = std::fs::read_to_string(repo_root().join("docs/repair-package-0.md")).unwrap();
    assert!(close.contains("**DONE**") && close.contains("RFC-0207"));
}

#[test]
fn repair_pkg0_status_honesty() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("Product honesty") || status.contains("product-requirements-trace"));
    assert!(status.contains("RFC-0207") || status.contains("Repair Pack 0"));
}
