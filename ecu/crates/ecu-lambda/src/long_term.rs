use serde::{Deserialize, Serialize};
use std::string::String;
use std::vec;
use std::vec::Vec;

use crate::gate::{GateDecision, GateReason};

/// Cell-aware long-term fuel trim (LTFT). Kept strictly separate from base VE
/// and from short-term trim. Learning requires sustained dwell in a cell and a
/// valid gate; merging into a base map is an explicit, reviewable action.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LongTermTrimTable {
    pub rpm_axis: Vec<f32>,
    pub map_axis: Vec<f32>,
    /// map-major: index = map_row * rpm_cols + rpm_col
    pub trim: Vec<f32>,
    pub samples: Vec<u32>,
    pub dwell_ms: Vec<f32>,
    pub last_update_ms: Vec<u64>,
    pub revision: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LtftConfig {
    pub max_trim: f32,
    pub learning_gain: f32,
    pub max_step: f32,
    pub deadband: f32,
    pub min_dwell_ms: f32,
    pub min_update_interval_ms: u64,
    pub min_samples: u32,
}

impl Default for LtftConfig {
    fn default() -> Self {
        Self {
            max_trim: 0.20,
            learning_gain: 0.03,
            max_step: 0.002,
            deadband: 0.002,
            min_dwell_ms: 1500.0,
            min_update_interval_ms: 100,
            min_samples: 20,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LtftUpdate {
    pub row: usize,
    pub col: usize,
    pub old_trim: f32,
    pub new_trim: f32,
    pub samples: u32,
    pub confidence: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LtftError {
    EmptyAxis,
    LengthMismatch,
    InvalidTrim,
}

impl std::fmt::Display for LtftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LtftError::EmptyAxis => f.write_str("LTFT axes must not be empty"),
            LtftError::LengthMismatch => f.write_str("LTFT table length mismatch"),
            LtftError::InvalidTrim => f.write_str("LTFT trim outside permitted limits"),
        }
    }
}

impl LongTermTrimTable {
    pub fn new(rpm_axis: Vec<f32>, map_axis: Vec<f32>) -> Result<Self, LtftError> {
        if rpm_axis.len() < 2 || map_axis.len() < 2 {
            return Err(LtftError::EmptyAxis);
        }
        if !strictly_increasing(&rpm_axis) || !strictly_increasing(&map_axis) {
            return Err(LtftError::EmptyAxis);
        }
        let n = rpm_axis.len() * map_axis.len();
        Ok(Self {
            rpm_axis,
            map_axis,
            trim: vec![0.0; n],
            samples: vec![0; n],
            dwell_ms: vec![0.0; n],
            last_update_ms: vec![0; n],
            revision: 1,
        })
    }

    pub fn cell_count(&self) -> usize {
        self.trim.len()
    }

    pub fn index(&self, rpm: f32, map: f32) -> Option<usize> {
        let col = nearest_index(&self.rpm_axis, rpm)?;
        let row = nearest_index(&self.map_axis, map)?;
        Some(row * self.rpm_axis.len() + col)
    }

    pub fn confidence(&self, idx: usize, cfg: &LtftConfig) -> f32 {
        if idx >= self.trim.len() {
            return 0.0;
        }
        (self.samples[idx] as f32 / cfg.min_samples.max(1) as f32).min(1.0)
            * if self.dwell_ms[idx] >= cfg.min_dwell_ms {
                1.0
            } else {
                0.5
            }
    }

    /// Learn from an active, valid STFT. Returns an update only when the cell is
    /// eligible; otherwise the caller must show why no update happened.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        now_ms: u64,
        rpm: f32,
        map_kpa: f32,
        stft: f32,
        stft_active: bool,
        gate: &GateDecision,
        dt_ms: f32,
        cfg: &LtftConfig,
    ) -> Option<LtftUpdate> {
        if !gate.allowed
            || !stft_active
            || !stft.is_finite()
            || !rpm.is_finite()
            || !map_kpa.is_finite()
        {
            return None;
        }
        let idx = self.index(rpm, map_kpa)?;
        let dt = dt_ms.clamp(0.0, 1000.0);
        self.dwell_ms[idx] += dt;
        if self.dwell_ms[idx] < cfg.min_dwell_ms {
            return None;
        }
        if now_ms.saturating_sub(self.last_update_ms[idx]) < cfg.min_update_interval_ms
            && self.last_update_ms[idx] != 0
        {
            return None;
        }
        let old = self.trim[idx];
        let error = stft - old;
        if error.abs() < cfg.deadband {
            return None;
        }
        let step = (error * cfg.learning_gain).clamp(-cfg.max_step, cfg.max_step);
        let new = (old + step).clamp(-cfg.max_trim, cfg.max_trim);
        if (new - old).abs() < 1e-8 {
            return None;
        }
        self.trim[idx] = new;
        self.last_update_ms[idx] = now_ms;
        self.samples[idx] = self.samples[idx].saturating_add(1);
        self.revision = self.revision.wrapping_add(1);
        Some(LtftUpdate {
            row: idx / self.rpm_axis.len(),
            col: idx % self.rpm_axis.len(),
            old_trim: old,
            new_trim: new,
            samples: self.samples[idx],
            confidence: self.confidence(idx, cfg),
        })
    }

