//! Schema list/validate (Analyze-81).

use std::process::ExitCode;

use aira_schema::find_repo_root;
use anyhow::{Context, Result};

use crate::cli::SchemaCommands;
use crate::support::load_schema_registry;

pub(crate) fn run(command: SchemaCommands) -> Result<ExitCode> {
    match command {
        SchemaCommands::List { schemas_dir } => {
            let reg = load_schema_registry(schemas_dir)?;
            for id in reg.list_ids() {
                println!("{id}");
            }
            Ok(ExitCode::SUCCESS)
        }
        SchemaCommands::Validate {
            schema,
            file,
            fixtures,
            schemas_dir,
        } => {
            let reg = load_schema_registry(schemas_dir)?;
            if let Some(fixtures_root) = fixtures {
                let root_repo = if fixtures_root.as_os_str() == "fixtures"
                    || fixtures_root.ends_with("fixtures")
                {
                    find_repo_root(std::env::current_dir()?)?
                } else {
                    fixtures_root
                };
                let report = reg.validate_fixtures(&root_repo)?;
                println!(
                    "fixtures: passed={} failed={}",
                    report.passed, report.failed
                );
                for f in &report.failures {
                    eprintln!("FAIL: {f}");
                }
                if report.failed > 0 {
                    return Ok(ExitCode::FAILURE);
                }
                return Ok(ExitCode::SUCCESS);
            }

            let schema = schema.context("--schema is required unless --fixtures is set")?;
            let file = file.context("--file is required unless --fixtures is set")?;
            match reg.validate_file(&schema, &file) {
                Ok(()) => {
                    println!("OK: {} validates against {schema}", file.display());
                    Ok(ExitCode::SUCCESS)
                }
                Err(e) => {
                    eprintln!("FAIL: {e}");
                    Ok(ExitCode::FAILURE)
                }
            }
        }
    }
}
