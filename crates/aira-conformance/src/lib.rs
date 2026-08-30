//! AIRA C0–C5 conformance runners + security/alpha (Issue Set Epic 9 / #63–#70, Epic 11 / #78–#80, Analyze-46).

mod alpha;
mod c0;
mod c1;
mod c2;
mod c3;
mod c4;
mod c5;
mod report;
mod runner;
mod security;

pub use alpha::run_alpha_acceptance;
pub use c0::run_c0;
pub use c1::run_c1;
pub use c2::run_c2;
pub use c3::run_c3;
pub use c4::run_c4;
pub use c5::run_c5;
pub use report::{
    AiraInfo, ConformanceProfile, ConformanceReport, FailureRecord, ImplementationInfo,
    ResultCounters,
};
pub use runner::{CaseOutcome, CaseResult, ConformanceError, SuiteResult};
pub use security::run_security_baseline;

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Run a profile suite into `artifact_root`.
pub fn run_profile(
    profile: ConformanceProfile,
    artifact_root: impl AsRef<std::path::Path>,
) -> Result<SuiteResult, ConformanceError> {
    match profile {
        ConformanceProfile::C0 => run_c0(artifact_root),
        ConformanceProfile::C1 => run_c1(artifact_root),
        ConformanceProfile::C2 => run_c2(artifact_root),
        ConformanceProfile::C3 => run_c3(artifact_root),
        ConformanceProfile::C4 => run_c4(artifact_root),
        ConformanceProfile::C5 => run_c5(artifact_root),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn c0_suite_passes_and_emits_immutable_report() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("reports");
        let suite = run_c0(&root).unwrap();
        assert_eq!(suite.report.aira.profile, ConformanceProfile::C0);
        assert_eq!(suite.report.results.failed, 0);
        assert!(suite.report.results.passed >= 5);
        assert!(!suite.report_artifact_id.as_str().is_empty());

        let v = serde_json::to_value(&suite.report).unwrap();
        let repo = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = aira_schema::SchemaRegistry::load(repo.join("schemas")).unwrap();
        reg.validate("aira:schema:conformance:report:0.1", &v)
            .unwrap();
    }

    #[test]
    fn c1_suite_passes_and_emits_report() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("reports");
        let suite = run_c1(&root).unwrap();
        assert_eq!(suite.report.aira.profile, ConformanceProfile::C1);
        assert_eq!(
            suite.report.results.failed, 0,
            "failures={:?}",
            suite.report.failures
        );
        assert!(suite.report.results.passed >= 6);
        assert_eq!(suite.cases.len(), 6);
    }

    #[test]
    fn c2_suite_passes_and_emits_report() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("reports");
        let suite = run_c2(&root).unwrap();
        assert_eq!(suite.report.aira.profile, ConformanceProfile::C2);
        assert_eq!(
            suite.report.results.failed, 0,
            "failures={:?}",
            suite.report.failures
        );
        assert_eq!(suite.report.results.passed, 11);
        assert_eq!(suite.cases.len(), 11);
    }

    #[test]
    fn c3_suite_passes_and_emits_report() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("reports");
        let suite = run_c3(&root).unwrap();
        assert_eq!(suite.report.aira.profile, ConformanceProfile::C3);
        assert_eq!(
            suite.report.results.failed, 0,
            "failures={:?}",
            suite.report.failures
        );
        assert_eq!(suite.report.results.passed, 8);
        assert_eq!(suite.cases.len(), 8);
        assert!(suite
            .cases
            .iter()
            .any(|c| c.test_id == "c3.capability.advertisement"));
        assert!(suite
            .cases
            .iter()
            .any(|c| c.test_id == "c3.federation.export_deny"));
        assert!(suite
            .cases
            .iter()
            .any(|c| c.test_id == "c3.crp.reject_node_route"));
        assert!(suite
            .cases
            .iter()
            .any(|c| c.test_id == "c3.crp.route_candidate"));
    }

    #[test]
    fn c4_suite_passes_and_emits_report() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("reports");
        let suite = run_c4(&root).unwrap();
        assert_eq!(suite.report.aira.profile, ConformanceProfile::C4);
        assert_eq!(
            suite.report.results.failed, 0,
            "failures={:?}",
            suite.report.failures
        );
        assert_eq!(suite.report.results.passed, 3);
        assert_eq!(suite.cases.len(), 3);
        assert!(suite
            .cases
            .iter()
            .any(|c| c.test_id == "c4.settlement.receipt_emit_verify"));
        assert!(suite
            .cases
            .iter()
            .any(|c| c.test_id == "c4.settlement.privacy_reject"));
        assert!(suite
            .cases
            .iter()
            .any(|c| c.test_id == "c4.settlement.link_prior_route"));
    }

    #[test]
    fn c5_suite_passes_and_emits_report() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("reports");
        let suite = run_c5(&root).unwrap();
        assert_eq!(suite.report.aira.profile, ConformanceProfile::C5);
        assert_eq!(
            suite.report.results.failed, 0,
            "failures={:?}",
            suite.report.failures
        );
        assert_eq!(suite.report.results.passed, 3);
        assert_eq!(suite.cases.len(), 3);
        assert!(suite
            .cases
            .iter()
            .any(|c| c.test_id == "c5.research.separation"));
        assert!(suite
            .cases
            .iter()
            .any(|c| c.test_id == "c5.promotion.gate_reject"));
        assert!(suite
            .cases
            .iter()
            .any(|c| c.test_id == "c5.promotion.candidate_schema"));
    }

    #[test]
    fn run_profile_dispatch() {
        let dir = tempfile::tempdir().unwrap();
        let suite = run_profile(ConformanceProfile::C0, dir.path().join("p")).unwrap();
        assert_eq!(suite.cases.len(), 10);
        let suite2 = run_profile(ConformanceProfile::C2, dir.path().join("p2")).unwrap();
        assert_eq!(suite2.cases.len(), 11);
        let suite3 = run_profile(ConformanceProfile::C3, dir.path().join("p3")).unwrap();
        assert_eq!(suite3.cases.len(), 8);
        let suite4 = run_profile(ConformanceProfile::C4, dir.path().join("p4")).unwrap();
        assert_eq!(suite4.cases.len(), 3);
        let suite5 = run_profile(ConformanceProfile::C5, dir.path().join("p5")).unwrap();
        assert_eq!(suite5.cases.len(), 3);
    }

    #[test]
    fn security_baseline_passes() {
        let dir = tempfile::tempdir().unwrap();
        let suite = run_security_baseline(dir.path().join("sec")).unwrap();
        assert_eq!(
            suite.report.results.failed, 0,
            "failures={:?}",
            suite.report.failures
        );
        assert_eq!(suite.report.results.passed, 10);
    }

    #[test]
    fn alpha_acceptance_passes() {
        let dir = tempfile::tempdir().unwrap();
        let suite = run_alpha_acceptance(dir.path()).unwrap();
        assert_eq!(
            suite.report.results.failed, 0,
            "failures={:?}",
            suite.report.failures
        );
        assert!(suite.report.results.passed >= 4);
    }
}
