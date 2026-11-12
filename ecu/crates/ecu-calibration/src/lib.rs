//! Interpolation, table editing, calibration documents, starter templates and
//! preflight validation.
//!
//! `Table` supports ranks 1..=4. The first axis is the slowest-varying index
//! (row axis), matching a conventional fuel map with load rows and RPM columns.
//! Interpolation clamps at the axis ends; it never extrapolates.
//!
//! Calibration documents are immutable-by-convention snapshots. Editing
//! produces a new revision; `seal` computes a SHA-256 over canonical JSON with
//! the checksum field blank.

#![forbid(unsafe_code)]

pub mod preflight;
pub mod table;
pub mod vehicle;
pub mod wizard;

pub use preflight::*;
pub use table::*;
pub use vehicle::*;
pub use wizard::*;

use ecu_types::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::string::{String, ToString};
use std::vec::Vec;

pub const CALIBRATION_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Calibration {
    pub schema_version: u16,
    pub id: String,
    pub revision: u32,
    pub parent: Option<String>,
    pub created_unix_ms: u64,
    pub engine: EngineConfig,
    pub fuel: FuelConfig,
    pub sensors: SensorSetup,
    pub io: IoConfig,
    pub tables: BTreeMap<String, Table>,
    pub ignition_policy: IgnitionPolicy,
    pub notes: Vec<String>,
    /// SHA-256 of the document with this field set to the empty string.
    pub checksum: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IgnitionPolicy {
    /// No verified, engine-specific timing source exists. A synthetic table may
    /// exist for simulation, but physical hardware exports are blocked.
    SyntheticSimulationOnly { warning: String, table_key: String },
    /// A licensed, engine-specific reference was supplied. Still requires
    /// physical timing-light verification before first start.
    VerifiedReference {
        citation: String,
        licence: String,
        verified_by: String,
        timing_light_required: bool,
    },
    /// No table and no source. The ECU must not invent timing.
    InsufficientEvidence { reason: String },
}

impl IgnitionPolicy {
    pub fn hardware_eligible(&self) -> bool {
        matches!(self, IgnitionPolicy::VerifiedReference { .. })
    }

    pub fn warning(&self) -> Option<&str> {
        match self {
            IgnitionPolicy::SyntheticSimulationOnly { warning, .. } => Some(warning),
            IgnitionPolicy::InsufficientEvidence { reason } => Some(reason),
            IgnitionPolicy::VerifiedReference {
                timing_light_required,
                ..
            } => {
                if *timing_light_required {
                    Some("Verified timing reference still requires timing-light synchronisation.")
                } else {
                    None
                }
            }
        }
    }
}

impl Calibration {
    /// Compute and store the document checksum. Safe to call repeatedly.
    pub fn seal(&mut self) {
        let mut copy = self.clone();
        copy.checksum = String::new();
        let bytes = serde_json::to_vec(&copy).expect("calibration serialisation");
        self.checksum = hex_sha256(&bytes);
    }

    pub fn verify(&self) -> bool {
        let mut copy = self.clone();
        copy.checksum = String::new();
        let bytes = serde_json::to_vec(&copy).expect("calibration serialisation");
        hex_sha256(&bytes) == self.checksum
    }

    pub fn table(&self, key: &str) -> Result<&Table, CalibrationError> {
        self.tables
            .get(key)
            .ok_or_else(|| CalibrationError::MissingTable(key.to_string()))
    }

    pub fn table_mut(&mut self, key: &str) -> Result<&mut Table, CalibrationError> {
        self.tables
            .get_mut(key)
            .ok_or_else(|| CalibrationError::MissingTable(key.to_string()))
    }

    /// Create a child revision with a fresh id and parent pointer. The caller
    /// must seal it after edits; this function seals immediately so an unsigned
    /// document can never be saved by accident.
    pub fn fork(&self, next_revision: u32, now_unix_ms: u64) -> Self {
        let mut next = self.clone();
        next.parent = Some(self.id.clone());
        next.revision = next_revision;
        next.id = format!(
            "cal-{}-{}",
            next_revision,
            &hex_sha256(&serde_json::to_vec(self).unwrap_or_default())[..12]
        );
        next.created_unix_ms = now_unix_ms;
        next.seal();
        next
    }

    pub fn validate(&self) -> Vec<ValidationIssue> {
        let mut issues = self.engine.validate();
        issues.extend(validate_fuel_config(&self.fuel));
        if self.schema_version != CALIBRATION_SCHEMA_VERSION {
            issues.push(ValidationIssue {
                code: "CAL_SCHEMA".into(),
                field: "schema_version".into(),
                message: "Unsupported calibration schema version".into(),
                severity: Severity::Error,
            });
        }
        if self.tables.is_empty() {
            issues.push(ValidationIssue {
                code: "CAL_TABLES_EMPTY".into(),
                field: "tables".into(),
                message: "Calibration has no tables".into(),
                severity: Severity::Error,
            });
        }
        for (key, table) in &self.tables {
            for i in table.validate() {
                issues.push(ValidationIssue {
                    code: format!("TABLE_{}", i.code),
                    field: format!("tables.{key}.{}", i.field),
                    message: i.message,
                    severity: i.severity,
                });
            }
        }
        if !self.verify() {
            issues.push(ValidationIssue {
                code: "CAL_CHECKSUM".into(),
                field: "checksum".into(),
                message: "Calibration checksum does not match content".into(),
                severity: Severity::Error,
            });
        }
        for map in ["ve", "target_lambda", "ignition"] {
            if !self.tables.contains_key(map) {
                issues.push(ValidationIssue {
                    code: "CAL_REQUIRED_TABLE".into(),
                    field: format!("tables.{map}"),
                    message: format!("Required table '{map}' is missing"),
                    severity: Severity::Error,
                });
            }
        }
        issues
    }

    /// Replace a table's values after validating shape and bounds. Returns a
    /// sealed child revision. Never mutates the input snapshot.
    pub fn with_table_values(
        &self,
        key: &str,
        values: Vec<f32>,
        now_unix_ms: u64,
    ) -> Result<Self, CalibrationError> {
        let mut next = self.fork(self.revision + 1, now_unix_ms);
        let table = next.table_mut(key)?;
        table.replace_values(values)?;
        table.revision += 1;
        next.seal();
        Ok(next)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CalibrationError {
    MissingTable(String),
    InvalidValues(String),
}

impl core::fmt::Display for CalibrationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CalibrationError::MissingTable(k) => write!(f, "missing calibration table: {k}"),
            CalibrationError::InvalidValues(m) => write!(f, "invalid table values: {m}"),
        }
    }
}

