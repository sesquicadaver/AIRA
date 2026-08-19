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
