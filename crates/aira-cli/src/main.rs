//! AIRA CLI — local node, identity, CSU registry, problem/result/event commands.

use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod support;
mod tenant_secret;

use cli::Cli;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    commands::dispatch(cli.root, cli.command)
}

#[cfg(test)]
mod clap_secret_hex_file {
    use super::*;
    use clap::Parser;

    #[test]
    fn register_xor_secret_flags() {
        let err = Cli::try_parse_from([
            "aira",
            "identity",
            "csu-tenant",
            "register",
            "--csu-id",
            "aira:csu:x",
            "--publisher",
            "aira:identity:y",
            "--secret-hex",
            "aa",
            "--secret-hex-file",
            "seed.hex",
        ])
        .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("cannot be used with") || msg.contains("conflict"));
        assert!(!msg.contains("abababab"));
    }

    #[test]
    fn rotate_xor_secret_flags() {
        let err = Cli::try_parse_from([
            "aira",
            "identity",
            "csu-tenant",
            "rotate",
            "--csu-id",
            "aira:csu:x",
            "--secret-hex",
            "aa",
            "--secret-hex-file",
            "seed.hex",
        ])
        .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("cannot be used with") || msg.contains("conflict"));
        assert!(!msg.contains("abababab"));
    }
}

#[cfg(test)]
mod phase_n_cli_parse {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_peer_port_reachability_rendezvous() {
        for args in [
            vec!["aira", "peer", "port", "status"],
            vec!["aira", "peer", "port", "select", "--class", "udp-discv"],
            vec!["aira", "peer", "reachability", "status"],
            vec!["aira", "peer", "reachability", "check", "--port", "49157"],
            vec!["aira", "peer", "rendezvous", "status"],
            vec![
                "aira",
                "peer",
                "rendezvous",
                "publish",
                "--ttl-secs",
                "3600",
            ],
            vec![
                "aira",
                "peer",
                "rendezvous",
                "query",
                "--identity",
                "aira:identity:x",
            ],
        ] {
            Cli::try_parse_from(args).expect("phase-n peer CLI should parse");
        }
    }
}

#[cfg(test)]
mod problem_submit_cli_parity {
    //! `#347` / RFC-0230: supported-contract flags only; no unsupported no-op knobs.
    use super::*;
    use clap::Parser;
    use cli::{Commands, ProblemCommands};

    #[test]
    fn parses_allowed_and_excluded_model_refs() {
        let cli = Cli::try_parse_from([
            "aira",
            "problem",
            "submit",
            "--text",
            "Summarize the local Problem Statement",
            "--allowed-model-ref",
            "aira:model:a",
            "--allowed-model-ref",
            "aira:model:b",
            "--excluded-model-ref",
            "aira:model:a",
            "--placement",
            "local",
            "--reuse-policy",
            "allow_reuse",
        ])
        .expect("supported submit flags must parse");
        match cli.command {
            Commands::Problem {
                command:
                    ProblemCommands::Submit {
                        allowed_model_refs,
                        excluded_model_refs,
                        model_ref,
                        placement,
                        reuse_policy,
                        text,
                    },
            } => {
                assert_eq!(text, "Summarize the local Problem Statement");
                assert!(model_ref.is_none());
                assert_eq!(
                    allowed_model_refs,
                    vec!["aira:model:a".to_string(), "aira:model:b".to_string()]
                );
                assert_eq!(excluded_model_refs, vec!["aira:model:a".to_string()]);
                assert_eq!(placement.as_deref(), Some("local"));
                assert_eq!(reuse_policy.as_deref(), Some("allow_reuse"));
            }
            _ => panic!("expected problem submit"),
        }
    }

    #[test]
    fn rejects_removed_temperature_flag() {
        let err = Cli::try_parse_from([
            "aira",
            "problem",
            "submit",
            "--text",
            "hello",
            "--temperature",
            "0.2",
        ])
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("unexpected argument") || msg.contains("--temperature"),
            "{msg}"
        );
    }

    #[test]
    fn rejects_removed_privacy_and_fallback_flags() {
        for flag in [
            "--privacy-class",
            "--allow-model-fallback",
            "--allow-placement-fallback",
        ] {
            let err = Cli::try_parse_from(["aira", "problem", "submit", "--text", "hello", flag])
                .unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("unexpected argument") || msg.contains(flag),
                "flag {flag}: {msg}"
            );
        }
    }
}
