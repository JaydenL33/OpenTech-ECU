#![forbid(unsafe_code)]

pub mod scenario;

use std::collections::VecDeque;
use std::string::String;
use std::vec::Vec;

use ecu_calibration::preflight::{preflight, PreflightReport};
use ecu_calibration::Calibration;
use ecu_fuel::{calculate, FuelDecision, FuelInputs};
use ecu_ignition::{limit_dwell, RevLimiter};
use ecu_lambda::gate::{evaluate as evaluate_gate, GateConfig, GateInputs};
use ecu_lambda::{ClosedLoop, ClosedLoopConfig, LongTermTrimTable, LtftConfig};
use ecu_protection::{
    Dtc, ProtectionController, ProtectionDecision, ProtectionInputs, ProtectionLimits,
    ProtectionState as CoreProtectionState, Reaction,
};
use ecu_scheduler::{build_schedule, EventKind, EventSchedule, ScheduleRequest};
use ecu_trigger::{MissingToothConfig, MissingToothDecoder, SyncState as TriggerSync};
use ecu_types::{
    EngineState, LogEvent, LogFile, LogMetadata, ProtectionState, SensorStatus, SyncState,
    TelemetryFrame, WidebandState,
};
use serde::{Deserialize, Serialize};

use crate::scenario::{DeterministicRng, FaultWindow, Scenario, SimFault};

const AIR_R: f32 = 287.05;
const LHV_J_PER_KG: f32 = 43.5e6;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimError {
    CalibrationInvalid(String),
    PreflightBlocked(String),
    InvalidScenario(String),
}

impl std::fmt::Display for SimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimError::CalibrationInvalid(s) => write!(f, "calibration invalid: {s}"),
            SimError::PreflightBlocked(s) => write!(f, "preflight blocked virtual run: {s}"),
            SimError::InvalidScenario(s) => write!(f, "invalid scenario: {s}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimSnapshot {
    pub scenario_id: String,
    pub scenario_name: String,
    pub time_ms: u64,
    pub rpm: f32,
    pub map_kpa: f32,
    pub lambda: f32,
    pub target_lambda: f32,
    pub sync: String,
    pub engine_state: String,
    pub protection_state: String,
    pub calibration_revision: u32,
    pub sample_count: usize,
}

pub struct SimEngine {
    pub scenario: Scenario,
    pub calibration: Calibration,
    preflight: PreflightReport,
    rng: DeterministicRng,
    time_us: u64,
    step_count: u64,
    sample_every: u64,

    // True plant state. These values are hidden from the ECU except through the
    // modelled sensor path.
    true_rpm: f32,
    true_angle_deg: f32,
    true_map_kpa: f32,
    throttle_pct: f32,
    true_clt_c: f32,
    true_iat_c: f32,
    true_oil_kpa: f32,
    true_fuel_kpa: f32,
    battery_v: f32,
    load_torque_nm: f32,
    dyno_integral: f32,
    idle_integral: f32,
    starter_on: bool,
    auto_starter: bool,

    // Sensor path
    map_sensor: f32,
    tps_sensor: f32,
    clt_sensor: f32,
    iat_sensor: f32,
    oil_sensor: f32,
    fuel_pressure_sensor: f32,
    lambda_sensor: f32,
    lambda_ready: bool,
    lambda_status: SensorStatus,
    wideband: WidebandState,
    lambda_delay: VecDeque<(u64, f32)>,
    lambda_delay_target: f32,
    exhaust_delayed: bool,

    // Trigger and scheduling
    decoder: MissingToothDecoder,
    schedule: EventSchedule,
    injections_this_cycle: u32,
    sparks_this_cycle: u32,
    last_injection_angle: f32,
    last_spark_angle: f32,

    // Cycle mixture accumulation
    air_cycle_kg: f64,
    fuel_cycle_kg: f64,
    cycle_lambda_actual: f32,
    misfire_this_cycle: bool,

    // ECU outputs
    ve: f32,
    ve_row: u16,
    ve_col: u16,
    target_lambda: f32,
    fuel: Option<FuelDecision>,
    fuel_cut: bool,
    spark_cut: bool,
    advance_deg: f32,
    dwell_ms: f32,
    limiter: RevLimiter,
    closed_loop: ClosedLoop,
    ltft: LongTermTrimTable,
    ltft_cfg: LtftConfig,
    last_stft_ms: u64,
    last_ltft_ms: u64,
    protection: ProtectionController,
    protection_decision: ProtectionDecision,
    last_protection: CoreProtectionState,
    gate_cfg: GateConfig,

    // Recording
    frames: Vec<TelemetryFrame>,
    events: Vec<LogEvent>,
    seq: u64,
    last_faults: Vec<SimFault>,
    schedule_error_logged: bool,
    cycle_wrapped: bool,
}

