//! EVM rendezvous adapter (QUEUE #236 / Phase N).
//!
//! `EvmRendezvousProvider` implements [`RendezvousProvider`] with a deterministic
//! local ledger double for CI. Amoy (`80002`) and Polygon mainnet (`137`) are
//! config profiles only — live JSON-RPC publish/query deepen in `#237`.
//! EVM tx sender is never treated as AIRA identity.

use aira_object::ContentHash;

use crate::error::PeerError;
use crate::presence::NodePresenceRecord;
use crate::rendezvous::{MockRendezvousProvider, RendezvousProvider};

/// Adapter kind for EVM-shaped rendezvous.
pub const RENDEZVOUS_KIND_EVM: &str = "evm";

/// CI / unit local double chain id (not a public network).
pub const EVM_CHAIN_LOCAL_DOUBLE: u64 = 31337;
/// Polygon Amoy testnet.
pub const EVM_CHAIN_AMOY: u64 = 80002;
/// Polygon PoS mainnet.
pub const EVM_CHAIN_POLYGON: u64 = 137;

/// Placeholder contract for local double (not on-chain).
pub const EVM_LOCAL_CONTRACT_PLACEHOLDER: &str = "0x000000000000000000000000000000000000a12a";
/// Documented Amoy RPC hook default (not dialed in `#236`).
pub const EVM_AMOY_RPC_DEFAULT: &str = "https://rpc-amoy.polygon.technology/";
/// Documented Polygon mainnet RPC hook default (not dialed in `#236`).
pub const EVM_POLYGON_RPC_DEFAULT: &str = "https://polygon-rpc.com/";

/// Which EVM profile the adapter is configured for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvmChainProfile {
    /// Deterministic in-process double (CI).
    LocalDouble,
    /// Reference public testnet config hooks.
    Amoy,
    /// Production-compatible mainnet config hooks.
    PolygonMainnet,
}

impl EvmChainProfile {
    /// Chain id for this profile.
    pub fn chain_id(self) -> u64 {
        match self {
            Self::LocalDouble => EVM_CHAIN_LOCAL_DOUBLE,
            Self::Amoy => EVM_CHAIN_AMOY,
            Self::PolygonMainnet => EVM_CHAIN_POLYGON,
        }
    }

    /// Default RPC URL hook (never dialed by `#236` local path).
    pub fn default_rpc_url(self) -> &'static str {
        match self {
            Self::LocalDouble => "aira://evm-local-double",
            Self::Amoy => EVM_AMOY_RPC_DEFAULT,
            Self::PolygonMainnet => EVM_POLYGON_RPC_DEFAULT,
        }
    }
}

/// Config for [`EvmRendezvousProvider`] (Amoy/mainnet hooks + local double).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmRendezvousConfig {
    pub profile: EvmChainProfile,
    pub chain_id: u64,
    pub rpc_url: String,
    pub contract_address: String,
    /// When true, storage is the in-process double (required for CI / `#236`).
    pub use_local_double: bool,
}

impl EvmRendezvousConfig {
    /// Fail-closed: Amoy/mainnet profiles must keep documented chain ids.
    pub fn validate(&self) -> Result<(), PeerError> {
        let expected = self.profile.chain_id();
        if self.chain_id != expected {
            return Err(PeerError::Rendezvous(format!(
                "evm chain_id mismatch: profile {:?} expects {expected}, got {}",
                self.profile, self.chain_id
            )));
        }
        if self.rpc_url.trim().is_empty() {
            return Err(PeerError::Rendezvous("evm rpc_url empty".into()));
        }
        let addr = self.contract_address.trim();
        if !addr.starts_with("0x") || addr.len() != 42 {
            return Err(PeerError::Rendezvous(
                "evm contract_address must be 0x + 40 hex chars".into(),
            ));
        }
        if !addr[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(PeerError::Rendezvous(
                "evm contract_address must be hex".into(),
            ));
        }
        if matches!(
            self.profile,
            EvmChainProfile::Amoy | EvmChainProfile::PolygonMainnet
        ) && !self.use_local_double
        {
            return Err(PeerError::Rendezvous(
                "live EVM RPC publish/query is #237; use_local_double=true or LocalDouble profile"
                    .into(),
            ));
        }
        if self.profile == EvmChainProfile::LocalDouble && !self.use_local_double {
            return Err(PeerError::Rendezvous(
                "LocalDouble profile requires use_local_double=true".into(),
            ));
        }
        Ok(())
    }

