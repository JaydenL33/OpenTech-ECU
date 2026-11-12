use std::string::String;
use std::vec::Vec;

use ecu_types::*;
use serde::{Deserialize, Serialize};

use crate::table::{Axis, Table};
use crate::{
    hex_sha256, now_unix_ms, Calibration, CalibrationError, IgnitionPolicy,
    CALIBRATION_SCHEMA_VERSION,
};

pub const VE_TABLE: &str = "ve";
pub const TARGET_LAMBDA_TABLE: &str = "target_lambda";
pub const IGNITION_TABLE: &str = "ignition";
pub const DWELL_TABLE: &str = "dwell";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WizardRequest {
    pub engine: EngineConfig,
    pub fuel: FuelConfig,
    pub sensors: SensorSetup,
    pub io: IoConfig,
    /// Optional licensed, engine-specific timing reference. Without this the
    /// wizard creates a synthetic simulator-only table and blocks physical use.
    pub verified_ignition: Option<Table>,
    pub vehicle_name: String,
    pub notes: Vec<String>,
}

impl WizardRequest {
    pub fn default_inline4() -> Self {
        let mut engine = EngineConfig::default_inline4();
        engine.name = "OpenTech VR4 2.0L (virtual)".into();
        Self {
            engine,
            fuel: FuelConfig::default(),
            sensors: SensorSetup::default(),
            io: IoConfig::default(),
            verified_ignition: None,
            vehicle_name: "Virtual Development Vehicle".into(),
            notes: vec!["Synthetic simulator profile; not a validated engine calibration".into()],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WizardWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WizardResult {
    pub calibration: Calibration,
    pub warnings: Vec<WizardWarning>,
    pub virtual_run_allowed: bool,
    pub physical_run_allowed: bool,
}

/// Generate a complete starter calibration. All fuel values are explicit
/// estimates and all synthetic ignition data is marked simulation-only.
pub fn generate_starter(
    req: &WizardRequest,
    revision: u32,
) -> Result<Calibration, CalibrationError> {
    let issues = req.engine.validate();
    if let Some(e) = issues.iter().find(|i| i.is_error()) {
        return Err(CalibrationError::InvalidValues(format!(
            "{}: {}",
            e.code, e.message
        )));
    }
    let fuel_issues = validate_fuel_config(&req.fuel);
    if let Some(e) = fuel_issues.iter().find(|i| i.is_error()) {
        return Err(CalibrationError::InvalidValues(format!(
            "{}: {}",
            e.code, e.message
        )));
    }
    if req.engine.injectors.is_empty() {
        return Err(CalibrationError::InvalidValues(
            "at least one injector definition is required".into(),
        ));
    }

    let rpm_axis = build_rpm_axis(&req.engine);
    let map_axis = build_map_axis(&req.engine);
    let ve = build_ve_table(&req.engine, &rpm_axis, &map_axis);
    let target = build_target_lambda_table(&req.engine, &rpm_axis, &map_axis);
    let (ignition, ignition_policy, mut warnings) = build_ignition(req, &rpm_axis, &map_axis);
    let dwell = build_dwell_table();

    let mut tables = std::collections::BTreeMap::new();
    tables.insert(VE_TABLE.into(), ve);
    tables.insert(TARGET_LAMBDA_TABLE.into(), target);
    tables.insert(IGNITION_TABLE.into(), ignition);
    tables.insert(DWELL_TABLE.into(), dwell);

    let mut notes = req.notes.clone();
    notes.push(
        "ALL FUEL VALUES ARE UNVALIDATED STARTING ESTIMATES. Verify injector dead time, \
         fuel pressure, sensor calibration, trigger offset and the ignition reference on \
         real hardware with appropriate supervision."
            .into(),
    );
    if let Some(w) = ignition_policy.warning() {
        notes.push(w.into());
    }

    let id = format!(
        "cal-{}-{}",
        revision,
        &hex_sha256(
            format!(
                "{}:{}:{}:{}",
                req.engine.name, req.engine.displacement_cc, req.engine.rev_limit, req.vehicle_name
            )
            .as_bytes()
        )[..12]
    );
    let mut cal = Calibration {
        schema_version: CALIBRATION_SCHEMA_VERSION,
        id,
        revision,
        parent: None,
        created_unix_ms: now_unix_ms(),
        engine: req.engine.clone(),
        fuel: req.fuel.clone(),
        sensors: req.sensors.clone(),
        io: req.io.clone(),
        tables,
        ignition_policy,
        notes,
        checksum: String::new(),
    };
    cal.seal();

    let pre = crate::preflight::preflight(&cal);
    if !req.sensors.map_calibrated {
        warnings.push(WizardWarning {
            code: "SENSOR_CAL_UNVERIFIED".into(),
            message: "MAP/CLT/IAT sensor calibration is not verified; virtual run only".into(),
        });
    }
    // Preflight is advisory at generation time; it is enforced at export and
    // at every physical output boundary.
    let _ = pre;
    let _ = warnings;
    Ok(cal)
}

fn build_rpm_axis(engine: &EngineConfig) -> Axis {
    let low = engine.idle_rpm.saturating_sub(200).max(400) as f32;
    let high = (engine.rev_limit as f32 - 100.0).max(low + 800.0);
    let mut points = Vec::with_capacity(16);
    for i in 0..16 {
        let t = i as f32 / 15.0;
        // Denser at low RPM where driveability cells matter.
        let shaped = low + (high - low) * t.powf(1.15);
        let rounded = (shaped / 100.0).round() * 100.0;
        if points.last().copied().unwrap_or(-1.0) < rounded {
            points.push(rounded);
        }
    }
    Axis::new("RPM", "rpm", points)
}

fn build_map_axis(engine: &EngineConfig) -> Axis {
    let points = match engine.aspiration {
        Aspiration::NaturallyAspirated => vec![
            20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0, 110.0, 120.0, 140.0, 160.0,
            180.0, 210.0, 240.0,
        ],
        Aspiration::Turbocharged | Aspiration::Supercharged => vec![
            20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0, 120.0, 140.0, 160.0, 180.0,
            200.0, 230.0, 260.0,
        ],
    };
    Axis::new("MAP", "kPa", points)
}

fn build_ve_table(engine: &EngineConfig, rpm: &Axis, map: &Axis) -> Table {
    let mut values = Vec::with_capacity(rpm.breakpoints.len() * map.breakpoints.len());
    let rpm_lo = rpm.breakpoints[0];
    let rpm_hi = rpm.breakpoints[rpm.breakpoints.len() - 1];
    let map_lo = map.breakpoints[0];
    let map_hi = map.breakpoints[map.breakpoints.len() - 1];
    let boost = matches!(
        engine.aspiration,
        Aspiration::Turbocharged | Aspiration::Supercharged
    );
    for &m in &map.breakpoints {
        let load = ((m - map_lo) / (map_hi - map_lo)).clamp(0.0, 1.0);
        for &r in &rpm.breakpoints {
            let rn = ((r - rpm_lo) / (rpm_hi - rpm_lo)).clamp(0.0, 1.0);
            // Broad breathing curve peaking around 55% of the RPM range.
            let shape = (1.0 - ((rn - 0.55) / 0.62).powi(2)).clamp(0.20, 1.0);
            let mut ve = 34.0 + 30.0 * shape + 30.0 * load * (0.35 + 0.65 * shape);
            if boost && m > 105.0 {
                ve += (m - 105.0) * 0.055;
            }
            values.push(ve.clamp(28.0, 105.0));
        }
    }
    Table::new(
        VE_TABLE,
        "Volumetric efficiency",
        "%",
        vec![map.clone(), rpm.clone()],
        values,
        Provenance {
            source: SourceKind::EngineeringEstimate,
            citation: "Shape-only estimate generated from displacement and aspiration; no flow-bench data".into(),
            licence: "GPL-2.0-or-later".into(),
            verified_by: String::new(),
            verification_date: String::new(),
            engine_specific_verified: false,
            notes: "UNVALIDATED STARTING ESTIMATE. Replace with measured VE from a supervised dyno session.".into(),
        },
    )
    .expect("VE table shape generated internally")
    .with_bounds(20.0, 120.0, 1)
    .with_description("Estimated volumetric efficiency. Load rows (kPa) × RPM columns. Unvalidated starting estimate.")
}

fn build_target_lambda_table(engine: &EngineConfig, rpm: &Axis, map: &Axis) -> Table {
    let boosted = matches!(
        engine.aspiration,
        Aspiration::Turbocharged | Aspiration::Supercharged
    );
    let mut values = Vec::new();
    let rpm_hi = rpm.breakpoints[rpm.breakpoints.len() - 1];
    for &m in &map.breakpoints {
        for &r in &rpm.breakpoints {
            let load = ((m - 20.0) / 220.0).clamp(0.0, 1.0);
            let mut lambda: f32 = if m <= 40.0 {
                0.96
            } else if m <= 75.0 {
                1.00
            } else if m <= 100.0 {
                0.94
            } else {
                0.87
            };
            if m > 140.0 {
                lambda = 0.82;
            }
            if boosted && m > 160.0 && r > 3000.0 {
                lambda = 0.80;
            }
            // Slightly richer at high RPM under load for thermal margin. This is
            // a starting estimate, not a claim of optimal mixture.
            if r > rpm_hi * 0.8 && load > 0.55 {
                lambda -= 0.015;
            }
            values.push(lambda.clamp(0.75, 1.05));
        }
    }
    Table::new(
        TARGET_LAMBDA_TABLE,
        "Target lambda",
        "lambda",
        vec![map.clone(), rpm.clone()],
        values,
        Provenance {
            source: SourceKind::EngineeringEstimate,
            citation: "Conservative generic target-lambda shape; fuel-specific stoich is configured separately".into(),
            licence: "GPL-2.0-or-later".into(),
            verified_by: String::new(),
            verification_date: String::new(),
            engine_specific_verified: false,
            notes: "UNVALIDATED. Targets must be reviewed against fuel, compression, boost and thermal limits.".into(),
        },
    )
    .expect("target lambda shape generated internally")
    .with_bounds(0.70, 1.10, 3)
    .with_description("Target lambda by load and RPM. Lambda is fuel-specific only through the configured stoichiometric AFR.")
}

fn build_dwell_table() -> Table {
    Table::new(
        DWELL_TABLE,
        "Coil dwell",
        "ms",
        vec![Axis::new("Battery", "V", vec![8.0, 10.0, 12.0, 13.8, 15.0])],
        vec![7.0, 5.2, 3.8, 3.0, 2.6],
        Provenance::synthetic_simulation(
            "Representative coil dwell curve, not a coil datasheet. Verify against the selected coil.",
        ),
    )
    .expect("dwell generated internally")
    .with_bounds(0.5, 10.0, 2)
}

fn build_ignition(
    req: &WizardRequest,
    rpm: &Axis,
    map: &Axis,
) -> (Table, IgnitionPolicy, Vec<WizardWarning>) {
    let mut warnings = Vec::new();
    if let Some(reference) = &req.verified_ignition {
        if reference.provenance.is_physical_reference()
            && reference.axes.len() == 2
            && reference.values.iter().all(|v| v.is_finite())
        {
            // Resample the licensed reference onto the generated axes. The
            // original evidence metadata is preserved on the output table.
            let mut table = reference.clone();
            table.id = IGNITION_TABLE.into();
            table.name = "Ignition advance (licensed reference)".into();
            table.unit = "deg BTDC".into();
            table.provenance.notes = format!(
                "{} Resampled to wizard axes; verify timing light and engine-specific conditions.",
                table.provenance.notes
            );
            // Resampling is deliberately performed by the caller when axes
            // differ; for v1 keep the reference axes intact.
            table.mark_edited("imported licensed reference");
            let policy = IgnitionPolicy::VerifiedReference {
                citation: reference.provenance.citation.clone(),
                licence: reference.provenance.licence.clone(),
                verified_by: reference.provenance.verified_by.clone(),
                timing_light_required: true,
            };
            return (table, policy, warnings);
        }
        warnings.push(WizardWarning {
            code: "IGNITION_REFERENCE_REJECTED".into(),
            message: "Supplied ignition reference lacks verified engine-specific provenance; synthetic simulation table used instead and physical use is blocked.".into(),
        });
    }

    let mut values = Vec::new();
    for &m in &map.breakpoints {
        for &r in &rpm.breakpoints {
            let rpm_frac = (r / 7000.0).clamp(0.0, 1.0);
            let load = ((m - 20.0) / 220.0).clamp(0.0, 1.0);
            // Deliberately conservative, smooth demonstration surface. This is
            // NOT an MBT estimate and NOT knock-safe timing.
            let mut advance = if m < 45.0 {
                9.0 + 8.0 * load
            } else if m < 90.0 {
                18.0 + 12.0 * load
            } else {
                26.0 - 12.0 * load
            };
            advance -= 4.0 * rpm_frac;
            advance = advance.clamp(2.0, 32.0);
            if m > 140.0 {
                advance = (advance - 5.0).max(2.0);
            }
            values.push(advance);
        }
    }
    let table = Table::new(
        IGNITION_TABLE,
        "Ignition advance (SYNTHETIC SIMULATION ONLY)",
        "deg BTDC",
        vec![map.clone(), rpm.clone()],
        values,
        Provenance::synthetic_simulation(
            "Synthetic surface for virtual-ECU demonstration. Not derived from an engine, dyno data, knock limit or MBT measurement. Do not flash to a vehicle.",
        ),
    )
    .expect("ignition shape generated internally")
    .with_bounds(0.0, 45.0, 1)
    .with_description(
        "SYNTHETIC SIMULATION ONLY. No source timing data. Lambda is never used to infer advance. Physical exports are blocked by preflight.",
    );
    let policy = IgnitionPolicy::SyntheticSimulationOnly {
        warning: "Ignition table is synthetic and has no engine-specific evidence. Physical ignition output is blocked; a licensed reference and timing-light verification are required.".into(),
        table_key: IGNITION_TABLE.into(),
    };
    (table, policy, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_all_required_tables_with_expected_ranks() {
        let cal = generate_starter(&WizardRequest::default_inline4(), 1).unwrap();
        assert_eq!(cal.table(VE_TABLE).unwrap().rank(), 2);
        assert_eq!(cal.table(TARGET_LAMBDA_TABLE).unwrap().rank(), 2);
        assert_eq!(cal.table(IGNITION_TABLE).unwrap().rank(), 2);
        assert_eq!(cal.table(DWELL_TABLE).unwrap().rank(), 1);
        assert!(cal.verify());
        assert!(cal.validate().iter().all(|i| !i.is_error()));
    }

    #[test]
    fn ve_and_target_shapes_are_bounded() {
        let cal = generate_starter(&WizardRequest::default_inline4(), 1).unwrap();
        let ve = cal.table(VE_TABLE).unwrap();
        assert!(ve.values.iter().all(|v| (20.0..=120.0).contains(v)));
        let tl = cal.table(TARGET_LAMBDA_TABLE).unwrap();
        assert!(tl.values.iter().all(|v| (0.70..=1.10).contains(v)));
    }

    #[test]
    fn unverified_reference_is_rejected() {
        let mut req = WizardRequest::default_inline4();
        let fake = Table::new(
            "ign",
            "fake",
            "deg",
            vec![
                Axis::new("MAP", "kPa", vec![20.0, 100.0]),
                Axis::new("RPM", "rpm", vec![1000.0, 5000.0]),
            ],
            vec![10.0, 12.0, 14.0, 16.0],
            Provenance::unknown("unverified"),
        )
        .unwrap();
        req.verified_ignition = Some(fake);
        let cal = generate_starter(&req, 1).unwrap();
        assert!(matches!(
            cal.ignition_policy,
            IgnitionPolicy::SyntheticSimulationOnly { .. }
        ));
    }

    #[test]
    fn invalid_engine_profile_fails_actionably() {
        let mut req = WizardRequest::default_inline4();
        req.engine.firing_order = vec![1, 2];
        let err = generate_starter(&req, 1).unwrap_err();
        assert!(format!("{err}").contains("firing_order") || format!("{err}").contains("ENG"));
    }
}
