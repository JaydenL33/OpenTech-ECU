use std::string::String;
use std::vec::Vec;

use ecu_types::*;
use serde::{Deserialize, Serialize};

use crate::{Calibration, IgnitionPolicy};

/// There is no code path in v0.1 that energises a physical injector or coil.
/// This constant is the single explicit interlock; it must not be turned on
/// without the independent reviews documented in docs/SAFETY_CASE.md.
pub const PHYSICAL_OUTPUT_INTERLOCK_ENGAGED: bool = true;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    NotApplicable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreflightCheck {
    pub id: String,
    pub title: String,
    pub status: CheckStatus,
    pub detail: String,
    pub blocks_physical: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreflightReport {
    pub checks: Vec<PreflightCheck>,
    pub error_count: usize,
    pub warning_count: usize,
    /// True only if every technical check passes. Still not a safety release.
    pub hardware_candidate: bool,
    /// Always false in v0.1 due to the physical output interlock.
    pub physical_export_allowed: bool,
    /// A valid simulator calibration may always run in virtual mode; it cannot
    /// actuate hardware.
    pub virtual_run_allowed: bool,
    pub summary: String,
}

pub fn preflight(cal: &Calibration) -> PreflightReport {
    let mut checks = Vec::new();
    macro_rules! add {
        ($id:expr, $title:expr, $status:expr, $detail:expr, $blocks:expr $(,)?) => {
            checks.push(PreflightCheck {
                id: $id.into(),
                title: $title.into(),
                status: $status,
                detail: $detail,
                blocks_physical: $blocks,
            })
        };
    }

    add!(
        "CAL_CHECKSUM",
        "Calibration checksum",
        if cal.verify() {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        if cal.verify() {
            "SHA-256 matches content".into()
        } else {
            "Calibration content has changed or the checksum is corrupt. Import is refused.".into()
        },
        true,
    );

    let engine_issues = cal.engine.validate();
    let engine_errors: Vec<_> = engine_issues.iter().filter(|i| i.is_error()).collect();
    add!(
        "ENGINE_PROFILE",
        "Engine profile",
        if engine_errors.is_empty() {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        if engine_errors.is_empty() {
            format!(
                "{} cylinders, {:.0} cc, firing order {:?}",
                cal.engine.cylinders, cal.engine.displacement_cc, cal.engine.firing_order
            )
        } else {
            engine_errors
                .iter()
                .map(|i| format!("{}: {}", i.code, i.message))
                .collect::<Vec<_>>()
                .join("; ")
        },
        true,
    );

    let fuel_errors: Vec<_> = validate_fuel_config(&cal.fuel)
        .into_iter()
        .filter(|i| i.is_error())
        .collect();
    add!(
        "FUEL_CONFIG",
        "Fuel configuration",
        if fuel_errors.is_empty() {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        if fuel_errors.is_empty() {
            format!(
                "Stoich AFR {:.1}, density {:.3} g/cc, {:?}",
                cal.fuel.stoich_afr, cal.fuel.fuel_density_g_cc, cal.fuel.load_strategy
            )
        } else {
            fuel_errors
                .iter()
                .map(|i| i.message.clone())
                .collect::<Vec<_>>()
                .join("; ")
        },
        true,
    );

    let mut table_ok = true;
    let mut table_detail = Vec::new();
    for key in ["ve", "target_lambda", "ignition", "dwell"] {
        match cal.tables.get(key) {
            None => {
                table_ok = false;
                table_detail.push(format!("{key}: missing"));
            }
            Some(t) => {
                let errors: Vec<_> = t.validate().into_iter().filter(|i| i.is_error()).collect();
                if !errors.is_empty() {
                    table_ok = false;
                    table_detail.push(format!("{key}: {} validation errors", errors.len()));
                } else {
                    table_detail.push(format!("{key}: {}D, {} cells", t.rank(), t.values.len()));
                }
            }
        }
    }
    add!(
        "MAPS",
        "Required maps",
        if table_ok {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        table_detail.join("; "),
        true,
    );

    // Shape compatibility: VE and target lambda must share axes.
    let shape_ok = match (cal.tables.get("ve"), cal.tables.get("target_lambda")) {
        (Some(ve), Some(tl)) => {
            ve.axes.len() == 2
                && tl.axes.len() == 2
                && ve.axes[0].breakpoints == tl.axes[0].breakpoints
                && ve.axes[1].breakpoints == tl.axes[1].breakpoints
        }
        _ => false,
    };
    add!(
        "MAP_AXIS_COMPAT",
        "Fuel/target axis compatibility",
        if shape_ok {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        if shape_ok {
            "VE and target-lambda tables share the same load/RPM axes".into()
        } else {
            "VE and target-lambda axes differ; mixture-map learning requires aligned cells".into()
        },
        true,
    );

    let injectors_ok = !cal.engine.injectors.is_empty()
        && cal
            .engine
            .injectors
            .iter()
            .all(|i| i.flow_cc_min > 0.0 && i.dead_time_ms > 0.0);
    add!(
        "INJECTOR_CHARACTERISATION",
        "Injector characterisation",
        if injectors_ok {
            CheckStatus::Warn
        } else {
            CheckStatus::Fail
        },
        if injectors_ok {
            "Rated flow and dead time are declared, but no independent flow-bench verification is recorded".into()
        } else {
            "Injector flow/dead time missing; fuel calculation cannot be trusted".into()
        },
        true,
    );

    let sensor_flags =
        cal.sensors.map_calibrated && cal.sensors.clt_calibrated && cal.sensors.iat_calibrated;
    add!(
        "SENSOR_CALIBRATION",
        "Sensor calibration",
        if sensor_flags {
            CheckStatus::Pass
        } else {
            CheckStatus::Warn
        },
        if sensor_flags {
            "MAP, CLT and IAT calibration flags are set; their actual transfer functions must still be reviewed".into()
        } else {
            "MAP/CLT/IAT calibration flags are not set. Virtual run permitted; physical use blocked"
                .into()
        },
        true,
    );

    add!(
        "WIDEBAND",
        "Wideband assignment",
        if cal.sensors.wideband_banks > 0 {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        if cal.sensors.wideband_banks > 0 {
            format!(
                "{} bank(s) assigned; readiness and ageing are checked at runtime",
                cal.sensors.wideband_banks
            )
        } else {
            "No wideband bank assigned; fuel learning and closed loop cannot be enabled".into()
        },
        true,
    );

    let lambda_bounds_ok = cal.tables.get("target_lambda").is_some_and(|t| {
        t.values
            .iter()
            .all(|v| v.is_finite() && (0.60..=1.10).contains(v))
    });
    add!(
        "TARGET_LAMBDA_BOUNDS",
        "Target lambda plausibility",
        if lambda_bounds_ok {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        if lambda_bounds_ok {
            "All target-lambda cells are within 0.60..=1.10".into()
        } else {
            "Target lambda contains implausible or missing values".into()
        },
        true,
    );

    let trigger_ok = matches!(
        cal.engine.trigger.pattern,
        TriggerPattern::MissingTooth { .. }
    );
    add!(
        "TRIGGER",
        "Crank trigger",
        if trigger_ok {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        format!(
            "{:?}; cam required={}; base offset {:.1}°",
            cal.engine.trigger.pattern,
            cal.engine.trigger.cam_required,
            cal.engine.trigger.sync_offset_deg
        ),
        true,
    );

    let ignition_block = !cal.ignition_policy.hardware_eligible();
    add!(
        "IGNITION_EVIDENCE",
        "Ignition timing evidence",
        if ignition_block { CheckStatus::Fail } else { CheckStatus::Pass },
        match &cal.ignition_policy {
            IgnitionPolicy::SyntheticSimulationOnly { warning, .. } => format!(
                "SIMULATION ONLY: {warning} Lambda data cannot establish safe timing."
            ),
            IgnitionPolicy::InsufficientEvidence { reason } => {
                format!("INSUFFICIENT EVIDENCE: {reason}")
            }
            IgnitionPolicy::VerifiedReference { citation, .. } => format!(
                "Licensed engine-specific reference: {citation}. Timing light synchronisation still mandatory."
            ),
        },
        true,
    );

    let timing_light = match &cal.ignition_policy {
        IgnitionPolicy::VerifiedReference { .. } => CheckStatus::Warn,
        _ => CheckStatus::NotApplicable,
    };
    add!(
        "TIMING_LIGHT",
        "Timing-light synchronisation",
        timing_light,
        "No physical run is permitted without checking base offset with a timing light, immobilising the vehicle, and recording the measurement.".into(),
        true,
    );

    // Stable ignition offset is a physical calibration value and is not inferred
    // from the trigger configuration.
    add!(
        "IGNITION_OFFSET",
        "Absolute ignition offset",
        CheckStatus::Warn,
        "Trigger sync offset is not proof of TDC. A physical timing-light measurement (or OEM service data) is required.".into(),
        true,
    );

    if cal.fuel.pressure_reference_kpa < 100.0 {
        add!(
            "FUEL_PRESSURE",
            "Fuel pressure reference",
            CheckStatus::Fail,
            "Fuel pressure reference is out of range".into(),
            true,
        );
    } else {
        add!(
            "FUEL_PRESSURE",
            "Fuel pressure reference",
            CheckStatus::Warn,
            format!(
                "Reference {:.0} kPa configured; sensor plausibility and regulator behaviour are not verified",
                cal.fuel.pressure_reference_kpa
            ),
            true,
        );
    }

    let mut seen = std::collections::BTreeMap::<String, usize>::new();
    let mut conflicts = Vec::new();
    for io in &cal.io.assignments {
        *seen.entry(io.pin.clone()).or_insert(0) += 1;
    }
    for (pin, count) in seen {
        if count > 1 {
            conflicts.push(pin);
        }
    }
    add!(
        "IO_CONFLICTS",
        "I/O pin conflicts",
        if conflicts.is_empty() {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        if conflicts.is_empty() {
            "No duplicate pin assignments in the declared I/O map".into()
        } else {
            format!("Duplicate pin assignments: {}", conflicts.join(", "))
        },
        true,
    );

    add!(
        "PHYSICAL_OUTPUT_INTERLOCK",
        "Physical output interlock",
        if PHYSICAL_OUTPUT_INTERLOCK_ENGAGED { CheckStatus::Fail } else { CheckStatus::Pass },
        "v0.1 has no physical output path. Injector and spark outputs are simulated only. Independent engineering review is required before this interlock may be changed.".into(),
        true,
    );

    let error_count = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Fail)
        .count();
    let warning_count = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Warn)
        .count();
    let hardware_candidate = checks
        .iter()
        .filter(|c| c.blocks_physical)
        .all(|c| c.status != CheckStatus::Fail)
        && !ignition_block;
    let physical_export_allowed = hardware_candidate && !PHYSICAL_OUTPUT_INTERLOCK_ENGAGED;
    let virtual_run_allowed = cal.verify()
        && engine_errors.is_empty()
        && fuel_errors.is_empty()
        && table_ok
        && shape_ok
        && lambda_bounds_ok;
    let summary = if !virtual_run_allowed {
        format!("Virtual run blocked: {error_count} failed checks. Resolve errors before starting the simulator.")
    } else if !physical_export_allowed {
        format!(
            "Virtual run allowed. Physical use blocked ({error_count} blocking checks, {warning_count} warnings). This is not an engine-safe calibration."
        )
    } else {
        "Checks complete. Physical interlock still requires independent validation before any hardware path exists.".into()
    };

    PreflightReport {
        checks,
        error_count,
        warning_count,
        hardware_candidate,
        physical_export_allowed,
        virtual_run_allowed,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wizard::{generate_starter, WizardRequest};

    #[test]
    fn synthetic_calibration_is_virtual_only() {
        let cal = generate_starter(&WizardRequest::default_inline4(), 1).unwrap();
        let report = preflight(&cal);
        assert!(report.virtual_run_allowed);
        assert!(!report.physical_export_allowed);
        assert!(report
            .checks
            .iter()
            .any(|c| c.id == "IGNITION_EVIDENCE" && c.status == CheckStatus::Fail));
        assert!(report
            .checks
            .iter()
            .any(|c| c.id == "PHYSICAL_OUTPUT_INTERLOCK" && c.status == CheckStatus::Fail));
    }

    #[test]
    fn corrupted_checksum_blocks_virtual_run() {
        let mut cal = generate_starter(&WizardRequest::default_inline4(), 1).unwrap();
        cal.notes.push("tampered".into());
        let report = preflight(&cal);
        assert!(!report.virtual_run_allowed);
        assert!(report.error_count >= 1);
    }

    #[test]
    fn every_check_has_human_readable_detail_and_id() {
        let cal = generate_starter(&WizardRequest::default_inline4(), 1).unwrap();
        let report = preflight(&cal);
        assert!(!report.checks.is_empty());
        assert!(report
            .checks
            .iter()
            .all(|c| !c.id.is_empty() && !c.title.is_empty() && !c.detail.is_empty()));
    }
}
