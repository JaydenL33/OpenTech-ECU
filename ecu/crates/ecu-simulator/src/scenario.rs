use serde::{Deserialize, Serialize};
use std::string::String;
use std::vec::Vec;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Phase {
    pub start_ms: u64,
    pub throttle_pct: f32,
    /// If present, a dyno controller holds this RPM by adjusting load torque.
    pub target_rpm: Option<f32>,
    /// Applied load when not dyno-controlled.
    pub load_nm: f32,
    pub note: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimFault {
    WidebandStuckLean,
    WidebandStuckRich,
    WidebandNotReady,
    ExhaustLeak,
    Misfire,
    FuelPressureLow,
    FuelPressureSensorBlind,
    CoolantSensorStale,
    MapSensorStuck,
    TriggerDropout,
    KnockHigh,
    CoolantOverheat,
    InjectorFlowLow,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FaultWindow {
    pub fault: SimFault,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub description: String,
    pub seed: u64,
    pub duration_ms: u64,
    pub sample_hz: u16,
    pub initial_rpm: f32,
    pub start_coolant_c: f32,
    pub ambient_c: f32,
    pub cranking: bool,
    pub dyno_mode: bool,
    /// Multiplicative true-VE error at low load, percent. The ECU's VE table is
    /// wrong by this amount; the Mixture Map should recover it.
    pub true_ve_error_pct: f32,
    pub ve_error_load_gradient_pct: f32,
    pub ve_error_rpm_gradient_pct: f32,
    pub sensor_noise: f32,
    pub exhaust_delay_ms: u16,
    pub wideband_tau_ms: u16,
    pub phases: Vec<Phase>,
    pub faults: Vec<FaultWindow>,
    pub notes: Vec<String>,
}

impl Scenario {
    pub fn phase_at(&self, ms: u64) -> &Phase {
        let mut chosen = &self.phases[0];
        for p in &self.phases {
            if ms >= p.start_ms {
                chosen = p;
            }
        }
        chosen
    }

    pub fn fault_active(&self, fault: SimFault, ms: u64) -> bool {
        self.faults
            .iter()
            .any(|f| f.fault == fault && ms >= f.start_ms && ms < f.end_ms)
    }

    pub fn duration_seconds(&self) -> f32 {
        self.duration_ms as f32 / 1000.0
    }
}

fn idle_phase(start_ms: u64) -> Phase {
    Phase {
        start_ms,
        throttle_pct: 0.0,
        target_rpm: None,
        load_nm: 0.0,
        note: "idle".into(),
    }
}

fn sweep_phase(start_ms: u64, rpm: f32, throttle: f32, note: &str) -> Phase {
    Phase {
        start_ms,
        throttle_pct: throttle,
        target_rpm: Some(rpm),
        load_nm: 0.0,
        note: note.into(),
    }
}

pub fn builtin_scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            id: "idle_warmup".into(),
            name: "Cold crank to idle".into(),
            description: "Cranks from rest, synchronises crank/cam, starts, warms up and settles at idle. Exercises trigger sync, fuel scheduling, wideband readiness and protection recovery."
                .into(),
            seed: 0xC0FFEE,
            duration_ms: 25_000,
            sample_hz: 50,
            initial_rpm: 180.0,
            start_coolant_c: 22.0,
            ambient_c: 22.0,
            cranking: true,
            dyno_mode: false,
            true_ve_error_pct: 3.0,
            ve_error_load_gradient_pct: -1.0,
            ve_error_rpm_gradient_pct: 1.0,
            sensor_noise: 0.003,
            exhaust_delay_ms: 60,
            wideband_tau_ms: 80,
            phases: vec![idle_phase(0)],
            faults: Vec::new(),
            notes: vec![
                "Starter torque is applied until the engine catches.".into(),
                "Wideband heater readiness is delayed to exercise the learning gate.".into(),
            ],
        },
        Scenario {
            id: "mixture_sweep".into(),
            name: "Guided lambda mixture sweep".into(),
            description: "Warm steady-state dyno sweep across load and RPM with a deliberately biased true VE table. Used to demonstrate offline Mixture Map recovery."
                .into(),
            seed: 0xA11CE,
            duration_ms: 42_000,
            sample_hz: 50,
            initial_rpm: 900.0,
            start_coolant_c: 88.0,
            ambient_c: 24.0,
            cranking: false,
            dyno_mode: true,
            true_ve_error_pct: 8.0,
            ve_error_load_gradient_pct: -2.0,
            ve_error_rpm_gradient_pct: 1.5,
            sensor_noise: 0.004,
            exhaust_delay_ms: 60,
            wideband_tau_ms: 80,
            phases: vec![
                sweep_phase(0, 900.0, 0.0, "warm idle"),
                sweep_phase(4_000, 2200.0, 22.0, "light cruise"),
                sweep_phase(10_000, 3000.0, 40.0, "medium load"),
                sweep_phase(16_000, 3500.0, 78.0, "high load"),
                sweep_phase(22_000, 2000.0, 18.0, "low load"),
                sweep_phase(28_000, 4500.0, 55.0, "upper mid"),
                sweep_phase(34_000, 3200.0, 95.0, "WOT boundary (excluded by default)"),
            ],
            faults: Vec::new(),
            notes: vec![
                "Warm start so coolant gates pass; the point is mixture recovery, not warm-up.".into(),
                "WOT phase exists to show explicit exclusion rather than silent learning.".into(),
            ],
        },
        Scenario {
            id: "transient".into(),
            name: "Throttle transients".into(),
            description: "Repeated throttle ramps. Steady-state learning must abstain during transients; closed-loop trim may still operate within its own gates."
                .into(),
            seed: 0xBEE5,
            duration_ms: 28_000,
            sample_hz: 50,
            initial_rpm: 900.0,
            start_coolant_c: 88.0,
            ambient_c: 24.0,
            cranking: false,
            dyno_mode: false,
            true_ve_error_pct: 5.0,
            ve_error_load_gradient_pct: 0.0,
            ve_error_rpm_gradient_pct: 0.0,
            sensor_noise: 0.004,
            exhaust_delay_ms: 60,
            wideband_tau_ms: 80,
            phases: vec![
                Phase { start_ms: 0, throttle_pct: 0.0, target_rpm: None, load_nm: 0.0, note: "idle".into() },
                Phase { start_ms: 4_000, throttle_pct: 55.0, target_rpm: None, load_nm: 60.0, note: "tip-in".into() },
                Phase { start_ms: 8_000, throttle_pct: 0.0, target_rpm: None, load_nm: 5.0, note: "tip-out".into() },
                Phase { start_ms: 12_000, throttle_pct: 80.0, target_rpm: None, load_nm: 90.0, note: "load step".into() },
                Phase { start_ms: 17_000, throttle_pct: 20.0, target_rpm: None, load_nm: 15.0, note: "part load".into() },
                Phase { start_ms: 22_000, throttle_pct: 45.0, target_rpm: None, load_nm: 45.0, note: "settle".into() },
            ],
            faults: Vec::new(),
            notes: vec!["Transients are excluded from offline learning by rate gates.".into()],
        },
        Scenario {
            id: "fault_recovery".into(),
            name: "Fault injection and recovery".into(),
            description: "Over-temperature, low oil pressure, a stuck-lean wideband and a crank dropout. Demonstrates DTCs, latched cuts, freeze frames and recovery."
                .into(),
            seed: 0xFA017,
            duration_ms: 34_000,
            sample_hz: 50,
            initial_rpm: 900.0,
            start_coolant_c: 88.0,
            ambient_c: 24.0,
            cranking: false,
            dyno_mode: false,
            true_ve_error_pct: 4.0,
            ve_error_load_gradient_pct: 0.0,
            ve_error_rpm_gradient_pct: 0.0,
            sensor_noise: 0.003,
            exhaust_delay_ms: 60,
            wideband_tau_ms: 80,
            phases: vec![idle_phase(0)],
            faults: vec![
                FaultWindow { fault: SimFault::CoolantOverheat, start_ms: 6_000, end_ms: 12_000 },
                FaultWindow { fault: SimFault::FuelPressureLow, start_ms: 14_000, end_ms: 17_000 },
                FaultWindow { fault: SimFault::WidebandStuckLean, start_ms: 20_000, end_ms: 24_000 },
                FaultWindow { fault: SimFault::TriggerDropout, start_ms: 27_000, end_ms: 27_500 },
                FaultWindow { fault: SimFault::KnockHigh, start_ms: 29_000, end_ms: 31_000 },
            ],
            notes: vec![
                "Latched faults require explicit clearing through the diagnostics API.".into(),
                "Trigger dropout is short; the decoder times out only if configured timeout is exceeded.".into(),
            ],
        },
        Scenario {
            id: "sensor_stuck_lean".into(),
            name: "Stuck-lean wideband trap".into(),
            description: "A steady mixture sweep with the wideband frozen lean. The offline analyzer must detect a stuck signal and abstain rather than add fuel everywhere."
                .into(),
            seed: 0x57CC,
            duration_ms: 30_000,
            sample_hz: 50,
            initial_rpm: 900.0,
            start_coolant_c: 88.0,
            ambient_c: 24.0,
            cranking: false,
            dyno_mode: true,
            true_ve_error_pct: 2.0,
            ve_error_load_gradient_pct: 0.0,
            ve_error_rpm_gradient_pct: 0.0,
            sensor_noise: 0.0,
            exhaust_delay_ms: 60,
            wideband_tau_ms: 80,
            phases: vec![
                sweep_phase(0, 1500.0, 20.0, "low"),
                sweep_phase(6_000, 3000.0, 45.0, "mid"),
                sweep_phase(12_000, 4200.0, 80.0, "high"),
                sweep_phase(18_000, 2500.0, 30.0, "low-mid"),
                sweep_phase(24_000, 3500.0, 60.0, "mid-high"),
            ],
            faults: vec![FaultWindow {
                fault: SimFault::WidebandStuckLean,
                start_ms: 3_000,
                end_ms: 30_000,
            }],
            notes: vec!["No ignition estimates are produced by this scenario.".into()],
        },
        Scenario {
            id: "trigger_loss".into(),
            name: "Crank signal loss".into(),
            description: "Idle with two long crank-tooth dropouts that exceed the decoder timeout, verifying sync loss, fuel/spark cut and re-synchronisation."
                .into(),
            seed: 0x7A16,
            duration_ms: 22_000,
            sample_hz: 50,
            initial_rpm: 900.0,
            start_coolant_c: 88.0,
            ambient_c: 24.0,
            cranking: false,
            dyno_mode: false,
            true_ve_error_pct: 3.0,
            ve_error_load_gradient_pct: 0.0,
            ve_error_rpm_gradient_pct: 0.0,
            sensor_noise: 0.003,
            exhaust_delay_ms: 60,
            wideband_tau_ms: 80,
            phases: vec![idle_phase(0)],
            faults: vec![
                FaultWindow { fault: SimFault::TriggerDropout, start_ms: 6_000, end_ms: 7_000 },
                FaultWindow { fault: SimFault::TriggerDropout, start_ms: 12_000, end_ms: 13_000 },
            ],
            notes: Vec::new(),
        },
    ]
}

