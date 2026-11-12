//! Speed-density fuel calculation.
//!
//! Formula (documented in docs/FORMULAS.md):
//!   rho_air   = MAP[Pa] / (R_air * IAT[K])              [kg/m^3]
//!   m_air     = rho_air * Vd_cyl[m^3] * VE/100          [kg / 720 deg]
//!   m_fuel    = m_air / (stoich_AFR * target_lambda)    [kg / 720 deg]
//!   flow      = rated_cc_min * density * pressure_factor [g/s]
//!   PW        = m_fuel/flow*1000 + dead_time(volts)     [ms]
//!   duty      = PW / (120000/RPM)                       [fraction]
//!
//! Assumptions and limits:
//! - stoichiometric AFR is fuel-specific and supplied by calibration;
//! - ideal gas, one intake event per cylinder per 720 crank degrees;
//! - no wall-wetting compensation in v1;
//! - pressure factor is sqrt(actual/rated) for a returnless system and is only
//!   applied when calibrated on;
//! - the function returns `Inhibited` rather than guessing when inputs are not
//!   trustworthy or duty is excessive.
//!
//! This module contains no lambda-feedback logic. Trims are explicit inputs so
//! the caller can log and audit exactly what was applied.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

use core::fmt;

use ecu_types::{InjectorConfig, LoadStrategy};
use serde::{Deserialize, Serialize};

pub const R_SPECIFIC_AIR: f32 = 287.05; // J/(kg*K)
pub const MAX_VE_PERCENT: f32 = 150.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuelInputs {
    pub rpm: f32,
    pub map_kpa: f32,
    pub iat_k: f32,
    pub tps_pct: f32,
    /// Volumetric efficiency, percent (100 = nominal displacement filling).
    pub ve_percent: f32,
    pub target_lambda: f32,
    pub stoich_afr: f32,
    /// Fuel density in g/cc; gasoline is about 0.745, E85 about 0.785.
    pub fuel_density_g_cc: f32,
    pub battery_v: f32,
    pub fuel_pressure_kpa: f32,
    /// Short-term trim as a fraction (0.05 = +5%).
    pub stft: f32,
    /// Long-term trim as a fraction.
    pub ltft: f32,
    pub displacement_cc: f32,
    pub cylinders: u8,
    pub pressure_compensation: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuelDecision {
    pub air_mass_mg: f32,
    pub requested_fuel_mg: f32,
    /// Fuel mass the injector can actually deliver with the clamped pulse.
    /// At minimum pulse width this can be greater than requested.
    pub delivered_fuel_mg: f32,
    pub pw_ms: f32,
    /// PW before the minimum-pulse clamp, for diagnostics.
    pub pw_uncorrected_ms: f32,
    pub dead_time_ms: f32,
    pub effective_flow_cc_min: f32,
    pub duty_percent: f32,
    pub total_trim: f32,
    pub min_pulse_clamped: bool,
    pub inhibited: Option<InhibitReason>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InhibitReason {
    EngineNotTurning,
    InvalidRpm,
    InvalidMap,
    InvalidAirTemperature,
    InvalidVe,
    InvalidTargetLambda,
    InvalidStoichiometry,
    InvalidBatteryVoltage,
    InvalidFuelPressure,
    InvalidInjector,
    InvalidTrim,
    DutyExceeded,
    NoCylinders,
}

impl fmt::Display for InhibitReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            InhibitReason::EngineNotTurning => "engine not turning",
            InhibitReason::InvalidRpm => "invalid RPM",
            InhibitReason::InvalidMap => "invalid MAP",
            InhibitReason::InvalidAirTemperature => "invalid IAT",
            InhibitReason::InvalidVe => "invalid VE",
            InhibitReason::InvalidTargetLambda => "invalid target lambda",
            InhibitReason::InvalidStoichiometry => "invalid stoichiometric AFR",
            InhibitReason::InvalidBatteryVoltage => "invalid battery voltage",
            InhibitReason::InvalidFuelPressure => "invalid fuel pressure",
            InhibitReason::InvalidInjector => "invalid injector characterisation",
            InhibitReason::InvalidTrim => "trim outside permitted bounds",
            InhibitReason::DutyExceeded => "injector duty limit exceeded",
            InhibitReason::NoCylinders => "cylinder count is zero",
        };
        f.write_str(s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FuelError {
    Inhibited(InhibitReason),
}

impl fmt::Display for FuelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FuelError::Inhibited(r) => write!(f, "fuel inhibited: {r}"),
        }
    }
}