    /// Merge selected learned cells into a copy of base VE values. The source
    /// snapshot is never mutated. This is a review action, not runtime control.
    pub fn merge_into_values(
        &self,
        base_values: &[f32],
        selected: &[usize],
        max_total_fraction: f32,
    ) -> Result<(Vec<f32>, Vec<LtftMergeCell>), LtftError> {
        if base_values.len() != self.trim.len() {
            return Err(LtftError::LengthMismatch);
        }
        let mut out = base_values.to_vec();
        let mut merged = Vec::new();
        for &idx in selected {
            if idx >= out.len() {
                return Err(LtftError::LengthMismatch);
            }
            let old = out[idx];
            let factor =
                (1.0 + self.trim[idx]).clamp(1.0 - max_total_fraction, 1.0 + max_total_fraction);
            let new = old * factor;
            if !new.is_finite() || new <= 0.0 {
                return Err(LtftError::InvalidTrim);
            }
            out[idx] = new;
            merged.push(LtftMergeCell {
                index: idx,
                old_ve: old,
                new_ve: new,
                applied_fraction: new / old - 1.0,
            });
        }
        Ok((out, merged))
    }

    pub fn reset_cell(&mut self, idx: usize) -> bool {
        if idx >= self.trim.len() {
            return false;
        }
        self.trim[idx] = 0.0;
        self.samples[idx] = 0;
        self.dwell_ms[idx] = 0.0;
        self.last_update_ms[idx] = 0;
        self.revision = self.revision.wrapping_add(1);
        true
    }