pub fn by_id(id: &str) -> Option<Scenario> {
    builtin_scenarios().into_iter().find(|s| s.id == id)
}

/// SplitMix64. Deterministic, dependency-free and adequate for simulation noise,
/// not cryptography.
#[derive(Clone, Debug)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [-1, 1].
    pub fn bipolar(&mut self) -> f32 {
        let v = (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32;
        v * 2.0 - 1.0
    }

    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_have_unique_ids_and_sorted_phases() {
        let all = builtin_scenarios();
        let mut ids = Vec::new();
        for s in &all {
            assert!(!ids.contains(&s.id));
            ids.push(s.id.clone());
            assert!(!s.phases.is_empty());
            assert!(s.phases.windows(2).all(|w| w[0].start_ms < w[1].start_ms));
            assert!(s.duration_ms > 0);
        }
    }

    #[test]
    fn phase_lookup_is_last_started_phase() {
        let s = by_id("mixture_sweep").unwrap();
        assert_eq!(s.phase_at(0).target_rpm, Some(900.0));
        assert_eq!(s.phase_at(4_500).target_rpm, Some(2200.0));
        assert_eq!(s.phase_at(100_000).target_rpm, Some(3200.0));
    }

    #[test]
    fn prng_is_repeatable_and_bounded() {
        let mut a = DeterministicRng::new(42);
        let mut b = DeterministicRng::new(42);
        for _ in 0..1000 {
            let x = a.bipolar();
            let y = b.bipolar();
            assert_eq!(x, y);
            assert!((-1.0..=1.0).contains(&x));
        }
    }

    #[test]
    fn faults_are_windowed() {
        let s = by_id("fault_recovery").unwrap();
        assert!(!s.fault_active(SimFault::CoolantOverheat, 0));
        assert!(s.fault_active(SimFault::CoolantOverheat, 8_000));
        assert!(!s.fault_active(SimFault::CoolantOverheat, 13_000));
    }
}
