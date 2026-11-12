//! Vehicle reference profiles.
//!
//! The Mitsubishi Evo VIII GSR profile is deliberately split into:
//! - verified metadata (identity, engine family, transmission family);
//! - simulation-only assumptions used by the virtual ECU;
//! - blocking unknowns that must be resolved against the physical vehicle.
//!
//! No pin, connector, trigger tooth count or injector characteristic in this
//! module may be treated as verified because it appears here. The physical
//! request is refused until the audit evidence is complete.

use std::string::String;
use std::vec::Vec;

use ecu_types::{
    Aspiration, EngineConfig, FuelConfig, InjectorConfig, SensorSetup, TriggerConfig,
    TriggerPattern,
};
use serde::{Deserialize, Serialize};

use crate::wizard::{generate_starter, WizardRequest, WizardWarning};
use crate::Calibration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Unverified,
    PartiallyVerified,
    VerifiedForBench,
    VerifiedForVehicle,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnverifiedItem {
    pub id: String,
    pub title: String,
    pub blocking_reason: String,
    pub verification_method: String,
    pub gate: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FactorySystem {
    pub name: String,
    pub status: String,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VehicleProfile {
    pub id: String,
    pub make: String,
    pub model: String,
    pub variant: String,
    pub market: String,
    pub chassis: String,
    pub engine: String,
    pub transmission: String,
    pub aspiration: String,
    pub verification: VerificationStatus,
    pub ecu_part_number: Option<String>,
    pub harness_documented: bool,
    pub connector_documented: bool,
    pub pinout_documented: bool,
    pub trigger_verified: bool,
    pub injector_data_verified: bool,
    pub ignition_data_verified: bool,
    pub unverified: Vec<UnverifiedItem>,
    pub factory_systems: Vec<FactorySystem>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VehicleConfigError {
    pub code: String,
    pub message: String,
    pub missing: Vec<String>,
}

impl std::fmt::Display for VehicleConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} ({} blocking items)",
            self.code,
            self.message,
            self.missing.len()
        )
    }
}

pub fn builtin_profiles() -> Vec<VehicleProfile> {
    vec![evo8_jdm_gsr()]
}

pub fn by_id(id: &str) -> Option<VehicleProfile> {
    builtin_profiles().into_iter().find(|p| p.id == id)
}

