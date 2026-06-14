//! Error types for manifest parsing and validation.

use thiserror::Error;

/// Errors that can occur when loading or validating a Ferroway Manifest.
#[derive(Debug, Error)]
pub enum ManifestError {
    /// The manifest file could not be read from disk.
    #[error("failed to read manifest file: {0}")]
    FileRead(#[from] std::io::Error),

    /// The manifest YAML could not be parsed into the expected structure.
    #[error("failed to parse manifest YAML: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    /// A correlation references a driver channel ID that is not declared
    /// among the manifest's Tier-1 channels.
    #[error("correlation '{correlation}' has undeclared driver channel '{driver}'")]
    InvalidCorrelationDriver {
        /// Name of the offending correlation.
        correlation: String,
        /// The channel ID that was referenced but not declared.
        driver: String,
    },

    /// A correlation references an influenced channel ID that is not declared
    /// among the manifest's Tier-1 channels.
    #[error("correlation '{correlation}' has undeclared influenced channel '{channel}'")]
    InvalidCorrelationInfluenced {
        /// Name of the offending correlation.
        correlation: String,
        /// The channel ID that was referenced but not declared.
        channel: String,
    },
}
