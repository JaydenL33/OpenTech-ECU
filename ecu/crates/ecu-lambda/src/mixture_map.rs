use std::collections::BTreeMap;
use std::string::String;
use std::vec::Vec;

use ecu_calibration::table::{diff_tables, TableDiff};
use ecu_calibration::{hex_sha256, Calibration};
use ecu_types::{EngineState, LogFile, ProtectionState, SensorStatus, SyncState, TelemetryFrame};
use serde::{Deserialize, Serialize};

use crate::gate::{evaluate, GateConfig, GateInputs};

pub const VE_TABLE_KEY: &str = "ve";

/// Offline Mixture Map: replay a log, align lambda to the fuel command that
/// produced it, filter unsafe/unsteady samples, and propose bounded VE changes.
///
/// It never changes ignition. It never applies automatically: the proposal is
/// data, and `apply_proposal` requires an explicit selection and matching base
/// checksum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MixtureMapOptions {
    pub min_samples_per_cell: u32,
    pub min_dwell_ms: f32,
    pub max_lambda_stddev: f32,
    pub max_cell_correction: f32,
    pub max_total_correction: f32,
    pub min_confidence: f32,
    pub allow_wot: bool,
    pub require_steady_state: bool,
    pub min_fuel_pressure_kpa: f32,
    pub min_oil_pressure_kpa: f32,
    pub bank: u8,
    pub time_align_exhaust: bool,
    /// Extra settling time (ms) added to the exhaust transport delay. A lambda
    /// sample is only used if no transient, misfire, fuel/spark cut or trim
    /// change occurred anywhere in `[t - exhaust_delay - holdoff, t]`, because
    /// the sensor reading reflects fuel delivered that long ago. This is a
    /// proxy: `TelemetryFrame` carries no accel-enrichment channel, so
    /// transient enrichment is detected from TPS/MAP/RPM rates only.
    #[serde(default = "default_transient_holdoff_ms")]
    pub transient_holdoff_ms: u32,
    /// Maximum permitted spread of (STFT + LTFT) inside the latency window.
    /// A moving trim means the fuel actually delivered is not the fuel the
    /// aligned frame reports. Initial value is an unvalidated engineering
    /// guess, not a calibrated figure.
    #[serde(default = "default_max_trim_swing")]
    pub max_trim_swing: f32,
}

fn default_transient_holdoff_ms() -> u32 {
    250
}

fn default_max_trim_swing() -> f32 {
    0.06
}

