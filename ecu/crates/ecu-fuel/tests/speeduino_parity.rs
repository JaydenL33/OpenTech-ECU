//! Numerical parity pin against Speeduino's documented fuel arithmetic.
//!
//! Speeduino is GPL-2.0 and retained in this repository as the behavioural
//! oracle. These are derived formula comparisons, not a binary build of the C++
//! firmware. No claim of full parity is made.
//!
//! Speeduino `calcPrimaryPulseWidth` uses:
//!   PW = reqFuel * VE% * MAP_multiplier * corrections% + injOpenTime
//! (see `speeduino/fuel_calcs.cpp` and
//! `test/test_fuel/test_pw_calcPrimaryPulseWidth.cpp`).
//!
//! OpenTech-ECU computes fuel from physical air mass and injector flow. At
//! MAP=100 kPa, VE=100%, lambda=1 and matched injector/fuel data, the
//! physical calculation should collapse to the same req-fuel form.

use ecu_fuel::{air_mass_mg, calculate, FuelInputs};
use ecu_types::InjectorConfig;

fn injector() -> InjectorConfig {
    InjectorConfig {
        id: "parity".into(),
        flow_cc_min: 440.0,
        rated_pressure_kpa: 300.0,
        dead_time_ms: 1.0,
        dead_time_curve: vec![(13.8, 1.0)],
        min_pw_ms: 0.5,
        max_duty: 0.95,
        cylinders: vec![1, 2, 3, 4],
    }
}

fn inputs(ve: f32, map_kpa: f32) -> FuelInputs {
    FuelInputs {
        rpm: 3000.0,
        map_kpa,
        iat_k: 293.15,
        tps_pct: 50.0,
        ve_percent: ve,
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
fn physical_fuel_model_matches_speeduino_req_fuel_form_at_reference_conditions() {
    let inj = injector();
    // reqFuel equivalent: pulse time for one cylinder at VE=100, MAP=100,
    // lambda=1, without dead time. Speeduino stores reqFuel in ms (integer
    // tenths in the UI); our equivalent is continuous for comparison.
    let air_mg = air_mass_mg(100.0, 293.15, 100.0, 1998.0, 4).unwrap();
    let fuel_mg = air_mg / 14.7;
    let flow_g_s = 440.0 * 0.745 / 60.0;
    let req_fuel_ms = fuel_mg / flow_g_s;

    for (ve, map, speeduino_multiplier) in [
        (75.0f32, 100.0f32, 1.0f32),
        (50.0, 100.0, 1.0),
        (100.0, 100.0, 1.0),
        (75.0, 80.0, 0.8),
    ] {
        let d = calculate(&inputs(ve, map), &inj).unwrap();
        let expected = req_fuel_ms * (ve / 100.0) * speeduino_multiplier + inj.dead_time_ms;
        assert!(
            (d.pw_ms - expected).abs() < 0.002,
            "ve={ve} map={map}: OpenTech {:.6} ms vs Speeduino-form {:.6} ms",
            d.pw_ms,
            expected
        );
    }
}

/// Speeduino's paired 4-stroke rule halves reqFuel (one squirt per revolution);
/// sequential leaves it unchanged. Vectors are from
/// `test/test_fuel/test_pw_calculateRequiredFuel.cpp` (reqFuel = 11 ms).
#[test]
fn speeduino_required_fuel_paired_rule_matches_upstream_vectors() {
    fn required_fuel_us(req_fuel_ms_x10: u16, four_stroke: bool, sequential: bool) -> u16 {
        let mut req = req_fuel_ms_x10 * 100;
        if four_stroke && !sequential {
            req /= 2;
        }
        req
    }
    assert_eq!(required_fuel_us(11, false, false), 1100);
    assert_eq!(required_fuel_us(11, false, true), 1100);
    assert_eq!(required_fuel_us(11, true, false), 550);
    assert_eq!(required_fuel_us(11, true, true), 1100);
}