    /// CI / unit deterministic double.
    pub fn local_double() -> Self {
        Self {
            profile: EvmChainProfile::LocalDouble,
            chain_id: EVM_CHAIN_LOCAL_DOUBLE,
            rpc_url: EvmChainProfile::LocalDouble.default_rpc_url().into(),
            contract_address: EVM_LOCAL_CONTRACT_PLACEHOLDER.into(),
            use_local_double: true,
        }
    }

    /// Amoy config hooks with local storage (no network in `#236`).
    pub fn amoy_local_double(contract_address: impl Into<String>) -> Self {
        Self {
            profile: EvmChainProfile::Amoy,
            chain_id: EVM_CHAIN_AMOY,
            rpc_url: EVM_AMOY_RPC_DEFAULT.into(),
            contract_address: contract_address.into(),
            use_local_double: true,
        }
    }

    /// Polygon mainnet config hooks with local storage (no network in `#236`).
    pub fn polygon_mainnet_local_double(contract_address: impl Into<String>) -> Self {
        Self {
            profile: EvmChainProfile::PolygonMainnet,
            chain_id: EVM_CHAIN_POLYGON,
            rpc_url: EVM_POLYGON_RPC_DEFAULT.into(),
            contract_address: contract_address.into(),
            use_local_double: true,
        }
    }

    /// Override RPC URL on a config (still not dialed when `use_local_double`).
    pub fn with_rpc_url(mut self, rpc_url: impl Into<String>) -> Self {
        self.rpc_url = rpc_url.into();
        self
    }
}

/// SHA-256 hex of `identity_ref` bytes — contract `identity_hash` key shape.
pub fn evm_identity_hash(identity_ref: &str) -> String {
    ContentHash::sha256_bytes(identity_ref.as_bytes())
        .as_str()
        .to_string()
}

/// EVM-shaped rendezvous provider (local double backend in `#236`).
#[derive(Debug, Clone)]
pub struct EvmRendezvousProvider {
    config: EvmRendezvousConfig,
    inner: MockRendezvousProvider,
}

impl EvmRendezvousProvider {
    /// Build from validated config (local double only in this atom).
    pub fn new(config: EvmRendezvousConfig) -> Result<Self, PeerError> {
        config.validate()?;
        if !config.use_local_double {
            return Err(PeerError::Rendezvous(
                "EvmRendezvousProvider #236 requires use_local_double".into(),
            ));
        }
        Ok(Self {
            config,
            inner: MockRendezvousProvider::new(),
        })
    }

    /// Shortcut: local double profile.
    pub fn local_double() -> Self {
        Self::new(EvmRendezvousConfig::local_double()).expect("local_double config valid")
    }

    /// Active config (chain_id / rpc / contract hooks).
    pub fn config(&self) -> &EvmRendezvousConfig {
        &self.config
    }

    /// Contract-facing identity hash for a presence (not EVM account).
    pub fn identity_hash_for(record: &NodePresenceRecord) -> String {
        evm_identity_hash(&record.identity_ref)
    }
}

impl RendezvousProvider for EvmRendezvousProvider {
    fn publish_presence(&mut self, record: NodePresenceRecord) -> Result<(), PeerError> {
        // Authenticity is AIRA Ed25519 on the record — never the EVM payer.
        let _ = Self::identity_hash_for(&record);
        self.inner.publish_presence(record)
    }

    fn update_presence(&mut self, record: NodePresenceRecord) -> Result<(), PeerError> {
        let _ = Self::identity_hash_for(&record);
        self.inner.update_presence(record)
    }

    fn remove_or_expire_presence(
        &mut self,
        identity_ref: &str,
        as_of: &str,
        force: bool,
    ) -> Result<bool, PeerError> {
        self.inner
            .remove_or_expire_presence(identity_ref, as_of, force)
    }

    fn query_active_peers(&self, as_of: &str) -> Result<Vec<NodePresenceRecord>, PeerError> {
        self.inner.query_active_peers(as_of)
    }

    fn query_identity(&self, identity_ref: &str) -> Result<Option<NodePresenceRecord>, PeerError> {
        self.inner.query_identity(identity_ref)
    }

    fn query_relays(&self, as_of: &str) -> Result<Vec<NodePresenceRecord>, PeerError> {
        self.inner.query_relays(as_of)
    }

    fn provider_kind(&self) -> &'static str {
        RENDEZVOUS_KIND_EVM
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use aira_flow::NodePaths;
    use aira_object::{ensure_trust_defaults, sign_with_key, AiraRef, Keyring};
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

    use crate::presence::{
        empty_capabilities_hash, PresenceDirectEndpoint, PresenceDraft, PresenceReachability,
    };