    pub fn validate(&self) -> Result<(), LtftError> {
        let n = self.rpm_axis.len() * self.map_axis.len();
        if self.trim.len() != n
            || self.samples.len() != n
            || self.dwell_ms.len() != n
            || self.last_update_ms.len() != n
        {
            return Err(LtftError::LengthMismatch);
        }
        if self.trim.iter().any(|v| !v.is_finite() || v.abs() > 0.5) {
            return Err(LtftError::InvalidTrim);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LtftMergeCell {
    pub index: usize,
    pub old_ve: f32,
    pub new_ve: f32,
    pub applied_fraction: f32,
}

/// Small, explicit bank container. Separate controllers are required because a
/// single sensor cannot observe both banks.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BankTrims {
    pub bank1: LongTermTrimTable,
    pub bank2: Option<LongTermTrimTable>,
}

impl BankTrims {
    pub fn single(t: LongTermTrimTable) -> Self {
        Self {
            bank1: t,
            bank2: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_bank(
        &mut self,
        bank: u8,
        now_ms: u64,
        rpm: f32,
        map_kpa: f32,
        stft: f32,
        stft_active: bool,
        gate: &GateDecision,
        dt_ms: f32,
        cfg: &LtftConfig,
    ) -> Option<LtftUpdate> {
        match bank {
            1 => self
                .bank1
                .update(now_ms, rpm, map_kpa, stft, stft_active, gate, dt_ms, cfg),
            2 => self
                .bank2
                .as_mut()
                .and_then(|t| t.update(now_ms, rpm, map_kpa, stft, stft_active, gate, dt_ms, cfg)),
            _ => None,
        }
    }
}

fn strictly_increasing(v: &[f32]) -> bool {
    v.windows(2)
        .all(|w| w[0].is_finite() && w[1].is_finite() && w[1] > w[0])
}

fn nearest_index(axis: &[f32], value: f32) -> Option<usize> {
    if axis.is_empty() || !value.is_finite() {
        return None;
    }
    let mut best = 0;
    let mut distance = f32::INFINITY;
    for (i, v) in axis.iter().enumerate() {
        let d = (value - *v).abs();
        if d < distance {
            best = i;
            distance = d;
        }
    }
    Some(best)
}

/// Helper used by tests and the service to explain a non-update.
pub fn explain_block(decision: &GateDecision) -> String {
    if decision.allowed {
        "gates satisfied".into()
    } else {
        decision
            .reasons
            .iter()
            .map(|r| r.code.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Prevent dead-code warnings for the public reason helper while keeping the
/// diagnostic type available to callers.
pub fn gate_reason_codes(decision: &GateDecision) -> Vec<String> {
    decision
        .reasons
        .iter()
        .map(|r: &GateReason| r.code.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate::{evaluate, GateConfig, GateInputs};
    use ecu_types::SensorStatus;

    fn gate() -> GateDecision {
        evaluate(
            &GateConfig::default(),
            &GateInputs {
                engine_running: true,
                sync_sequential: true,
                coolant_c: 90.0,
                rpm: 3000.0,
                map_kpa: 60.0,
                tps_pct: 25.0,
                lambda: 1.0,
                target_lambda: 1.0,
                lambda_status: SensorStatus::Ok,
                lambda_ready: true,
                lambda_age_ms: 20,
                fuel_pressure_ok: true,
                injector_duty_pct: 40.0,
                ..GateInputs::default()
            },
        )
    }

    fn table() -> LongTermTrimTable {
        LongTermTrimTable::new(vec![1000.0, 2000.0, 3000.0, 4000.0], vec![30.0, 60.0, 90.0])
            .unwrap()
    }

    #[test]
    fn learns_toward_positive_stft_with_bounds() {
        let mut t = table();
        let cfg = LtftConfig::default();
        let g = gate();
        let mut prev = 0.0;
        for i in 0..500u64 {
            let now = i * 100;
            if let Some(u) = t.update(now, 3000.0, 60.0, 0.10, true, &g, 100.0, &cfg) {
                assert!(u.new_trim >= u.old_trim - 1e-6);
                prev = u.new_trim;
            }
        }
        assert!(prev > 0.05, "learned={prev}");
        assert!(prev <= cfg.max_trim + 1e-6);
        assert!(t.samples[t.index(3000.0, 60.0).unwrap()] > 0);
    }

    #[test]
    fn does_not_learn_without_dwell_or_gate() {
        let mut t = table();
        let cfg = LtftConfig::default();
        let g = gate();
        let u = t.update(100, 3000.0, 60.0, 0.1, true, &g, 50.0, &cfg);
        assert!(u.is_none());
        let blocked = GateDecision::blocked(vec![GateReason {
            code: "KNOCK".into(),
            message: "test".into(),
        }]);
        for i in 0..100 {
            assert!(t
                .update(i * 200, 3000.0, 60.0, 0.1, true, &blocked, 200.0, &cfg)
                .is_none());
        }
        assert!(t.trim.iter().all(|v| v.abs() < 1e-9));
    }

    #[test]
    fn merge_is_selective_bounded_and_preserves_source() {
        let mut t = table();
        let i1 = t.index(3000.0, 60.0).unwrap();
        let i2 = t.index(2000.0, 30.0).unwrap();
        t.trim[i1] = 0.10;
        t.trim[i2] = 0.30; // above max but stored
        let base = vec![80.0; t.cell_count()];
        let idx = t.index(3000.0, 60.0).unwrap();
        let (out, cells) = t.merge_into_values(&base, &[idx], 0.15).unwrap();
        assert!((cells[0].applied_fraction - 0.10).abs() < 1e-5);
        assert!((out[idx] - 88.0).abs() < 0.01);
        assert!((base[idx] - 80.0).abs() < 1e-6);
        assert_eq!(out[t.index(2000.0, 30.0).unwrap()], 80.0);
    }

    #[test]
    fn serialization_round_trip_preserves_learning() {
        let mut t = table();
        t.trim[3] = 0.04;
        let s = serde_json::to_string(&t).unwrap();
        let back: LongTermTrimTable = serde_json::from_str(&s).unwrap();
        assert_eq!(t, back);
        assert!(back.validate().is_ok());
    }

    #[test]
    fn bank_two_is_independent_and_optional() {
        let mut b = BankTrims::single(table());
        let g = gate();
        let cfg = LtftConfig::default();
        for i in 0..100 {
            b.update_bank(1, i * 100, 3000.0, 60.0, 0.1, true, &g, 100.0, &cfg);
            assert!(b
                .update_bank(2, i * 100, 3000.0, 60.0, 0.1, true, &g, 100.0, &cfg)
                .is_none());
        }
        assert!(b.bank1.trim.iter().any(|v| v.abs() > 0.0));
        assert!(b.bank2.is_none());
    }
}