impl SimEngine {
    pub fn new(calibration: Calibration, scenario: Scenario) -> Result<Self, SimError> {
        let issues = calibration.validate();
        if let Some(e) = issues.iter().find(|i| i.is_error()) {
            return Err(SimError::CalibrationInvalid(format!(
                "{}: {}",
                e.code, e.message
            )));
        }
        let preflight = preflight(&calibration);
        if !preflight.virtual_run_allowed {
            return Err(SimError::PreflightBlocked(preflight.summary));
        }
        if scenario.phases.is_empty() || scenario.sample_hz == 0 || scenario.duration_ms == 0 {
            return Err(SimError::InvalidScenario(
                "phases, sample rate and duration are required".into(),
            ));
        }
        let ve_table = calibration
            .table("ve")
            .map_err(|e| SimError::CalibrationInvalid(e.to_string()))?;
        let ltft = LongTermTrimTable::new(
            ve_table.axes[1].breakpoints.clone(),
            ve_table.axes[0].breakpoints.clone(),
        )
        .map_err(|e| SimError::CalibrationInvalid(e.to_string()))?;
        let motor_cfg = calibration.engine.trigger.clone();
        let (teeth, missing) = match motor_cfg.pattern {
            ecu_types::TriggerPattern::MissingTooth { teeth, missing } => (teeth, missing),
            _ => {
                return Err(SimError::CalibrationInvalid(
                    "v1 simulator requires a missing-tooth trigger".into(),
                ))
            }
        };
        let decoder = MissingToothDecoder::new(MissingToothConfig {
            teeth,
            missing,
            cam_required: motor_cfg.cam_required,
            sync_offset_deg: motor_cfg.sync_offset_deg,
            timeout_ms: motor_cfg.tooth_timeout_ms,
        })
        .map_err(|e| SimError::CalibrationInvalid(format!("{e:?}")))?;

        let sample_hz = scenario.sample_hz.max(1) as u64;
        let auto_starter = scenario.id == "fault_recovery" || scenario.id == "trigger_loss";
        let mut engine = Self {
            rng: DeterministicRng::new(scenario.seed),
            time_us: 0,
            step_count: 0,
            sample_every: (1000 / sample_hz).max(1),
            true_rpm: scenario.initial_rpm.max(0.0),
            true_angle_deg: 0.0,
            true_map_kpa: if scenario.cranking { 85.0 } else { 28.0 },
            throttle_pct: 0.0,
            true_clt_c: scenario.start_coolant_c,
            true_iat_c: scenario.ambient_c,
            true_oil_kpa: 80.0 + scenario.initial_rpm * 0.06,
            true_fuel_kpa: calibration.fuel.pressure_reference_kpa.max(250.0),
            battery_v: if scenario.cranking { 11.8 } else { 13.8 },
            load_torque_nm: 0.0,
            dyno_integral: 0.0,
            idle_integral: 0.0,
            starter_on: scenario.cranking,
            auto_starter,
            map_sensor: 0.0,
            tps_sensor: 0.0,
            clt_sensor: scenario.start_coolant_c,
            iat_sensor: scenario.ambient_c,
            oil_sensor: 0.0,
            fuel_pressure_sensor: calibration.fuel.pressure_reference_kpa.max(250.0),
            lambda_sensor: 1.0,
            lambda_ready: false,
            lambda_status: SensorStatus::NotReady,
            wideband: WidebandState::not_ready(1),
            lambda_delay: VecDeque::new(),
            lambda_delay_target: 1.0,
            exhaust_delayed: false,
            decoder,
            schedule: EventSchedule::empty(),
            injections_this_cycle: 0,
            sparks_this_cycle: 0,
            last_injection_angle: 0.0,
            last_spark_angle: 0.0,
            air_cycle_kg: 0.0,
            fuel_cycle_kg: 0.0,
            cycle_lambda_actual: 1.0,
            misfire_this_cycle: false,
            ve: 0.0,
            ve_row: 0,
            ve_col: 0,
            target_lambda: 1.0,
            fuel: None,
            fuel_cut: false,
            spark_cut: false,
            advance_deg: 0.0,
            dwell_ms: 0.0,
            limiter: RevLimiter::new(calibration.engine.rev_limit as f32, 100.0),
            closed_loop: ClosedLoop::new(ClosedLoopConfig::default()),
            ltft,
            ltft_cfg: LtftConfig::default(),
            last_stft_ms: 0,
            last_ltft_ms: 0,
            protection: ProtectionController::new(ProtectionLimits::default()),
            protection_decision: ProtectionController::new(ProtectionLimits::default())
                .evaluate(&ProtectionInputs::default(), 0),
            last_protection: CoreProtectionState::Normal,
            gate_cfg: GateConfig::for_closed_loop(),
            frames: Vec::new(),
            events: Vec::new(),
            seq: 0,
            last_faults: Vec::new(),
            schedule_error_logged: false,
            cycle_wrapped: false,
            scenario,
            calibration,
            preflight,
        };
        engine.update_sensors();
        engine.compute_ecu_outputs();
        engine.rebuild_schedule();
        Ok(engine)
    }

    pub fn preflight(&self) -> &PreflightReport {
        &self.preflight
    }

    pub fn frames(&self) -> &[TelemetryFrame] {
        &self.frames
    }

    pub fn events(&self) -> &[LogEvent] {
        &self.events
    }

    pub fn last_frame(&self) -> Option<&TelemetryFrame> {
        self.frames.last()
    }

    pub fn time_ms(&self) -> u64 {
        self.time_us / 1000
    }

    pub fn true_rpm(&self) -> f32 {
        self.true_rpm
    }

    pub fn is_finished(&self) -> bool {
        self.time_ms() >= self.scenario.duration_ms
    }

    pub fn protection_faults(&self) -> Vec<ecu_protection::ActiveFault> {
        self.protection.active_faults().copied().collect()
    }

    /// Explicit operator action. Only faults whose physical condition has
    /// cleared and whose hold has elapsed can be removed.
    pub fn clear_latched_faults(&mut self) -> usize {
        self.protection.clear_latched(self.time_us)
    }

    pub fn duty_percent(&self) -> f32 {
        self.fuel.as_ref().map(|f| f.duty_percent).unwrap_or(0.0)
    }

    pub fn snapshot(&self) -> SimSnapshot {
        let f = self.frames.last();
        SimSnapshot {
            scenario_id: self.scenario.id.clone(),
            scenario_name: self.scenario.name.clone(),
            time_ms: self.time_ms(),
            rpm: f.map(|f| f.rpm).unwrap_or(self.true_rpm),
            map_kpa: f.map(|f| f.map_kpa).unwrap_or(self.true_map_kpa),
            lambda: f.map(|f| f.lambda_measured).unwrap_or(0.0),
            target_lambda: f.map(|f| f.lambda_target).unwrap_or(1.0),
            sync: format!("{:?}", self.decoder.state()),
            engine_state: format!("{:?}", self.engine_state()),
            protection_state: format!("{:?}", self.protection_decision.state),
            calibration_revision: self.calibration.revision,
            sample_count: self.frames.len(),
        }
    }

    pub fn log_metadata(&self) -> LogMetadata {
        LogMetadata {
            schema_version: ecu_types::SCHEMA_VERSION,
            software_version: ecu_types::SOFTWARE_VERSION.into(),
            scenario_id: self.scenario.id.clone(),
            scenario_name: self.scenario.name.clone(),
            seed: self.scenario.seed,
            calibration_id: self.calibration.id.clone(),
            calibration_revision: self.calibration.revision,
            calibration_checksum: self.calibration.checksum.clone(),
            started_unix_ms: 0,
            sample_hz: self.scenario.sample_hz,
            notes: self.scenario.notes.clone(),
        }
    }

    pub fn log_file(&self) -> LogFile {
        LogFile {
            metadata: self.log_metadata(),
            frames: self.frames.clone(),
            events: self.events.clone(),
        }
    }

    pub fn advance_ms(&mut self, wall_ms: u64) {
        let end = (self.time_ms() + wall_ms).min(self.scenario.duration_ms);
        while self.time_ms() < end {
            if self.step_count.is_multiple_of(self.sample_every) {
                self.push_telemetry();
            }
            self.step_once();
        }
        if self.frames.is_empty()
            || self.frames.last().map(|f| f.scenario_time_ms).unwrap_or(0) < end
        {
            self.push_telemetry();
        }
    }

    pub fn run_to_end(&mut self) {
        while self.time_ms() < self.scenario.duration_ms {
            self.advance_ms(500);
        }
    }

