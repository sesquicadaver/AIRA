use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;

use crate::cli::PeerStunCommands;

pub(super) async fn run(root: &Path, command: PeerStunCommands) -> Result<ExitCode> {
    match command {
        PeerStunCommands::Query { stun_server } => {
            let server = stun_server.ok_or_else(|| {
                anyhow::anyhow!(
                    "stun server required — pass --stun-server host:port or set AIRA_STUN_SERVER"
                )
            })?;
            let rec = aira_peer::query_and_save_stun_reflexive(
                root,
                &server,
                aira_peer::STUN_QUERY_TIMEOUT,
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("stun reflexive {}", rec.addr);
            println!("stun_server {}", rec.stun_server);
            println!(
                "stun_reflexive {}",
                aira_peer::StunReflexiveRecord::path(root).display()
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}