impl Default for MixtureMapOptions {
    fn default() -> Self {
        Self {
            min_samples_per_cell: 20,
            min_dwell_ms: 1500.0,
            max_lambda_stddev: 0.02,
            max_cell_correction: 0.05,
            max_total_correction: 0.20,
            min_confidence: 0.55,
            allow_wot: false,
            require_steady_state: true,
            min_fuel_pressure_kpa: 200.0,
            min_oil_pressure_kpa: 40.0,
            bank: 1,
            time_align_exhaust: true,
            transient_holdoff_ms: default_transient_holdoff_ms(),
            max_trim_swing: default_max_trim_swing(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellStatus {
    Proposed,
    OnTarget,
    InsufficientSamples,
    HighVariance,
    LowConfidence,
    WotExcluded,
    NoSamples,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MixtureCellReport {
    pub row: usize,
    pub col: usize,
    pub rpm: f32,
    pub map_kpa: f32,
    pub samples_seen: u32,
    pub samples_used: u32,
    pub dwell_ms: f32,
    pub mean_lambda: Option<f32>,
    pub mean_target_lambda: Option<f32>,
    pub lambda_stddev: f32,
    pub mean_stft: f32,
    pub mean_ltft: f32,
    pub mean_total_trim: Option<f32>,
    pub current_ve: f32,
    pub proposed_ve: f32,
    pub correction_fraction: f32,
    pub confidence: f32,
    pub status: CellStatus,
    pub exclusion_reasons: BTreeMap<String, u32>,
    /// Samples that passed the instantaneous gate but were dropped because of a
    /// disturbance inside the latency window (`WINDOW_*` reasons). Counted once
    /// per sample, not once per reason.
    #[serde(default)]
    pub window_rejected_samples: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProposedVeCell {
    pub row: usize,
    pub col: usize,
    pub rpm: f32,
    pub map_kpa: f32,
    pub current_ve: f32,
    pub proposed_ve: f32,
    pub correction_fraction: f32,
    pub confidence: f32,
    pub samples: u32,
    pub mean_lambda: f32,
    pub target_lambda: f32,
    pub mean_trim: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VeProposal {
    pub proposal_id: String,
    pub calibration_id: String,
    pub calibration_revision: u32,
    pub base_checksum: String,
    pub table_key: String,
    pub cells: Vec<ProposedVeCell>,
    pub source_scenario: String,
    pub source_seed: u64,
    pub source_log_checksum: String,
    /// This workflow has no ignition output by construction.
    pub ignition_effect: String,
    pub requires_approval: bool,
    pub created_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MixtureMapReport {
    pub source_scenario: String,
    pub source_seed: u64,
    pub source_log_checksum: String,
    pub calibration_id: String,
    pub calibration_revision: u32,
    pub calibration_checksum: String,
    pub bank: u8,
    pub total_frames: usize,
    pub bank_frames: usize,
    pub samples_used: u32,
    pub samples_rejected: u32,
    pub gate_reason_counts: BTreeMap<String, u32>,
    /// Used samples taken while the STFT loop reported active.
    #[serde(default)]
    pub closed_loop_samples_used: u32,
    /// Used samples taken with the STFT loop inactive (open loop). The
    /// frame's applied STFT/LTFT is still folded into the pre-trim estimate.
    #[serde(default)]
    pub open_loop_samples_used: u32,
    pub cells: Vec<MixtureCellReport>,
    pub proposal: Option<VeProposal>,
    pub hard_faults: Vec<String>,
    pub ignition_untouched: bool,
    pub notes: Vec<String>,
}

impl MixtureMapReport {
    pub fn proposed_cells(&self) -> Vec<&ProposedVeCell> {
        self.proposal
            .as_ref()
            .map(|p| p.cells.iter().collect())
            .unwrap_or_default()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MixtureMapError {
    CalibrationTampered,
    MissingTable(String),
    AxisMismatch,
    NoFrames,
    InvalidOptions(String),
}

impl std::fmt::Display for MixtureMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MixtureMapError::CalibrationTampered => f.write_str("calibration checksum is invalid"),
            MixtureMapError::MissingTable(t) => write!(f, "missing table {t}"),
            MixtureMapError::AxisMismatch => f.write_str("VE and target-lambda axes do not match"),
            MixtureMapError::NoFrames => f.write_str("log has no frames"),
            MixtureMapError::InvalidOptions(s) => write!(f, "invalid mixture-map options: {s}"),
        }
    }
}

#[derive(Default)]
struct Acc {
    seen: u32,
    used: u32,
    sum_lambda: f64,
    sum_lambda_sq: f64,
    sum_target: f64,
    sum_stft: f64,
    sum_ltft: f64,
    sum_trim: f64,
    dwell_ms: f32,
    window_rejected: u32,
    exclusion: BTreeMap<String, u32>,
}

pub fn analyze(
    cal: &Calibration,
    frames: &[TelemetryFrame],
    options: &MixtureMapOptions,
) -> Result<MixtureMapReport, MixtureMapError> {
    if !cal.verify() {
        return Err(MixtureMapError::CalibrationTampered);
    }
    if frames.is_empty() {
        return Err(MixtureMapError::NoFrames);
    }
    if options.min_samples_per_cell == 0
        || options.max_cell_correction <= 0.0
        || options.max_cell_correction > 0.5
        || options.max_total_correction <= 0.0
        || options.max_total_correction > 0.5
        || options.min_confidence < 0.0
        || options.min_confidence > 1.0
        || !options.max_lambda_stddev.is_finite()
        || options.max_lambda_stddev <= 0.0
        || !options.max_trim_swing.is_finite()
        || options.max_trim_swing <= 0.0
    {
        return Err(MixtureMapError::InvalidOptions(
            "bounds must be finite and within documented limits".into(),
        ));
    }
    let ve = cal
        .table(VE_TABLE_KEY)
        .map_err(|_| MixtureMapError::MissingTable(VE_TABLE_KEY.into()))?;
    let target = cal
        .table("target_lambda")
        .map_err(|_| MixtureMapError::MissingTable("target_lambda".into()))?;
    if ve.rank() != 2 || target.rank() != 2 {
        return Err(MixtureMapError::AxisMismatch);
    }
    if ve.axes[0].breakpoints != target.axes[0].breakpoints
        || ve.axes[1].breakpoints != target.axes[1].breakpoints
    {
        return Err(MixtureMapError::AxisMismatch);
    }
    if ve.axes[0].breakpoints.iter().any(|v| !v.is_finite())
        || ve.axes[1].breakpoints.iter().any(|v| !v.is_finite())
    {
        return Err(MixtureMapError::AxisMismatch);
    }
    let rows = ve.axes[0].breakpoints.len();
    let cols = ve.axes[1].breakpoints.len();
    let mut acc: Vec<Acc> = (0..rows * cols).map(|_| Acc::default()).collect();
    let mut gate_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut used = 0u32;
    let mut rejected = 0u32;
    let mut bank_frames = 0usize;
    let log_checksum = hex_sha256(&serde_json::to_vec(frames).unwrap_or_default());

    // Stuck-sensor trap: a constant lambda across a wide operating range is not
    // credible mixture evidence. This catches sensors that still report Ok.
    let mut hard_faults = Vec::new();
    let mut run_len = 0u32;
    let mut run_prev = f32::NAN;
    let mut run_rpm_min = f32::MAX;
    let mut run_rpm_max = f32::MIN;
    let mut run_map_min = f32::MAX;
    let mut run_map_max = f32::MIN;
    for f in frames.iter() {
        if f.lambda_bank != options.bank
            || !f.lambda_ready
            || f.lambda_status != SensorStatus::Ok
            || !f.lambda_measured.is_finite()
        {
            continue;
        }
        if run_len == 0 || (f.lambda_measured - run_prev).abs() > 1e-5 {
            run_len = 1;
            run_prev = f.lambda_measured;
            run_rpm_min = f.rpm;
            run_rpm_max = f.rpm;
            run_map_min = f.map_kpa;
            run_map_max = f.map_kpa;
        } else {
            run_len += 1;
            run_rpm_min = run_rpm_min.min(f.rpm);
            run_rpm_max = run_rpm_max.max(f.rpm);
            run_map_min = run_map_min.min(f.map_kpa);
            run_map_max = run_map_max.max(f.map_kpa);
        }
        if run_len >= 50 && (run_rpm_max - run_rpm_min > 1000.0 || run_map_max - run_map_min > 30.0)
        {
            if !hard_faults.contains(&"LAMBDA_STUCK_SUSPECTED".to_string()) {
                hard_faults.push("LAMBDA_STUCK_SUSPECTED".into());
            }
            *gate_counts
                .entry("LAMBDA_STUCK_SUSPECTED".into())
                .or_insert(0) += 1;
        }
    }

    let learning_gate_cfg = GateConfig {
        allow_wot_adaptation: options.allow_wot,
        require_steady_state: options.require_steady_state,
        require_fuel_pressure_ok: true,
        require_oil_pressure_ok: true,
        ..GateConfig::default()
    };

    // Per-frame event flags for the latency-window check below. Thresholds
    // match the instantaneous gate so the two checks stay consistent.
    const EV_TRANSIENT: u8 = 1;
    const EV_MISFIRE: u8 = 2;
    const EV_CUT: u8 = 4;
    let event_flags: Vec<u8> = frames
        .iter()
        .enumerate()
        .map(|(j, fr)| {
            let prev = j.checked_sub(1).map(|k| &frames[k]);
            let (rr, mr, tr) = rates(prev, fr);
            let mut flags = 0u8;
            if options.require_steady_state
                && (!rr.is_finite()
                    || !mr.is_finite()
                    || !tr.is_finite()
                    || rr.abs() > learning_gate_cfg.max_rpm_rate_rpm_s
                    || mr.abs() > learning_gate_cfg.max_map_rate_kpa_s
                    || tr.abs() > learning_gate_cfg.max_tps_rate_pct_s)
            {
                flags |= EV_TRANSIENT;
            }
            if fr
                .active_faults
                .iter()
                .any(|s| s.contains("MISFIRE") || s.contains("EXHAUST_LEAK"))
            {
                flags |= EV_MISFIRE;
            }
            if fr.fuel_cut || fr.spark_cut {
                flags |= EV_CUT;
            }
            flags
        })
        .collect();
    let mut closed_loop_used = 0u32;
    let mut open_loop_used = 0u32;

    for (i, f) in frames.iter().enumerate() {
        if f.lambda_bank != options.bank {
            continue;
        }
        bank_frames += 1;
        let prev = i.checked_sub(1).map(|j| &frames[j]);
        let aligned_idx = if options.time_align_exhaust {
            aligned_index(frames, i, f.exhaust_delay_ms as u64)
        } else {
            Some(i)
        };
        let Some(ai) = aligned_idx else {
            *gate_counts
                .entry("NO_ALIGNED_FUEL_SAMPLE".into())
                .or_insert(0) += 1;
            rejected += 1;
            continue;
        };
        let a = &frames[ai];
        // Only samples whose bank matches the analyzed bank can inform that
        // bank's cells.
        if a.lambda_bank != options.bank {
            *gate_counts
                .entry("ALIGNED_BANK_MISMATCH".into())
                .or_insert(0) += 1;
            rejected += 1;
            continue;
        }
        let (rpm_rate, map_rate, tps_rate) = rates(prev, f);
        let faults = &f.active_faults;
        let fault = |needle: &str| faults.iter().any(|s| s.contains(needle));
        let overrun = f.tps_pct < 1.0 && f.rpm > 1500.0 && !f.fuel_cut;
        let critical = matches!(f.protection, ProtectionState::Limp | ProtectionState::Cut)
            || f.engine_state == EngineState::Stalled;
        let sensor_stale = (f.map_kpa == 0.0 && f.rpm > 0.0)
            || !f.clt_c.is_finite()
            || !f.iat_c.is_finite()
            || !f.oil_pressure_kpa.is_finite();
        let gate = evaluate(
            &learning_gate_cfg,
            &GateInputs {
                engine_running: f.engine_state == EngineState::Running,
                cranking: f.engine_state == EngineState::Cranking,
                sync_sequential: f.sync == SyncState::Sequential,
                coolant_c: f.clt_c,
                rpm: f.rpm,
                map_kpa: f.map_kpa,
                tps_pct: f.tps_pct,
                lambda: f.lambda_measured,
                target_lambda: f.lambda_target,
                lambda_status: f.lambda_status,
                lambda_ready: f.lambda_ready,
                lambda_age_ms: f.lambda_age_ms,
                fuel_cut: f.fuel_cut,
                overrun,
                misfire: fault("MISFIRE"),
                exhaust_leak: fault("EXHAUST_LEAK"),
                knock_active: f.knock_level > 2.0 || fault("KNOCK"),
                overboost: fault("OVERBOOST") || f.boost_kpa > 260.0,
                critical_dtc: critical,
                sensor_stale,
                fuel_pressure_ok: f.fuel_pressure_kpa >= options.min_fuel_pressure_kpa,
                oil_pressure_ok: f.oil_pressure_kpa >= options.min_oil_pressure_kpa,
                injector_duty_pct: f.injector_duty_pct,
                rpm_rate_rpm_s: rpm_rate,
                map_rate_kpa_s: map_rate,
                tps_rate_pct_s: tps_rate,
            },
        );

        // Use the fuel command at injection time for cell/trim alignment. This is
        // the pre-trim equivalent; the gate and operating point come from the
        // aligned sample so the measurement is attributed to the cell that
        // produced it.
        let Some(indices) = ve.cell_indices(&[a.map_kpa, a.rpm]) else {
            rejected += 1;
            continue;
        };
        let idx = indices[0] * cols + indices[1];
        acc[idx].seen += 1;
        if !gate.allowed {
            rejected += 1;
            for r in &gate.reasons {
                *gate_counts.entry(r.code.clone()).or_insert(0) += 1;
                *acc[idx].exclusion.entry(r.code.clone()).or_insert(0) += 1;
            }
            continue;
        }
        if gate.high_load && !options.allow_wot {
            rejected += 1;
            *gate_counts.entry("WOT_BOUNDARY".into()).or_insert(0) += 1;
            *acc[idx].exclusion.entry("WOT_BOUNDARY".into()).or_insert(0) += 1;
            continue;
        }
        let target_l = f.lambda_target;
        if !f.lambda_measured.is_finite() || !target_l.is_finite() || target_l <= 0.0 {
            rejected += 1;
            *gate_counts.entry("LAMBDA_IMPLAUSIBLE".into()).or_insert(0) += 1;
            *acc[idx]
                .exclusion
                .entry("LAMBDA_IMPLAUSIBLE".into())
                .or_insert(0) += 1;
            continue;
        }

        // Latency window: the lambda reading at `f` reflects fuel delivered
        // `exhaust_delay_ms` earlier, and needs `transient_holdoff_ms` to
        // settle. Any disturbance anywhere in that span invalidates it, even
        // if the current frame itself looks steady. A window that starts
        // before the log is clipped (no evidence of a disturbance).
        let transport_ms: u64 = if options.time_align_exhaust {
            f.exhaust_delay_ms as u64
        } else {
            0
        };
        let lead_ms: u64 = transport_ms + options.transient_holdoff_ms as u64;
        let start_us = f.timestamp_us.saturating_sub(lead_ms * 1000);
        let start = frames[..=i].partition_point(|x| x.timestamp_us < start_us);
        let window_flags = event_flags[start..=i].iter().fold(0u8, |m, b| m | b);
        let mut trim_min = f32::MAX;
        let mut trim_max = f32::MIN;
        let mut trim_finite = true;
        let mut saw_closed = false;
        let mut saw_open = false;
        for w in &frames[start..=i] {
            let t = w.stft + w.ltft;
            if t.is_finite() {
                trim_min = trim_min.min(t);
                trim_max = trim_max.max(t);
            } else {
                trim_finite = false;
            }
            if w.closed_loop {
                saw_closed = true;
            } else {
                saw_open = true;
            }
        }
        let mut window_reasons: Vec<&'static str> = Vec::new();
        if window_flags & EV_TRANSIENT != 0 {
            window_reasons.push("WINDOW_TRANSIENT");
        }
        if window_flags & EV_MISFIRE != 0 {
            window_reasons.push("WINDOW_MISFIRE");
        }
        if window_flags & EV_CUT != 0 {
            window_reasons.push("WINDOW_FUEL_OR_SPARK_CUT");
        }
        if !trim_finite || trim_max - trim_min > options.max_trim_swing {
            window_reasons.push("WINDOW_TRIM_UNSTABLE");
        }
        if saw_closed && saw_open {
            window_reasons.push("WINDOW_LOOP_STATE_CHANGE");
        }
        if !window_reasons.is_empty() {
            rejected += 1;
            acc[idx].window_rejected += 1;
            for code in window_reasons {
                *gate_counts.entry(code.to_string()).or_insert(0) += 1;
                *acc[idx].exclusion.entry(code.to_string()).or_insert(0) += 1;
            }
            continue;
        }
        if f.closed_loop {
            closed_loop_used += 1;
        } else {
            open_loop_used += 1;
        }

        let trim = (a.stft + a.ltft).clamp(-0.6, 0.6);
        acc[idx].used += 1;
        acc[idx].sum_lambda += f.lambda_measured as f64;
        acc[idx].sum_lambda_sq += (f.lambda_measured as f64).powi(2);
        acc[idx].sum_target += target_l as f64;
        acc[idx].sum_stft += a.stft as f64;
        acc[idx].sum_ltft += a.ltft as f64;
        acc[idx].sum_trim += trim as f64;
        // Approximate stable dwell from consecutive accepted samples.
        let dt = prev
            .map(|p| {
                ((f.timestamp_us.saturating_sub(p.timestamp_us)) as f32 / 1000.0).clamp(0.0, 200.0)
            })
            .unwrap_or(0.0);
        acc[idx].dwell_ms += dt;
        used += 1;
    }

    let mut cells = Vec::with_capacity(rows * cols);
    let mut proposed = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            let idx = row * cols + col;
            let a = &acc[idx];
            let current_ve = ve.values[idx];
            let confidence = confidence(a, options);
            let mean_lambda = if a.used > 0 {
                Some((a.sum_lambda / a.used as f64) as f32)
            } else {
                None
            };
            let mean_target = if a.used > 0 {
                Some((a.sum_target / a.used as f64) as f32)
            } else {
                None
            };
            let mean_trim = if a.used > 0 {
                Some((a.sum_trim / a.used as f64) as f32)
            } else {
                None
            };
            let stddev = if a.used >= 2 {
                let mean = a.sum_lambda / a.used as f64;
                let var = (a.sum_lambda_sq / a.used as f64 - mean * mean).max(0.0);
                var.sqrt() as f32
            } else {
                0.0
            };
            let status = classify(a, stddev, confidence, options);
            let mut proposed_ve = current_ve;
            if let (CellStatus::Proposed, Some(mean_lambda), Some(mean_target), Some(mean_trim)) =
                (status, mean_lambda, mean_target, mean_trim)
            {
                // Proportional relation for steady state, explicitly corrected
                // for any active trims so base VE is not double-corrected:
                // factor = (lambda_measured/lambda_target) * (1 + stft + ltft).
                let raw = (mean_lambda / mean_target) * (1.0 + mean_trim);
                let bounded = raw.clamp(
                    1.0 - options.max_total_correction,
                    1.0 + options.max_total_correction,
                );
                // Per-application bound is independent of the lifetime bound.
                let per_cell = bounded.clamp(
                    1.0 - options.max_cell_correction,
                    1.0 + options.max_cell_correction,
                );
                proposed_ve = (current_ve * per_cell).clamp(ve.value_min, ve.value_max);
                proposed.push(ProposedVeCell {
                    row,
                    col,
                    rpm: ve.axes[1].breakpoints[col],
                    map_kpa: ve.axes[0].breakpoints[row],
                    current_ve,
                    proposed_ve,
                    correction_fraction: proposed_ve / current_ve - 1.0,
                    confidence,
                    samples: a.used,
                    mean_lambda,
                    target_lambda: mean_target,
                    mean_trim,
                });
            }
            cells.push(MixtureCellReport {
                row,
                col,
                rpm: ve.axes[1].breakpoints[col],
                map_kpa: ve.axes[0].breakpoints[row],
                samples_seen: a.seen,
                samples_used: a.used,
                dwell_ms: a.dwell_ms,
                mean_lambda,
                mean_target_lambda: mean_target,
                lambda_stddev: stddev,
                mean_stft: if a.used > 0 {
                    (a.sum_stft / a.used as f64) as f32
                } else {
                    0.0
                },
                mean_ltft: if a.used > 0 {
                    (a.sum_ltft / a.used as f64) as f32
                } else {
                    0.0
                },
                mean_total_trim: mean_trim,
                current_ve,
                proposed_ve,
                correction_fraction: if current_ve > 0.0 {
                    proposed_ve / current_ve - 1.0
                } else {
                    0.0
                },
                confidence,
                status,
                exclusion_reasons: a.exclusion.clone(),
                window_rejected_samples: a.window_rejected,
            });
        }
    }

    let proposal = if proposed.is_empty() || !hard_faults.is_empty() {
        None
    } else {
        let mut hash_input = String::new();
        hash_input.push_str(&cal.checksum);
        hash_input.push_str(
            &frames
                .first()
                .map(|f| f.scenario_time_ms.to_string())
                .unwrap_or_default(),
        );
        hash_input.push_str(&frames.len().to_string());
        hash_input.push_str(&proposed.len().to_string());
        for cell in &proposed {
            hash_input.push_str(&format!(
                "{},{},{:.4};",
                cell.row, cell.col, cell.proposed_ve
            ));
        }
        let proposal_id = format!("mix-{}", &hex_sha256(hash_input.as_bytes())[..16]);
        Some(VeProposal {
            proposal_id,
            calibration_id: cal.id.clone(),
            calibration_revision: cal.revision,
            base_checksum: cal.checksum.clone(),
            table_key: VE_TABLE_KEY.into(),
            cells: proposed,
            source_scenario: cal.engine.name.clone(),
            source_seed: 0,
            source_log_checksum: log_checksum.clone(),
            ignition_effect: "none: lambda/fuel data is never used as ignition evidence".into(),
            requires_approval: true,
            created_unix_ms: 0,
        })
    };

    let notes = vec![
        "Measured cells only; unvisited cells are not fabricated or interpolated.".into(),
        "Proportional VE correction is a starting estimate and requires validation.".into(),
        "No ignition advance is proposed or implied. Lambda cannot establish safe timing.".into(),
        "Samples are rejected if a transient, misfire, fuel/spark cut, trim change or loop-state change occurred inside the exhaust-delay + holdoff window; accel enrichment is inferred from TPS/MAP/RPM rates only.".into(),
    ];

    Ok(MixtureMapReport {
        source_scenario: cal.engine.name.clone(),
        source_seed: 0,
        source_log_checksum: log_checksum,
        calibration_id: cal.id.clone(),
        calibration_revision: cal.revision,
        calibration_checksum: cal.checksum.clone(),
        bank: options.bank,
        total_frames: frames.len(),
        bank_frames,
        samples_used: used,
        samples_rejected: rejected,
        gate_reason_counts: gate_counts,
        closed_loop_samples_used: closed_loop_used,
        open_loop_samples_used: open_loop_used,
        cells,
        proposal,
        hard_faults,
        ignition_untouched: true,
        notes,
    })
}

/// Analyze a full session log while preserving its identity in the report and
/// proposal. This is the entry point used by the service and CLI.
pub fn analyze_log(
    cal: &Calibration,
    log: &LogFile,
    options: &MixtureMapOptions,
) -> Result<MixtureMapReport, MixtureMapError> {
    let mut report = analyze(cal, &log.frames, options)?;
    report.source_scenario = log.metadata.scenario_name.clone();
    report.source_seed = log.metadata.seed;
    if let Some(proposal) = &mut report.proposal {
        proposal.source_scenario = log.metadata.scenario_name.clone();
        proposal.source_seed = log.metadata.seed;
    }
    Ok(report)
}

fn rates(prev: Option<&TelemetryFrame>, cur: &TelemetryFrame) -> (f32, f32, f32) {
    let Some(p) = prev else {
        return (0.0, 0.0, 0.0);
    };
    let dt_s = (cur.timestamp_us.saturating_sub(p.timestamp_us)) as f32 / 1_000_000.0;
    if dt_s <= 0.0 {
        return (f32::INFINITY, f32::INFINITY, f32::INFINITY);
    }
    (
        (cur.rpm - p.rpm) / dt_s,
        (cur.map_kpa - p.map_kpa) / dt_s,
        (cur.tps_pct - p.tps_pct) / dt_s,
    )
}

fn aligned_index(frames: &[TelemetryFrame], current: usize, delay_ms: u64) -> Option<usize> {
    let target = frames[current].timestamp_us.saturating_sub(delay_ms * 1000);
    // Largest timestamp <= target. Frames are expected to be time-ordered.
    let mut lo = 0usize;
    let mut hi = current + 1;
    let mut found = None;
    while lo < hi {
        let mid = (lo + hi) / 2;
        if frames[mid].timestamp_us <= target {
            found = Some(mid);
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    found
}

fn confidence(a: &Acc, options: &MixtureMapOptions) -> f32 {
    if a.used == 0 {
        return 0.0;
    }
    let sample = (a.used as f32 / options.min_samples_per_cell as f32).min(1.0);
    let mean = a.sum_lambda / a.used as f64;
    let var = (a.sum_lambda_sq / a.used as f64 - mean * mean).max(0.0);
    let sd = var.sqrt() as f32;
    let variance = (1.0 - (sd / options.max_lambda_stddev).min(1.0)).max(0.0);
    let dwell = (a.dwell_ms / options.min_dwell_ms.max(1.0)).min(1.0);
    let base = 0.45 * sample + 0.35 * variance + 0.20 * dwell;
    (base * window_factor(a)).clamp(0.0, 1.0)
}

/// Maximum fraction of confidence removed when every otherwise-gate-clean
/// sample in a cell was dropped by the latency window. The weight is an
/// unvalidated engineering choice, not a calibrated figure.
const WINDOW_CONFIDENCE_PENALTY: f32 = 0.30;

/// Confidence multiplier in `[1 - WINDOW_CONFIDENCE_PENALTY, 1]`. A cell whose
/// instantaneously-clean samples were mostly lost to the latency window is
/// mostly visited during transients/trim movement, so the few survivors are
/// less representative of its steady behaviour.
fn window_factor(a: &Acc) -> f32 {
    let clean = a.used + a.window_rejected;
    if clean == 0 {
        return 1.0;
    }
    1.0 - WINDOW_CONFIDENCE_PENALTY * (a.window_rejected as f32 / clean as f32)
}

fn classify(a: &Acc, stddev: f32, confidence: f32, options: &MixtureMapOptions) -> CellStatus {
    if a.used == 0 {
        return CellStatus::NoSamples;
    }
    if a.used < options.min_samples_per_cell || a.dwell_ms < options.min_dwell_ms {
        return CellStatus::InsufficientSamples;
    }
    if stddev > options.max_lambda_stddev {
        return CellStatus::HighVariance;
    }
    if confidence < options.min_confidence {
        return CellStatus::LowConfidence;
    }
    let mean_lambda = (a.sum_lambda / a.used as f64) as f32;
    let mean_target = (a.sum_target / a.used as f64) as f32;
    let mean_trim = (a.sum_trim / a.used as f64) as f32;
    if mean_target <= 0.0 {
        return CellStatus::LowConfidence;
    }
    let raw = (mean_lambda / mean_target) * (1.0 + mean_trim);
    if (raw - 1.0).abs() < 0.002 {
        CellStatus::OnTarget
    } else {
        CellStatus::Proposed
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyError {
    StaleProposal,
    CalibrationTampered,
    MissingCell { row: usize, col: usize },
    NoCellsSelected,
    InvalidValue(String),
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplyError::StaleProposal => {
                f.write_str("proposal was generated against a different calibration revision")
            }
            ApplyError::CalibrationTampered => f.write_str("calibration checksum is invalid"),
            ApplyError::MissingCell { row, col } => {
                write!(f, "proposal cell ({row},{col}) is missing")
            }
            ApplyError::NoCellsSelected => f.write_str("no proposal cells selected"),
            ApplyError::InvalidValue(s) => write!(f, "invalid proposed value: {s}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApplyResult {
    pub calibration: Calibration,
    pub applied_cells: usize,
    pub diff: TableDiff,
    pub undo_revision: u32,
}

/// Apply selected cells of a proposal to a new, sealed calibration revision.
/// The input calibration is never modified; the service stores it as the undo
/// snapshot. Applying to simulator injector outputs is impossible here: this
/// function only produces calibration data.
pub fn apply_proposal(
    cal: &Calibration,
    proposal: &VeProposal,
    selected: Option<&[(usize, usize)]>,
    now_unix_ms: u64,
) -> Result<ApplyResult, ApplyError> {
    if !cal.verify() {
        return Err(ApplyError::CalibrationTampered);
    }
    if cal.checksum != proposal.base_checksum
        || cal.id != proposal.calibration_id
        || cal.revision != proposal.calibration_revision
    {
        return Err(ApplyError::StaleProposal);
    }
    let chosen: Vec<&ProposedVeCell> = match selected {
        None => proposal.cells.iter().collect(),
        Some(sel) => {
            if sel.is_empty() {
                return Err(ApplyError::NoCellsSelected);
            }
            let mut unique: Vec<(usize, usize)> = Vec::new();
            for (r, c) in sel {
                if !unique.contains(&(*r, *c)) {
                    unique.push((*r, *c));
                }
            }
            for (r, c) in &unique {
                if !proposal
                    .cells
                    .iter()
                    .any(|cell| cell.row == *r && cell.col == *c)
                {
                    return Err(ApplyError::MissingCell { row: *r, col: *c });
                }
            }
            proposal
                .cells
                .iter()
                .filter(|cell| unique.iter().any(|(r, c)| *r == cell.row && *c == cell.col))
                .collect()
        }
    };
    if chosen.is_empty() {
        return Err(ApplyError::NoCellsSelected);
    }
    let old_table = cal
        .table(VE_TABLE_KEY)
        .map_err(|_| ApplyError::InvalidValue("missing VE table".into()))?
        .clone();
    let mut next = cal.fork(cal.revision + 1, now_unix_ms);
    {
        let table = next
            .table_mut(VE_TABLE_KEY)
            .map_err(|_| ApplyError::InvalidValue("missing VE table".into()))?;
        for cell in &chosen {
            if cell.proposed_ve < table.value_min || cell.proposed_ve > table.value_max {
                return Err(ApplyError::InvalidValue(format!(
                    "{} outside {}..{}",
                    cell.proposed_ve, table.value_min, table.value_max
                )));
            }
            let idx = cell.row * table.axes[1].breakpoints.len() + cell.col;
            if idx >= table.values.len() {
                return Err(ApplyError::MissingCell {
                    row: cell.row,
                    col: cell.col,
                });
            }
            table.values[idx] = cell.proposed_ve;
        }
        table.provenance.notes = format!(
            "{} Mixture Map applied {} cell(s) from proposal {}; review required.",
            table.provenance.notes,
            chosen.len(),
            proposal.proposal_id
        );
        table.revision += 1;
    }
    next.notes.push(format!(
        "Applied Mixture Map proposal {} to {} cells. Base map was {} revision {}.",
        proposal.proposal_id,
        chosen.len(),
        cal.id,
        cal.revision
    ));
    next.seal();
    let new_table = next
        .table(VE_TABLE_KEY)
        .map_err(|_| ApplyError::InvalidValue("missing VE table".into()))?
        .clone();
    let diff = diff_tables(&old_table, &new_table);
    Ok(ApplyResult {
        calibration: next,
        applied_cells: chosen.len(),
        diff,
        undo_revision: cal.revision,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecu_calibration::wizard::{generate_starter, WizardRequest, VE_TABLE};
    use ecu_types::SensorStatus;

    fn cal() -> Calibration {
        generate_starter(&WizardRequest::default_inline4(), 1).unwrap()
    }

    fn frame(t: u64, rpm: f32, map: f32, lambda: f32, trim: f32) -> TelemetryFrame {
        TelemetryFrame {
            timestamp_us: t * 1_000,
            scenario_time_ms: t,
            rpm,
            map_kpa: map,
            tps_pct: 25.0,
            clt_c: 90.0,
            iat_c: 25.0,
            lambda_measured: lambda,
            lambda_target: 1.0,
            lambda_status: SensorStatus::Ok,
            lambda_ready: true,
            lambda_age_ms: 20,
            lambda_bank: 1,
            fuel_pressure_kpa: 300.0,
            oil_pressure_kpa: 250.0,
            injector_duty_pct: 40.0,
            sync: SyncState::Sequential,
            engine_state: EngineState::Running,
            protection: ProtectionState::Normal,
            stft: trim,
            exhaust_delay_ms: 0,
            ..TelemetryFrame::default()
        }
    }

    fn steady_log(rpm: f32, map: f32, lambda: f32, trim: f32, n: usize) -> Vec<TelemetryFrame> {
        let ve = 70.0;
        let mut frames = Vec::new();
        for i in 0..n {
            let mut f = frame(i as u64 * 20, rpm, map, lambda, trim);
            f.ve = ve;
            frames.push(f);
        }
        frames
    }

    #[test]
    fn lean_steady_cell_proposes_ve_increase() {
        let c = cal();
        let opts = MixtureMapOptions {
            min_dwell_ms: 100.0,
            min_samples_per_cell: 10,
            max_cell_correction: 0.20,
            ..MixtureMapOptions::default()
        };
        let report = analyze(&c, &steady_log(3000.0, 60.0, 1.10, 0.0, 30), &opts).unwrap();
        let proposal = report.proposal.expect("must propose");
        assert_eq!(proposal.cells.len(), 1);
        assert!((proposal.cells[0].correction_fraction - 0.10).abs() < 0.02);
        assert!(proposal.cells[0].proposed_ve > proposal.cells[0].current_ve);
        assert!(report.ignition_untouched);
        assert!(proposal.ignition_effect.contains("none"));
    }

    #[test]
    fn rich_cell_proposes_ve_decrease() {
        let c = cal();
        let opts = MixtureMapOptions {
            min_dwell_ms: 100.0,
            min_samples_per_cell: 10,
            ..MixtureMapOptions::default()
        };
        let report = analyze(&c, &steady_log(3000.0, 60.0, 0.92, 0.0, 30), &opts).unwrap();
        let cell = report.proposal.unwrap().cells.into_iter().next().unwrap();
        assert!(cell.proposed_ve < cell.current_ve);
    }

    #[test]
    fn active_trims_are_included_in_pre_trim_equivalent() {
        let c = cal();
        let opts = MixtureMapOptions {
            min_dwell_ms: 100.0,
            min_samples_per_cell: 10,
            max_cell_correction: 0.20,
            ..MixtureMapOptions::default()
        };
        // Lambda at target, but +10% trim has been added: the base VE needs +10%.
        let report = analyze(&c, &steady_log(3000.0, 60.0, 1.0, 0.10, 30), &opts).unwrap();
        let cell = report.proposal.unwrap().cells.into_iter().next().unwrap();
        assert!(
            (cell.correction_fraction - 0.10).abs() < 0.015,
            "{}",
            cell.correction_fraction
        );
    }

    #[test]
    fn too_few_samples_abstains() {
        let c = cal();
        let opts = MixtureMapOptions::default();
        let report = analyze(&c, &steady_log(3000.0, 60.0, 1.1, 0.0, 5), &opts).unwrap();
        assert!(report.proposal.is_none());
        assert!(report
            .cells
            .iter()
            .any(|c| c.status == CellStatus::InsufficientSamples));
    }

    #[test]
    fn noisy_samples_are_rejected() {
        let c = cal();
        let mut frames = steady_log(3000.0, 60.0, 1.10, 0.0, 40);
        for (i, f) in frames.iter_mut().enumerate() {
            f.lambda_measured += if i % 2 == 0 { 0.15 } else { -0.15 };
        }
        let opts = MixtureMapOptions {
            min_dwell_ms: 100.0,
            min_samples_per_cell: 10,
            max_lambda_stddev: 0.02,
            ..MixtureMapOptions::default()
        };
        let report = analyze(&c, &frames, &opts).unwrap();
        assert!(report
            .cells
            .iter()
            .any(|c| c.status == CellStatus::HighVariance));
        assert!(report.proposal.is_none());
    }

    #[test]
    fn cold_engine_and_stale_sensor_are_explained() {
        let c = cal();
        let opts = MixtureMapOptions {
            min_dwell_ms: 50.0,
            min_samples_per_cell: 5,
            ..MixtureMapOptions::default()
        };
        let mut frames = steady_log(3000.0, 60.0, 1.1, 0.0, 40);
        for f in frames.iter_mut() {
            f.clt_c = 20.0;
            f.lambda_status = SensorStatus::Stale;
        }
        let report = analyze(&c, &frames, &opts).unwrap();
        assert!(report.proposal.is_none());
        assert!(report.gate_reason_counts.contains_key("COOLANT_COLD"));
        assert!(report.gate_reason_counts.contains_key("LAMBDA_STATUS"));
    }

    #[test]
    fn fuel_cut_misfire_and_overrun_are_excluded() {
        let c = cal();
        let opts = MixtureMapOptions {
            min_dwell_ms: 50.0,
            min_samples_per_cell: 5,
            ..MixtureMapOptions::default()
        };
        let mut frames = steady_log(3000.0, 60.0, 1.1, 0.0, 40);
        for (i, f) in frames.iter_mut().enumerate() {
            match i % 4 {
                0 => f.active_faults.push("P030X_MISFIRE".into()),
                1 => f.fuel_cut = true,
                2 => f.tps_pct = 0.0,
                _ => {}
            }
        }
        let report = analyze(&c, &frames, &opts).unwrap();
        assert!(report.proposal.is_none());
        assert!(report.gate_reason_counts.contains_key("MISFIRE"));
        assert!(report.gate_reason_counts.contains_key("FUEL_CUT"));
        assert!(report.gate_reason_counts.contains_key("OVERRUN"));
    }

    #[test]
    fn wot_cells_require_explicit_opt_in() {
        let c = cal();
        let mut frames = steady_log(4000.0, 150.0, 1.05, 0.0, 40);
        for f in frames.iter_mut() {
            f.tps_pct = 95.0;
        }
        let opts = MixtureMapOptions {
            min_dwell_ms: 50.0,
            min_samples_per_cell: 5,
            ..MixtureMapOptions::default()
        };
        let blocked = analyze(&c, &frames, &opts).unwrap();
        assert!(blocked.proposal.is_none());
        assert!(blocked.gate_reason_counts.contains_key("WOT_BOUNDARY"));
        let opts = MixtureMapOptions {
            allow_wot: true,
            ..opts
        };
        let allowed = analyze(&c, &frames, &opts).unwrap();
        assert!(allowed.proposal.is_some());
    }

    #[test]
    fn deterministic_for_same_log() {
        let c = cal();
        let opts = MixtureMapOptions {
            min_dwell_ms: 100.0,
            min_samples_per_cell: 10,
            ..MixtureMapOptions::default()
        };
        let log = steady_log(3000.0, 60.0, 1.08, 0.03, 50);
        let a = analyze(&c, &log, &opts).unwrap();
        let b = analyze(&c, &log, &opts).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn apply_is_sealed_revisioned_and_diffable() {
        let c = cal();
        let opts = MixtureMapOptions {
            min_dwell_ms: 100.0,
            min_samples_per_cell: 10,
            ..MixtureMapOptions::default()
        };
        let report = analyze(&c, &steady_log(3000.0, 60.0, 1.08, 0.0, 40), &opts).unwrap();
        let proposal = report.proposal.unwrap();
        let applied = apply_proposal(&c, &proposal, None, 123).unwrap();
        assert_eq!(applied.calibration.revision, c.revision + 1);
        assert_eq!(applied.calibration.parent.as_deref(), Some(c.id.as_str()));
        assert!(applied.calibration.verify());
        assert_eq!(applied.applied_cells, 1);
        assert_eq!(applied.diff.changes.len(), 1);
        assert!(
            applied.calibration.table(VE_TABLE).unwrap().values.len()
                == c.table(VE_TABLE).unwrap().values.len()
        );
        assert!(c.verify());
    }

    #[test]
    fn stale_proposal_is_refused() {
        let c = cal();
        let opts = MixtureMapOptions {
            min_dwell_ms: 100.0,
            min_samples_per_cell: 10,
            ..MixtureMapOptions::default()
        };
        let report = analyze(&c, &steady_log(3000.0, 60.0, 1.08, 0.0, 40), &opts).unwrap();
        let proposal = report.proposal.unwrap();
        let changed = c.fork(c.revision + 1, 1);
        assert!(apply_proposal(&changed, &proposal, None, 2).is_err());
    }

    fn window_opts() -> MixtureMapOptions {
        MixtureMapOptions {
            min_dwell_ms: 50.0,
            min_samples_per_cell: 5,
            transient_holdoff_ms: 200,
            ..MixtureMapOptions::default()
        }
    }

    fn delayed_log(n: usize) -> Vec<TelemetryFrame> {
        let mut frames = steady_log(3000.0, 60.0, 1.05, 0.0, n);
        for f in frames.iter_mut() {
            f.exhaust_delay_ms = 100;
        }
        frames
    }

    #[test]
    fn throttle_blip_invalidates_later_steady_looking_samples() {
        let c = cal();
        let mut frames = delayed_log(80);
        // One-frame throttle blip at frame 30 (t = 600 ms).
        frames[30].tps_pct = 40.0;
        let report = analyze(&c, &frames, &window_opts()).unwrap();
        let n = report
            .gate_reason_counts
            .get("WINDOW_TRANSIENT")
            .copied()
            .unwrap_or(0);
        assert!(n > 0, "{:?}", report.gate_reason_counts);

        // With no holdoff and no transport delay the same log has no window
        // rejections: the option really controls the behaviour.
        let mut frames = delayed_log(80);
        frames[30].tps_pct = 40.0;
        for f in frames.iter_mut() {
            f.exhaust_delay_ms = 0;
        }
        let opts = MixtureMapOptions {
            transient_holdoff_ms: 0,
            ..window_opts()
        };
        let report = analyze(&c, &frames, &opts).unwrap();
        assert!(!report.gate_reason_counts.contains_key("WINDOW_TRANSIENT"));
    }

    #[test]
    fn misfire_inside_latency_window_rejects_following_samples() {
        let c = cal();
        let mut frames = delayed_log(80);
        frames[30].active_faults.push("P0301_MISFIRE".into());
        let report = analyze(&c, &frames, &window_opts()).unwrap();
        // The misfire frame itself is rejected by the instantaneous gate; the
        // 15 frames after it (t = 620..900 ms, window = 100 + 200 ms) are
        // rejected by the window check.
        assert_eq!(
            report.gate_reason_counts.get("WINDOW_MISFIRE").copied(),
            Some(15),
            "{:?}",
            report.gate_reason_counts
        );
        assert!(report.gate_reason_counts.contains_key("MISFIRE"));
    }

    #[test]
    fn fuel_cut_inside_window_is_rejected_even_when_current_frame_is_clean() {
        let c = cal();
        let mut frames = delayed_log(80);
        frames[30].fuel_cut = true;
        let report = analyze(&c, &frames, &window_opts()).unwrap();
        assert!(report
            .gate_reason_counts
            .contains_key("WINDOW_FUEL_OR_SPARK_CUT"));
    }

    #[test]
    fn moving_trim_and_loop_state_change_are_rejected() {
        let c = cal();
        let mut frames = delayed_log(80);
        // Loop engages at frame 40 and immediately carries a 10% trim.
        for f in frames.iter_mut().skip(40) {
            f.closed_loop = true;
            f.stft = 0.10;
        }
        let report = analyze(&c, &frames, &window_opts()).unwrap();
        assert!(report
            .gate_reason_counts
            .contains_key("WINDOW_TRIM_UNSTABLE"));
        assert!(report
            .gate_reason_counts
            .contains_key("WINDOW_LOOP_STATE_CHANGE"));
    }

    #[test]
    fn open_and_closed_loop_samples_are_counted_separately() {
        let c = cal();
        let open = analyze(&c, &steady_log(3000.0, 60.0, 1.05, 0.0, 40), &window_opts()).unwrap();
        assert!(open.open_loop_samples_used > 0);
        assert_eq!(open.closed_loop_samples_used, 0);
        assert_eq!(open.open_loop_samples_used, open.samples_used);

        let mut frames = steady_log(3000.0, 60.0, 1.05, 0.02, 40);
        for f in frames.iter_mut() {
            f.closed_loop = true;
        }
        let closed = analyze(&c, &frames, &window_opts()).unwrap();
        assert!(closed.closed_loop_samples_used > 0);
        assert_eq!(closed.open_loop_samples_used, 0);
        assert_eq!(closed.closed_loop_samples_used, closed.samples_used);
    }

    #[test]
    fn invalid_trim_swing_option_is_refused() {
        let c = cal();
        let opts = MixtureMapOptions {
            max_trim_swing: f32::NAN,
            ..window_opts()
        };
        assert!(matches!(
            analyze(&c, &steady_log(3000.0, 60.0, 1.05, 0.0, 40), &opts),
            Err(MixtureMapError::InvalidOptions(_))
        ));
    }

    #[test]
    fn options_json_without_new_fields_still_deserializes() {
        let mut v = serde_json::to_value(MixtureMapOptions::default()).unwrap();
        let obj = v.as_object_mut().unwrap();
        obj.remove("transient_holdoff_ms");
        obj.remove("max_trim_swing");
        let back: MixtureMapOptions = serde_json::from_value(v).unwrap();
        assert_eq!(back, MixtureMapOptions::default());
    }

    #[test]
    fn window_rejections_are_counted_per_cell_and_lower_confidence() {
        let c = cal();
        let opts = window_opts();
        let clean = analyze(&c, &delayed_log(80), &opts).unwrap();
        let clean_cell = clean.cells.iter().find(|x| x.samples_used > 0).unwrap();
        assert_eq!(clean_cell.window_rejected_samples, 0);

        let mut frames = delayed_log(80);
        frames[30].active_faults.push("P0301_MISFIRE".into());
        let hit = analyze(&c, &frames, &opts).unwrap();
        let hit_cell = hit
            .cells
            .iter()
            .find(|x| x.row == clean_cell.row && x.col == clean_cell.col)
            .unwrap();
        // Same 15 post-misfire frames as the latency-window test, counted once
        // per sample.
        assert_eq!(hit_cell.window_rejected_samples, 15);
        assert_eq!(
            hit_cell.exclusion_reasons.get("WINDOW_MISFIRE").copied(),
            Some(15)
        );
        let total: u32 = hit.cells.iter().map(|x| x.window_rejected_samples).sum();
        assert_eq!(total, 15);
        assert!(
            hit_cell.confidence < clean_cell.confidence,
            "{} !< {}",
            hit_cell.confidence,
            clean_cell.confidence
        );
        // The penalty is bounded by the documented share.
        assert!(hit_cell.confidence >= clean_cell.confidence * (1.0 - WINDOW_CONFIDENCE_PENALTY));
    }

    #[test]
    fn window_factor_is_bounded_and_neutral_without_rejections() {
        assert_eq!(window_factor(&Acc::default()), 1.0);
        let none = Acc {
            used: 10,
            ..Acc::default()
        };
        assert_eq!(window_factor(&none), 1.0);
        let all = Acc {
            window_rejected: 10,
            ..Acc::default()
        };
        assert!((window_factor(&all) - (1.0 - WINDOW_CONFIDENCE_PENALTY)).abs() < 1e-6);
    }

    #[test]
    fn report_json_without_window_rejected_field_still_deserializes() {
        let c = cal();
        let report = analyze(&c, &delayed_log(80), &window_opts()).unwrap();
        let mut v = serde_json::to_value(&report).unwrap();
        for cell in v["cells"].as_array_mut().unwrap() {
            cell.as_object_mut()
                .unwrap()
                .remove("window_rejected_samples");
        }
        let back: MixtureMapReport = serde_json::from_value(v).unwrap();
        assert!(back.cells.iter().all(|x| x.window_rejected_samples == 0));
    }

    #[test]
    fn selection_only_applies_chosen_cells() {
        let c = cal();
        let opts = MixtureMapOptions {
            min_dwell_ms: 100.0,
            min_samples_per_cell: 10,
            max_cell_correction: 0.20,
            ..MixtureMapOptions::default()
        };
        let report = analyze(&c, &steady_log(3000.0, 60.0, 1.10, 0.0, 40), &opts).unwrap();
        let proposal = report.proposal.unwrap();
        let cell = proposal.cells[0].clone();
        let applied = apply_proposal(&c, &proposal, Some(&[(cell.row, cell.col)]), 3).unwrap();
        assert_eq!(applied.applied_cells, 1);
        // A selection that is not in the proposal is refused, not silently applied.
        assert!(apply_proposal(&c, &proposal, Some(&[(0, 0)]), 4).is_err());
    }
}