pub fn evo8_jdm_gsr() -> VehicleProfile {
    VehicleProfile {
        id: "mitsubishi-evo8-jdm-gsr-6mt".into(),
        make: "Mitsubishi".into(),
        model: "Lancer Evolution VIII".into(),
        variant: "GSR".into(),
        market: "Japan (JDM)".into(),
        chassis: "CT9A".into(),
        engine: "4G63 turbocharged 2.0L inline-4".into(),
        transmission: "6-speed manual (W6MAA expected, unconfirmed)".into(),
        aspiration: "turbocharged".into(),
        verification: VerificationStatus::Unverified,
        ecu_part_number: None,
        harness_documented: false,
        connector_documented: false,
        pinout_documented: false,
        trigger_verified: false,
        injector_data_verified: false,
        ignition_data_verified: false,
        unverified: vec![
            item("EVO-ID-001", "Exact ECU part number", "Cannot select a connector or adapter without it.", "Photograph the label and connector housings; record all markings.", 3),
            item("EVO-PIN-001", "ECU connector pinout", "A wrong pin assignment can destroy the ECU or vehicle circuits.", "Key-off continuity and pin-to-pin checks against factory service data; second-person review.", 3),
            item("EVO-CON-001", "Mating connector part numbers", "Plug-and-play cannot be claimed without the correct housing, terminals and seals.", "Identify housing markings, keying, terminal series and service connector references.", 3),
            item("EVO-TRIG-001", "Crank/cam trigger pattern", "Wrong decoding loses sync or schedules spark at the wrong angle.", "Scope crank and cam at cranking; verify tooth count, gap, phase and sensor type.", 3),
            item("EVO-INJ-001", "Injector type and flow", "Fuel model and duty limits depend on rated flow, dead time, resistance and driver type.", "Read injector markings; bench flow/dead-time test or obtain manufacturer data.", 3),
            item("EVO-IGN-001", "Coil/igniter topology", "Direct coil drive vs external igniter changes output stage and safe defaults.", "Trace ignition outputs, measure trigger polarity and verify coil primary resistance.", 3),
            item("EVO-KNK-001", "Knock sensor interface", "Knock input conditioning and thresholds are engine-specific.", "Measure sensor type, shielding and factory module behaviour; validate with controlled excitation.", 3),
            item("EVO-FUEL-001", "Fuel pump/relay control", "FP relay polarity and failsafe state are unknown.", "Verify relay coil voltage/current and factory control strategy key-off/key-on.", 3),
            item("EVO-ACD-001", "ACD/AYC CAN interaction", "Replacement ECU may break factory chassis control or dashboard warnings.", "Capture CAN traffic with factory ECU installed; identify frames only from authoritative data.", 4),
            item("EVO-IMM-001", "Immobiliser/security", "Bypassing security is prohibited; loss of immobiliser function can strand the vehicle.", "Document factory behaviour and obtain a legitimate integration strategy.", 4),
            item("EVO-BOOST-001", "Boost control solenoid", "Solenoid type, frequency and safe default are unknown.", "Measure resistance/inductance and factory control; default output off.", 3),
            item("EVO-VSS-001", "Vehicle speed / gear / clutch signals", "Six-speed functions need verified speed and driveline signals.", "Scope VSS and clutch/neutral inputs; verify gear ratios and sensor type.", 4),
        ],
        factory_systems: vec![
            FactorySystem { name: "Engine management".into(), status: "UNVERIFIED".into(), note: "Primary replacement target; no pinout or trigger data verified.".into() },
            FactorySystem { name: "ACD (Active Centre Differential)".into(), status: "FURTHER RESEARCH".into(), note: "May communicate with the engine ECU; do not disable or spoof without review.".into() },
            FactorySystem { name: "AYC (Active Yaw Control)".into(), status: "FURTHER RESEARCH".into(), note: "JDM-specific interaction with ABS/ACD; unsupported until CAN and electrical behaviour are documented.".into() },
            FactorySystem { name: "Immobiliser".into(), status: "MUST NOT BYPASS".into(), note: "No security bypass. A legitimate integration approach is required.".into() },
            FactorySystem { name: "Dashboard / tachometer".into(), status: "UNVERIFIED".into(), note: "Tach output and warning lamps need signal verification.".into() },
            FactorySystem { name: "Air conditioning".into(), status: "FURTHER RESEARCH".into(), note: "Compressor request/clutch control and idle-up behaviour unknown.".into() },
            FactorySystem { name: "Cooling fans".into(), status: "UNVERIFIED".into(), note: "Relay polarity, PWM or relay control and failsafe state unknown.".into() },
            FactorySystem { name: "Fuel pump control".into(), status: "UNVERIFIED".into(), note: "Factory may use multi-speed or PWM control; do not assume a simple relay.".into() },
            FactorySystem { name: "Intercooler spray".into(), status: "FURTHER RESEARCH".into(), note: "Output polarity and automatic behaviour unknown.".into() },
            FactorySystem { name: "Factory diagnostics (MUT-III/OBD)".into(), status: "FURTHER RESEARCH".into(), note: "Protocol, pins and diagnostic session behaviour must be captured; cannot claim support.".into() },
        ],
        notes: vec![
            "JDM GSR is not assumed to share wiring with USDM, UKDM, MR, RS or Evo IX models.".into(),
            "No value in this profile is a vehicle-ready calibration. Simulation values are explicitly synthetic.".into(),
            "Physical integration remains blocked at Gate 3 until the pinout, connector, trigger and electrical checks are complete.".into(),
        ],
    }
}

fn item(
    id: &str,
    title: &str,
    blocking_reason: &str,
    verification_method: &str,
    gate: u8,
) -> UnverifiedItem {
    UnverifiedItem {
        id: id.into(),
        title: title.into(),
        blocking_reason: blocking_reason.into(),
        verification_method: verification_method.into(),
        gate,
    }
}

impl VehicleProfile {
    pub fn blocking_items(&self) -> Vec<&UnverifiedItem> {
        self.unverified.iter().collect()
    }

    pub fn physical_ready(&self) -> bool {
        self.verification == VerificationStatus::VerifiedForVehicle
            && self.pinout_documented
            && self.connector_documented
            && self.trigger_verified
            && self.injector_data_verified
            && self.ignition_data_verified
    }

