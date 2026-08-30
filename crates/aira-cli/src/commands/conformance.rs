//! Conformance runners (Analyze-81).

use std::path::Path;
use std::process::ExitCode;

use aira_conformance::{run_profile, ConformanceProfile};
use anyhow::{bail, Result};

use crate::cli::ConformanceCommands;

pub(crate) fn run(root: &Path, command: ConformanceCommands) -> Result<ExitCode> {
    match command {
        ConformanceCommands::Run { profile, out } => {
            let profile = match profile.to_uppercase().as_str() {
                "C0" => ConformanceProfile::C0,
                "C1" => ConformanceProfile::C1,
                "C2" => ConformanceProfile::C2,
                "C3" => ConformanceProfile::C3,
                "C4" => ConformanceProfile::C4,
                "C5" => ConformanceProfile::C5,
                other => bail!("unsupported profile {other} (use C0, C1, C2, C3, C4, or C5)"),
            };
            let out = out.unwrap_or_else(|| root.join("conformance").join("reports"));
            std::fs::create_dir_all(&out)?;
            let suite = run_profile(profile, &out).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("profile {}", suite.report.aira.profile.as_str());
            println!(
                "results total={} passed={} failed={} skipped={}",
                suite.report.results.total,
                suite.report.results.passed,
                suite.report.results.failed,
                suite.report.results.skipped
            );
            println!("report_artifact {}", suite.report_artifact_id);
            for f in &suite.report.failures {
                eprintln!("FAIL {}: {}", f.test_id, f.reason);
            }
            if suite.report.results.failed > 0 {
                Ok(ExitCode::FAILURE)
            } else {
                Ok(ExitCode::SUCCESS)
            }
        }
    }
}