    fn step_once(&mut self) {
        const DT_S: f32 = 0.001;
        let step_start_us = self.time_us;
        let phase = self.scenario.phase_at(self.time_ms()).clone();
        self.step_count += 1;
        self.cycle_wrapped = false;

        if self.scenario.cranking && self.true_rpm < 500.0 && self.time_ms() < 8000 {
            self.starter_on = true;
        }
        if self.auto_starter && self.true_rpm < 350.0 {
            self.starter_on = true;
        }
        if self.starter_on && self.true_rpm > 650.0 {
            self.starter_on = false;
        }

        self.update_driver(&phase);
        self.update_engine_physics(DT_S, step_start_us);
        self.update_sensors();
        self.advance_trigger(DT_S, step_start_us);
        self.decoder.tick(self.time_us);
        self.compute_ecu_outputs();
        self.evaluate_protection();
        self.apply_events(step_start_us);
        self.update_wideband(DT_S);
        self.track_fault_events();
        self.time_us += 1000;
    }

    fn update_driver(&mut self, phase: &crate::scenario::Phase) {
        // A dyno target near idle still needs idle-air control; the dyno only
        // supplies positive load and cannot keep an engine alive at closed
        // throttle.
        let idle_target = phase.target_rpm.unwrap_or(900.0);
        let idle = phase.throttle_pct < 1.0 && !self.starter_on && idle_target <= 950.0;
        let commanded = if idle {
            let rpm_error = idle_target - self.true_rpm;
            self.idle_integral = (self.idle_integral + rpm_error * 0.0012).clamp(0.0, 22.0);
            (rpm_error * 0.05 + self.idle_integral).clamp(0.0, 28.0)
        } else {
            phase.throttle_pct
        };
        let tau_s: f32 = 0.06;
        self.throttle_pct += (commanded - self.throttle_pct) * (0.001 / tau_s).min(1.0);

        let baro = 100.0;
        let throttle = (self.throttle_pct / 100.0).clamp(0.0, 1.0);
        let mut map_target = 20.0 + (baro - 20.0) * throttle.powf(0.65);
        if self.starter_on && !self.decoder.is_sequential() {
            map_target = 70.0 + 25.0 * throttle;
        }
        // Pumping vacuum rises with speed at part throttle.
        map_target -= ((self.true_rpm - 900.0).max(0.0) / 6500.0) * 7.0 * (1.0 - throttle);
        if let ecu_types::Aspiration::Turbocharged | ecu_types::Aspiration::Supercharged =
            self.calibration.engine.aspiration
        {
            if self.true_rpm > 2200.0 && self.throttle_pct > 55.0 {
                let boost = ((self.true_rpm - 2200.0) / 35.0).min(120.0)
                    * ((self.throttle_pct - 55.0) / 45.0).clamp(0.0, 1.0);
                map_target += boost;
            }
        }
        let map_target = map_target.clamp(15.0, 260.0);
        self.true_map_kpa += (map_target - self.true_map_kpa) * 0.03;
    }

    fn update_engine_physics(&mut self, dt_s: f32, _step_start_us: u64) {
        // Thermal model: slow warm-up, or forced overheat injection.
        let coolant_target = if self
            .scenario
            .fault_active(SimFault::CoolantOverheat, self.time_ms())
        {
            125.0
        } else {
            90.0
        };
        self.true_clt_c += (coolant_target - self.true_clt_c) * (dt_s / 35.0);
        self.true_iat_c += ((self.scenario.ambient_c + 8.0) - self.true_iat_c) * (dt_s / 120.0);

        let oil_target = if self
            .scenario
            .fault_active(SimFault::FuelPressureLow, self.time_ms())
        {
            // Keep oil independent of fuel pressure; reserved for future faults.
            80.0 + self.true_rpm * 0.06
        } else {
            80.0 + self.true_rpm * 0.06
        };
        self.true_oil_kpa += (oil_target.clamp(20.0, 600.0) - self.true_oil_kpa) * (dt_s / 0.3);

        let fp_target = if self
            .scenario
            .fault_active(SimFault::FuelPressureLow, self.time_ms())
        {
            155.0
        } else if self
            .scenario
            .fault_active(SimFault::FuelPressureSensorBlind, self.time_ms())
        {
            170.0
        } else {
            self.calibration.fuel.pressure_reference_kpa.max(250.0)
        };
        self.true_fuel_kpa += (fp_target - self.true_fuel_kpa) * (dt_s / 0.4);

        self.battery_v = if self.starter_on {
            11.6
        } else {
            13.8 - self.true_rpm * 0.00005
        };

        // True VE differs from the ECU's table by the scenario bias.
        let base_ve = self
            .lookup_ve(self.true_rpm, self.true_map_kpa)
            .unwrap_or(60.0);
        let load_frac = (self.true_map_kpa / 100.0 - 0.5).clamp(-0.5, 1.5);
        let rpm_frac = (self.true_rpm / 6000.0 - 0.5).clamp(-0.5, 1.0);
        let error = 1.0
            + self.scenario.true_ve_error_pct / 100.0
            + self.scenario.ve_error_load_gradient_pct / 100.0 * load_frac
            + self.scenario.ve_error_rpm_gradient_pct / 100.0 * rpm_frac;
        let true_ve = (base_ve * error).clamp(10.0, 130.0);
        let rho = (self.true_map_kpa * 1000.0) / (AIR_R * (self.true_iat_c + 273.15));
        let vd_m3 = self.calibration.engine.displacement_cc * 1e-6;
        let air_flow = rho * vd_m3 * (true_ve / 100.0) * (self.true_rpm / 120.0);
        self.air_cycle_kg += (air_flow * dt_s) as f64;

        // Instantaneous fuel flow for torque, from the last scheduled injection
        // command. Actual flow is reduced if the pressure sensor is blind.
        let fuel = self.fuel.as_ref();
        let mut fuel_kg_s = 0.0f32;
        if let Some(f) = fuel {
            if !self.fuel_cut && self.decoder.is_sequential() {
                let actual_pressure = self.true_fuel_kpa.max(1.0);
                let sensed_pressure = self.fuel_pressure_sensor.max(1.0);
                let blind_factor = (actual_pressure / sensed_pressure).sqrt().clamp(0.2, 1.5);
                let injector_factor = if self
                    .scenario
                    .fault_active(SimFault::InjectorFlowLow, self.time_ms())
                {
                    0.80
                } else {
                    1.0
                };
                let per_cycle_mg = f.delivered_fuel_mg * blind_factor * injector_factor;
                fuel_kg_s = per_cycle_mg
                    * 1e-6
                    * self.calibration.engine.cylinders as f32
                    * (self.true_rpm / 120.0);
            }
        }

        let lambda_for_torque = if self.cycle_lambda_actual.is_finite() {
            self.cycle_lambda_actual.clamp(0.55, 1.4)
        } else {
            0.90
        };
        let mixture_eff = (1.0 - 1.4 * (lambda_for_torque - 0.90).powi(2)).clamp(0.35, 1.0);
        let warmup = (0.65 + 0.35 * ((self.true_clt_c - 20.0) / 70.0)).clamp(0.65, 1.0);
        let rpm_eff = (0.72 + 0.28 * (self.true_rpm / 4500.0).min(1.0))
            * if self.true_rpm > 6500.0 { 0.85 } else { 1.0 };
        let misfire = self
            .scenario
            .fault_active(SimFault::Misfire, self.time_ms());
        let combustion_efficiency =
            0.32 * mixture_eff * warmup * rpm_eff * if misfire { 0.55 } else { 1.0 };
        let fuel_power_w = fuel_kg_s * LHV_J_PER_KG * combustion_efficiency;
        let omega = self.true_rpm * core::f32::consts::TAU / 60.0;
        let indicated_torque = if omega > 1.0 {
            fuel_power_w / omega
        } else {
            0.0
        };
        let friction = 12.0 + 0.0015 * self.true_rpm + 1.2e-7 * self.true_rpm * self.true_rpm;
        let pumping = ((100.0 - self.true_map_kpa).max(0.0) * 0.06
            + (self.true_map_kpa - 100.0).max(0.0) * 0.03)
            * (self.true_rpm / 900.0).max(0.4);
        let accessory = if self.true_rpm > 50.0 { 1.5 } else { 0.0 };

        let dyno_load = if self.scenario.dyno_mode {
            if let Some(target) = self.scenario.phase_at(self.time_ms()).target_rpm {
                let err = self.true_rpm - target;
                self.dyno_integral = (self.dyno_integral + err * 0.01).clamp(0.0, 250.0);
                (err * 0.35 + self.dyno_integral).clamp(0.0, 300.0)
            } else {
                0.0
            }
        } else {
            self.scenario.phase_at(self.time_ms()).load_nm
        };
        self.load_torque_nm = dyno_load;
        let starter_torque = if self.starter_on { 45.0 } else { 0.0 };
        let net = indicated_torque + starter_torque - friction - pumping - accessory - dyno_load;
        let inertia = 0.28;
        let mut new_omega = omega + net / inertia * dt_s;
        if new_omega < 0.0 {
            new_omega = 0.0;
        }
        let new_rpm = (new_omega * 60.0 / core::f32::consts::TAU).clamp(0.0, 9000.0);
        self.true_rpm = new_rpm;

        self.cycle_lambda_actual = if self.fuel_cycle_kg > 1e-9 {
            (self.air_cycle_kg / (self.fuel_cycle_kg * self.calibration.fuel.stoich_afr as f64))
                as f32
        } else {
            2.0
        };
        self.cycle_lambda_actual = self.cycle_lambda_actual.clamp(0.5, 2.0);
        if misfire
            || self
                .scenario
                .fault_active(SimFault::ExhaustLeak, self.time_ms())
        {
            // Unburned oxygen and external air both bias the sensor lean. They are
            // not evidence that the fuel table is lean; the gate must reject them.
            self.cycle_lambda_actual =
                (self.cycle_lambda_actual + if misfire { 0.10 } else { 0.12 }).min(2.0);
        }
    }

