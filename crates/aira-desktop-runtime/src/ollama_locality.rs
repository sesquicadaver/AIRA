//! Locality predicate for Ollama models (`#371`).
//!
//! Distinguishes **server-local** (on the Ollama server, not its cloud) from
//! **this computer** (server-local **and** the configured endpoint is this
//! machine). Empty `remote_host` / `remote_model` mean server-local only when
//! the API contract is known; for an unknown version they stay Unknown.
//! Neither `localhost` in the endpoint nor a `:cloud` name suffix alone proves
//! locality.

use crate::ollama::{effective_ollama_host, OllamaJsonField, OllamaTagsModel};

/// How empty `remote_*` fields should be read for this Ollama API version (`#371`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OllamaApiLocalityContract {
    /// Supported contract: empty `remote_host` / `remote_model` ⇒ server-local.
    EmptyMeansServerLocal,
    /// Unknown / unsupported API version: empty fields stay Unknown (not local).
    Unknown,
}

/// Where a model runs relative to the Ollama server and this Desktop host (`#371`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelLocality {
    /// Not enough API evidence (or unknown empty semantics).
    Unknown,
    /// On the Ollama server, not its remote/cloud — server may still be remote to Desktop.
    ServerLocal,
    /// Remote/cloud relative to the Ollama server (`remote_host` / `remote_model` set).
    RemoteFromServer,
    /// [`ServerLocal`](Self::ServerLocal) and the configured endpoint is this machine.
    ThisComputer,
}

impl ModelLocality {
    /// Stable label for tests / technical details.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::ServerLocal => "server_local",
            Self::RemoteFromServer => "remote_from_server",
            Self::ThisComputer => "this_computer",
        }
    }

    /// True only for confirmed on-this-machine execution.
    pub fn is_this_computer(self) -> bool {
        matches!(self, Self::ThisComputer)
    }

    /// True when the model is local to the Ollama server (including this computer).
    pub fn is_server_local(self) -> bool {
        matches!(self, Self::ServerLocal | Self::ThisComputer)
    }
}

/// Classify locality from tags/show fields, endpoint, and API contract (`#371`).
///
/// - Name suffix `:cloud` is **ignored** as evidence.
/// - Endpoint `localhost` alone does **not** yield [`ModelLocality::ThisComputer`].
/// - [`OllamaApiLocalityContract::Unknown`] + empty Known fields → [`ModelLocality::Unknown`].
pub fn classify_model_locality(
    model: &OllamaTagsModel,
    endpoint: &str,
    contract: OllamaApiLocalityContract,
) -> ModelLocality {
    classify_remote_fields(
        &model.remote_host,
        &model.remote_model,
        endpoint,
        contract,
    )
}

/// Same as [`classify_model_locality`] with explicit remote fields.
pub fn classify_remote_fields(
    remote_host: &OllamaJsonField<String>,
    remote_model: &OllamaJsonField<String>,
    endpoint: &str,
    contract: OllamaApiLocalityContract,
) -> ModelLocality {
    let relative = match (remote_host, remote_model, contract) {
        (OllamaJsonField::Unknown, _, _) | (_, OllamaJsonField::Unknown, _) => {
            ModelLocality::Unknown
        }
        (OllamaJsonField::Known(h), OllamaJsonField::Known(m), _)
            if !h.trim().is_empty() || !m.trim().is_empty() =>
        {
            ModelLocality::RemoteFromServer
        }
        (OllamaJsonField::Known(_), OllamaJsonField::Known(_), OllamaApiLocalityContract::Unknown) => {
            // Both present and empty, but empty≠local for unknown API versions.
            ModelLocality::Unknown
        }
        (
            OllamaJsonField::Known(_),
            OllamaJsonField::Known(_),
            OllamaApiLocalityContract::EmptyMeansServerLocal,
        ) => ModelLocality::ServerLocal,
    };

    match relative {
        ModelLocality::ServerLocal if endpoint_is_this_machine(endpoint) => {
            ModelLocality::ThisComputer
        }
        other => other,
    }
}

/// True when the configured Ollama base points at this machine (loopback).
///
/// Used only together with server-local evidence — never alone as locality proof.
pub fn endpoint_is_this_machine(endpoint: &str) -> bool {
    let base = effective_ollama_host(Some(endpoint));
    let Some(rest) = base
        .trim()
        .trim_end_matches('/')
        .strip_prefix("http://")
        .or_else(|| {
            base.trim()
                .trim_end_matches('/')
                .strip_prefix("https://")
        })
    else {
        return false;
    };
    let host = rest
        .split('/')
        .next()
        .unwrap_or("")
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(rest.split('/').next().unwrap_or(""));
    // Strip IPv6 brackets and port.
    let host = host.strip_prefix('[').and_then(|h| h.split(']').next()).unwrap_or(host);
    let host = host.split(':').next().unwrap_or(host);
    let host = host.trim().to_ascii_lowercase();
    matches!(host.as_str(), "127.0.0.1" | "localhost" | "::1")
}

