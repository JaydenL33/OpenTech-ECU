use serde::{Deserialize, Serialize};
use std::string::String;
use std::vec::Vec;

use ecu_types::SensorStatus;

/// Conditions that must hold before any lambda-derived correction is used or
/// learned. The same gate is applied to online trim and offline mixture-map
/// analysis so the reasons are comparable between workflows.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GateConfig {
    pub min_coolant_c: f32,
    pub max_coolant_c: f32,
    pub min_rpm: f32,
    pub max_lambda_age_ms: u32,
    pub min_lambda: f32,
    pub max_lambda: f32,
    pub max_rpm_rate_rpm_s: f32,
    pub max_map_rate_kpa_s: f32,
    pub max_tps_rate_pct_s: f32,
    pub max_injector_duty_pct: f32,
    pub require_sequential_sync: bool,
    pub require_steady_state: bool,
    pub allow_wot_adaptation: bool,
    pub wot_tps_pct: f32,
    pub wot_map_kpa: f32,
    pub require_fuel_pressure_ok: bool,
    pub require_oil_pressure_ok: bool,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            min_coolant_c: 70.0,
            max_coolant_c: 110.0,
            min_rpm: 500.0,
            max_lambda_age_ms: 300,
            min_lambda: 0.70,
            max_lambda: 1.30,
            max_rpm_rate_rpm_s: 400.0,
            max_map_rate_kpa_s: 8.0,
            max_tps_rate_pct_s: 5.0,
            max_injector_duty_pct: 92.0,
            require_sequential_sync: true,
            require_steady_state: true,
            allow_wot_adaptation: false,
            wot_tps_pct: 85.0,
            wot_map_kpa: 100.0,
            require_fuel_pressure_ok: true,
            require_oil_pressure_ok: false,
        }
    }
}

impl GateConfig {
    pub fn for_closed_loop() -> Self {
        Self {
            require_steady_state: false,
            require_oil_pressure_ok: true,
            ..Self::default()
        }
    }
}

/// Snapshot of all conditions evaluated at one sample. Flags are booleans
/// because upstream diagnostics own the detection thresholds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GateInputs {
    pub engine_running: bool,
    pub cranking: bool,
    pub sync_sequential: bool,
    pub coolant_c: f32,
    pub rpm: f32,
    pub map_kpa: f32,
    pub tps_pct: f32,
    pub lambda: f32,
    pub target_lambda: f32,
    pub lambda_status: SensorStatus,
    pub lambda_ready: bool,
    pub lambda_age_ms: u32,
    pub fuel_cut: bool,
    pub overrun: bool,
    pub misfire: bool,
    pub exhaust_leak: bool,
    pub knock_active: bool,
    pub overboost: bool,
    pub critical_dtc: bool,
    pub sensor_stale: bool,
    pub fuel_pressure_ok: bool,
    pub oil_pressure_ok: bool,
    pub injector_duty_pct: f32,
    pub rpm_rate_rpm_s: f32,
    pub map_rate_kpa_s: f32,
    pub tps_rate_pct_s: f32,
}

