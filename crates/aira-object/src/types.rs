//! Common scalar types aligned with Schema Pack §3.

use std::fmt;
use std::str::FromStr;

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// Parse / validation errors for core scalars.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)] // intentional Invalid* prefix for TypeError variants
pub enum TypeError {
    #[error("invalid AIRA ref: {0}")]
    InvalidRef(String),
    #[error("invalid content hash: {0}")]
    InvalidHash(String),
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),
}

/// Canonical AIRA reference: `aira:<kind>:<id>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AiraRef(String);

impl AiraRef {
    /// Parse and validate an AIRA reference string.
    pub fn parse(s: impl AsRef<str>) -> Result<Self, TypeError> {
        let s = s.as_ref();
        // Schema: ^aira:[a-z][a-z0-9_-]*:[a-zA-Z0-9_.:-]+$
        static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        let re = RE.get_or_init(|| {
            Regex::new(r"^aira:[a-z][a-z0-9_-]*:[a-zA-Z0-9_.:-]+$").expect("ref regex")
        });
        if !re.is_match(s) {
            return Err(TypeError::InvalidRef(s.to_string()));
        }
        Ok(Self(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AiraRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for AiraRef {
    type Err = TypeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Serialize for AiraRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for AiraRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(s).map_err(serde::de::Error::custom)
    }
}

/// Content-addressed hash: `sha256:…` or `sha512:…`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentHash(String);

impl ContentHash {
    pub fn parse(s: impl AsRef<str>) -> Result<Self, TypeError> {
        let s = s.as_ref();
        static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        let re =
            RE.get_or_init(|| Regex::new(r"^(sha256|sha512):[a-fA-F0-9]+$").expect("hash regex"));
        if !re.is_match(s) {
            return Err(TypeError::InvalidHash(s.to_string()));
        }
        Ok(Self(s.to_string()))
    }

    /// Hash canonical JSON bytes with SHA-256.
    pub fn sha256_bytes(bytes: &[u8]) -> Self {
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest(bytes);
        Self(format!("sha256:{}", hex::encode(digest)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for ContentHash {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(s).map_err(serde::de::Error::custom)
    }
}

/// Ed25519 (or other) signature envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    pub algorithm: String,
    pub key_ref: AiraRef,
    pub signature_value: String,
}

/// RFC3339 / JSON Schema `date-time` timestamp (stored as string).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timestamp(String);

impl Timestamp {
    pub fn parse(s: impl AsRef<str>) -> Result<Self, TypeError> {
        let s = s.as_ref().to_string();
        // Minimal check: contains 'T' and ends with Z or offset-ish length.
        if !s.contains('T') || s.len() < 16 {
            return Err(TypeError::InvalidTimestamp(s));
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