    fn write_node(root: &std::path::Path, name: &str, seed: [u8; 32]) -> (AiraRef, String) {
        let paths = NodePaths::new(root);
        fs::create_dir_all(paths.identity_dir()).unwrap();
        let sk = SigningKey::from_bytes(&seed);
        let pub_hex = hex::encode(sk.verifying_key().to_bytes());
        let id = format!("aira:identity:{name}");
        let id_ref = AiraRef::parse(&id).unwrap();
        fs::write(
            paths.identity_key(),
            format!("{}\n", hex::encode(sk.to_bytes())),
        )
        .unwrap();
        let sig = sign_with_key(id_ref.clone(), &sk, id.as_bytes());
        let desc = serde_json::json!({
            "identity_id": id,
            "identity_type": "local",
            "display_name": name,
            "public_key": { "algorithm": "ed25519", "key_hex": pub_hex },
            "created_at": "2026-07-16T00:00:00Z",
            "key_path": "identity/local.ed25519",
            "signature": sig
        });
        fs::write(
            paths.identity_json(),
            serde_json::to_string_pretty(&desc).unwrap(),
        )
        .unwrap();
        let _ = ensure_trust_defaults(root).unwrap();
        let (loaded_id, _ring): (AiraRef, Keyring) = Keyring::load_node_identity(root).unwrap();
        assert_eq!(loaded_id, id_ref);
        (id_ref, pub_hex)
    }

    fn signed(
        root: &std::path::Path,
        id: &AiraRef,
        pub_hex: &str,
        sequence: u64,
    ) -> NodePresenceRecord {
        NodePresenceRecord::draft(PresenceDraft {
            identity_ref: id.as_str().into(),
            identity_public_key: pub_hex.into(),
            sequence,
            created_at: "2026-09-05T12:00:00Z".into(),
            expires_at: "2026-09-12T12:00:00Z".into(),
            direct_endpoints: vec![PresenceDirectEndpoint {
                transport: "tcp-peer".into(),
                host: "127.0.0.1".into(),
                port: 49157,
                reachability_state: PresenceReachability::Unknown,
                observed_at: "2026-09-05T12:00:00Z".into(),
            }],
            relay_endpoints: vec![],
            capabilities_hash: empty_capabilities_hash(),
        })
        .unwrap()
        .sign_for_node_root(root)
        .unwrap()
    }

    #[test]
    fn local_double_roundtrip_and_kind() {
        let dir = tempdir().unwrap();
        let (id, pk) = write_node(dir.path(), "evm-alice", [51u8; 32]);
        let mut evm = EvmRendezvousProvider::local_double();
        assert_eq!(evm.provider_kind(), RENDEZVOUS_KIND_EVM);
        assert_eq!(evm.config().chain_id, EVM_CHAIN_LOCAL_DOUBLE);
        assert!(evm.config().use_local_double);
        let rec = signed(dir.path(), &id, &pk, 1);
        let hash = EvmRendezvousProvider::identity_hash_for(&rec);
        assert!(hash.starts_with("sha256:"));
        evm.publish_presence(rec).unwrap();
        assert_eq!(
            evm.query_active_peers("2026-09-06T00:00:00Z")
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn amoy_and_polygon_config_hooks() {
        let amoy = EvmRendezvousConfig::amoy_local_double(EVM_LOCAL_CONTRACT_PLACEHOLDER);
        amoy.validate().unwrap();
        assert_eq!(amoy.chain_id, EVM_CHAIN_AMOY);
        assert_eq!(amoy.rpc_url, EVM_AMOY_RPC_DEFAULT);
        let poly =
            EvmRendezvousConfig::polygon_mainnet_local_double(EVM_LOCAL_CONTRACT_PLACEHOLDER);
        poly.validate().unwrap();
        assert_eq!(poly.chain_id, EVM_CHAIN_POLYGON);
        let provider = EvmRendezvousProvider::new(amoy).unwrap();
        assert_eq!(provider.config().profile, EvmChainProfile::Amoy);
    }

    #[test]
    fn rejects_live_remote_without_237() {
        let mut cfg = EvmRendezvousConfig::amoy_local_double(EVM_LOCAL_CONTRACT_PLACEHOLDER);
        cfg.use_local_double = false;
        assert!(cfg.validate().is_err());
        assert!(EvmRendezvousProvider::new(cfg).is_err());
    }

    #[test]
    fn rejects_chain_id_mismatch() {
        let mut cfg = EvmRendezvousConfig::local_double();
        cfg.chain_id = 1;
        assert!(cfg.validate().is_err());
    }
}