    fn update_sensors(&mut self) {
        let noise = self.scenario.sensor_noise;
        self.map_sensor = if self
            .scenario
            .fault_active(SimFault::MapSensorStuck, self.time_ms())
        {
            55.0
        } else {
            self.true_map_kpa + self.rng.bipolar() * noise * 4.0
        };
        self.tps_sensor = (self.throttle_pct + self.rng.bipolar() * noise * 2.0).clamp(0.0, 100.0);
        self.clt_sensor = if self
            .scenario
            .fault_active(SimFault::CoolantSensorStale, self.time_ms())
        {
            35.0
        } else {
            self.true_clt_c + self.rng.bipolar() * 0.2
        };
        self.iat_sensor = self.true_iat_c + self.rng.bipolar() * 0.2;
        self.oil_sensor = self.true_oil_kpa + self.rng.bipolar() * 2.0;
        self.fuel_pressure_sensor = if self
            .scenario
            .fault_active(SimFault::FuelPressureSensorBlind, self.time_ms())
        {
            self.calibration.fuel.pressure_reference_kpa.max(250.0)
        } else {
            self.true_fuel_kpa + self.rng.bipolar() * 2.0
        };
        let heater_time = if self.scenario.start_coolant_c > 60.0 {
            600
        } else {
            5000
        };
        self.lambda_ready = self.time_ms() >= heater_time
            && !self
                .scenario
                .fault_active(SimFault::WidebandNotReady, self.time_ms());
        self.lambda_status = if self.lambda_ready {
            if self
                .scenario
                .fault_active(SimFault::WidebandStuckLean, self.time_ms())
                || self
                    .scenario
                    .fault_active(SimFault::WidebandStuckRich, self.time_ms())
            {
                // Deliberately keep status Ok: a stuck signal must be caught by
                // plausibility/coverage logic, not only by a status bit.
                SensorStatus::Ok
            } else {
                SensorStatus::Ok
            }
        } else {
            SensorStatus::NotReady
        };
        self.wideband = WidebandState {
            ready: self.lambda_ready,
            heater_on: self.lambda_ready,
            pumping_current_ma: (self.lambda_sensor - 1.0) * 8.0,
            sensor_temperature_c: if self.lambda_ready { 780.0 } else { 20.0 },
            status: self.lambda_status,
            response_tau_ms: self.scenario.wideband_tau_ms,
            transport_delay_ms: self.scenario.exhaust_delay_ms,
            bank: 1,
            controller_id: "virtual-lsu49".into(),
        };
    }

    fn advance_trigger(&mut self, dt_s: f32, step_start_us: u64) {
        let advance_deg = self.true_rpm * 6.0 * dt_s;
        let old = self.true_angle_deg;
        let new = (old + advance_deg).rem_euclid(720.0);
        let wrapped = new < old;
        let dropout = self
            .scenario
            .fault_active(SimFault::TriggerDropout, self.time_ms());
        if !dropout && advance_deg > 0.0 {
            let mut events: Vec<(f32, u64, bool)> = Vec::new();
            for rev in 0..2 {
                for tooth in 0..(self.decoder.config().teeth as usize) {
                    let pos = rev as f32 * 360.0
                        + tooth as f32 * (360.0 / self.decoder.config().teeth as f32);
                    if tooth
                        == (self.decoder.config().teeth - self.decoder.config().missing) as usize
                    {
                        continue;
                    }
                    if crossed(old, new, pos) {
                        let offset = (pos - old).rem_euclid(720.0);
                        let t = step_start_us + ((offset / advance_deg.max(1e-6)) * 1000.0) as u64;
                        events.push((pos, t, pos == 690.0));
                    }
                }
            }
            events.sort_by_key(|e| e.1);
            for (_pos, ts, cam) in events {
                // A cam edge may coincide with a crank tooth; both inputs must be
                // delivered. Dropping the crank tooth would break sync.
                self.decoder.on_tooth(ts);
                if cam {
                    self.decoder.on_cam(ts);
                }
            }
        }
        self.true_angle_deg = new;
        if wrapped {
            self.cycle_wrapped = true;
            self.complete_cycle();
        }
    }

