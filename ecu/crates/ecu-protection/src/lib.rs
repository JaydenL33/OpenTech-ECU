//! Independent engine protection and diagnostic state.
//!
//! The controller is deliberately separate from fuel/ignition calculation. It
//! consumes measurements and returns bounded reactions plus DTC/freeze-frame
//! records. A fault that requires latching remains active until an explicit
//! clear, even if the condition disappears; this avoids silent recovery from
//! low oil pressure, over-temperature or over-boost.
//!
//! This is a simulator/bench implementation. Thresholds are configurable
//! engineering placeholders and are not validated engine limits.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

use core::fmt;
use serde::{Deserialize, Serialize};

pub const MAX_FAULTS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dtc {
    CrankSyncLost,
    CoolantOverTemperature,
    LowOilPressure,
    LowFuelPressure,
    LeanUnderLoad,
    Overboost,
    LowBatteryVoltage,
    WidebandNotReady,
    SensorPlausibility,
    CalibrationChecksum,
    Watchdog,
    CanBusLost,
    InjectorDutyHigh,
    KnockHigh,
}

impl Dtc {
    pub const ALL: [Dtc; 14] = [
        Dtc::CrankSyncLost,
        Dtc::CoolantOverTemperature,
        Dtc::LowOilPressure,
        Dtc::LowFuelPressure,
        Dtc::LeanUnderLoad,
        Dtc::Overboost,
        Dtc::LowBatteryVoltage,
        Dtc::WidebandNotReady,
        Dtc::SensorPlausibility,
        Dtc::CalibrationChecksum,
        Dtc::Watchdog,
        Dtc::CanBusLost,
        Dtc::InjectorDutyHigh,
        Dtc::KnockHigh,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Dtc::CrankSyncLost => "P0335",
            Dtc::CoolantOverTemperature => "P0217",
            Dtc::LowOilPressure => "P0524",
            Dtc::LowFuelPressure => "P0087",
            Dtc::LeanUnderLoad => "P0171",
            Dtc::Overboost => "P0234",
            Dtc::LowBatteryVoltage => "P0562",
            Dtc::WidebandNotReady => "P0133",
            Dtc::SensorPlausibility => "P0113",
            Dtc::CalibrationChecksum => "P0601",
            Dtc::Watchdog => "P0606",
            Dtc::CanBusLost => "U0100",
            Dtc::InjectorDutyHigh => "P0213",
            Dtc::KnockHigh => "P0325",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Dtc::CrankSyncLost => "Crank/cam sync lost",
            Dtc::CoolantOverTemperature => "Coolant over-temperature",
            Dtc::LowOilPressure => "Low oil pressure",
            Dtc::LowFuelPressure => "Low fuel pressure",
            Dtc::LeanUnderLoad => "Lean mixture under load",
            Dtc::Overboost => "Overboost",
            Dtc::LowBatteryVoltage => "Low battery voltage",
            Dtc::WidebandNotReady => "Wideband not ready",
            Dtc::SensorPlausibility => "Sensor plausibility failure",
            Dtc::CalibrationChecksum => "Calibration checksum failure",
            Dtc::Watchdog => "Watchdog fault",
            Dtc::CanBusLost => "CAN bus lost",
            Dtc::InjectorDutyHigh => "Injector duty too high",
            Dtc::KnockHigh => "Knock level high",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectionState {
    Normal,
    Warning,
    Derated,
    Limp,
    Cut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaultAction {
    Warn,
    Derate,
    CutFuel,
    CutAll,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FreezeFrame {
    pub timestamp_us: u64,
    pub rpm: f32,
    pub map_kpa: f32,
    pub tps_pct: f32,
    pub coolant_c: f32,
    pub oil_pressure_kpa: f32,
    pub fuel_pressure_kpa: f32,
    pub battery_v: f32,
    pub lambda: f32,
    pub target_lambda: f32,
    pub injector_duty_pct: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActiveFault {
    pub dtc: Dtc,
    pub action: FaultAction,
    pub latched: bool,
    pub condition_active: bool,
    pub first_seen_us: u64,
    pub last_seen_us: u64,
    pub clear_since_us: Option<u64>,
    pub occurrences: u32,
    pub freeze_frame: FreezeFrame,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProtectionInputs {
    pub rpm: f32,
    pub map_kpa: f32,
    pub tps_pct: f32,
    pub coolant_c: f32,
    pub oil_pressure_kpa: f32,
    pub fuel_pressure_kpa: f32,
    pub battery_v: f32,
    pub lambda: f32,
    pub target_lambda: f32,
    pub injector_duty_pct: f32,
    pub knock_level: f32,
    pub sync_ok: bool,
    pub wideband_ready: bool,
    pub sensor_plausible: bool,
    pub calibration_crc_ok: bool,
    pub watchdog_ok: bool,
    pub can_ok: bool,
}

impl Default for ProtectionInputs {
    fn default() -> Self {
        Self {
            rpm: 0.0,
            map_kpa: 100.0,
            tps_pct: 0.0,
            coolant_c: 85.0,
            oil_pressure_kpa: 250.0,
            fuel_pressure_kpa: 300.0,
            battery_v: 13.8,
            lambda: 1.0,
            target_lambda: 1.0,
            injector_duty_pct: 20.0,
            knock_level: 0.0,
            sync_ok: true,
            wideband_ready: true,
            sensor_plausible: true,
            calibration_crc_ok: true,
            watchdog_ok: true,
            can_ok: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProtectionLimits {
    pub derate_coolant_c: f32,
    pub cut_coolant_c: f32,
    pub coolant_clear_c: f32,
    pub min_oil_pressure_kpa: f32,
    pub oil_clear_kpa: f32,
    pub min_fuel_pressure_kpa: f32,
    pub fuel_clear_kpa: f32,
    pub lean_derate_lambda: f32,
    pub lean_cut_lambda: f32,
    pub lean_load_map_kpa: f32,
    pub max_boost_gauge_kpa: f32,
    pub derate_boost_gauge_kpa: f32,
    pub low_voltage_derate_v: f32,
    pub low_voltage_cut_v: f32,
    pub max_injector_duty_pct: f32,
    pub knock_retard_start: f32,
    pub knock_derate_level: f32,
    pub clear_hold_us: u64,
}

impl Default for ProtectionLimits {
    fn default() -> Self {
        Self {
            derate_coolant_c: 110.0,
            cut_coolant_c: 118.0,
            coolant_clear_c: 106.0,
            min_oil_pressure_kpa: 60.0,
            oil_clear_kpa: 90.0,
            min_fuel_pressure_kpa: 180.0,
            fuel_clear_kpa: 210.0,
            lean_derate_lambda: 1.12,
            lean_cut_lambda: 1.22,
            lean_load_map_kpa: 90.0,
            max_boost_gauge_kpa: 120.0,
            derate_boost_gauge_kpa: 95.0,
            low_voltage_derate_v: 10.0,
            low_voltage_cut_v: 8.0,
            max_injector_duty_pct: 95.0,
            knock_retard_start: 1.5,
            knock_derate_level: 4.0,
            clear_hold_us: 1_000_000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reaction {
    pub cut_fuel: bool,
    pub cut_spark: bool,
    pub derate: bool,
    pub max_rpm: u32,
    pub boost_limit_gauge_kpa: f32,
    pub knock_retard_deg: f32,
    pub learning_inhibited: bool,
}

impl Reaction {
    pub const NONE: Self = Self {
        cut_fuel: false,
        cut_spark: false,
        derate: false,
        max_rpm: 20_000,
        boost_limit_gauge_kpa: 400.0,
        knock_retard_deg: 0.0,
        learning_inhibited: false,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProtectionDecision {
    pub state: ProtectionState,
    pub active: [Option<Dtc>; MAX_FAULTS],
    pub active_count: usize,
    pub reaction: Reaction,
    pub changed: bool,
    pub latched_count: usize,
}

impl ProtectionDecision {
    pub fn contains(&self, dtc: Dtc) -> bool {
        self.active.iter().flatten().any(|d| *d == dtc)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProtectionController {
    limits: ProtectionLimits,
    faults: [Option<ActiveFault>; MAX_FAULTS],
    state: ProtectionState,
    last_signature: u64,
}

impl ProtectionController {
    pub fn new(limits: ProtectionLimits) -> Self {
        Self {
            limits,
            faults: [None; MAX_FAULTS],
            state: ProtectionState::Normal,
            last_signature: 0,
        }
    }

    pub fn limits(&self) -> ProtectionLimits {
        self.limits
    }

    pub fn faults(&self) -> [Option<ActiveFault>; MAX_FAULTS] {
        self.faults
    }

    pub fn active_faults(&self) -> impl Iterator<Item = &ActiveFault> {
        self.faults.iter().filter_map(|f| f.as_ref())
    }

    /// Explicit clear. Latched faults whose physical condition is still active
    /// cannot be cleared. Non-latched faults are handled automatically after the
    /// clear-hold interval.
    pub fn clear_latched(&mut self, now_us: u64) -> usize {
        let mut cleared = 0;
        for slot in &mut self.faults {
            if let Some(f) = slot {
                if f.latched
                    && !f.condition_active
                    && f.clear_since_us
                        .is_some_and(|t| now_us.saturating_sub(t) >= self.limits.clear_hold_us)
                {
                    *slot = None;
                    cleared += 1;
                } else if f.latched && !f.condition_active {
                    // Start the hold window if it was not recorded.
                    let mut nf = *f;
                    nf.clear_since_us.get_or_insert(now_us);
                    *slot = Some(nf);
                }
            }
        }
        if cleared > 0 {
            self.last_signature = 0;
        }
        cleared
    }

    pub fn evaluate(&mut self, i: &ProtectionInputs, now_us: u64) -> ProtectionDecision {
        let limits = self.limits;
        let freeze = FreezeFrame {
            timestamp_us: now_us,
            rpm: i.rpm,
            map_kpa: i.map_kpa,
            tps_pct: i.tps_pct,
            coolant_c: i.coolant_c,
            oil_pressure_kpa: i.oil_pressure_kpa,
            fuel_pressure_kpa: i.fuel_pressure_kpa,
            battery_v: i.battery_v,
            lambda: i.lambda,
            target_lambda: i.target_lambda,
            injector_duty_pct: i.injector_duty_pct,
        };
        let boost_gauge = i.map_kpa - 100.0;

        self.set(
            Dtc::CrankSyncLost,
            !i.sync_ok,
            FaultAction::CutAll,
            false,
            now_us,
            freeze,
            limits,
        );
        self.set(
            Dtc::CalibrationChecksum,
            !i.calibration_crc_ok,
            FaultAction::CutAll,
            true,
            now_us,
            freeze,
            limits,
        );
        self.set(
            Dtc::Watchdog,
            !i.watchdog_ok,
            FaultAction::CutAll,
            true,
            now_us,
            freeze,
            limits,
        );
        self.set(
            Dtc::CoolantOverTemperature,
            i.coolant_c >= limits.derate_coolant_c,
            FaultAction::Derate,
            false,
            now_us,
            freeze,
            limits,
        );
        if i.coolant_c >= limits.cut_coolant_c {
            self.set(
                Dtc::CoolantOverTemperature,
                true,
                FaultAction::CutAll,
                true,
                now_us,
                freeze,
                limits,
            );
        }
        let oil_condition = i.rpm > 1200.0 && i.oil_pressure_kpa < limits.min_oil_pressure_kpa;
        if oil_condition {
            self.set(
                Dtc::LowOilPressure,
                true,
                FaultAction::CutAll,
                true,
                now_us,
                freeze,
                limits,
            );
        } else if i.rpm < 500.0 || i.oil_pressure_kpa > limits.oil_clear_kpa {
            self.set(
                Dtc::LowOilPressure,
                false,
                FaultAction::CutAll,
                true,
                now_us,
                freeze,
                limits,
            );
        }
        self.set(
            Dtc::LowFuelPressure,
            i.fuel_pressure_kpa < limits.min_fuel_pressure_kpa,
            FaultAction::CutFuel,
            false,
            now_us,
            freeze,
            limits,
        );
        self.set(
            Dtc::LeanUnderLoad,
            i.map_kpa >= limits.lean_load_map_kpa && i.lambda > limits.lean_derate_lambda,
            FaultAction::Derate,
            false,
            now_us,
            freeze,
            limits,
        );
        if i.map_kpa >= limits.lean_load_map_kpa && i.lambda > limits.lean_cut_lambda {
            self.set(
                Dtc::LeanUnderLoad,
                true,
                FaultAction::CutFuel,
                true,
                now_us,
                freeze,
                limits,
            );
        }
        self.set(
            Dtc::Overboost,
            boost_gauge > limits.derate_boost_gauge_kpa,
            FaultAction::Derate,
            false,
            now_us,
            freeze,
            limits,
        );
        if boost_gauge > limits.max_boost_gauge_kpa {
            self.set(
                Dtc::Overboost,
                true,
                FaultAction::CutFuel,
                true,
                now_us,
                freeze,
                limits,
            );
        }
        self.set(
            Dtc::LowBatteryVoltage,
            i.battery_v < limits.low_voltage_derate_v,
            FaultAction::Derate,
            false,
            now_us,
            freeze,
            limits,
        );
        if i.battery_v < limits.low_voltage_cut_v {
            self.set(
                Dtc::LowBatteryVoltage,
                true,
                FaultAction::CutFuel,
                true,
                now_us,
                freeze,
                limits,
            );
        }
        self.set(
            Dtc::WidebandNotReady,
            !i.wideband_ready,
            FaultAction::Warn,
            false,
            now_us,
            freeze,
            limits,
        );
        self.set(
            Dtc::SensorPlausibility,
            !i.sensor_plausible,
            FaultAction::Derate,
            false,
            now_us,
            freeze,
            limits,
        );
        self.set(
            Dtc::CanBusLost,
            !i.can_ok,
            FaultAction::Warn,
            false,
            now_us,
            freeze,
            limits,
        );
        self.set(
            Dtc::InjectorDutyHigh,
            i.injector_duty_pct > limits.max_injector_duty_pct,
            FaultAction::Derate,
            false,
            now_us,
            freeze,
            limits,
        );
        self.set(
            Dtc::KnockHigh,
            i.knock_level >= limits.knock_retard_start,
            if i.knock_level >= limits.knock_derate_level {
                FaultAction::Derate
            } else {
                FaultAction::Warn
            },
            false,
            now_us,
            freeze,
            limits,
        );

        // Clear non-latched faults whose condition has been absent long enough.
        for slot in &mut self.faults {
            if let Some(f) = slot {
                if !f.latched && !f.condition_active {
                    if let Some(since) = f.clear_since_us {
                        // Sync restoration must resume control promptly; holding a
                        // fuel cut for a full second after valid teeth would stall
                        // the engine. Other faults retain the full clear hold.
                        let hold = if f.dtc == Dtc::CrankSyncLost {
                            100_000
                        } else {
                            limits.clear_hold_us
                        };
                        if now_us.saturating_sub(since) >= hold {
                            *slot = None;
                        }
                    } else {
                        let mut nf = *f;
                        nf.clear_since_us = Some(now_us);
                        *slot = Some(nf);
                    }
                }
            }
        }

        let mut state = ProtectionState::Normal;
        let mut reaction = Reaction::NONE;
        let mut active = [None; MAX_FAULTS];
        let mut active_count = 0;
        let mut latched_count = 0;
        for f in self.faults.iter().flatten() {
            if active_count < MAX_FAULTS {
                active[active_count] = Some(f.dtc);
                active_count += 1;
            }
            if f.latched {
                latched_count += 1;
            }
            match f.action {
                FaultAction::Warn => state = state.max(ProtectionState::Warning),
                FaultAction::Derate => {
                    state = state.max(ProtectionState::Derated);
                    reaction.derate = true;
                    reaction.learning_inhibited = true;
                }
                FaultAction::CutFuel => {
                    state = state.max(ProtectionState::Cut);
                    reaction.cut_fuel = true;
                    reaction.learning_inhibited = true;
                }
                FaultAction::CutAll => {
                    state = state.max(ProtectionState::Cut);
                    reaction.cut_fuel = true;
                    reaction.cut_spark = true;
                    reaction.learning_inhibited = true;
                }
            }
        }
        if i.knock_level >= limits.knock_retard_start {
            let retard = ((i.knock_level - limits.knock_retard_start + 0.5) * 1.5).clamp(0.0, 6.0);
            reaction.knock_retard_deg = retard;
            reaction.learning_inhibited = true;
        }
        // Derate caps are conservative defaults for the simulator only.
        if reaction.derate {
            reaction.max_rpm = 4500;
            reaction.boost_limit_gauge_kpa = limits.derate_boost_gauge_kpa;
        }
        if reaction.cut_fuel {
            reaction.max_rpm = 0;
            reaction.boost_limit_gauge_kpa = 0.0;
        }

        let signature = signature(&active, active_count, state, &reaction);
        let changed = signature != self.last_signature;
        self.last_signature = signature;
        self.state = state;
        ProtectionDecision {
            state,
            active,
            active_count,
            reaction,
            changed,
            latched_count,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn set(
        &mut self,
        dtc: Dtc,
        condition: bool,
        action: FaultAction,
        latch: bool,
        now_us: u64,
        freeze: FreezeFrame,
        _limits: ProtectionLimits,
    ) {
        let existing = self
            .faults
            .iter()
            .position(|f| f.as_ref().is_some_and(|x| x.dtc == dtc));
        match (condition, existing) {
            (true, Some(idx)) => {
                if let Some(f) = &mut self.faults[idx] {
                    if !f.condition_active {
                        f.occurrences = f.occurrences.saturating_add(1);
                    }
                    f.condition_active = true;
                    f.last_seen_us = now_us;
                    f.clear_since_us = None;
                    f.latched |= latch;
                    // A higher-severity action wins until the fault clears.
                    if action_rank(action) > action_rank(f.action) {
                        f.action = action;
                    }
                }
            }
            (true, None) => {
                if let Some(slot) = self.faults.iter_mut().find(|f| f.is_none()) {
                    *slot = Some(ActiveFault {
                        dtc,
                        action,
                        latched: latch,
                        condition_active: true,
                        first_seen_us: now_us,
                        last_seen_us: now_us,
                        clear_since_us: None,
                        occurrences: 1,
                        freeze_frame: freeze,
                    });
                }
            }
            (false, Some(idx)) => {
                if let Some(f) = &mut self.faults[idx] {
                    f.condition_active = false;
                    if f.clear_since_us.is_none() {
                        f.clear_since_us = Some(now_us);
                    }
                }
            }
            (false, None) => {}
        }
    }
}

fn action_rank(a: FaultAction) -> u8 {
    match a {
        FaultAction::Warn => 0,
        FaultAction::Derate => 1,
        FaultAction::CutFuel => 2,
        FaultAction::CutAll => 3,
    }
}

fn signature(
    active: &[Option<Dtc>; MAX_FAULTS],
    count: usize,
    state: ProtectionState,
    reaction: &Reaction,
) -> u64 {
    let mut h = state as u64 + 1;
    for d in active.iter().take(count).flatten() {
        h = h.wrapping_mul(1099511628211).wrapping_add(*d as u64 + 1);
    }
    if reaction.cut_fuel {
        h ^= 0xA5;
    }
    if reaction.cut_spark {
        h ^= 0x5A;
    }
    h.wrapping_add((reaction.knock_retard_deg * 10.0) as u64)
}

impl fmt::Display for ProtectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtectionState::Normal => f.write_str("normal"),
            ProtectionState::Warning => f.write_str("warning"),
            ProtectionState::Derated => f.write_str("derated"),
            ProtectionState::Limp => f.write_str("limp"),
            ProtectionState::Cut => f.write_str("cut"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> ProtectionInputs {
        ProtectionInputs::default()
    }

    fn ctrl() -> ProtectionController {
        ProtectionController::new(ProtectionLimits::default())
    }

    #[test]
    fn nominal_state_has_no_faults() {
        let d = ctrl().evaluate(&good(), 1_000_000);
        assert_eq!(d.state, ProtectionState::Normal);
        assert_eq!(d.active_count, 0);
        assert_eq!(d.reaction, Reaction::NONE);
    }

    #[test]
    fn sync_loss_cuts_fuel_and_spark_then_recovers() {
        let mut c = ctrl();
        let mut i = good();
        i.rpm = 2000.0;
        i.sync_ok = false;
        let d = c.evaluate(&i, 1_000_000);
        assert_eq!(d.state, ProtectionState::Cut);
        assert!(d.reaction.cut_fuel && d.reaction.cut_spark);
        assert!(d.contains(Dtc::CrankSyncLost));
        i.sync_ok = true;
        let d = c.evaluate(&i, 2_000_000);
        // Non-latched clear requires hold: still present immediately.
        assert!(d.contains(Dtc::CrankSyncLost));
        let d = c.evaluate(&i, 3_200_000);
        assert!(!d.contains(Dtc::CrankSyncLost), "{d:?}");
        assert_eq!(d.state, ProtectionState::Normal);
    }

    #[test]
    fn overtemp_latches_and_requires_explicit_clear() {
        let mut c = ctrl();
        let mut i = good();
        i.coolant_c = 120.0;
        let d = c.evaluate(&i, 1_000_000);
        assert!(d.reaction.cut_fuel);
        assert!(d.contains(Dtc::CoolantOverTemperature));
        i.coolant_c = 90.0;
        let d = c.evaluate(&i, 2_000_000);
        assert!(d.contains(Dtc::CoolantOverTemperature));
        assert_eq!(d.state, ProtectionState::Cut);
        // The clear hold started when the condition cleared; the first explicit
        // clear attempt arms it, and a later attempt after the hold succeeds.
        assert_eq!(c.clear_latched(2_100_000), 0);
        assert_eq!(c.clear_latched(4_000_000), 1);
        let d = c.evaluate(&i, 4_100_000);
        assert!(!d.contains(Dtc::CoolantOverTemperature));
        assert_eq!(d.state, ProtectionState::Normal);
    }

    #[test]
    fn low_oil_pressure_cuts_and_latches() {
        let mut c = ctrl();
        let mut i = good();
        i.rpm = 3000.0;
        i.oil_pressure_kpa = 30.0;
        let d = c.evaluate(&i, 5);
        assert!(d.reaction.cut_fuel);
        assert!(d.contains(Dtc::LowOilPressure));
        i.oil_pressure_kpa = 250.0;
        let d = c.evaluate(&i, 2_000_000);
        assert!(d.contains(Dtc::LowOilPressure));
    }

    #[test]
    fn overboost_derates_then_cuts_with_hysteresis() {
        let mut c = ctrl();
        let mut i = good();
        i.map_kpa = 230.0;
        let d = c.evaluate(&i, 1);
        assert!(d.contains(Dtc::Overboost));
        assert!(d.reaction.cut_fuel);
        i.map_kpa = 205.0;
        let d = c.evaluate(&i, 2_000_000);
        assert!(d.state == ProtectionState::Derated || d.state == ProtectionState::Cut);
    }

    #[test]
    fn lean_under_load_is_a_protection_fault_but_not_an_ignition_signal() {
        let mut c = ctrl();
        let mut i = good();
        i.map_kpa = 110.0;
        i.lambda = 1.16;
        let d = c.evaluate(&i, 1);
        assert!(d.contains(Dtc::LeanUnderLoad));
        assert!(d.reaction.learning_inhibited);
        // The protection API has no ignition-advance output at all.
        assert_eq!(d.reaction.knock_retard_deg, 0.0);
    }

    #[test]
    fn calibration_crc_failure_latches_a_cut() {
        let mut c = ctrl();
        let mut i = good();
        i.calibration_crc_ok = false;
        let d = c.evaluate(&i, 10);
        assert!(d.reaction.cut_fuel && d.reaction.cut_spark);
        assert!(d.contains(Dtc::CalibrationChecksum));
        assert_eq!(d.latched_count, 1);
    }

    #[test]
    fn knock_produces_bounded_retard_not_fuel_learning() {
        let mut c = ctrl();
        let mut i = good();
        i.map_kpa = 120.0;
        i.knock_level = 3.0;
        let d = c.evaluate(&i, 1);
        assert!(d.contains(Dtc::KnockHigh));
        assert!(d.reaction.knock_retard_deg > 0.0 && d.reaction.knock_retard_deg <= 6.0);
        assert!(d.reaction.learning_inhibited);
    }

    #[test]
    fn wideband_not_ready_warns_without_cutting_engine() {
        let mut c = ctrl();
        let mut i = good();
        i.wideband_ready = false;
        let d = c.evaluate(&i, 1);
        assert_eq!(d.state, ProtectionState::Warning);
        assert!(d.contains(Dtc::WidebandNotReady));
        assert!(!d.reaction.cut_fuel);
        assert!(!d.reaction.learning_inhibited); // learning gate will reject sensor readiness separately
    }

    #[test]
    fn fault_free_recovery_after_hold() {
        let mut c = ctrl();
        let mut i = good();
        i.battery_v = 9.0;
        let d = c.evaluate(&i, 1_000_000);
        assert!(d.reaction.derate);
        i.battery_v = 13.8;
        let d = c.evaluate(&i, 2_000_000);
        assert!(d.reaction.derate);
        let d = c.evaluate(&i, 3_500_000);
        assert!(!d.reaction.derate, "{d:?}");
    }
}
