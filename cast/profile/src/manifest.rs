//! Typed Rust structs for the Ferroway Manifest YAML schema.

use std::collections::HashSet;
use std::path::Path;
use std::str::FromStr;

use serde::Deserialize;

use crate::error::ManifestError;

// ── Top-level ─────────────────────────────────────────────────────────────────

/// Top-level Ferroway device manifest.
///
/// Describes a single simulated device: its identity, raw sensor channels,
/// inter-channel correlations, injectable faults, and transport configuration.
/// Load with [`Manifest::from_file`] or via `s.parse::<Manifest>()`.
#[derive(Debug, Deserialize)]
pub struct Manifest {
    /// Device identity and metadata.
    pub device: DeviceInfo,
    /// Raw sensor channels (Tier 1).
    pub channels: Vec<Channel>,
    /// Inter-channel correlation relationships.
    #[serde(default)]
    pub correlations: Vec<Correlation>,
    /// Named injectable fault modes.
    #[serde(default)]
    pub faults: Vec<Fault>,
    /// Wire encoding and Kafka/file transport configuration.
    pub transport: Transport,
}

impl Manifest {
    /// Load and validate a manifest from a YAML file on disk.
    ///
    /// Reads the file, parses YAML, then calls [`Self::validate`].
    /// Returns the first error encountered, if any.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    /// Validate tier-separation rules after parsing.
    ///
    /// Ensures every correlation `driver` and `influenced` ID is declared as a
    /// Tier-1 channel in this manifest. Returns the first violation found as
    /// [`ManifestError::InvalidCorrelationDriver`] or
    /// [`ManifestError::InvalidCorrelationInfluenced`].
    pub fn validate(&self) -> Result<(), ManifestError> {
        let channel_ids: HashSet<&str> = self.channels.iter().map(|c| c.id.as_str()).collect();

        for corr in &self.correlations {
            if !channel_ids.contains(corr.driver.as_str()) {
                return Err(ManifestError::InvalidCorrelationDriver {
                    correlation: corr.name.clone(),
                    driver: corr.driver.clone(),
                });
            }
            if !channel_ids.contains(corr.influenced.as_str()) {
                return Err(ManifestError::InvalidCorrelationInfluenced {
                    correlation: corr.name.clone(),
                    channel: corr.influenced.clone(),
                });
            }
        }

        Ok(())
    }
}

impl FromStr for Manifest {
    type Err = ManifestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let manifest: Self = serde_yaml::from_str(s)?;
        manifest.validate()?;
        Ok(manifest)
    }
}

// ── Device ────────────────────────────────────────────────────────────────────

/// Identity and metadata for the simulated device.
#[derive(Debug, Deserialize)]
pub struct DeviceInfo {
    /// Unique profile identifier used in Kafka topic names and telemetry messages.
    pub profile_id: String,
    /// Human-readable display name.
    pub name: String,
    /// Industry domain this device belongs to.
    pub domain: Domain,
    /// Asset classification within the domain (e.g. `"construction-equipment"`).
    pub asset_class: String,
    /// Firmware version string.
    pub firmware_version: String,
    /// Manufacturer name.
    pub manufacturer: String,
    /// Optional extended description.
    pub description: Option<String>,
}

/// Industry domain of the simulated device.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    /// Road and off-road vehicles, construction equipment.
    Vehicular,
    /// Agricultural machinery.
    Agricultural,
    /// Aviation and space systems.
    Aerospace,
    /// Industrial automation and process control.
    Industrial,
}

// ── Channels ──────────────────────────────────────────────────────────────────

/// A raw Tier-1 sensor channel declaration.
#[derive(Debug, Deserialize)]
pub struct Channel {
    /// Unique channel identifier used in Kafka topic names.
    pub id: String,
    /// Human-readable channel name.
    pub name: String,
    /// Wire type used in the `TelemetryMessage` oneof value field.
    pub data_type: DataType,
    /// Physical unit label (e.g. `"rpm"`, `"celsius"`, `"bar"`).
    pub unit: String,
    /// `[min, max]` physical range the sensor can produce.
    pub range: [f64; 2],
    /// `[low, high]` bounds of the healthy operating range.
    pub nominal: [f64; 2],
    /// Steady-state idle value used as the baseline in correlation equations.
    pub idle: f64,
    /// Target emission frequency in Hz.
    pub sample_rate_hz: u32,
    /// Noise profile applied during simulation.
    pub noise: NoiseConfig,
    /// Optional human-readable description.
    pub description: Option<String>,
}