    fn complete_cycle(&mut self) {
        // This is the only place the cycle lambda is updated. Recomputing it
        // during the next cycle would use a partially filled air/fuel accumulator
        // and produce a false lean spike every cycle.
        let mut lambda = if self.fuel_cycle_kg > 1e-9 {
            (self.air_cycle_kg / (self.fuel_cycle_kg * self.calibration.fuel.stoich_afr as f64))
                as f32
        } else {
            2.0
        };
        lambda = lambda.clamp(0.5, 2.0);
        if self
            .scenario
            .fault_active(SimFault::Misfire, self.time_ms())
        {
            lambda = (lambda + 0.10).min(2.0);
        }
        if self
            .scenario
            .fault_active(SimFault::ExhaustLeak, self.time_ms())
        {
            lambda = (lambda + 0.12).min(2.0);
        }
        self.cycle_lambda_actual = lambda;
        self.lambda_delay.push_back((self.time_us, lambda));
        self.air_cycle_kg = 0.0;
        self.fuel_cycle_kg = 0.0;
        self.injections_this_cycle = 0;
        self.sparks_this_cycle = 0;
        self.misfire_this_cycle = false;
        self.exhaust_delayed = false;
    }

    fn update_wideband(&mut self, dt_s: f32) {
        let delay_us = self.scenario.exhaust_delay_ms as u64 * 1000;
        while let Some((t, v)) = self.lambda_delay.front().copied() {
            if t + delay_us <= self.time_us {
                self.lambda_delay_target = v;
                self.lambda_delay.pop_front();
                self.exhaust_delayed = true;
            } else {
                break;
            }
        }
        let tau = (self.scenario.wideband_tau_ms as f32 / 1000.0).max(0.005);
        let alpha = (dt_s / (tau + dt_s)).clamp(0.0, 1.0);
        if self.lambda_ready {
            let stuck = if self
                .scenario
                .fault_active(SimFault::WidebandStuckLean, self.time_ms())
            {
                Some(1.18)
            } else if self
                .scenario
                .fault_active(SimFault::WidebandStuckRich, self.time_ms())
            {
                Some(0.78)
            } else {
                None
            };
            self.lambda_sensor = if let Some(v) = stuck {
                v
            } else {
                let noisy =
                    self.lambda_delay_target + self.rng.bipolar() * self.scenario.sensor_noise;
                self.lambda_sensor += alpha * (noisy - self.lambda_sensor);
                self.lambda_sensor
            };
        } else {
            self.lambda_sensor = 1.0;
        }
        self.lambda_sensor = self.lambda_sensor.clamp(0.55, 2.0);
    }

    fn compute_ecu_outputs(&mut self) {
        let measured_rpm = self.decoder.rpm();
        let rpm = if measured_rpm.is_finite() {
            measured_rpm
        } else {
            0.0
        };
        let map = self.map_sensor.clamp(5.0, 300.0);
        let lambda_target = self.lookup_target(rpm, map).unwrap_or(1.0).clamp(0.7, 1.1);
        self.target_lambda = lambda_target;
        let ve = self.lookup_ve(rpm, map).unwrap_or(0.0);
        self.ve = ve;
        if let Ok(t) = self.calibration.table("ve") {
            if let Some(idx) = t.cell_indices(&[map, rpm]) {
                self.ve_row = idx[0] as u16;
                self.ve_col = idx[1] as u16;
            }
        }

        // Closed-loop trim: evaluate gates on the current measured state.
        let base_cell_ltft = self
            .ltft
            .index(rpm, map)
            .map(|i| self.ltft.trim[i])
            .unwrap_or(0.0);
        let gate = self.learning_gate(rpm, map);
        let stft = self.closed_loop.current_trim();
        let injector = &self.calibration.engine.injectors[0];
        let inputs = FuelInputs {
            rpm,
            map_kpa: map,
            iat_k: self.iat_sensor + 273.15,
            tps_pct: self.tps_sensor,
            ve_percent: ve,
            target_lambda: lambda_target,
            stoich_afr: self.calibration.fuel.stoich_afr,
            fuel_density_g_cc: self.calibration.fuel.fuel_density_g_cc,
            battery_v: self.battery_v,
            fuel_pressure_kpa: self.fuel_pressure_sensor,
            stft,
            ltft: base_cell_ltft,
            displacement_cc: self.calibration.engine.displacement_cc,
            cylinders: self.calibration.engine.cylinders,
            pressure_compensation: self.calibration.fuel.pressure_compensation,
        };
        let mut fuel = calculate(&inputs, injector).ok();
        let mut fuel_cut = !self.decoder.is_sequential();
        if let Some(f) = &fuel {
            if f.inhibited.is_some() {
                fuel_cut = true;
            }
        } else {
            fuel_cut = true;
        }
        if rpm < 50.0 || !self.decoder.is_sequential() {
            fuel_cut = true;
        }

        let ignition_base = self.lookup_advance(rpm, map).unwrap_or(0.0);
        let mut advance = ignition_base.clamp(-10.0, 55.0);
        let dwell_base = self.lookup_dwell(self.battery_v).unwrap_or(3.0);
        let dwell = limit_dwell(
            rpm.max(1.0),
            dwell_base,
            1,
            self.calibration.engine.cylinders,
        )
        .unwrap_or(3.0);
        self.limiter.update(rpm);
        let limiter_cut = self.limiter.cut && rpm > self.calibration.engine.rev_limit as f32 * 0.95;

        // Protection has not run yet this step; use the previous decision for the
        // one-millisecond actuator command, then re-evaluate immediately below.
        if self.protection_decision.reaction.cut_spark {
            // Handled by evaluate_protection after; keep the schedule conservative
            // by marking cut now.
        }
        advance -= self.protection_decision.reaction.knock_retard_deg;
        advance = advance.clamp(-10.0, 55.0);

        self.advance_deg = advance;
        self.dwell_ms = dwell;
        self.fuel_cut = fuel_cut;
        self.spark_cut = limiter_cut;
        self.fuel = fuel.take();

        // Closed-loop update at 20 Hz once gates allow it.
        if self.time_ms().saturating_sub(self.last_stft_ms) >= 50 {
            self.last_stft_ms = self.time_ms();
            let dt_ms = 50.0;
            let _ = dt_ms;
            if self.lambda_ready && rpm > 300.0 {
                let out = self.closed_loop.update(
                    self.time_ms(),
                    self.lambda_sensor,
                    lambda_target,
                    &gate,
                );
                if out.active && !gate.allowed {
                    // Defensive: controller must never report active behind a gate.
                    self.fuel_cut = true;
                }
            } else {
                let blocked = ecu_lambda::GateDecision::blocked(vec![ecu_lambda::GateReason {
                    code: "LAMBDA_NOT_READY".into(),
                    message: "wideband is not ready".into(),
                }]);
                self.closed_loop.update(
                    self.time_ms(),
                    self.lambda_sensor,
                    lambda_target,
                    &blocked,
                );
            }
        }
        // LTFT learns from an active STFT only under the stricter learning gate.
        if self.time_ms().saturating_sub(self.last_ltft_ms) >= 100 {
            let dt_ms = 100.0;
            self.last_ltft_ms = self.time_ms();
            let stft_now = self.closed_loop.current_trim();
            let active = self.closed_loop.is_active();
            let gate = self.learning_gate(rpm, map);
            if let Some(idx) = self.ltft.index(rpm, map) {
                if let Some(update) = self.ltft.update(
                    self.time_ms(),
                    rpm,
                    map,
                    stft_now,
                    active,
                    &gate,
                    dt_ms,
                    &self.ltft_cfg,
                ) {
                    if update.samples == 1 || update.samples % 100 == 0 {
                        self.events.push(LogEvent {
                            timestamp_us: self.time_us,
                            severity: "info".into(),
                            source: "ltft".into(),
                            code: "LTFT_UPDATE".into(),
                            message: format!(
                                "cell ({},{}) trim {:.2}% confidence {:.2}",
                                update.row,
                                update.col,
                                update.new_trim * 100.0,
                                update.confidence
                            ),
                        });
                    }
                    let _ = idx;
                }
            }
        }
    }

