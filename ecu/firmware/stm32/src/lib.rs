//! OpenTech-ECU STM32 safety skeleton.
//!
//! This crate is deliberately a *skeleton*, not a working engine controller.
//! It demonstrates the no_std boundary, capture-to-decoder path, independent
//! protection and fail-closed output policy. It has no HAL bindings, no timer
//! capture implementation, no injector/ignition driver, no watchdog integration
//! and no physical output authorisation.
//!
//! `PHYSICAL_OUTPUTS_AUTHORISED` is false. Outputs are forced inactive even if
//! [`Controller::arm`] is called. Changing this requires the Gate 1-4 evidence
//! and independent review described in AUDIT/RELEASE_GATES.md.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

use ecu_protection::{
    ProtectionController, ProtectionDecision, ProtectionInputs, ProtectionLimits, ProtectionState,
    Reaction,
};
use ecu_trigger::{MissingToothConfig, MissingToothDecoder, SyncState};

/// Hard interlock. There is no physical output code path in v0.1.
pub const PHYSICAL_OUTPUTS_AUTHORISED: bool = false;

pub const MAX_OUTPUTS: usize = 8;

/// The absolute minimum hardware contract. Implementations must default every
/// output to its de-energised state at reset and on MCU fault.
pub trait EngineOutputs {
    fn set_injector(&mut self, channel: u8, energised: bool);
    fn set_ignition(&mut self, channel: u8, energised: bool);
    fn set_fuel_pump(&mut self, energised: bool);
    fn all_off(&mut self);
}

/// A deliberately limited "arm" token. Arming the software does not authorise
/// physical outputs; it only permits the simulated output state to be observed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BenchArmToken(u16);

