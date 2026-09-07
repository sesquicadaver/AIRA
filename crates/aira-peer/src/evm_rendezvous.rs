//! EVM rendezvous adapter (QUEUE #236 / Phase N; live JSON-RPC `#248`; honesty `#271`).
//!
//! `EvmRendezvousProvider` implements [`RendezvousProvider`] with either a
//! deterministic local double (`use_local_double=true`) or a live HTTP JSON-RPC
//! dial (`use_local_double=false`) against anvil / Amoy hooks / the
//! [`crate::evm_rendezvous_rpc::ReferenceEvmRendezvousRpc`] stand-in.
//! Reference/Mock HTTP is **PARTIAL** — never an on-chain Polygon ledger claim.
//! Declared Amoy/mainnet `https://` URLs validate in config; dial remains via
//! `http://` gateway/reference until a TLS client ships. EVM tx sender ≠ AIRA identity.

use aira_object::ContentHash;
use serde_json::{json, Value};

use crate::error::PeerError;
use crate::evm_rendezvous_rpc::{
    RPC_ETH_CHAIN_ID, RPC_PUBLISH, RPC_QUERY_ACTIVE, RPC_QUERY_IDENTITY, RPC_QUERY_RELAYS,
    RPC_REMOVE, RPC_UPDATE,
};
use crate::json_rpc_http::json_rpc_call;
use crate::presence::NodePresenceRecord;
use crate::rendezvous::{MockRendezvousProvider, RendezvousProvider};

/// Adapter kind for EVM-shaped rendezvous.
pub const RENDEZVOUS_KIND_EVM: &str = "evm";

/// Honesty label for what an EVM adapter path actually proved (`#271`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvmLedgerClaim {
    /// In-process [`MockRendezvousProvider`] double — not a chain.
    LocalMockDouble,
    /// Live dial to HTTP JSON-RPC (anvil / [`crate::ReferenceEvmRendezvousRpc`]).
    /// Socket + custom `aira_rendezvous_*` methods — **not** an on-chain ledger.
    ReferenceHttpJsonRpc,
    /// Config stores declared `https://` Amoy/mainnet URL; dial not completed here.
    DeclaredHttpsConfig,
}

impl EvmLedgerClaim {
    /// Wire / docs token.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalMockDouble => "local_mock_double",
            Self::ReferenceHttpJsonRpc => "reference_http_json_rpc",
            Self::DeclaredHttpsConfig => "declared_https_config",
        }
    }

    /// True only for a real on-chain claim — always false in this crate (`#271` PARTIAL).
    pub fn is_on_chain_ledger(self) -> bool {
        false
    }
}

/// CI / unit local double chain id (not a public network).
pub const EVM_CHAIN_LOCAL_DOUBLE: u64 = 31337;
/// Polygon Amoy testnet.
pub const EVM_CHAIN_AMOY: u64 = 80002;
/// Polygon PoS mainnet.
pub const EVM_CHAIN_POLYGON: u64 = 137;

/// Placeholder contract for local double (not on-chain).
pub const EVM_LOCAL_CONTRACT_PLACEHOLDER: &str = "0x000000000000000000000000000000000000a12a";
/// Documented Amoy RPC hook default (dialed only when `use_local_double=false`).
pub const EVM_AMOY_RPC_DEFAULT: &str = "https://rpc-amoy.polygon.technology/";
/// Documented Polygon mainnet RPC hook default (dialed only when live).
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

    /// Default RPC URL hook.
    pub fn default_rpc_url(self) -> &'static str {
        match self {
            Self::LocalDouble => "aira://evm-local-double",
            Self::Amoy => EVM_AMOY_RPC_DEFAULT,
            Self::PolygonMainnet => EVM_POLYGON_RPC_DEFAULT,
        }
    }
}

/// Config for [`EvmRendezvousProvider`] (Amoy/mainnet hooks + local double + live).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmRendezvousConfig {
    pub profile: EvmChainProfile,
    pub chain_id: u64,
    pub rpc_url: String,
    pub contract_address: String,
    /// When true, storage is the in-process double (no network).
    pub use_local_double: bool,
}