    fn evaluate_protection(&mut self) {
        let measured_rpm = self.decoder.rpm();
        let lambda = if self.lambda_ready {
            self.lambda_sensor
        } else {
            self.target_lambda
        };
        let duty = self.fuel.as_ref().map(|f| f.duty_percent).unwrap_or(0.0);
        let knock = if self
            .scenario
            .fault_active(SimFault::KnockHigh, self.time_ms())
        {
            3.0
        } else {
            0.0
        };
        let inputs = ProtectionInputs {
            rpm: measured_rpm,
            map_kpa: self.map_sensor,
            tps_pct: self.tps_sensor,
            coolant_c: self.clt_sensor,
            oil_pressure_kpa: self.oil_sensor,
            fuel_pressure_kpa: self.fuel_pressure_sensor,
            battery_v: self.battery_v,
            lambda,
            target_lambda: self.target_lambda,
            injector_duty_pct: duty,
            knock_level: knock,
            sync_ok: self.decoder.is_sequential(),
            wideband_ready: self.lambda_ready,
            sensor_plausible: !self
                .scenario
                .fault_active(SimFault::CoolantSensorStale, self.time_ms())
                && !self
                    .scenario
                    .fault_active(SimFault::MapSensorStuck, self.time_ms()),
            calibration_crc_ok: self.calibration.verify(),
            watchdog_ok: true,
            can_ok: true,
        };
        self.protection_decision = self.protection.evaluate(&inputs, self.time_us);
        let reaction: Reaction = self.protection_decision.reaction;
        if reaction.cut_fuel {
            self.fuel_cut = true;
        }
        if reaction.cut_spark {
            self.spark_cut = true;
        }
        self.advance_deg = (self.advance_deg - reaction.knock_retard_deg).clamp(-10.0, 55.0);
        if self.protection_decision.state != self.last_protection {
            self.events.push(LogEvent {
                timestamp_us: self.time_us,
                severity: if matches!(self.protection_decision.state, CoreProtectionState::Cut) {
                    "error".into()
                } else {
                    "warning".into()
                },
                source: "protection".into(),
                code: "STATE_CHANGE".into(),
                message: format!(
                    "{:?} -> {:?}; active: {}",
                    self.last_protection,
                    self.protection_decision.state,
                    self.active_fault_codes().join(",")
                ),
            });
            self.last_protection = self.protection_decision.state;
        }
        if self.cycle_wrapped {
            self.rebuild_schedule();
        }
    }

    fn rebuild_schedule(&mut self) {
        let cycle = (self.true_angle_deg / 720.0).floor() as u64;
        let _ = cycle;
        let req = ScheduleRequest {
            rpm: self.decoder.rpm().max(1.0),
            pulse_width_ms: self
                .fuel
                .as_ref()
                .map(|f| if self.fuel_cut { 0.0 } else { f.pw_ms })
                .unwrap_or(0.0),
            dwell_ms: self.dwell_ms.max(0.5),
            advance_deg: self.advance_deg,
            injection_end_btdc_deg: 240.0,
            spark_cut: self.spark_cut,
            fuel_cut: self.fuel_cut,
        };
        match build_schedule(&self.calibration.engine.firing_order, &req) {
            Ok(s) => self.schedule = s,
            Err(e) => {
                if !self.schedule_error_logged {
                    self.schedule_error_logged = true;
                    self.events.push(LogEvent {
                        timestamp_us: self.time_us,
                        severity: "error".into(),
                        source: "scheduler".into(),
                        code: "SCHEDULE_ERROR".into(),
                        message: format!("{e}"),
                    });
                }
                self.schedule = EventSchedule::empty();
            }
        }
    }

    fn apply_events(&mut self, _step_start_us: u64) {
        let now_angle = self.true_angle_deg;
        let prev_angle = (now_angle - self.true_rpm * 6.0 * 0.001).rem_euclid(720.0);
        for e in self.schedule.events_crossed(prev_angle, now_angle) {
            if !self.decoder.is_sequential() {
                continue;
            }
            match e.kind {
                EventKind::InjectionStart => {
                    if !self.fuel_cut && e.duration_ms > 0.0 {
                        let inj = &self.calibration.engine.injectors[0];
                        let flow_g_s = inj.flow_cc_min * self.calibration.fuel.fuel_density_g_cc
                            / 60.0
                            * (self.fuel_pressure_sensor / inj.rated_pressure_kpa)
                                .max(0.0)
                                .sqrt();
                        let dead = inj.dead_time_at(self.battery_v);
                        let mg = ((e.duration_ms - dead).max(0.0) * flow_g_s).max(0.0);
                        let actual_blind = (self.true_fuel_kpa
                            / self.fuel_pressure_sensor.max(1.0))
                        .sqrt()
                        .clamp(0.2, 1.5);
                        let injector_factor = if self
                            .scenario
                            .fault_active(SimFault::InjectorFlowLow, self.time_ms())
                        {
                            0.80
                        } else {
                            1.0
                        };
                        self.fuel_cycle_kg += (mg * actual_blind * injector_factor) as f64 * 1e-6;
                        self.injections_this_cycle += 1;
                        self.last_injection_angle = e.angle_deg;
                    }
                }
                EventKind::Spark => {
                    if !self.spark_cut {
                        self.sparks_this_cycle += 1;
                        self.last_spark_angle = e.angle_deg;
                    }
                }
                EventKind::DwellStart => {}
            }
        }
    }

