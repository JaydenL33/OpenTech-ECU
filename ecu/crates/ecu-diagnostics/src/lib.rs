//! Session logging, replay and summaries.
//!
//! Logs are the evidence trail for offline mixture analysis. Frame rows include
//! the calibration checksum, seeds and every gate input needed to reproduce a
//! learning decision.

#![forbid(unsafe_code)]

use std::string::String;
use std::vec::Vec;

use ecu_types::{LogEvent, LogFile, LogMetadata, TelemetryFrame};

#[derive(Clone, Debug)]
pub struct SessionLog {
    metadata: LogMetadata,
    frames: Vec<TelemetryFrame>,
    events: Vec<LogEvent>,
    capacity: usize,
}

impl SessionLog {
    pub fn new(metadata: LogMetadata, capacity: usize) -> Self {
        Self {
            metadata,
            frames: Vec::new(),
            events: Vec::new(),
            capacity: capacity.max(1),
        }
    }

    pub fn push_frame(&mut self, f: TelemetryFrame) {
        if self.frames.len() >= self.capacity {
            // Ring-buffer semantics: preserve the most recent evidence.
            self.frames.remove(0);
        }
        self.frames.push(f);
    }

    pub fn push_event(
        &mut self,
        timestamp_us: u64,
        severity: &str,
        source: &str,
        code: &str,
        message: &str,
    ) {
        if self.events.len() >= self.capacity {
            self.events.remove(0);
        }
        self.events.push(LogEvent {
            timestamp_us,
            severity: severity.into(),
            source: source.into(),
            code: code.into(),
            message: message.into(),
        });
    }

    pub fn frames(&self) -> &[TelemetryFrame] {
        &self.frames
    }

    pub fn events(&self) -> &[LogEvent] {
        &self.events
    }

    pub fn metadata(&self) -> &LogMetadata {
        &self.metadata
    }

    pub fn update_calibration(&mut self, id: &str, revision: u32, checksum: &str) {
        self.metadata.calibration_id = id.into();
        self.metadata.calibration_revision = revision;
        self.metadata.calibration_checksum = checksum.into();
    }

    pub fn finish(self) -> LogFile {
        LogFile {
            metadata: self.metadata,
            frames: self.frames,
            events: self.events,
        }
    }