impl Default for GateInputs {
    fn default() -> Self {
        Self {
            engine_running: false,
            cranking: false,
            sync_sequential: false,
            coolant_c: 20.0,
            rpm: 0.0,
            map_kpa: 0.0,
            tps_pct: 0.0,
            lambda: f32::NAN,
            target_lambda: 1.0,
            lambda_status: SensorStatus::NotReady,
            lambda_ready: false,
            lambda_age_ms: 0,
            fuel_cut: false,
            overrun: false,
            misfire: false,
            exhaust_leak: false,
            knock_active: false,
            overboost: false,
            critical_dtc: false,
            sensor_stale: false,
            fuel_pressure_ok: false,
            oil_pressure_ok: true,
            injector_duty_pct: 0.0,
            rpm_rate_rpm_s: 0.0,
            map_rate_kpa_s: 0.0,
            tps_rate_pct_s: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GateDecision {
    pub allowed: bool,
    pub reasons: Vec<GateReason>,
    pub high_load: bool,
}

impl GateDecision {
    pub fn blocked(reasons: Vec<GateReason>) -> Self {
        Self {
            allowed: false,
            reasons,
            high_load: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateReason {
    pub code: String,
    pub message: String,
}

impl GateReason {
    fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

pub fn evaluate(cfg: &GateConfig, i: &GateInputs) -> GateDecision {
    let mut reasons = Vec::new();
    macro_rules! add {
        ($code:expr, $message:expr $(,)?) => {
            reasons.push(GateReason::new($code, $message))
        };
    }

    if !i.engine_running || i.cranking {
        add!("ENGINE_NOT_RUNNING", "engine is not running steadily");
    }
    if cfg.require_sequential_sync && !i.sync_sequential {
        add!(
            "SYNC_NOT_SEQUENTIAL",
            "crank/cam phase is not sequentially synchronised"
        );
    }
    if i.coolant_c < cfg.min_coolant_c {
        add!("COOLANT_COLD", "coolant is below the learning temperature");
    }
    if i.coolant_c > cfg.max_coolant_c {
        add!(
            "COOLANT_HOT",
            "coolant is above the learning temperature window"
        );
    }
    if !i.rpm.is_finite() || i.rpm < cfg.min_rpm {
        add!("RPM_LOW", "RPM is below the learning threshold");
    }
    if !i.lambda_ready {
        add!("LAMBDA_NOT_READY", "wideband controller is not ready");
    }
    if i.lambda_status != SensorStatus::Ok {
        add!("LAMBDA_STATUS", "wideband sensor status is not valid");
    }
    if i.lambda_age_ms > cfg.max_lambda_age_ms {
        add!("LAMBDA_STALE", "wideband sample is older than permitted");
    }
    if !i.lambda.is_finite() || i.lambda < cfg.min_lambda || i.lambda > cfg.max_lambda {
        add!("LAMBDA_IMPLAUSIBLE", "lambda is outside plausible bounds");
    }
    if i.lambda_status == SensorStatus::Saturated {
        add!("LAMBDA_SATURATED", "wideband signal is saturated");
    }
    if i.fuel_cut {
        add!("FUEL_CUT", "fuel cut is active");
    }
    if i.overrun {
        add!("OVERRUN", "overrun fuel cut / no-load condition");
    }
    if i.misfire {
        add!("MISFIRE", "misfire indication can read false-lean");
    }
    if i.exhaust_leak {
        add!(
            "EXHAUST_LEAK",
            "exhaust leak indication can read false-lean"
        );
    }
    if i.knock_active {
        add!("KNOCK", "knock is active; adaptation suspended");
    }
    if i.overboost {
        add!("OVERBOOST", "overboost protection is active");
    }
    if i.critical_dtc {
        add!(
            "CRITICAL_DTC",
            "an active critical diagnostic fault is present"
        );
    }
    if i.sensor_stale {
        add!("SENSOR_STALE", "a required sensor is stale or invalid");
    }
    if cfg.require_fuel_pressure_ok && !i.fuel_pressure_ok {
        add!("FUEL_PRESSURE", "fuel pressure is not plausible");
    }
    if cfg.require_oil_pressure_ok && !i.oil_pressure_ok {
        add!("OIL_PRESSURE", "oil pressure is not plausible");
    }
    if !i.injector_duty_pct.is_finite() || i.injector_duty_pct > cfg.max_injector_duty_pct {
        add!(
            "INJECTOR_DUTY",
            "injector duty is too high to infer mixture error"
        );
    }
    if cfg.require_steady_state {
        if !i.rpm_rate_rpm_s.is_finite() || i.rpm_rate_rpm_s.abs() > cfg.max_rpm_rate_rpm_s {
            add!("RPM_TRANSIENT", "RPM is not steady");
        }
        if !i.map_rate_kpa_s.is_finite() || i.map_rate_kpa_s.abs() > cfg.max_map_rate_kpa_s {
            add!("MAP_TRANSIENT", "MAP is not steady");
        }
        if !i.tps_rate_pct_s.is_finite() || i.tps_rate_pct_s.abs() > cfg.max_tps_rate_pct_s {
            add!("TPS_TRANSIENT", "throttle is not steady");
        }
    }

    let high_load = i.tps_pct >= cfg.wot_tps_pct || i.map_kpa >= cfg.wot_map_kpa;
    if high_load && !cfg.allow_wot_adaptation {
        add!(
            "WOT_BOUNDARY",
            "high-load/WOT cell excluded unless explicitly authorised",
        );
    }

    GateDecision {
        allowed: reasons.is_empty(),
        reasons,
        high_load,
    }
}

/// Convenience iterator for reason codes used in UI counters.
pub fn reason_codes(decision: &GateDecision) -> Vec<String> {
    decision.reasons.iter().map(|r| r.code.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> GateInputs {
        GateInputs {
            engine_running: true,
            sync_sequential: true,
            coolant_c: 88.0,
            rpm: 2500.0,
            map_kpa: 55.0,
            tps_pct: 20.0,
            lambda: 1.0,
            target_lambda: 1.0,
            lambda_status: SensorStatus::Ok,
            lambda_ready: true,
            lambda_age_ms: 30,
            fuel_pressure_ok: true,
            injector_duty_pct: 30.0,
            rpm_rate_rpm_s: 0.0,
            map_rate_kpa_s: 0.0,
            tps_rate_pct_s: 0.0,
            ..GateInputs::default()
        }
    }

    #[test]
    fn clean_steady_sample_is_allowed() {
        let d = evaluate(&GateConfig::default(), &good());
        assert!(d.allowed, "{:?}", d.reasons);
    }

    #[test]
    fn every_invalidity_is_explained() {
        let mut i = good();
        i.lambda_ready = false;
        i.lambda_status = SensorStatus::Stale;
        i.lambda_age_ms = 1000;
        i.misfire = true;
        i.exhaust_leak = true;
        i.fuel_cut = true;
        i.knock_active = true;
        i.critical_dtc = true;
        let d = evaluate(&GateConfig::default(), &i);
        assert!(!d.allowed);
        let codes = reason_codes(&d);
        for expected in [
            "LAMBDA_NOT_READY",
            "LAMBDA_STATUS",
            "LAMBDA_STALE",
            "MISFIRE",
            "EXHAUST_LEAK",
            "FUEL_CUT",
            "KNOCK",
            "CRITICAL_DTC",
        ] {
            assert!(
                codes.iter().any(|c| c == expected),
                "missing {expected}: {codes:?}"
            );
        }
    }

    #[test]
    fn high_load_is_excluded_by_default_and_opt_in() {
        let mut i = good();
        i.map_kpa = 120.0;
        i.tps_pct = 95.0;
        let d = evaluate(&GateConfig::default(), &i);
        assert!(!d.allowed);
        assert!(reason_codes(&d).contains(&"WOT_BOUNDARY".to_string()));
        let cfg = GateConfig {
            allow_wot_adaptation: true,
            ..GateConfig::default()
        };
        let d = evaluate(&cfg, &i);
        assert!(d.allowed, "{:?}", d.reasons);
        assert!(d.high_load);
    }

    #[test]
    fn transient_checks_only_apply_for_learning() {
        let mut i = good();
        i.map_rate_kpa_s = 80.0;
        assert!(!evaluate(&GateConfig::default(), &i).allowed);
        let cfg = GateConfig::for_closed_loop();
        assert!(
            evaluate(&cfg, &i).allowed,
            "{:?}",
            evaluate(&cfg, &i).reasons
        );
    }

    #[test]
    fn nan_lambda_rejected() {
        let mut i = good();
        i.lambda = f32::NAN;
        assert!(!evaluate(&GateConfig::default(), &i).allowed);
    }
}
