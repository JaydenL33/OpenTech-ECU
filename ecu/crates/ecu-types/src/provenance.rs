use alloc::string::String;
use serde::{Deserialize, Serialize};

/// Where a calibration or recommendation came from. This is a first-class
/// field, not prose, because ignition and fuel data have different evidence
/// requirements.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    /// A named, licensed, engine-specific reference with an identifiable owner.
    VerifiedReference,
    /// Manufacturer-published data, not independently verified on this engine.
    ManufacturerPublished,
    /// Engineering estimate derived from documented assumptions. Never safe.
    EngineeringEstimate,
    /// Synthetic data generated for simulation/testing. Never engine-safe.
    SyntheticSimulationOnly,
    /// No credible source. Callers must not invent values.
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub source: SourceKind,
    pub citation: String,
    pub licence: String,
    pub verified_by: String,
    pub verification_date: String,
    /// True only when the source is an engine-specific verified reference.
    pub engine_specific_verified: bool,
    pub notes: String,
}

impl Provenance {
    pub fn synthetic_simulation(reason: &str) -> Self {
        Self {
            source: SourceKind::SyntheticSimulationOnly,
            citation: "OpenTech ECU synthetic simulator template".into(),
            licence: "GPL-2.0-or-later".into(),
            verified_by: String::new(),
            verification_date: String::new(),
            engine_specific_verified: false,
            notes: reason.into(),
        }
    }

    pub fn unknown(reason: &str) -> Self {
        Self {
            source: SourceKind::Unknown,
            citation: String::new(),
            licence: String::new(),
            verified_by: String::new(),
            verification_date: String::new(),
            engine_specific_verified: false,
            notes: reason.into(),
        }
    }

    /// A calibration is only eligible for physical use when this returns true.
    /// Fuel and ignition require their own verified sources in addition.
    pub fn is_physical_reference(&self) -> bool {
        self.source == SourceKind::VerifiedReference && self.engine_specific_verified
    }
}

impl Default for Provenance {
    fn default() -> Self {
        Self::unknown("no provenance recorded")
    }
}
