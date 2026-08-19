//! Problem / result / artifact / event (Analyze-81).

use std::path::Path;
use std::process::ExitCode;

use aira_flow::{LocalSession, SubmitOutcome};
use anyhow::Result;

use crate::cli::{ArtifactCommands, EventCommands, ProblemCommands, ResultCommands};
use crate::support::ensure_init;

pub(crate) fn problem(root: &Path, command: ProblemCommands) -> Result<ExitCode> {
    match command {
        ProblemCommands::Submit { text } => {
            ensure_init(root)?;
            let mut session = LocalSession::open(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            let out = session
                .submit_problem(&text)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            match out {
                SubmitOutcome::Completed {
                    problem_id,
                    verified_artifact_id,
                    result,
                } => {
                    println!("problem_ref {}", problem_id);
                    println!("result_ref {}", verified_artifact_id);
                    println!("status completed");
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                SubmitOutcome::NeedsHumanCollapse { field_artifact_id } => {
                    let pref = session
                        .plane()
                        .problem_ref()
                        .map(|r| r.as_str().to_string())
                        .unwrap_or_default();
                    println!("problem_ref {pref}");
                    println!("field_ref {}", field_artifact_id);
                    println!("status needs_human_collapse");
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        ProblemCommands::Status { problem_ref } => {
            ensure_init(root)?;
            let session = LocalSession::open(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            let rec = session
                .problem_status(&problem_ref)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{}", serde_json::to_string_pretty(&rec)?);
            Ok(ExitCode::SUCCESS)
        }
    }
}

pub(crate) fn result(root: &Path, command: ResultCommands) -> Result<ExitCode> {
    match command {
        ResultCommands::Get { result_ref } => {
            ensure_init(root)?;
            let session = LocalSession::open(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            let v = session
                .get_result(&result_ref)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{}", serde_json::to_string_pretty(&v)?);
            Ok(ExitCode::SUCCESS)
        }
    }
}

pub(crate) fn artifact(root: &Path, command: ArtifactCommands) -> Result<ExitCode> {
    match command {
        ArtifactCommands::Get { artifact_ref, raw } => {
            ensure_init(root)?;
            let session = LocalSession::open(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            let (desc, bytes) = session
                .get_artifact(&artifact_ref)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            if raw {
                match std::str::from_utf8(&bytes) {
                    Ok(s) => print!("{s}"),
                    Err(_) => println!("{}", hex::encode(&bytes)),
                }
            } else {
                let mut out = serde_json::Map::new();
                out.insert("descriptor".into(), desc);
                if let Ok(body) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                    out.insert("payload".into(), body);
                } else {
                    out.insert("payload_hex".into(), serde_json::json!(hex::encode(&bytes)));
                }
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::Value::Object(out))?
                );
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

pub(crate) fn event(root: &Path, command: EventCommands) -> Result<ExitCode> {
    match command {
        EventCommands::Tail { limit } => {
            ensure_init(root)?;
            let session = LocalSession::open(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            let events = session
                .event_tail(limit)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            for e in events {
                println!(
                    "{}\t{:?}\t{}",
                    e.event_id,
                    e.event_type,
                    e.payload_ref.as_deref().unwrap_or("-")
                );
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}