    /// Build a simulator-only request. Every unverified vehicle property is
    /// replaced by an explicitly labelled synthetic assumption. This does not
    /// certify any value for the real car.
    pub fn simulation_request(&self) -> WizardRequest {
        if self.id != "mitsubishi-evo8-jdm-gsr-6mt" {
            return WizardRequest::default_inline4();
        }
        let mut engine = EngineConfig::default_inline4();
        engine.name = "Mitsubishi Evo VIII GSR — SIMULATION TEMPLATE (UNVERIFIED)".into();
        engine.displacement_cc = 1997.0;
        engine.cylinders = 4;
        engine.firing_order = vec![1, 3, 4, 2];
        engine.compression_ratio = 8.8;
        engine.aspiration = Aspiration::Turbocharged;
        engine.rev_limit = 7500;
        engine.idle_rpm = 850;
        // Synthetic assumptions. The real 4G63 trigger is unverified; 36-1 is
        // used only because the simulator supports it and is clearly labelled.
        engine.trigger = TriggerConfig {
            pattern: TriggerPattern::MissingTooth {
                teeth: 36,
                missing: 1,
            },
            cam_required: true,
            sync_offset_deg: 0.0,
            tooth_timeout_ms: 250,
        };
        engine.injectors = vec![InjectorConfig {
            id: "SIM-560-UNVERIFIED".into(),
            flow_cc_min: 560.0,
            rated_pressure_kpa: 300.0,
            dead_time_ms: 1.0,
            dead_time_curve: vec![(8.0, 1.8), (12.0, 1.2), (13.8, 1.0), (15.0, 0.9)],
            min_pw_ms: 0.8,
            max_duty: 0.9,
            cylinders: vec![1, 2, 3, 4],
        }];
        engine.cylinder_banks = vec![1, 1, 1, 1];
        let _ = engine.validate();
        let fuel = FuelConfig {
            stoich_afr: 14.7,
            ..FuelConfig::default()
        };
        WizardRequest {
            engine,
            fuel,
            sensors: SensorSetup::default(),
            io: Default::default(),
            verified_ignition: None,
            vehicle_name: "Mitsubishi Evo VIII GSR — virtual bench reference".into(),
            notes: vec![
                "SIMULATION ONLY. Trigger pattern, injector flow, dead time and sensor data are synthetic placeholders, not verified Evo VIII values.".into(),
                "Physical pinout, connector, trigger, injector and ignition data are UNVERIFIED. Do not build or connect a vehicle harness from this profile.".into(),
                "No vehicle security, ACD/AYC or factory safety system is modelled or bypassed.".into(),
            ],
        }
    }

    pub fn simulation_calibration(
        &self,
    ) -> Result<(Calibration, Vec<WizardWarning>), VehicleConfigError> {
        let request = self.simulation_request();
        let cal = generate_starter(&request, 1).map_err(|e| VehicleConfigError {
            code: "EVO_SIM_GENERATION".into(),
            message: e.to_string(),
            missing: self.unverified.iter().map(|u| u.id.clone()).collect(),
        })?;
        let warnings = vec![WizardWarning {
            code: "UNVERIFIED_EVO_SIMULATION".into(),
            message: "Evo VIII profile is a synthetic virtual-bench template. Physical outputs remain blocked.".into(),
        }];
        Ok((cal, warnings))
    }

    /// Physical configuration is intentionally refused until audit evidence is
    /// complete. This is a fail-closed API, not a TODO.
    pub fn physical_request(&self) -> Result<WizardRequest, VehicleConfigError> {
        if !self.physical_ready() {
            return Err(VehicleConfigError {
                code: "EVO_PHYSICAL_CONFIG_BLOCKED".into(),
                message: "Vehicle configuration is not verified for physical use.".into(),
                missing: self.unverified.iter().map(|u| u.id.clone()).collect(),
            });
        }
        Err(VehicleConfigError {
            code: "EVO_PHYSICAL_CONFIG_NOT_IMPLEMENTED".into(),
            message: "Even with metadata complete, generating an engine-safe physical calibration is not authorised.".into(),
            missing: self.unverified.iter().map(|u| u.id.clone()).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evo_profile_is_fail_closed() {
        let p = evo8_jdm_gsr();
        assert!(!p.physical_ready());
        assert_eq!(p.verification, VerificationStatus::Unverified);
        assert!(p.unverified.iter().any(|u| u.id == "EVO-PIN-001"));
        assert!(p.physical_request().is_err());
    }

    #[test]
    fn simulation_profile_is_explicitly_unverified() {
        let p = evo8_jdm_gsr();
        let (cal, warnings) = p.simulation_calibration().unwrap();
        assert!(cal.verify());
        assert!(cal.notes.iter().any(|n| n.contains("SIMULATION ONLY")));
        assert!(warnings
            .iter()
            .any(|w| w.code == "UNVERIFIED_EVO_SIMULATION"));
        assert!(matches!(
            cal.ignition_policy,
            crate::IgnitionPolicy::SyntheticSimulationOnly { .. }
        ));
        let pre = crate::preflight::preflight(&cal);
        assert!(pre.virtual_run_allowed);
        assert!(!pre.physical_export_allowed);
    }

    #[test]
    fn jdm_variant_notes_are_present() {
        let p = evo8_jdm_gsr();
        assert!(p.notes.iter().any(|n| n.contains("JDM GSR is not assumed")));
        assert!(!p.pinout_documented);
    }
}
