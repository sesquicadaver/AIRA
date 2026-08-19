//! Init / status (Analyze-81).

use std::path::Path;
use std::process::ExitCode;

use aira_flow::{init_node, node_config_present, LocalSession};
use anyhow::Result;

pub(crate) fn init(root: &Path) -> Result<ExitCode> {
    let paths = init_node(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("initialized {}", paths.root.display());
    println!("config {}", paths.config().display());
    println!("sqlite {}", paths.sqlite().display());
    println!("artifacts {}", paths.artifacts().display());
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn status(root: &Path) -> Result<ExitCode> {
    println!("aira {}", env!("CARGO_PKG_VERSION"));
    println!("status: C1 Conformance ready (Epic 9)");
    if node_config_present(root) {
        let session = LocalSession::open(root).map_err(|e| anyhow::anyhow!("{e}"))?;
        println!(
            "node: mode={} profile={}",
            session.config.node.mode, session.config.node.profile
        );
        println!("root: {}", session.paths.root.display());
        let has_id = session.paths.identity_json().exists();
        println!("identity: {}", if has_id { "present" } else { "missing" });
    } else {
        println!(
            "root: {} (not initialized — run `aira init`)",
            root.display()
        );
    }
    Ok(ExitCode::SUCCESS)
}
