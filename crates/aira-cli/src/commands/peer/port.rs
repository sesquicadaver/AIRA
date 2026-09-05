//! Phase N `#243`: `aira peer port …`

use std::path::Path;
use std::process::ExitCode;

use aira_object::Keyring;
use anyhow::{bail, Result};

use crate::cli::PeerPortCommands;

fn parse_class(s: &str) -> Result<aira_peer::TransportClass> {
    match s.trim() {
        "tcp-peer" | "tcp" => Ok(aira_peer::TransportClass::TcpPeer),
        "udp-discv" | "udp" => Ok(aira_peer::TransportClass::UdpDiscv),
        other => bail!("unknown transport class {other} (want tcp-peer|udp-discv)"),
    }
}

pub(super) async fn run(root: &Path, command: PeerPortCommands) -> Result<ExitCode> {
    let (id, _) = Keyring::load_node_identity(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    match command {
        PeerPortCommands::Status { class } => {
            let class = parse_class(&class)?;
            let preferred = aira_peer::preferred_port(id.as_str(), class);
            let suggested = aira_peer::suggested_aira_port(id.as_str(), class);
            let idx = aira_peer::preferred_port_index(id.as_str(), class);
            println!("identity {}", id.as_str());
            println!("class {}", class.as_str());
            println!("preferred_port {preferred}");
            println!("preferred_index {idx}");
            println!("suggested_port {suggested}");
            println!("p_aira_count {}", aira_peer::P_AIRA_COUNT);
            Ok(ExitCode::SUCCESS)
        }
        PeerPortCommands::Select { class } => {
            let class = parse_class(&class)?;
            match class {
                aira_peer::TransportClass::TcpPeer => {
                    let (listener, addr) =
                        aira_peer::select_available_loopback_tcp_for(id.as_str())
                            .map_err(|e| anyhow::anyhow!("{e}"))?;
                    drop(listener);
                    println!("identity {}", id.as_str());
                    println!("class {}", class.as_str());
                    println!("selected_bind {addr}");
                    println!("selected_port {}", addr.port());
                }
                aira_peer::TransportClass::UdpDiscv => {
                    let (_sock, addr) = aira_peer::select_available_loopback_udp_for(id.as_str())
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    println!("identity {}", id.as_str());
                    println!("class {}", class.as_str());
                    println!("selected_bind {addr}");
                    println!("selected_port {}", addr.port());
                }
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}