impl BenchArmToken {
    /// Only a locally reviewed constant can construct this token. The value is
    /// intentionally not a secret or a bypass; it is a compile-time tripwire.
    pub const fn reviewed(value: u16) -> Option<Self> {
        if value == 0x4F50 {
            Some(Self(value))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirmwareFault {
    NotSynchronised,
    OutputsNotAuthorised,
    ProtectionCut,
    Watchdog,
    InvalidConfiguration,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SensorFrame {
    pub rpm: f32,
    pub map_kpa: f32,
    pub coolant_c: f32,
    pub oil_pressure_kpa: f32,
    pub fuel_pressure_kpa: f32,
    pub battery_v: f32,
    pub lambda: f32,
    pub target_lambda: f32,
    pub injector_duty_pct: f32,
    pub knock_level: f32,
    pub wideband_ready: bool,
    pub sensor_plausible: bool,
    pub calibration_crc_ok: bool,
    pub watchdog_ok: bool,
    pub can_ok: bool,
}

impl Default for SensorFrame {
    fn default() -> Self {
        Self {
            rpm: 0.0,
            map_kpa: 100.0,
            coolant_c: 85.0,
            oil_pressure_kpa: 300.0,
            fuel_pressure_kpa: 300.0,
            battery_v: 13.8,
            lambda: 1.0,
            target_lambda: 1.0,
            injector_duty_pct: 0.0,
            knock_level: 0.0,
            wideband_ready: true,
            sensor_plausible: true,
            calibration_crc_ok: true,
            watchdog_ok: true,
            can_ok: true,
        }
    }
}

pub struct Controller {
    decoder: MissingToothDecoder,
    protection: ProtectionController,
    armed: bool,
    last_decision: ProtectionDecision,
    last_sync: SyncState,
}

impl Controller {
    pub fn new(
        trigger: MissingToothConfig,
        limits: ProtectionLimits,
    ) -> Result<Self, FirmwareFault> {
        let decoder =
            MissingToothDecoder::new(trigger).map_err(|_| FirmwareFault::InvalidConfiguration)?;
        let protection = ProtectionController::new(limits);
        let last_decision =
            ProtectionController::new(limits).evaluate(&ProtectionInputs::default(), 0);
        Ok(Self {
            decoder,
            protection,
            armed: false,
            last_decision,
            last_sync: SyncState::Unsynced,
        })
    }

    /// Arming is not output authorisation. Physical outputs remain low because
    /// `PHYSICAL_OUTPUTS_AUTHORISED` is false.
    pub fn arm(&mut self, token: Option<BenchArmToken>) -> Result<(), FirmwareFault> {
        if token.is_none() {
            return Err(FirmwareFault::InvalidConfiguration);
        }
        self.armed = true;
        Ok(())
    }

    pub fn disarm(&mut self) {
        self.armed = false;
    }

    pub fn on_crank_tooth(&mut self, timestamp_us: u64) {
        self.decoder.on_tooth(timestamp_us);
    }

    pub fn on_cam_edge(&mut self, timestamp_us: u64) {
        self.decoder.on_cam(timestamp_us);
    }

    pub fn decoder_state(&self) -> SyncState {
        self.decoder.state()
    }

    pub fn protection(&self) -> ProtectionDecision {
        self.last_decision
    }

    /// Safety tick. Returns the reaction; the caller must apply it to outputs.
    /// If anything is not positively valid, outputs are forced inactive.
    pub fn tick(&mut self, now_us: u64, sensors: &SensorFrame) -> (Reaction, FirmwareFaultOrNone) {
        let _ = self.decoder.tick(now_us);
        let sync_ok = self.decoder.is_sequential();
        let inputs = ProtectionInputs {
            rpm: sensors.rpm,
            map_kpa: sensors.map_kpa,
            tps_pct: 0.0,
            coolant_c: sensors.coolant_c,
            oil_pressure_kpa: sensors.oil_pressure_kpa,
            fuel_pressure_kpa: sensors.fuel_pressure_kpa,
            battery_v: sensors.battery_v,
            lambda: sensors.lambda,
            target_lambda: sensors.target_lambda,
            injector_duty_pct: sensors.injector_duty_pct,
            knock_level: sensors.knock_level,
            sync_ok,
            wideband_ready: sensors.wideband_ready,
            sensor_plausible: sensors.sensor_plausible,
            calibration_crc_ok: sensors.calibration_crc_ok,
            watchdog_ok: sensors.watchdog_ok,
            can_ok: sensors.can_ok,
        };
        self.last_decision = self.protection.evaluate(&inputs, now_us);
        self.last_sync = self.decoder.state();

        let mut fault = FirmwareFaultOrNone::None;
        if !sensors.watchdog_ok {
            fault = FirmwareFaultOrNone::Some(FirmwareFault::Watchdog);
        } else if !sync_ok {
            fault = FirmwareFaultOrNone::Some(FirmwareFault::NotSynchronised);
        } else if self.last_decision.state == ProtectionState::Cut {
            fault = FirmwareFaultOrNone::Some(FirmwareFault::ProtectionCut);
        } else if !PHYSICAL_OUTPUTS_AUTHORISED {
            fault = FirmwareFaultOrNone::Some(FirmwareFault::OutputsNotAuthorised);
        }

        let reaction = if fault.is_some() || !self.armed {
            Reaction {
                cut_fuel: true,
                cut_spark: true,
                ..Reaction::NONE
            }
        } else {
            self.last_decision.reaction
        };
        (reaction, fault)
    }

    /// Apply a reaction fail-closed. This helper never energises an output while
    /// `PHYSICAL_OUTPUTS_AUTHORISED` is false, regardless of `armed`.
    pub fn apply_reaction<O: EngineOutputs>(&self, out: &mut O, reaction: &Reaction) {
        let safe = !self.armed
            || !PHYSICAL_OUTPUTS_AUTHORISED
            || reaction.cut_fuel
            || self.last_decision.state == ProtectionState::Cut;
        if safe {
            out.all_off();
            return;
        }
        // Future hardware path. Kept unreachable by the interlock above.
        out.all_off();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirmwareFaultOrNone {
    None,
    Some(FirmwareFault),
}

impl FirmwareFaultOrNone {
    pub fn is_some(self) -> bool {
        matches!(self, Self::Some(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecu_protection::ProtectionLimits;

    #[derive(Default)]
    struct MockOutputs {
        energised: bool,
        all_off_calls: u32,
    }
    impl EngineOutputs for MockOutputs {
        fn set_injector(&mut self, _channel: u8, energised: bool) {
            self.energised |= energised;
        }
        fn set_ignition(&mut self, _channel: u8, energised: bool) {
            self.energised |= energised;
        }
        fn set_fuel_pump(&mut self, energised: bool) {
            self.energised |= energised;
        }
        fn all_off(&mut self) {
            self.energised = false;
            self.all_off_calls += 1;
        }
    }

    fn controller() -> Controller {
        Controller::new(MissingToothConfig::default(), ProtectionLimits::default()).unwrap()
    }

    #[test]
    fn unarmed_controller_forces_outputs_off() {
        let mut c = controller();
        let (reaction, fault) = c.tick(1_000_000, &SensorFrame::default());
        assert!(reaction.cut_fuel && reaction.cut_spark);
        assert!(fault.is_some());
        let mut out = MockOutputs::default();
        c.apply_reaction(&mut out, &reaction);
        assert!(!out.energised);
        assert!(out.all_off_calls > 0);
    }

    #[test]
    fn arming_cannot_override_physical_interlock() {
        let mut c = controller();
        c.arm(BenchArmToken::reviewed(0x4F50)).unwrap();
        let (reaction, fault) = c.tick(1_000, &SensorFrame::default());
        assert!(fault.is_some());
        assert!(reaction.cut_fuel && reaction.cut_spark);
        let mut out = MockOutputs::default();
        c.apply_reaction(&mut out, &reaction);
        assert!(!out.energised);
    }

    #[test]
    fn rejected_token_cannot_arm() {
        let mut c = controller();
        assert!(c.arm(None).is_err());
        assert!(BenchArmToken::reviewed(0x0000).is_none());
    }

    #[test]
    fn watchdog_fault_is_fail_closed() {
        let mut c = controller();
        let _ = c.arm(BenchArmToken::reviewed(0x4F50));
        let sensors = SensorFrame {
            watchdog_ok: false,
            ..SensorFrame::default()
        };
        let (_, fault) = c.tick(1_000, &sensors);
        assert_eq!(fault, FirmwareFaultOrNone::Some(FirmwareFault::Watchdog));
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn physical_interlock_constant_is_false() {
        assert!(!PHYSICAL_OUTPUTS_AUTHORISED);
    }
}