    fn learning_gate(&self, rpm: f32, map: f32) -> ecu_lambda::GateDecision {
        let faults = self.active_fault_codes();
        let fault = |needle: &str| faults.iter().any(|s| s.contains(needle));
        let overrun = self.tps_sensor < 1.0 && rpm > 1500.0 && !self.fuel_cut;
        let inputs = GateInputs {
            engine_running: self.decoder.is_sequential() && self.true_rpm > 400.0,
            cranking: self.starter_on,
            sync_sequential: self.decoder.is_sequential(),
            coolant_c: self.clt_sensor,
            rpm,
            map_kpa: map,
            tps_pct: self.tps_sensor,
            lambda: self.lambda_sensor,
            target_lambda: self.target_lambda,
            lambda_status: self.lambda_status,
            lambda_ready: self.lambda_ready,
            lambda_age_ms: 20,
            fuel_cut: self.fuel_cut,
            overrun,
            misfire: self
                .scenario
                .fault_active(SimFault::Misfire, self.time_ms())
                || fault("MISFIRE"),
            exhaust_leak: self
                .scenario
                .fault_active(SimFault::ExhaustLeak, self.time_ms())
                || fault("EXHAUST_LEAK"),
            knock_active: fault("P0325")
                || self
                    .scenario
                    .fault_active(SimFault::KnockHigh, self.time_ms()),
            overboost: fault("P0234"),
            critical_dtc: matches!(self.protection_decision.state, CoreProtectionState::Cut),
            sensor_stale: fault("SENSOR"),
            fuel_pressure_ok: self.fuel_pressure_sensor >= 180.0,
            oil_pressure_ok: self.oil_sensor >= 40.0,
            injector_duty_pct: self.fuel.as_ref().map(|f| f.duty_percent).unwrap_or(0.0),
            rpm_rate_rpm_s: 0.0,
            map_rate_kpa_s: 0.0,
            tps_rate_pct_s: 0.0,
        };
        evaluate_gate(
            &GateConfig {
                require_steady_state: false,
                ..self.gate_cfg.clone()
            },
            &inputs,
        )
    }

    fn active_fault_codes(&self) -> Vec<String> {
        let mut codes: Vec<String> = self
            .protection_decision
            .active
            .iter()
            .flatten()
            .map(|d: &Dtc| d.code().to_string())
            .collect();
        if self
            .scenario
            .fault_active(SimFault::Misfire, self.time_ms())
        {
            codes.push("MISFIRE_SIM".into());
        }
        if self
            .scenario
            .fault_active(SimFault::ExhaustLeak, self.time_ms())
        {
            codes.push("EXHAUST_LEAK".into());
        }
        if self
            .scenario
            .fault_active(SimFault::CoolantSensorStale, self.time_ms())
            || self
                .scenario
                .fault_active(SimFault::MapSensorStuck, self.time_ms())
        {
            codes.push("SENSOR_STALE".into());
        }
        if self
            .scenario
            .fault_active(SimFault::InjectorFlowLow, self.time_ms())
        {
            codes.push("INJECTOR_FLOW_LOW".into());
        }
        codes
    }

    fn track_fault_events(&mut self) {
        let active: Vec<SimFault> = self
            .scenario
            .faults
            .iter()
            .filter(|w: &&FaultWindow| self.time_ms() >= w.start_ms && self.time_ms() < w.end_ms)
            .map(|w| w.fault)
            .collect();
        for f in &active {
            if !self.last_faults.contains(f) {
                self.events.push(LogEvent {
                    timestamp_us: self.time_us,
                    severity: "warning".into(),
                    source: "simulator".into(),
                    code: format!("SIM_FAULT_{f:?}"),
                    message: format!("injected simulated fault {f:?}"),
                });
            }
        }
        for f in &self.last_faults {
            if !active.contains(f) {
                self.events.push(LogEvent {
                    timestamp_us: self.time_us,
                    severity: "info".into(),
                    source: "simulator".into(),
                    code: format!("SIM_FAULT_CLEAR_{f:?}"),
                    message: format!("simulated fault {f:?} cleared"),
                });
            }
        }
        self.last_faults = active;
    }

    fn engine_state(&self) -> EngineState {
        if self.true_rpm < 20.0 {
            EngineState::Off
        } else if self.starter_on && !self.decoder.is_sequential() {
            EngineState::Cranking
        } else if self.fuel_cut {
            EngineState::Cut
        } else if self.decoder.is_sequential() {
            EngineState::Running
        } else {
            EngineState::Stalled
        }
    }

    fn sync_state(&self) -> SyncState {
        match self.decoder.state() {
            TriggerSync::Unsynced => SyncState::Unsynced,
            TriggerSync::CrankSynced => SyncState::CrankSynced,
            TriggerSync::Sequential => SyncState::Sequential,
            TriggerSync::Lost => SyncState::Lost,
        }
    }

    fn protection_state_public(&self) -> ProtectionState {
        match self.protection_decision.state {
            CoreProtectionState::Normal => ProtectionState::Normal,
            CoreProtectionState::Warning => ProtectionState::Warning,
            CoreProtectionState::Derated => ProtectionState::Derated,
            CoreProtectionState::Limp => ProtectionState::Limp,
            CoreProtectionState::Cut => ProtectionState::Cut,
        }
    }

    fn push_telemetry(&mut self) {
        if self.time_ms() > self.scenario.duration_ms {
            return;
        }
        let rpm = self.decoder.rpm();
        let ve = self.ve;
        let cell_ltft = self
            .ltft
            .index(rpm, self.map_sensor)
            .map(|i| self.ltft.trim[i])
            .unwrap_or(0.0);
        let cycle_angle = self.decoder.cycle_angle_at(self.time_us).unwrap_or(0.0);
        let f = TelemetryFrame {
            seq: self.seq,
            timestamp_us: self.time_us,
            rpm,
            map_kpa: self.map_sensor,
            tps_pct: self.tps_sensor,
            iat_c: self.iat_sensor,
            clt_c: self.clt_sensor,
            oil_pressure_kpa: self.oil_sensor,
            fuel_pressure_kpa: self.fuel_pressure_sensor,
            battery_v: self.battery_v,
            lambda_measured: if self.lambda_ready {
                self.lambda_sensor
            } else {
                0.0
            },
            lambda_target: self.target_lambda,
            lambda_status: self.lambda_status,
            lambda_ready: self.lambda_ready,
            lambda_age_ms: 20,
            lambda_bank: 1,
            exhaust_delay_ms: self.scenario.exhaust_delay_ms,
            wideband: self.wideband.clone(),
            ve,
            ve_row: self.ve_row,
            ve_col: self.ve_col,
            ignition_advance_deg: self.advance_deg,
            dwell_ms: self.dwell_ms,
            injector_pw_ms: self.fuel.as_ref().map(|f| f.pw_ms).unwrap_or(0.0),
            injector_duty_pct: self.fuel.as_ref().map(|f| f.duty_percent).unwrap_or(0.0),
            stft: self.closed_loop.current_trim(),
            ltft: cell_ltft,
            closed_loop: self.closed_loop.is_active(),
            sync: self.sync_state(),
            cycle_angle_deg: cycle_angle,
            engine_state: self.engine_state(),
            protection: self.protection_state_public(),
            active_faults: self.active_fault_codes(),
            fuel_cut: self.fuel_cut,
            spark_cut: self.spark_cut,
            knock_level: if self
                .scenario
                .fault_active(SimFault::KnockHigh, self.time_ms())
            {
                3.0
            } else {
                0.0
            },
            boost_kpa: (self.map_sensor - 100.0).max(0.0),
            speed_kph: (rpm / 40.0).clamp(0.0, 300.0),
            gear: 3,
            ethanol_pct: 0.0,
            starter_active: self.starter_on,
            throttle_commanded_pct: self.tps_sensor,
            dyno_load_nm: self.load_torque_nm,
            scenario_time_ms: self.time_ms(),
            log_note: self.scenario.phase_at(self.time_ms()).note.clone(),
        };
        self.frames.push(f);
        self.seq += 1;
    }