/// Name suffix heuristics are not locality evidence (`#371`).
pub fn name_suffix_proves_locality(_model_name: &str) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(
        name: &str,
        remote_host: OllamaJsonField<String>,
        remote_model: OllamaJsonField<String>,
    ) -> OllamaTagsModel {
        OllamaTagsModel {
            name: name.into(),
            digest: OllamaJsonField::Unknown,
            size: OllamaJsonField::Unknown,
            remote_host,
            remote_model,
            capabilities: OllamaJsonField::Unknown,
        }
    }

    /// `#371`: unknown API version — empty remote fields stay Unknown, not local.
    #[test]
    fn unknown_contract_empty_fields_are_unknown_not_local() {
        let m = model(
            "phi:latest",
            OllamaJsonField::Known(String::new()),
            OllamaJsonField::Known(String::new()),
        );
        let loc = classify_model_locality(
            &m,
            "http://127.0.0.1:11434",
            OllamaApiLocalityContract::Unknown,
        );
        assert_eq!(loc, ModelLocality::Unknown);
        assert_eq!(loc.as_str(), "unknown");
        assert!(!loc.is_this_computer());
        assert!(!loc.is_server_local());
    }

    /// `#371`: localhost endpoint alone does not prove this-computer locality.
    #[test]
    fn localhost_endpoint_alone_does_not_prove_this_computer() {
        let m = model(
            "phi:latest",
            OllamaJsonField::Unknown,
            OllamaJsonField::Unknown,
        );
        let loc = classify_model_locality(
            &m,
            "http://127.0.0.1:11434",
            OllamaApiLocalityContract::EmptyMeansServerLocal,
        );
        assert_eq!(loc, ModelLocality::Unknown);
        assert!(endpoint_is_this_machine("http://127.0.0.1:11434"));
        assert!(!loc.is_this_computer());
    }

    /// `#371`: `:cloud` suffix alone does not confirm remote or local.
    #[test]
    fn cloud_suffix_alone_does_not_prove_locality() {
        assert!(!name_suffix_proves_locality("kimi-k2.7-code:cloud"));
        let m = model(
            "kimi-k2.7-code:cloud",
            OllamaJsonField::Unknown,
            OllamaJsonField::Unknown,
        );
        let loc = classify_model_locality(
            &m,
            "http://127.0.0.1:11434",
            OllamaApiLocalityContract::EmptyMeansServerLocal,
        );
        assert_eq!(loc, ModelLocality::Unknown);
    }

    /// `#371`: known contract + empty remotes + loopback ⇒ this computer.
    #[test]
    fn known_contract_empty_on_loopback_is_this_computer() {
        let m = model(
            "phi:latest",
            OllamaJsonField::Known(String::new()),
            OllamaJsonField::Known(String::new()),
        );
        let loc = classify_model_locality(
            &m,
            "http://127.0.0.1:11434",
            OllamaApiLocalityContract::EmptyMeansServerLocal,
        );
        assert_eq!(loc, ModelLocality::ThisComputer);
    }

    /// `#371`: known contract + empty remotes on a remote endpoint ⇒ server-local only.
    #[test]
    fn known_contract_empty_on_remote_endpoint_is_server_local_not_this_computer() {
        let m = model(
            "phi:latest",
            OllamaJsonField::Known(String::new()),
            OllamaJsonField::Known(String::new()),
        );
        let loc = classify_model_locality(
            &m,
            "http://192.168.1.50:11434",
            OllamaApiLocalityContract::EmptyMeansServerLocal,
        );
        assert_eq!(loc, ModelLocality::ServerLocal);
        assert!(!loc.is_this_computer());
    }

    /// `#371`: non-empty remote_host ⇒ remote even on localhost Desktop.
    #[test]
    fn remote_host_set_is_remote_from_server() {
        let m = model(
            "kimi-k2.7-code:cloud",
            OllamaJsonField::Known("https://ollama.com".into()),
            OllamaJsonField::Known("kimi-k2.7-code".into()),
        );
        let loc = classify_model_locality(
            &m,
            "http://127.0.0.1:11434",
            OllamaApiLocalityContract::EmptyMeansServerLocal,
        );
        assert_eq!(loc, ModelLocality::RemoteFromServer);
    }
}