/// Precision and storage type for a channel's value.
///
/// Determines which `oneof` field is populated in `TelemetryMessage`.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    /// 32-bit IEEE float — populates `value_f32`.
    Float32,
    /// 64-bit IEEE float — populates `value_f64`.
    Float64,
    /// 16-bit signed integer — populates `value_i32`.
    Int16,
    /// 32-bit signed integer — populates `value_i32`.
    Int32,
    /// 64-bit signed integer — populates `value_i64`.
    Int64,
    /// 16-bit unsigned integer — populates `value_u32`.
    Uint16,
    /// 32-bit unsigned integer — populates `value_u32`.
    Uint32,
}

/// Noise model applied to a simulated channel.
///
/// Discriminated by the `profile` YAML key.
#[derive(Debug, Deserialize)]
#[serde(tag = "profile", rename_all = "snake_case")]
pub enum NoiseConfig {
    /// White Gaussian noise with a fixed standard deviation.
    Gaussian {
        /// Standard deviation of the noise distribution.
        std_dev: f64,
    },
    /// Slow random walk (Brownian drift) with optional per-sample noise.
    Drift {
        /// Mean rate of drift per sample.
        drift_rate: f64,
        /// Standard deviation of the per-sample noise component.
        std_dev: f64,
    },
    /// Discrete step changes — parameters TBD in Phase 2.
    Stepped,
    /// No noise — value tracks the base signal exactly.
    None,
}

// ── Correlations ──────────────────────────────────────────────────────────────

/// An inter-channel correlation linking a Tier-1 driver to an influenced channel.
///
/// The `driver` and `influenced` IDs must both be declared Tier-1 channel IDs.
/// Validated at parse time by [`Manifest::validate`].
#[derive(Debug, Deserialize)]
pub struct Correlation {
    /// Human-readable correlation name used in error messages and logs.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Tier-1 channel ID whose value drives the influenced channel.
    pub driver: String,
    /// Channel ID whose simulated value is modified by this correlation.
    pub influenced: String,
    /// Scaling factor applied in the relationship equation.
    pub coefficient: f64,
    /// Mathematical form of the relationship.
    ///
    /// Also carries any variant-specific fields (e.g. `lag_seconds`).
    #[serde(flatten)]
    pub relationship: CorrelationRelationship,
}

/// Mathematical form of a correlation relationship.
///
/// Discriminated by the `relationship` YAML key.
#[derive(Debug, Deserialize)]
#[serde(tag = "relationship", rename_all = "snake_case")]
pub enum CorrelationRelationship {
    /// `influenced(t) = idle + coefficient * driver(t)`
    Linear,
    /// `influenced(t) = idle + coefficient * driver(t − lag_seconds)`
    LaggedLinear {
        /// Seconds by which the driver's effect is delayed.
        lag_seconds: f64,
    },
    /// `influenced(t) = idle + coefficient * √driver(t)`
    Sqrt,
}

// ── Faults ────────────────────────────────────────────────────────────────────

/// A named injectable fault mode referenced by scenario files.
#[derive(Debug, Deserialize)]
pub struct Fault {
    /// Unique fault identifier referenced by scenario files.
    pub id: String,
    /// Human-readable fault name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Channels affected by this fault and the behavior applied to each.
    pub affected_channels: Vec<AffectedChannel>,
    /// Activation timing model.
    pub onset: FaultOnset,
    /// Whether the fault resolves on its own without operator intervention.
    pub recoverable: Option<bool>,
    /// For recoverable faults: seconds until automatic recovery.
    pub recovery_seconds: Option<f64>,
    /// Dropout timing parameters; present when any channel uses `behavior: dropout`.
    pub dropout: Option<DropoutConfig>,
    /// Recovery model; present when the fault uses the `recurring` onset type.
    pub recovery: Option<RecoveryConfig>,
    /// Operational impact classification.
    pub severity: FaultSeverity,
}

/// A single channel affected by a fault, together with the behavior to apply.
#[derive(Debug, Deserialize)]
pub struct AffectedChannel {
    /// ID of the channel this behavior is applied to.
    pub channel: String,
    /// Effect applied to the channel's emitted value.
    ///
    /// Carries the discriminant key `behavior` and any variant-specific fields.
    #[serde(flatten)]
    pub behavior: FaultBehavior,
}

