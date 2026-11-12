use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::sensor::{SensorStatus, WidebandState};
use crate::SCHEMA_VERSION;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncState {
    Unsynced,
    CrankSynced,
    Sequential,
    Lost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineState {
    Off,
    Cranking,
    Running,
    Stalled,
    Cut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectionState {
    Normal,
    Warning,
    Derated,
    Limp,
    Cut,
}

/// One deterministic telemetry observation. All fields are recorded so an
/// offline reviewer can reconstruct why a cell was learned or rejected.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetryFrame {
    pub seq: u64,
    pub timestamp_us: u64,
    pub rpm: f32,
    pub map_kpa: f32,
    pub tps_pct: f32,
    pub iat_c: f32,
    pub clt_c: f32,
    pub oil_pressure_kpa: f32,
    pub fuel_pressure_kpa: f32,
    pub battery_v: f32,
    pub lambda_measured: f32,
    pub lambda_target: f32,
    pub lambda_status: SensorStatus,
    pub lambda_ready: bool,
    pub lambda_age_ms: u32,
    pub lambda_bank: u8,
    pub exhaust_delay_ms: u16,
    pub wideband: WidebandState,
    pub ve: f32,
    pub ve_row: u16,
    pub ve_col: u16,
    pub ignition_advance_deg: f32,
    pub dwell_ms: f32,
    pub injector_pw_ms: f32,
    pub injector_duty_pct: f32,
    pub stft: f32,
    pub ltft: f32,
    pub closed_loop: bool,
    pub sync: SyncState,
    pub cycle_angle_deg: f32,
    pub engine_state: EngineState,
    pub protection: ProtectionState,
    pub active_faults: Vec<String>,
    pub fuel_cut: bool,
    pub spark_cut: bool,
    pub knock_level: f32,
    pub boost_kpa: f32,
    pub speed_kph: f32,
    pub gear: i8,
    pub ethanol_pct: f32,
    pub starter_active: bool,
    pub throttle_commanded_pct: f32,
    pub dyno_load_nm: f32,
    pub scenario_time_ms: u64,
    pub log_note: String,
}

impl Default for TelemetryFrame {
    fn default() -> Self {
        Self {
            seq: 0,
            timestamp_us: 0,
            rpm: 0.0,
            map_kpa: 0.0,
            tps_pct: 0.0,
            iat_c: 20.0,
            clt_c: 20.0,
            oil_pressure_kpa: 0.0,
            fuel_pressure_kpa: 300.0,
            battery_v: 12.6,
            lambda_measured: 0.0,
            lambda_target: 1.0,
            lambda_status: SensorStatus::NotReady,
            lambda_ready: false,
            lambda_age_ms: 0,
            lambda_bank: 1,
            exhaust_delay_ms: 60,
            wideband: WidebandState::not_ready(1),
            ve: 0.0,
            ve_row: 0,
            ve_col: 0,
            ignition_advance_deg: 0.0,
            dwell_ms: 0.0,
            injector_pw_ms: 0.0,
            injector_duty_pct: 0.0,
            stft: 0.0,
            ltft: 0.0,
            closed_loop: false,
            sync: SyncState::Unsynced,
            cycle_angle_deg: 0.0,
            engine_state: EngineState::Off,
            protection: ProtectionState::Normal,
            active_faults: Vec::new(),
            fuel_cut: false,
            spark_cut: false,
            knock_level: 0.0,
            boost_kpa: 0.0,
            speed_kph: 0.0,
            gear: 0,
            ethanol_pct: 0.0,
            starter_active: false,
            throttle_commanded_pct: 0.0,
            dyno_load_nm: 0.0,
            scenario_time_ms: 0,
            log_note: String::new(),
        }
    }
}

impl TelemetryFrame {
    pub fn schema_version(&self) -> u16 {
        SCHEMA_VERSION
    }

    /// A compact row for CSV export. Every quantity has an explicit unit in the
    /// header; do not reorder without changing `CSV_HEADER`.
    pub const CSV_HEADER: &'static str = "time_ms,rpm,map_kpa,tps_pct,iat_c,clt_c,oil_kpa,fuel_kpa,battery_v,lambda,lambda_target,lambda_status,lambda_ready,ve,ve_row,ve_col,advance_deg,dwell_ms,pw_ms,duty_pct,stft,ltft,closed_loop,sync,cycle_angle,engine_state,protection,fuel_cut,spark_cut,faults";

    pub fn csv_row(&self) -> String {
        alloc::format!(
            "{},{:.1},{:.2},{:.2},{:.2},{:.2},{:.1},{:.1},{:.2},{:.4},{:.4},{:?},{},{:.2},{},{},{:.2},{:.3},{:.3},{:.2},{:.4},{:.4},{},{:?},{:.2},{:?},{:?},{},{},{}",
            self.scenario_time_ms,
            self.rpm,
            self.map_kpa,
            self.tps_pct,
            self.iat_c,
            self.clt_c,
            self.oil_pressure_kpa,
            self.fuel_pressure_kpa,
            self.battery_v,
            self.lambda_measured,
            self.lambda_target,
            self.lambda_status,
            self.lambda_ready,
            self.ve,
            self.ve_row,
            self.ve_col,
            self.ignition_advance_deg,
            self.dwell_ms,
            self.injector_pw_ms,
            self.injector_duty_pct,
            self.stft,
            self.ltft,
            self.closed_loop,
            self.sync,
            self.cycle_angle_deg,
            self.engine_state,
            self.protection,
            self.fuel_cut,
            self.spark_cut,
            self.active_faults.join("|"),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogMetadata {
    pub schema_version: u16,
    pub software_version: String,
    pub scenario_id: String,
    pub scenario_name: String,
    pub seed: u64,
    pub calibration_id: String,
    pub calibration_revision: u32,
    pub calibration_checksum: String,
    pub started_unix_ms: u64,
    pub sample_hz: u16,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogFile {
    pub metadata: LogMetadata,
    pub frames: Vec<TelemetryFrame>,
    pub events: Vec<LogEvent>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogEvent {
    pub timestamp_us: u64,
    pub severity: String,
    pub source: String,
    pub code: String,
    pub message: String,
}

impl LogFile {
    pub fn to_csv(&self) -> String {
        let mut out = String::new();
        out.push_str(TelemetryFrame::CSV_HEADER);
        out.push('\n');
        for f in &self.frames {
            out.push_str(&f.csv_row());
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_has_stable_header_and_row_width() {
        let f = TelemetryFrame::default();
        let row = f.csv_row();
        assert_eq!(
            TelemetryFrame::CSV_HEADER.split(',').count(),
            row.split(',').count()
        );
    }
}