/// Air mass in mg per cylinder per 720-degree cycle.
pub fn air_mass_mg(
    map_kpa: f32,
    iat_k: f32,
    ve_percent: f32,
    displacement_cc: f32,
    cylinders: u8,
) -> Result<f32, FuelError> {
    if cylinders == 0 {
        return Err(FuelError::Inhibited(InhibitReason::NoCylinders));
    }
    if !in_range(map_kpa, 5.0, 600.0) {
        return Err(FuelError::Inhibited(InhibitReason::InvalidMap));
    }
    if !in_range(iat_k, 200.0, 400.0) {
        return Err(FuelError::Inhibited(InhibitReason::InvalidAirTemperature));
    }
    if !in_range(ve_percent, 1.0, MAX_VE_PERCENT) {
        return Err(FuelError::Inhibited(InhibitReason::InvalidVe));
    }
    if !in_range(displacement_cc, 50.0, 20_000.0) {
        return Err(FuelError::Inhibited(InhibitReason::NoCylinders));
    }
    let rho = (map_kpa * 1000.0) / (R_SPECIFIC_AIR * iat_k);
    let vd_cyl_m3 = displacement_cc * 1e-6 / cylinders as f32;
    Ok(rho * vd_cyl_m3 * (ve_percent / 100.0) * 1_000_000.0)
}

/// Compute the injector pulse and delivered fuel for one cylinder.
/// `injector` describes a single primary injector feeding one cylinder. Staged
/// and batch arrangements must call this per supported injector and sum masses.
pub fn calculate(
    inputs: &FuelInputs,
    injector: &InjectorConfig,
) -> Result<FuelDecision, FuelError> {
    if !in_range(inputs.rpm, 1.0, 20_000.0) {
        return Err(FuelError::Inhibited(if inputs.rpm == 0.0 {
            InhibitReason::EngineNotTurning
        } else {
            InhibitReason::InvalidRpm
        }));
    }
    if !in_range(inputs.stoich_afr, 8.0, 20.0) {
        return Err(FuelError::Inhibited(InhibitReason::InvalidStoichiometry));
    }
    if !in_range(inputs.fuel_density_g_cc, 0.65, 0.9) {
        return Err(FuelError::Inhibited(InhibitReason::InvalidStoichiometry));
    }
    if !in_range(inputs.target_lambda, 0.5, 1.5) {
        return Err(FuelError::Inhibited(InhibitReason::InvalidTargetLambda));
    }
    if !in_range(inputs.battery_v, 6.0, 18.0) {
        return Err(FuelError::Inhibited(InhibitReason::InvalidBatteryVoltage));
    }
    if !in_range(inputs.fuel_pressure_kpa, 50.0, 800.0) {
        return Err(FuelError::Inhibited(InhibitReason::InvalidFuelPressure));
    }
    if !in_range(inputs.stft, -0.6, 0.6) || !in_range(inputs.ltft, -0.6, 0.6) {
        return Err(FuelError::Inhibited(InhibitReason::InvalidTrim));
    }
    if !in_range(injector.flow_cc_min, 10.0, 5000.0)
        || !in_range(injector.rated_pressure_kpa, 50.0, 1000.0)
        || !in_range(injector.dead_time_ms, 0.0, 10.0)
        || !in_range(injector.max_duty, 0.05, 1.0)
    {
        return Err(FuelError::Inhibited(InhibitReason::InvalidInjector));
    }

    let air_mg = air_mass_mg(
        inputs.map_kpa,
        inputs.iat_k,
        inputs.ve_percent,
        inputs.displacement_cc,
        inputs.cylinders,
    )?;
    let base_fuel_mg = air_mg / (inputs.stoich_afr * inputs.target_lambda);
    let total_trim = 1.0 + inputs.stft + inputs.ltft;
    // A trim that would command <=0 is not a usable correction.
    if total_trim <= 0.05 {
        return Err(FuelError::Inhibited(InhibitReason::InvalidTrim));
    }
    let requested_fuel_mg = base_fuel_mg * total_trim;

    let pressure_factor = if inputs.pressure_compensation {
        if inputs.fuel_pressure_kpa <= 0.0 {
            return Err(FuelError::Inhibited(InhibitReason::InvalidFuelPressure));
        }
        libm::sqrtf(inputs.fuel_pressure_kpa / injector.rated_pressure_kpa)
    } else {
        1.0
    };
    let effective_flow_cc_min = injector.flow_cc_min * pressure_factor;

    let flow_g_s = effective_flow_cc_min * inputs.fuel_density_g_cc / 60.0;
    if !flow_g_s.is_finite() || flow_g_s <= 0.0 {
        return Err(FuelError::Inhibited(InhibitReason::InvalidInjector));
    }
    let dead_time = injector.dead_time_at(inputs.battery_v).clamp(0.0, 10.0);
    let pw_uncorrected = requested_fuel_mg / flow_g_s + dead_time;
    let pw = pw_uncorrected.max(injector.min_pw_ms).max(dead_time);
    let min_pulse_clamped = pw > pw_uncorrected + 1e-6;
    let delivered_fuel_mg = ((pw - dead_time).max(0.0) * flow_g_s).max(0.0);

    let cycle_ms = 120_000.0 / inputs.rpm;
    let duty = pw / cycle_ms;
    if duty > injector.max_duty {
        // Return an inhibited decision carrying the calculated values so the UI
        // and protection layer can show exactly what exceeded the limit.
        return Ok(FuelDecision {
            air_mass_mg: air_mg,
            requested_fuel_mg,
            delivered_fuel_mg,
            pw_ms: pw,
            pw_uncorrected_ms: pw_uncorrected,
            dead_time_ms: dead_time,
            effective_flow_cc_min,
            duty_percent: duty * 100.0,
            total_trim,
            min_pulse_clamped,
            inhibited: Some(InhibitReason::DutyExceeded),
        });
    }

    Ok(FuelDecision {
        air_mass_mg: air_mg,
        requested_fuel_mg,
        delivered_fuel_mg,
        pw_ms: pw,
        pw_uncorrected_ms: pw_uncorrected,
        dead_time_ms: dead_time,
        effective_flow_cc_min,
        duty_percent: duty * 100.0,
        total_trim,
        min_pulse_clamped,
        inhibited: None,
    })
}

