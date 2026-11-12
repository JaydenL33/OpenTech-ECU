//! Ignition timing calculation, dwell limiting and evidence-gated
//! recommendations.
//!
//! **Lambda is not an ignition input.** There is intentionally no lambda
//! parameter anywhere in this crate. A wideband sensor measures fuel/oxygen
//! consequences; it cannot establish MBT, knock margin or safe spark advance.
//!
//! Recommendation policy:
//! - no verified engine-specific reference -> `InsufficientEvidence`;
//! - a synthetic simulator surface -> `SimulationOnly`, never physical;
//! - a licensed reference -> resampling is permitted offline, still requiring
//!   timing-light verification and knock instrumentation.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use ecu_types::Provenance;
use serde::{Deserialize, Serialize};

pub const MIN_ADVANCE_DEG: f32 = -10.0;
pub const MAX_ADVANCE_DEG: f32 = 55.0;
pub const MIN_DWELL_MS: f32 = 0.5;
pub const MAX_DWELL_MS: f32 = 12.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct IgnitionOutput {
    pub advance_deg: f32,
    pub dwell_ms: f32,
    /// Spark cut, not retard. Used by rev limiter and protection.
    pub spark_cut: bool,
    /// Additional degrees removed by limiter/protection, for logging.
    pub retard_deg: f32,
    pub limiter_active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IgnitionInhibit {
    InvalidRpm,
    InvalidLoad,
    InvalidAdvance,
    InvalidDwell,
    DwellDutyExceeded,
}

impl fmt::Display for IgnitionInhibit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            IgnitionInhibit::InvalidRpm => "invalid RPM",
            IgnitionInhibit::InvalidLoad => "invalid load",
            IgnitionInhibit::InvalidAdvance => "invalid base advance",
            IgnitionInhibit::InvalidDwell => "invalid dwell",
            IgnitionInhibit::DwellDutyExceeded => "dwell duty exceeded",
        };
        f.write_str(s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RevLimiter {
    pub limit_rpm: f32,
    pub hysteresis_rpm: f32,
    pub cut: bool,
}

impl RevLimiter {
    pub fn new(limit_rpm: f32, hysteresis_rpm: f32) -> Self {
        Self {
            limit_rpm,
            hysteresis_rpm: hysteresis_rpm.max(10.0),
            cut: false,
        }
    }

    /// Hysteresis prevents spark chatter at the limit. Fuel cut is handled by
    /// the protection/fuel layer; spark cut alone is enough to arrest RPM.
    pub fn update(&mut self, rpm: f32) -> bool {
        if rpm >= self.limit_rpm {
            self.cut = true;
        } else if rpm <= self.limit_rpm - self.hysteresis_rpm {
            self.cut = false;
        }
        self.cut
    }
}

/// Coolant and IAT corrections are bounded and additive. They must come from a
/// reviewed calibration, not generated guesses.
pub fn corrected_advance(
    base_advance_deg: f32,
    coolant_correction_deg: f32,
    iat_correction_deg: f32,
    protection_retard_deg: f32,
) -> Result<f32, IgnitionInhibit> {
    if !base_advance_deg.is_finite() {
        return Err(IgnitionInhibit::InvalidAdvance);
    }
    let correction = coolant_correction_deg.clamp(-15.0, 15.0)
        + iat_correction_deg.clamp(-15.0, 15.0)
        + protection_retard_deg.clamp(0.0, 30.0);
    let advance = (base_advance_deg + correction).clamp(MIN_ADVANCE_DEG, MAX_ADVANCE_DEG);
    Ok(advance)
}

/// Limit dwell so a single coil is not charged for an implausible fraction of
/// the available cylinder period. `outputs_per_cylinder` is 1 for sequential
/// coils. Dwell timing is a hardware property, not a tuning preference.
pub fn limit_dwell(
    rpm: f32,
    requested_dwell_ms: f32,
    outputs_per_cylinder: u8,
    cylinders: u8,
) -> Result<f32, IgnitionInhibit> {
    if !(1.0..=20_000.0).contains(&rpm) || !rpm.is_finite() {
        return Err(IgnitionInhibit::InvalidRpm);
    }
    if !(MIN_DWELL_MS..=MAX_DWELL_MS).contains(&requested_dwell_ms)
        || !requested_dwell_ms.is_finite()
    {
        return Err(IgnitionInhibit::InvalidDwell);
    }
    if cylinders == 0 || outputs_per_cylinder == 0 {
        return Err(IgnitionInhibit::DwellDutyExceeded);
    }
    // Time between this output's sparks = one engine cycle / number of outputs
    // sharing it. For sequential 4-cyl, 720 deg / 4 = 180 deg of crank.
    let cycle_ms = 120_000.0 / rpm;
    let available_ms = cycle_ms * outputs_per_cylinder as f32 / cylinders as f32;
    let max_dwell = available_ms * 0.45; // never exceed 45% coil charge duty
    if requested_dwell_ms > max_dwell {
        // Clamp rather than inhibit: insufficient dwell is a real rev limit
        // condition; not charging the coil at all would cause a misfire.
        Ok(max_dwell.clamp(MIN_DWELL_MS, MAX_DWELL_MS))
    } else {
        Ok(requested_dwell_ms)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    InsufficientEvidence,
    SimulationOnly,
    VerifiedReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimingReference {
    pub provenance: Provenance,
    pub advance_deg: f32,
    pub rpm: f32,
    pub map_kpa: f32,
    pub compression_ratio: f32,
    pub fuel: String,
    pub aspiration: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IgnitionRecommendation {
    pub status: EvidenceStatus,
    /// `None` when evidence is insufficient. Never populated from lambda.
    pub proposed_advance_deg: Option<f32>,
    pub lower_bound_deg: Option<f32>,
    pub upper_bound_deg: Option<f32>,
    pub confidence: f32,
    pub reasons: Vec<String>,
    pub required_validation: Vec<String>,
}

/// Produce an offline recommendation from a verified reference only.
/// Deliberately does not accept lambda, fuel trim, AFR or wideband state.
pub fn recommend_from_reference(
    reference: Option<&TimingReference>,
    target_rpm: f32,
    target_map_kpa: f32,
    engine_compression_ratio: f32,
    engine_fuel: &str,
    engine_aspiration: &str,
) -> IgnitionRecommendation {
    let mut required = vec![
        "Verify base ignition offset with a timing light".into(),
        "Use calibrated knock detection and audible knock review".into(),
        "Confirm fuel octane, compression ratio, boost and thermal conditions match the reference"
            .into(),
        "Supervise changes on a load-bearing dyno; never advance unattended".into(),
    ];
    let Some(r) = reference else {
        return IgnitionRecommendation {
            status: EvidenceStatus::InsufficientEvidence,
            proposed_advance_deg: None,
            lower_bound_deg: None,
            upper_bound_deg: None,
            confidence: 0.0,
            reasons: vec![
                "No licensed, engine-specific verified timing reference was supplied.".into(),
                "Lambda and fuel-trim data are not evidence of safe ignition timing.".into(),
            ],
            required_validation: required,
        };
    };
    if !r.provenance.is_physical_reference() {
        return IgnitionRecommendation {
            status: EvidenceStatus::InsufficientEvidence,
            proposed_advance_deg: None,
            lower_bound_deg: None,
            upper_bound_deg: None,
            confidence: 0.0,
            reasons: vec![
                "The supplied timing reference is not a verified, engine-specific source.".into(),
            ],
            required_validation: required,
        };
    }

    let mut reasons = Vec::new();
    let mut confidence: f32 = 0.8;
    if (r.compression_ratio - engine_compression_ratio).abs() > 0.4 {
        confidence -= 0.25;
        reasons.push(format!(
            "Compression ratio differs: reference {:.1}, engine {:.1}",
            r.compression_ratio, engine_compression_ratio
        ));
    }
    if !r.fuel.eq_ignore_ascii_case(engine_fuel) {
        confidence -= 0.2;
        reasons.push(format!(
            "Fuel differs: reference '{}', engine '{}'",
            r.fuel, engine_fuel
        ));
    }
    if !r.aspiration.eq_ignore_ascii_case(engine_aspiration) {
        confidence -= 0.25;
        reasons.push(format!(
            "Aspiration differs: reference '{}', engine '{}'",
            r.aspiration, engine_aspiration
        ));
    }
    if (target_rpm - r.rpm).abs() > 500.0 || (target_map_kpa - r.map_kpa).abs() > 10.0 {
        required
            .push("Reference point is distant; interpolation uncertainty must be reviewed".into());
        confidence -= 0.1;
    }
    confidence = confidence.clamp(0.0, 1.0);
    let proposed = r.advance_deg.clamp(MIN_ADVANCE_DEG, MAX_ADVANCE_DEG);
    // Uncertainty is evidence divergence, not a safety margin guarantee. These
    // are review bounds, not a promise that timing is knock-free.
    let uncertainty = 2.0 + (1.0 - confidence) * 8.0;
    IgnitionRecommendation {
        status: EvidenceStatus::VerifiedReference,
        proposed_advance_deg: Some(proposed),
        lower_bound_deg: Some((proposed - uncertainty).max(MIN_ADVANCE_DEG)),
        upper_bound_deg: Some((proposed + uncertainty).min(MAX_ADVANCE_DEG)),
        confidence,
        reasons,
        required_validation: required,
    }
}

/// Synthetic simulator-only recommendation. This exists to make the virtual ECU
/// run; it is never exported as an engine tune.
pub fn simulation_only_recommendation(advance_deg: f32) -> IgnitionRecommendation {
    IgnitionRecommendation {
        status: EvidenceStatus::SimulationOnly,
        proposed_advance_deg: Some(advance_deg.clamp(MIN_ADVANCE_DEG, MAX_ADVANCE_DEG)),
        lower_bound_deg: None,
        upper_bound_deg: None,
        confidence: 0.0,
        reasons: vec![
            "Synthetic simulator surface. No engine-specific evidence.".into(),
            "Must not be written to a physical ignition output.".into(),
        ],
        required_validation: vec![
            "A licensed engine-specific ignition reference is required".into(),
            "Physical timing-light synchronisation and knock review are required".into(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecu_types::{Provenance, SourceKind};

    fn verified() -> TimingReference {
        TimingReference {
            provenance: Provenance {
                source: SourceKind::VerifiedReference,
                citation: "Test reference".into(),
                licence: "Used with permission".into(),
                verified_by: "Dyno operator".into(),
                verification_date: "2026-01-01".into(),
                engine_specific_verified: true,
                notes: String::new(),
            },
            advance_deg: 18.0,
            rpm: 4000.0,
            map_kpa: 100.0,
            compression_ratio: 9.5,
            fuel: "98 RON".into(),
            aspiration: "naturally_aspirated".into(),
        }
    }

    #[test]
    fn missing_reference_returns_insufficient_evidence_not_a_guess() {
        let r = recommend_from_reference(None, 4000.0, 100.0, 9.5, "98 RON", "naturally_aspirated");
        assert_eq!(r.status, EvidenceStatus::InsufficientEvidence);
        assert_eq!(r.proposed_advance_deg, None);
        assert!(r.confidence == 0.0);
        assert!(r.reasons.iter().any(|s| s.contains("Lambda")));
    }

    #[test]
    fn synthetic_or_unknown_reference_is_rejected() {
        let mut t = verified();
        t.provenance = Provenance::synthetic_simulation("not evidence");
        let r = recommend_from_reference(
            Some(&t),
            4000.0,
            100.0,
            9.5,
            "98 RON",
            "naturally_aspirated",
        );
        assert_eq!(r.status, EvidenceStatus::InsufficientEvidence);
        assert!(r.proposed_advance_deg.is_none());
    }

    #[test]
    fn verified_reference_produces_bounded_offline_recommendation() {
        let r = recommend_from_reference(
            Some(&verified()),
            4000.0,
            100.0,
            9.5,
            "98 RON",
            "naturally_aspirated",
        );
        assert_eq!(r.status, EvidenceStatus::VerifiedReference);
        assert!((r.proposed_advance_deg.unwrap() - 18.0).abs() < 1e-5);
        assert!(r.lower_bound_deg.unwrap() <= 18.0 && r.upper_bound_deg.unwrap() >= 18.0);
        assert!(r.confidence > 0.5);
    }

    #[test]
    fn mismatched_engine_lowers_confidence_and_explains() {
        let r = recommend_from_reference(
            Some(&verified()),
            4000.0,
            180.0,
            11.0,
            "E85",
            "turbocharged",
        );
        assert!(r.confidence < 0.5);
        assert!(r.reasons.len() >= 3);
    }

    #[test]
    fn rev_limiter_has_hysteresis() {
        let mut lim = RevLimiter::new(7000.0, 100.0);
        assert!(!lim.update(6900.0));
        assert!(lim.update(7000.0));
        assert!(lim.update(6950.0));
        assert!(!lim.update(6890.0));
    }

    #[test]
    fn advance_is_clamped_and_corrections_bounded() {
        let a = corrected_advance(20.0, 100.0, -100.0, 0.0).unwrap();
        assert!((a - 20.0).abs() < 1e-5); // +15 and -15 cancel
        let b = corrected_advance(50.0, 15.0, 15.0, 30.0).unwrap();
        assert!(b <= MAX_ADVANCE_DEG);
    }

    #[test]
    fn dwell_is_reduced_at_high_rpm_but_never_zero() {
        let d = limit_dwell(7000.0, 6.0, 1, 4).unwrap();
        assert!((MIN_DWELL_MS..6.0).contains(&d), "dwell={d}");
        assert!(limit_dwell(0.0, 3.0, 1, 4).is_err());
    }
}
