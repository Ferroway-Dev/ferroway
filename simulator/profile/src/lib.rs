//! Manifest parsing and validation for Ferroway Cast.
//!
//! The primary entry point is [`Manifest`], which can be loaded from a YAML file
//! with [`Manifest::from_file`] or parsed from a YAML string via the
//! [`std::str::FromStr`] implementation (i.e. `s.parse::<Manifest>()`).

pub mod error;
pub mod manifest;

pub use error::ManifestError;
pub use manifest::{
    AffectedChannel, Channel, Correlation, CorrelationRelationship, DataType, DeviceInfo, Domain,
    DropoutConfig, Encoding, Fault, FaultBehavior, FaultOnset, FaultSeverity, FileConfig,
    FileFormat, KafkaCompression, KafkaConfig, Manifest, NoiseConfig, RecoveryConfig, Transport,
};