/// Pressure-corrected flow used by injector sizing and diagnostics.
pub fn effective_flow_cc_min(
    rated_flow: f32,
    rated_pressure_kpa: f32,
    actual_pressure_kpa: f32,
) -> Option<f32> {
    if !rated_flow.is_finite()
        || rated_flow <= 0.0
        || !rated_pressure_kpa.is_finite()
        || rated_pressure_kpa <= 0.0
        || !actual_pressure_kpa.is_finite()
        || actual_pressure_kpa <= 0.0
    {
        return None;
    }
    Some(rated_flow * libm::sqrtf(actual_pressure_kpa / rated_pressure_kpa))
}

/// Blend speed-density MAP with alpha-N TPS. This is load selection, not a
/// correction factor.
pub fn blended_load(map_kpa: f32, tps_pct: f32, strategy: LoadStrategy) -> Result<f32, FuelError> {
    if !in_range(map_kpa, 5.0, 600.0) || !in_range(tps_pct, 0.0, 100.0) {
        return Err(FuelError::Inhibited(InhibitReason::InvalidMap));
    }
    Ok(match strategy {
        LoadStrategy::SpeedDensity => map_kpa,
        LoadStrategy::AlphaN => tps_pct,
        LoadStrategy::Blended { alpha_percent } => {
            let a = (alpha_percent.min(100) as f32) / 100.0;
            map_kpa * (1.0 - a) + tps_pct * a
        }
    })
}