    fn lookup_ve(&self, rpm: f32, map: f32) -> Option<f32> {
        self.calibration
            .table("ve")
            .ok()
            .and_then(|t| t.interpolate(&[map, rpm]).ok())
    }

    fn lookup_target(&self, rpm: f32, map: f32) -> Option<f32> {
        self.calibration
            .table("target_lambda")
            .ok()
            .and_then(|t| t.interpolate(&[map, rpm]).ok())
    }

    fn lookup_advance(&self, rpm: f32, map: f32) -> Option<f32> {
        self.calibration
            .table("ignition")
            .ok()
            .and_then(|t| t.interpolate(&[map, rpm]).ok())
    }

    fn lookup_dwell(&self, volts: f32) -> Option<f32> {
        self.calibration
            .table("dwell")
            .ok()
            .and_then(|t| t.interpolate(&[volts]).ok())
    }
}

fn crossed(old: f32, new: f32, event: f32) -> bool {
    let o = old.rem_euclid(720.0);
    let n = new.rem_euclid(720.0);
    let e = event.rem_euclid(720.0);
    if o <= n {
        e > o && e <= n
    } else {
        e > o || e <= n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::by_id;
    use ecu_calibration::wizard::{generate_starter, WizardRequest};

    fn cal() -> Calibration {
        generate_starter(&WizardRequest::default_inline4(), 1).unwrap()
    }

    #[test]
    fn starts_with_valid_preflight_and_emits_telemetry() {
        let mut s = SimEngine::new(cal(), by_id("idle_warmup").unwrap()).unwrap();
        s.advance_ms(2000);
        assert!(!s.frames().is_empty());
        let f = s.last_frame().unwrap();
        assert!(f.rpm > 0.0);
        assert!(f.timestamp_us > 0);
    }

    #[test]
    fn mixture_sweep_produces_running_engine_and_lambda() {
        let mut s = SimEngine::new(cal(), by_id("mixture_sweep").unwrap()).unwrap();
        s.run_to_end();
        let frames = s.frames();
        assert!(frames.len() > 1000);
        let running = frames
            .iter()
            .filter(|f| f.engine_state == EngineState::Running)
            .count();
        assert!(
            running > frames.len() / 2,
            "running={running} total={}",
            frames.len()
        );
        let valid = frames
            .iter()
            .filter(|f| f.lambda_ready && f.lambda_measured.is_finite() && f.lambda_measured > 0.5)
            .count();
        assert!(valid > 100, "valid lambda samples={valid}");
        assert!(frames.iter().any(|f| f.sync == SyncState::Sequential));
    }

    #[test]
    fn mixture_map_recovers_deliberate_ve_bias_and_applies() {
        let calibration = cal();
        let mut s = SimEngine::new(calibration.clone(), by_id("mixture_sweep").unwrap()).unwrap();
        s.run_to_end();
        let opts = ecu_lambda::MixtureMapOptions {
            min_dwell_ms: 800.0,
            min_samples_per_cell: 12,
            max_lambda_stddev: 0.035,
            min_confidence: 0.45,
            ..ecu_lambda::MixtureMapOptions::default()
        };
        let report = ecu_lambda::mixture_map::analyze(&calibration, s.frames(), &opts).unwrap();
        assert!(report.hard_faults.is_empty(), "{:?}", report.hard_faults);
        let proposal = report
            .proposal
            .expect("mixture sweep must propose at least one cell");
        assert!(proposal.cells.len() >= 3, "cells={}", proposal.cells.len());
        let mean = proposal
            .cells
            .iter()
            .map(|c| c.correction_fraction)
            .sum::<f32>()
            / proposal.cells.len() as f32;
        assert!(
            mean > 0.02,
            "expected lean recovery, mean correction={mean}"
        );
        assert!(proposal.ignition_effect.contains("none"));
        let applied = ecu_lambda::apply_proposal(&calibration, &proposal, None, 42).unwrap();
        assert!(applied.calibration.verify());
        assert!(applied.applied_cells >= 3);
    }

    #[test]
    fn stuck_wideband_is_detected_and_no_proposal_issued() {
        let calibration = cal();
        let mut s =
            SimEngine::new(calibration.clone(), by_id("sensor_stuck_lean").unwrap()).unwrap();
        s.run_to_end();
        let report = ecu_lambda::mixture_map::analyze(
            &calibration,
            s.frames(),
            &ecu_lambda::MixtureMapOptions::default(),
        )
        .unwrap();
        assert!(
            report
                .hard_faults
                .contains(&"LAMBDA_STUCK_SUSPECTED".to_string()),
            "hard={:?} frames={} bank={} used={} rejected={} gates={:?}",
            report.hard_faults,
            report.total_frames,
            report.bank_frames,
            report.samples_used,
            report.samples_rejected,
            report.gate_reason_counts
        );
        assert!(report.proposal.is_none());
    }

    #[test]
    fn cold_start_reaches_running_state() {
        let mut s = SimEngine::new(cal(), by_id("idle_warmup").unwrap()).unwrap();
        s.run_to_end();
        let frames = s.frames();
        assert!(frames
            .iter()
            .any(|f| f.engine_state == EngineState::Running));
        assert!(frames.iter().any(|f| f.sync == SyncState::Sequential));
        assert!(frames.last().unwrap().clt_c > 40.0);
    }

    #[test]
    fn trigger_loss_causes_cut_then_resync() {
        let mut s = SimEngine::new(cal(), by_id("trigger_loss").unwrap()).unwrap();
        s.run_to_end();
        let frames = s.frames();
        assert!(frames
            .iter()
            .any(|f| f.sync == SyncState::Lost || f.protection == ProtectionState::Cut));
        assert!(frames.last().unwrap().rpm > 0.0, "engine should recover");
    }
}