    pub fn summary(&self) -> SessionSummary {
        session_summary(&self.frames)
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SessionSummary {
    pub samples: usize,
    pub duration_ms: u64,
    pub mean_rpm: f32,
    pub max_rpm: f32,
    pub max_coolant_c: f32,
    pub min_oil_pressure_kpa: f32,
    pub mean_lambda: Option<f32>,
    pub lambda_samples: usize,
    pub fault_codes: Vec<String>,
    pub protection_cut_samples: usize,
}

pub fn session_summary(frames: &[TelemetryFrame]) -> SessionSummary {
    if frames.is_empty() {
        return SessionSummary {
            samples: 0,
            duration_ms: 0,
            mean_rpm: 0.0,
            max_rpm: 0.0,
            max_coolant_c: 0.0,
            min_oil_pressure_kpa: 0.0,
            mean_lambda: None,
            lambda_samples: 0,
            fault_codes: Vec::new(),
            protection_cut_samples: 0,
        };
    }
    let mut sum_rpm = 0.0f64;
    let mut max_rpm = 0.0f32;
    let mut max_coolant = f32::MIN;
    let mut min_oil = f32::MAX;
    let mut sum_lambda = 0.0f64;
    let mut lambda_n = 0usize;
    let mut faults = Vec::new();
    let mut cut = 0usize;
    for f in frames {
        sum_rpm += f.rpm as f64;
        max_rpm = max_rpm.max(f.rpm);
        max_coolant = max_coolant.max(f.clt_c);
        min_oil = min_oil.min(f.oil_pressure_kpa);
        if f.lambda_measured.is_finite() {
            sum_lambda += f.lambda_measured as f64;
            lambda_n += 1;
        }
        for code in &f.active_faults {
            if !faults.contains(code) {
                faults.push(code.clone());
            }
        }
        if f.fuel_cut || f.spark_cut || f.protection == ecu_types::ProtectionState::Cut {
            cut += 1;
        }
    }
    let first = frames.first().unwrap().timestamp_us;
    let last = frames.last().unwrap().timestamp_us;
    SessionSummary {
        samples: frames.len(),
        duration_ms: last.saturating_sub(first) / 1000,
        mean_rpm: (sum_rpm / frames.len() as f64) as f32,
        max_rpm,
        max_coolant_c: max_coolant,
        min_oil_pressure_kpa: min_oil,
        mean_lambda: if lambda_n > 0 {
            Some((sum_lambda / lambda_n as f64) as f32)
        } else {
            None
        },
        lambda_samples: lambda_n,
        fault_codes: faults,
        protection_cut_samples: cut,
    }
}

/// Deterministic log replay. Frames are assumed time-ordered; the cursor emits
/// frames due at the current playback time, never skips silently, and can be
/// reset for byte-identical repeat runs.
#[derive(Clone, Debug)]
pub struct ReplayCursor {
    frames: Vec<TelemetryFrame>,
    index: usize,
    playback_us: u64,
    speed: f32,
    base_timestamp_us: u64,
}

impl ReplayCursor {
    pub fn new(frames: Vec<TelemetryFrame>, speed: f32) -> Self {
        let base = frames.first().map(|f| f.timestamp_us).unwrap_or(0);
        Self {
            frames,
            index: 0,
            playback_us: 0,
            speed: speed.clamp(0.1, 20.0),
            base_timestamp_us: base,
        }
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.playback_us = 0;
    }

    pub fn is_finished(&self) -> bool {
        self.index >= self.frames.len()
    }

    pub fn playback_ms(&self) -> u64 {
        self.playback_us / 1000
    }

    /// Advance by wall-clock milliseconds and return due frames in order.
    pub fn step(&mut self, wall_dt_ms: f32) -> Vec<TelemetryFrame> {
        if self.frames.is_empty() {
            return Vec::new();
        }
        self.playback_us = self
            .playback_us
            .saturating_add((wall_dt_ms.max(0.0) * self.speed * 1000.0) as u64);
        let target = self.base_timestamp_us.saturating_add(self.playback_us);
        let mut out = Vec::new();
        while self.index < self.frames.len() && self.frames[self.index].timestamp_us <= target {
            out.push(self.frames[self.index].clone());
            self.index += 1;
        }
        out
    }

    pub fn seek_ms(&mut self, ms: u64) {
        self.playback_us = ms * 1000;
        let target = self.base_timestamp_us.saturating_add(self.playback_us);
        self.index = self.frames.partition_point(|f| f.timestamp_us < target);
    }

    pub fn remaining(&self) -> usize {
        self.frames.len().saturating_sub(self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecu_types::LogMetadata;

    fn meta() -> LogMetadata {
        LogMetadata {
            schema_version: 1,
            software_version: "0.1.0".into(),
            scenario_id: "test".into(),
            scenario_name: "test".into(),
            seed: 1,
            calibration_id: "cal".into(),
            calibration_revision: 1,
            calibration_checksum: "abc".into(),
            started_unix_ms: 0,
            sample_hz: 50,
            notes: Vec::new(),
        }
    }

    fn frame(t: u64, rpm: f32) -> TelemetryFrame {
        TelemetryFrame {
            timestamp_us: t * 1000,
            rpm,
            lambda_measured: 1.0,
            lambda_status: ecu_types::SensorStatus::Ok,
            ..TelemetryFrame::default()
        }
    }

    #[test]
    fn replay_emits_every_frame_once_in_order() {
        let frames = vec![frame(0, 900.0), frame(20, 950.0), frame(40, 1000.0)];
        let mut r = ReplayCursor::new(frames, 1.0);
        let mut all = Vec::new();
        for _ in 0..3 {
            all.extend(r.step(20.0));
        }
        assert_eq!(all.len(), 3);
        assert!(all
            .windows(2)
            .all(|w| w[0].timestamp_us <= w[1].timestamp_us));
        assert!(r.is_finished());
        r.reset();
        assert!(!r.is_finished());
    }

    #[test]
    fn replay_speed_and_seek_are_deterministic() {
        let frames = vec![frame(0, 900.0), frame(100, 950.0), frame(200, 1000.0)];
        let mut r = ReplayCursor::new(frames, 2.0);
        let first = r.step(50.0);
        assert_eq!(first.len(), 2); // 50 ms wall at 2x reaches t=100 ms
        r.seek_ms(150);
        let rest = r.step(50.0);
        assert_eq!(rest.len(), 1); // only the 200 ms frame remains
    }

    #[test]
    fn summary_reports_faults_and_lambda_coverage() {
        let mut a = frame(0, 1000.0);
        a.active_faults.push("P0217".into());
        a.fuel_cut = true;
        let b = frame(20, 3000.0);
        let s = session_summary(&[a, b]);
        assert_eq!(s.samples, 2);
        assert!(s.max_rpm >= 3000.0);
        assert!(s.fault_codes.contains(&"P0217".to_string()));
        assert_eq!(s.protection_cut_samples, 1);
        assert_eq!(s.lambda_samples, 2);
    }

    #[test]
    fn ring_buffer_keeps_recent_frames() {
        let mut log = SessionLog::new(meta(), 2);
        log.push_frame(frame(0, 900.0));
        log.push_frame(frame(20, 950.0));
        log.push_frame(frame(40, 1000.0));
        assert_eq!(log.frames().len(), 2);
        assert_eq!(log.frames()[0].timestamp_us, 20_000);
    }
}