impl std::error::Error for CalibrationError {}

pub fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

pub fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_cal() -> Calibration {
        wizard::generate_starter(&wizard::WizardRequest::default_inline4(), 1).unwrap()
    }

    #[test]
    fn checksum_detects_tampering() {
        let mut c = simple_cal();
        assert!(c.verify());
        c.engine.rev_limit = 7600;
        assert!(!c.verify());
        c.seal();
        assert!(c.verify());
    }

    #[test]
    fn edit_produces_new_revision_and_preserves_parent() {
        let c = simple_cal();
        let t = c.table("ve").unwrap();
        let mut values = t.values.clone();
        values[0] += 2.0;
        let next = c
            .with_table_values("ve", values.clone(), now_unix_ms())
            .unwrap();
        assert_eq!(next.revision, c.revision + 1);
        assert_eq!(next.parent.as_deref(), Some(c.id.as_str()));
        assert_eq!(next.table("ve").unwrap().values[0], values[0]);
        assert_ne!(next.checksum, c.checksum);
        assert!(next.verify());
    }

    #[test]
    fn calibration_validation_flags_bad_checksum() {
        let mut c = simple_cal();
        c.checksum = "deadbeef".into();
        assert!(c.validate().iter().any(|i| i.code == "CAL_CHECKSUM"));
    }

    #[test]
    fn starter_tables_are_marked_estimates() {
        let c = simple_cal();
        assert_eq!(
            c.table("ve").unwrap().provenance.source,
            SourceKind::EngineeringEstimate
        );
        assert!(matches!(
            c.ignition_policy,
            IgnitionPolicy::SyntheticSimulationOnly { .. }
        ));
        assert!(!c.ignition_policy.hardware_eligible());
    }
}