impl EvmRendezvousConfig {
    /// Fail-closed: profile chain ids; live path requires http(s) RPC URL.
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
        if !self.use_local_double {
            let url = self.rpc_url.trim();
            let http = url.starts_with("http://");
            let https = url.starts_with("https://");
            if !http && !https {
                return Err(PeerError::Rendezvous(format!(
                    "live EVM rpc_url must be http:// or https:// (#271), got {url}"
                )));
            }
        }
        Ok(())
    }

    /// True when live config points at a declared HTTPS Amoy/mainnet-style URL.
    pub fn is_declared_https_rpc(&self) -> bool {
        !self.use_local_double && self.rpc_url.trim().starts_with("https://")
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

    /// Amoy config hooks with local storage (no network).
    pub fn amoy_local_double(contract_address: impl Into<String>) -> Self {
        Self {
            profile: EvmChainProfile::Amoy,
            chain_id: EVM_CHAIN_AMOY,
            rpc_url: EVM_AMOY_RPC_DEFAULT.into(),
            contract_address: contract_address.into(),
            use_local_double: true,
        }
    }

    /// Polygon mainnet config hooks with local storage (no network).
    pub fn polygon_mainnet_local_double(contract_address: impl Into<String>) -> Self {
        Self {
            profile: EvmChainProfile::PolygonMainnet,
            chain_id: EVM_CHAIN_POLYGON,
            rpc_url: EVM_POLYGON_RPC_DEFAULT.into(),
            contract_address: contract_address.into(),
            use_local_double: true,
        }
    }

    /// Live Amoy-shaped profile: real JSON-RPC dial (`#248`).
    ///
    /// `rpc_url` may be `https://` (declared Amoy default) or `http://` gateway/reference.
    pub fn amoy_live(contract_address: impl Into<String>, rpc_url: impl Into<String>) -> Self {
        Self {
            profile: EvmChainProfile::Amoy,
            chain_id: EVM_CHAIN_AMOY,
            rpc_url: rpc_url.into(),
            contract_address: contract_address.into(),
            use_local_double: false,
        }
    }

    /// Amoy live hooks with the documented HTTPS RPC URL (`#271` config path).
    pub fn amoy_live_declared_https(contract_address: impl Into<String>) -> Self {
        Self::amoy_live(contract_address, EVM_AMOY_RPC_DEFAULT)
    }

    /// Polygon mainnet live hooks with the documented HTTPS RPC URL (`#271`).
    pub fn polygon_live_declared_https(contract_address: impl Into<String>) -> Self {
        Self {
            profile: EvmChainProfile::PolygonMainnet,
            chain_id: EVM_CHAIN_POLYGON,
            rpc_url: EVM_POLYGON_RPC_DEFAULT.into(),
            contract_address: contract_address.into(),
            use_local_double: false,
        }
    }

    /// Live dial against local anvil or [`crate::evm_rendezvous_rpc::ReferenceEvmRendezvousRpc`] (`#248`).
    pub fn anvil_live(contract_address: impl Into<String>, rpc_url: impl Into<String>) -> Self {
        Self {
            profile: EvmChainProfile::LocalDouble,
            chain_id: EVM_CHAIN_LOCAL_DOUBLE,
            rpc_url: rpc_url.into(),
            contract_address: contract_address.into(),
            use_local_double: false,
        }
    }

    /// Override RPC URL on a config.
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

#[derive(Debug, Clone)]
enum EvmBackend {
    Local(MockRendezvousProvider),
    JsonRpc,
}

/// EVM-shaped rendezvous provider (local double or live JSON-RPC).
#[derive(Debug, Clone)]
pub struct EvmRendezvousProvider {
    config: EvmRendezvousConfig,
    backend: EvmBackend,
}

impl EvmRendezvousProvider {
    /// Build from validated config (local double or live HTTP JSON-RPC).
    pub fn new(config: EvmRendezvousConfig) -> Result<Self, PeerError> {
        config.validate()?;
        let backend = if config.use_local_double {
            EvmBackend::Local(MockRendezvousProvider::new())
        } else {
            EvmBackend::JsonRpc
        };
        let provider = Self { config, backend };
        if matches!(provider.backend, EvmBackend::JsonRpc) {
            provider.verify_chain_id()?;
        }
        Ok(provider)
    }

    /// Shortcut: local double profile.
    pub fn local_double() -> Self {
        Self::new(EvmRendezvousConfig::local_double()).expect("local_double config valid")
    }

    /// Active config (chain_id / rpc / contract hooks).
    pub fn config(&self) -> &EvmRendezvousConfig {
        &self.config
    }

    /// True when dialing remote/reference JSON-RPC (not in-process double).
    ///
    /// Does **not** mean an on-chain Polygon ledger success — see [`Self::ledger_claim`].
    pub fn is_live_json_rpc(&self) -> bool {
        matches!(self.backend, EvmBackend::JsonRpc)
    }

    /// What this provider instance actually claims (`#271` honesty).
    pub fn ledger_claim(&self) -> EvmLedgerClaim {
        if self.config.use_local_double {
            return EvmLedgerClaim::LocalMockDouble;
        }
        if self.config.is_declared_https_rpc() {
            return EvmLedgerClaim::DeclaredHttpsConfig;
        }
        EvmLedgerClaim::ReferenceHttpJsonRpc
    }

    /// Contract-facing identity hash for a presence (not EVM account).
    pub fn identity_hash_for(record: &NodePresenceRecord) -> String {
        evm_identity_hash(&record.identity_ref)
    }

    fn verify_chain_id(&self) -> Result<(), PeerError> {
        let result = json_rpc_call(&self.config.rpc_url, RPC_ETH_CHAIN_ID, json!([]))?;
        let hex = result
            .as_str()
            .ok_or_else(|| PeerError::Rendezvous("eth_chainId result not string".into()))?;
        let remote = parse_hex_u64(hex)?;
        if remote != self.config.chain_id {
            return Err(PeerError::Rendezvous(format!(
                "live EVM chain_id mismatch: config {} vs eth_chainId {remote}",
                self.config.chain_id
            )));
        }
        Ok(())
    }

    fn rpc_params_record(&self, record: &NodePresenceRecord) -> Value {
        json!({
            "contract": self.config.contract_address,
            "record": record,
            "identity_hash": Self::identity_hash_for(record),
        })
    }

    fn decode_records(v: Value) -> Result<Vec<NodePresenceRecord>, PeerError> {
        serde_json::from_value(v).map_err(|e| PeerError::Rendezvous(e.to_string()))
    }

    fn decode_optional_record(v: Value) -> Result<Option<NodePresenceRecord>, PeerError> {
        if v.is_null() {
            return Ok(None);
        }
        serde_json::from_value(v).map_err(|e| PeerError::Rendezvous(e.to_string()))
    }
}

fn parse_hex_u64(hex: &str) -> Result<u64, PeerError> {
    let s = hex.trim().trim_start_matches("0x").trim_start_matches("0X");
    u64::from_str_radix(s, 16).map_err(|e| PeerError::Rendezvous(format!("bad chain id hex: {e}")))
}

impl RendezvousProvider for EvmRendezvousProvider {
    fn publish_presence(&mut self, record: NodePresenceRecord) -> Result<(), PeerError> {
        let _ = Self::identity_hash_for(&record);
        match &mut self.backend {
            EvmBackend::Local(inner) => inner.publish_presence(record),
            EvmBackend::JsonRpc => {
                let _ = json_rpc_call(
                    &self.config.rpc_url,
                    RPC_PUBLISH,
                    self.rpc_params_record(&record),
                )?;
                Ok(())
            }
        }
    }

    fn update_presence(&mut self, record: NodePresenceRecord) -> Result<(), PeerError> {
        let _ = Self::identity_hash_for(&record);
        match &mut self.backend {
            EvmBackend::Local(inner) => inner.update_presence(record),
            EvmBackend::JsonRpc => {
                let _ = json_rpc_call(
                    &self.config.rpc_url,
                    RPC_UPDATE,
                    self.rpc_params_record(&record),
                )?;
                Ok(())
            }
        }
    }

    fn remove_or_expire_presence(
        &mut self,
        identity_ref: &str,
        as_of: &str,
        force: bool,
    ) -> Result<bool, PeerError> {
        match &mut self.backend {
            EvmBackend::Local(inner) => inner.remove_or_expire_presence(identity_ref, as_of, force),
            EvmBackend::JsonRpc => {
                let result = json_rpc_call(
                    &self.config.rpc_url,
                    RPC_REMOVE,
                    json!({
                        "contract": self.config.contract_address,
                        "identity_ref": identity_ref,
                        "as_of": as_of,
                        "force": force,
                    }),
                )?;
                Ok(result
                    .get("removed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false))
            }
        }
    }

    fn query_active_peers(&self, as_of: &str) -> Result<Vec<NodePresenceRecord>, PeerError> {
        match &self.backend {
            EvmBackend::Local(inner) => inner.query_active_peers(as_of),
            EvmBackend::JsonRpc => {
                let result = json_rpc_call(
                    &self.config.rpc_url,
                    RPC_QUERY_ACTIVE,
                    json!({
                        "contract": self.config.contract_address,
                        "as_of": as_of,
                    }),
                )?;
                Self::decode_records(result)
            }
        }
    }

    fn query_identity(&self, identity_ref: &str) -> Result<Option<NodePresenceRecord>, PeerError> {
        match &self.backend {
            EvmBackend::Local(inner) => inner.query_identity(identity_ref),
            EvmBackend::JsonRpc => {
                let result = json_rpc_call(
                    &self.config.rpc_url,
                    RPC_QUERY_IDENTITY,
                    json!({
                        "contract": self.config.contract_address,
                        "identity_ref": identity_ref,
                    }),
                )?;
                Self::decode_optional_record(result)
            }
        }
    }

    fn query_relays(&self, as_of: &str) -> Result<Vec<NodePresenceRecord>, PeerError> {
        match &self.backend {
            EvmBackend::Local(inner) => inner.query_relays(as_of),
            EvmBackend::JsonRpc => {
                let result = json_rpc_call(
                    &self.config.rpc_url,
                    RPC_QUERY_RELAYS,
                    json!({
                        "contract": self.config.contract_address,
                        "as_of": as_of,
                    }),
                )?;
                Self::decode_records(result)
            }
        }
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

    use crate::evm_rendezvous_rpc::ReferenceEvmRendezvousRpc;
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
        assert!(!evm.is_live_json_rpc());
        assert_eq!(evm.ledger_claim(), EvmLedgerClaim::LocalMockDouble);
        assert!(!evm.ledger_claim().is_on_chain_ledger());
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
    fn live_amoy_config_validates_without_stub_error() {
        let cfg =
            EvmRendezvousConfig::amoy_live(EVM_LOCAL_CONTRACT_PLACEHOLDER, "http://127.0.0.1:8545");
        cfg.validate().unwrap();
        assert!(!cfg.use_local_double);
    }

    #[test]
    fn declared_https_amoy_and_polygon_config_validate() {
        let amoy = EvmRendezvousConfig::amoy_live_declared_https(EVM_LOCAL_CONTRACT_PLACEHOLDER);
        amoy.validate().unwrap();
        assert!(amoy.is_declared_https_rpc());
        assert_eq!(amoy.rpc_url, EVM_AMOY_RPC_DEFAULT);
        let poly = EvmRendezvousConfig::polygon_live_declared_https(EVM_LOCAL_CONTRACT_PLACEHOLDER);
        poly.validate().unwrap();
        assert!(poly.rpc_url.starts_with("https://"));
    }

    #[test]
    fn rejects_live_without_http_or_https_url() {
        let mut cfg = EvmRendezvousConfig::amoy_local_double(EVM_LOCAL_CONTRACT_PLACEHOLDER);
        cfg.use_local_double = false;
        cfg.rpc_url = "aira://not-http".into();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_chain_id_mismatch() {
        let mut cfg = EvmRendezvousConfig::local_double();
        cfg.chain_id = 1;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn live_json_rpc_roundtrip_via_reference_server() {
        let dir = tempdir().unwrap();
        let (id, pk) = write_node(dir.path(), "evm-live", [77u8; 32]);
        let server = ReferenceEvmRendezvousRpc::spawn(
            EVM_CHAIN_LOCAL_DOUBLE,
            EVM_LOCAL_CONTRACT_PLACEHOLDER,
        )
        .unwrap();
        let cfg = EvmRendezvousConfig::anvil_live(EVM_LOCAL_CONTRACT_PLACEHOLDER, server.rpc_url());
        let mut evm = EvmRendezvousProvider::new(cfg).unwrap();
        assert!(evm.is_live_json_rpc());
        assert_eq!(evm.ledger_claim(), EvmLedgerClaim::ReferenceHttpJsonRpc);
        assert!(!evm.ledger_claim().is_on_chain_ledger());
        let rec = signed(dir.path(), &id, &pk, 1);
        evm.publish_presence(rec.clone()).unwrap();
        let found = evm.query_identity(id.as_str()).unwrap().unwrap();
        assert_eq!(found.identity_ref, rec.identity_ref);
        assert_eq!(
            evm.query_active_peers("2026-09-06T00:00:00Z")
                .unwrap()
                .len(),
            1
        );
        let rec2 = signed(dir.path(), &id, &pk, 2);
        evm.update_presence(rec2).unwrap();
        assert!(evm
            .remove_or_expire_presence(id.as_str(), "2026-09-06T00:00:00Z", true)
            .unwrap());
        assert!(evm.query_identity(id.as_str()).unwrap().is_none());
    }

    #[test]
    fn declared_https_live_provider_build_is_partial_not_ledger() {
        let cfg = EvmRendezvousConfig::amoy_live_declared_https(EVM_LOCAL_CONTRACT_PLACEHOLDER);
        cfg.validate().unwrap();
        assert!(cfg.is_declared_https_rpc());
        // Constructing dials eth_chainId — https path fails closed as PARTIAL (not ledger).
        let err = EvmRendezvousProvider::new(cfg).unwrap_err().to_string();
        assert!(
            err.contains("PARTIAL") || err.contains("https"),
            "https live build must not pretend on-chain success: {err}"
        );
    }
}