/// The effect a fault has on an affected channel's emitted value.
///
/// Discriminated by the `behavior` YAML key.
#[derive(Debug, Deserialize)]
#[serde(tag = "behavior", rename_all = "snake_case")]
pub enum FaultBehavior {
    /// Value climbs at `rate_per_second` until it reaches `ceiling`.
    GradualRise {
        /// Rate of increase in engineering units per second.
        rate_per_second: f64,
        /// Upper bound; the value will not exceed this.
        ceiling: f64,
    },
    /// Value falls at `rate_per_second` until it reaches `floor`.
    GradualDrop {
        /// Rate of decrease in engineering units per second.
        rate_per_second: f64,
        /// Lower bound; the value will not fall below this.
        floor: f64,
    },
    /// Value is immediately offset by `magnitude`.
    StepIncrease {
        /// Absolute offset added to the channel's current value.
        magnitude: f64,
    },
    /// Channel stops emitting for the duration defined in the fault's `dropout` config.
    Dropout,
}

/// Timing model for when a fault activates after injection.
///
/// Discriminated by the `type` YAML key.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FaultOnset {
    /// Fault activates after a fixed delay from the moment it is injected.
    Delayed {
        /// Seconds from injection to activation.
        delay_seconds: f64,
    },
    /// Fault activates at the moment it is injected.
    Immediate,
    /// Fault activates within a random window and then recurs at a fixed interval.
    Recurring {
        /// `[earliest_seconds, latest_seconds]` window for the first occurrence.
        first_occurrence_window_seconds: [f64; 2],
    },
}

/// Timing configuration for a channel in dropout fault mode.
#[derive(Debug, Deserialize)]
pub struct DropoutConfig {
    /// How long the channel stays silent per dropout event, in seconds.
    pub duration_seconds: f64,
    /// Seconds between the end of one dropout and the start of the next.
    pub interval_seconds: f64,
}

/// Recovery model for a fault channel that self-resolves after each dropout.
///
/// Discriminated by the `type` YAML key.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RecoveryConfig {
    /// Channel recovers automatically after each dropout duration elapses.
    Automatic,
    /// Channel requires operator intervention to recover.
    Manual,
}

/// Operational impact classification of a fault.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FaultSeverity {
    /// Informational — no operational impact expected.
    Low,
    /// Degraded operation — monitoring recommended.
    Medium,
    /// Significant impact — immediate attention recommended.
    High,
    /// Safety-critical — immediate shutdown may be required.
    Critical,
}

// ── Transport ─────────────────────────────────────────────────────────────────

/// Wire encoding and message transport configuration.
#[derive(Debug, Deserialize)]
pub struct Transport {
    /// Default encoding applied to all channel messages.
    pub default_encoding: Encoding,
    /// Kafka producer configuration.
    pub kafka: KafkaConfig,
    /// Local file sink configuration.
    pub file: FileConfig,
}

/// Wire encoding format for telemetry messages.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Encoding {
    /// JSON encoding (Phase 1).
    Json,
    /// Avro with AWS Glue Schema Registry (Phase 2).
    Avro,
}

/// Kafka producer and topic configuration.
#[derive(Debug, Deserialize)]
pub struct KafkaConfig {
    /// Comma-separated `host:port` list of Kafka bootstrap servers.
    pub bootstrap_servers: String,
    /// Topic name template; placeholders: `{domain}`, `{profile_id}`, `{channel_id}`.
    pub topic_pattern: String,
    /// Compression algorithm applied to produced messages.
    pub compression: KafkaCompression,
    /// Producer acknowledgement level (`"all"`, `"1"`, or `"0"`).
    pub acks: String,
}

/// Compression algorithm for Kafka producer messages.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KafkaCompression {
    /// No compression.
    None,
    /// Gzip compression.
    Gzip,
    /// Snappy compression.
    Snappy,
    /// LZ4 compression.
    Lz4,
    /// Zstandard compression.
    Zstd,
}

/// Local file sink configuration.
#[derive(Debug, Deserialize)]
pub struct FileConfig {
    /// Directory path where output files are written.
    pub path: String,
    /// Output file format.
    pub format: FileFormat,
    /// Rotate to a new output file every N seconds; `None` disables rotation.
    pub rotation_seconds: Option<u64>,
}

/// Output file format for the local file sink.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FileFormat {
    /// Newline-delimited JSON — one JSON object per line.
    Jsonl,
    /// Comma-separated values.
    Csv,
    /// Apache Parquet columnar format.
    Parquet,
}