#[inline]
fn in_range(v: f32, lo: f32, hi: f32) -> bool {
    v.is_finite() && v >= lo && v <= hi
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecu_types::InjectorConfig;

    fn injector() -> InjectorConfig {
        InjectorConfig {
            id: "test".into(),
            flow_cc_min: 440.0,
            rated_pressure_kpa: 300.0,
            dead_time_ms: 1.0,
            dead_time_curve: vec![(10.0, 1.5), (13.8, 1.0), (15.0, 0.8)],
            min_pw_ms: 0.5,
            max_duty: 0.9,
            cylinders: vec![1],
        }
    }

    fn inputs() -> FuelInputs {
        FuelInputs {
            rpm: 3000.0,
            map_kpa: 100.0,
            iat_k: 293.15,
            tps_pct: 50.0,
            ve_percent: 100.0,
            target_lambda: 1.0,
            stoich_afr: 14.7,
            fuel_density_g_cc: 0.745,
            battery_v: 13.8,
            fuel_pressure_kpa: 300.0,
            stft: 0.0,
            ltft: 0.0,
            displacement_cc: 1998.0,
            cylinders: 4,
            pressure_compensation: true,
        }
    }

    #[test]
    fn air_mass_matches_hand_calculation() {
        let m = air_mass_mg(100.0, 293.15, 100.0, 2000.0, 4).unwrap();
        // rho=1.1885 kg/m3, Vd_cyl=0.0005 m3, mass=0.5943 g
        assert!((m - 594.3).abs() < 1.0);
    }

    #[test]
    fn injector_pulse_matches_hand_calculation() {
        let d = calculate(&inputs(), &injector()).unwrap();
        // fuel = 594.3mg / 14.7 = 40.4mg; flow = 440*0.745/60 = 5.463 g/s
        // fuel time = 7.40ms; +1.0ms dead time
        assert!((d.pw_ms - 8.40).abs() < 0.1, "pw={}", d.pw_ms);
        assert!(
            (d.duty_percent - 21.0).abs() < 0.5,
            "duty={}",
            d.duty_percent
        );
        assert!(d.inhibited.is_none());
        assert!((d.delivered_fuel_mg - 40.4).abs() < 0.2);
    }

    #[test]
    fn pressure_compensation_increases_flow_and_shortens_pulse() {
        let mut low = inputs();
        low.fuel_pressure_kpa = 200.0;
        let d = calculate(&low, &injector()).unwrap();
        assert!(d.effective_flow_cc_min < 440.0);
        assert!(d.pw_ms > 8.4);
    }

    #[test]
    fn dead_time_increases_at_low_voltage() {
        let mut low = inputs();
        low.battery_v = 10.0;
        let d = calculate(&low, &injector()).unwrap();
        assert!((d.dead_time_ms - 1.5).abs() < 1e-3);
        assert!(d.pw_ms > 8.8);
    }

    #[test]
    fn positive_trim_adds_fuel() {
        let mut rich = inputs();
        rich.stft = 0.10;
        let d = calculate(&rich, &injector()).unwrap();
        assert!((d.total_trim - 1.10).abs() < 1e-5);
        assert!(d.requested_fuel_mg > 40.4);
    }

    #[test]
    fn excessive_duty_is_reported_not_silently_clamped() {
        let mut hard = inputs();
        hard.rpm = 7000.0;
        hard.ve_percent = 105.0;
        hard.map_kpa = 180.0;
        hard.target_lambda = 0.8;
        let d = calculate(&hard, &injector()).unwrap();
        assert_eq!(d.inhibited, Some(InhibitReason::DutyExceeded));
        assert!(d.duty_percent > 90.0);
    }

    #[test]
    fn invalid_inputs_are_inhibited() {
        let mut bad = inputs();
        bad.target_lambda = 0.1;
        assert!(matches!(
            calculate(&bad, &injector()),
            Err(FuelError::Inhibited(_))
        ));
        let mut bad = inputs();
        bad.rpm = 0.0;
        assert_eq!(
            calculate(&bad, &injector()).unwrap_err(),
            FuelError::Inhibited(InhibitReason::EngineNotTurning)
        );
    }

    #[test]
    fn minimum_pulse_is_flagged_and_delivers_more_than_requested() {
        let mut idle = inputs();
        idle.rpm = 6000.0;
        idle.map_kpa = 20.0;
        idle.ve_percent = 30.0;
        let mut inj = injector();
        inj.min_pw_ms = 3.0;
        let d = calculate(&idle, &inj).unwrap();
        assert!(d.min_pulse_clamped);
        assert!(d.delivered_fuel_mg >= d.requested_fuel_mg);
    }

    #[test]
    fn blended_load_is_bounded() {
        assert_eq!(
            blended_load(80.0, 40.0, LoadStrategy::SpeedDensity).unwrap(),
            80.0
        );
        assert_eq!(
            blended_load(80.0, 40.0, LoadStrategy::AlphaN).unwrap(),
            40.0
        );
        let b = blended_load(80.0, 40.0, LoadStrategy::Blended { alpha_percent: 50 }).unwrap();
        assert!((b - 60.0).abs() < 1e-5);
    }

    #[test]
    fn air_mass_per_cylinder_scales_with_cylinder_count() {
        let base = air_mass_mg(100.0, 293.15, 100.0, 1998.0, 1).unwrap();
        for n in 1..=8 {
            assert!(
                (air_mass_mg(100.0, 293.15, 100.0, 1998.0, n).unwrap() * n as f32 - base).abs()
                    / base
                    < 1e-5
            );
        }
    }
}
