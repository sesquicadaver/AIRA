use thiserror::Error;

/// Acquisition gate / quarantine errors.
#[derive(Debug, Error)]
pub enum AcquisitionError {
    #[error("io: {0}")]
    Io(String),
    #[error("artifact: {0}")]
    Artifact(String),
    #[error("crypto: {0}")]
    Crypto(String),
    #[error("schema: {0}")]
    Schema(String),
    #[error("remote source rejected (local --source only): {0}")]
    RemoteSource(String),
    #[error("source is not a file: {0}")]
    SourceNotFile(String),
    #[error("source not found: {0}")]
    SourceMissing(String),
    #[error("quarantine path outside scoped models: {0}")]
    OutsideScope(String),
    #[error("no quarantine snapshot — run download --source first")]
    NoQuarantine,
    #[error("no verified snapshot — run models verify first")]
    NoVerified,
    #[error("no activated cache for model — run models activate first")]
    NoActivated,
    #[error("model artifact missing or invalid: {0}")]
    BadArtifact(String),
    #[error("invalid share visibility (use local|opt_in): {0}")]
    BadVisibility(String),
    #[error("{0}")]
    Other(String),
}
