//! Problem / result / artifact / event (Analyze-81).

use std::path::Path;
use std::process::ExitCode;

use aira_flow::{
    AdmissionConstraints, AdmissionSnapshot, FallbackRules, GenerationParameters, LocalSession,
    PlacementPreference, ReusePolicy, SubmitOutcome,
};
use anyhow::Result;

use crate::cli::{ArtifactCommands, EventCommands, ProblemCommands, ResultCommands};
use crate::support::ensure_init;

fn parse_placement(s: &str) -> Result<PlacementPreference> {
    match s.trim().to_ascii_lowercase().as_str() {
        "local" => Ok(PlacementPreference::Local),
        "remote_allowed" => Ok(PlacementPreference::RemoteAllowed),
        "remote_required" => Ok(PlacementPreference::RemoteRequired),
        other => {
            anyhow::bail!("unknown --placement {other} (local|remote_allowed|remote_required)")
        }
    }
}

fn parse_reuse_policy(s: &str) -> Result<ReusePolicy> {
    match s.trim().to_ascii_lowercase().as_str() {
        "allow_reuse" => Ok(ReusePolicy::AllowReuse),
        "require_new_execution" => Ok(ReusePolicy::RequireNewExecution),
        other => {
            anyhow::bail!("unknown --reuse-policy {other} (allow_reuse|require_new_execution)")
        }
    }
}

pub(crate) fn problem(root: &Path, command: ProblemCommands) -> Result<ExitCode> {
    match command {
        ProblemCommands::Submit {
            text,
            model_ref,
            allowed_model_refs,
            placement,
            reuse_policy,
            temperature,
            privacy_class,
            allow_model_fallback,
            allow_placement_fallback,
        } => {
            ensure_init(root)?;
            // #319 / RFC-0204: label executor before outcome so mock ≠ configured LLM.
            let executor = aira_flow::staff_executor_kind();
            if aira_flow::staff_executor_is_reference_mock() {
                println!("executor {executor} (reference)");
                println!("mode reference");
            } else {
                println!("executor {executor}");
            }
            let constraints = AdmissionConstraints {
                model_ref,
                allowed_model_refs,
                generation: GenerationParameters {
                    temperature,
                    ..Default::default()
                },
                placement: match placement {
                    Some(p) => parse_placement(&p)?,
                    None => PlacementPreference::default(),
                },
                privacy_class,
                fallback: FallbackRules {
                    allow_model_fallback,
                    allow_placement_fallback,
                },
                reuse_policy: match reuse_policy {
                    Some(p) => parse_reuse_policy(&p)?,
                    None => ReusePolicy::default(),
                },
                ..Default::default()
            };
            let snap = AdmissionSnapshot::from_text_and_constraints(&text, &constraints);
            let mut session = LocalSession::open(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            let out = session
                .submit_problem_with_admission(&text, snap)
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
                SubmitOutcome::Executed {
                    problem_id,
                    execution_artifact_id,
                    result,
                } => {
                    println!("problem_ref {}", problem_id);
                    println!("result_ref {}", execution_artifact_id);
                    println!("status executed");
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
