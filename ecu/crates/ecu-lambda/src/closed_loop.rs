use serde::{Deserialize, Serialize};
use std::vec::Vec;

use crate::gate::{GateDecision, GateReason};

/// Bounded short-term fuel trim (STFT) controller.
///
/// Sign convention: `error = measured_lambda - target_lambda`.
/// Lean (positive error) adds fuel. The output is a fractional trim applied to
/// base fuel: +0.05 means +5% pulse width.
///
/// Safety behaviour:
/// - output is hard-bounded and integrator windup is prevented;
/// - a blocked gate stops learning immediately;
/// - short invalid periods freeze the last trim;
/// - prolonged invalid sensor state decays the trim to zero instead of holding a
///   correction derived from a failed sensor;
/// - high-load/WOT gating comes from the shared gate, not this controller.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClosedLoopConfig {
    pub kp: f32,
    pub ki: f32,
    pub deadband_lambda: f32,
    pub max_stft: f32,
    pub invalid_freeze_ms: u64,
    pub invalid_decay_per_s: f32,
}

impl Default for ClosedLoopConfig {
    fn default() -> Self {
        Self {
            kp: 0.8,
            ki: 1.2,
            deadband_lambda: 0.002,
            max_stft: 0.25,
            invalid_freeze_ms: 1000,
            invalid_decay_per_s: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClosedLoopOutput {
    pub active: bool,
    pub error: f32,
    pub stft: f32,
    pub applied_trim: f32,
    pub p_term: f32,
    pub i_term: f32,
    pub saturated_high: bool,
    pub saturated_low: bool,
    pub gate_reasons: Vec<GateReason>,
}

#[derive(Clone, Debug)]
pub struct ClosedLoop {
    cfg: ClosedLoopConfig,
    integral: f32,
    stft: f32,
    active: bool,
    last_update_ms: Option<u64>,
    invalid_since_ms: Option<u64>,
}

impl ClosedLoop {
    pub fn new(cfg: ClosedLoopConfig) -> Self {
        Self {
            cfg,
            integral: 0.0,
            stft: 0.0,
            active: false,
            last_update_ms: None,
            invalid_since_ms: None,
        }
    }

    pub fn config(&self) -> ClosedLoopConfig {
        self.cfg
    }

    pub fn current_trim(&self) -> f32 {
        self.stft
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.stft = 0.0;
        self.active = false;
        self.last_update_ms = None;
        self.invalid_since_ms = None;
    }

    /// Update at the wideband/logging rate. `gate` must be produced by the shared
    /// learning gate for the same instant.
    pub fn update(
        &mut self,
        now_ms: u64,
        measured_lambda: f32,
        target_lambda: f32,
        gate: &GateDecision,
    ) -> ClosedLoopOutput {
        let dt_s = match self.last_update_ms {
            Some(prev) => ((now_ms.saturating_sub(prev)) as f32 / 1000.0).clamp(0.0, 0.5),
            None => 0.0,
        };
        self.last_update_ms = Some(now_ms);

        if !gate.allowed {
            self.active = false;
            let since = *self.invalid_since_ms.get_or_insert(now_ms);
            let invalid_ms = now_ms.saturating_sub(since);
            if invalid_ms > self.cfg.invalid_freeze_ms {
                let decay = self.cfg.invalid_decay_per_s * (dt_s.max(0.001));
                if self.stft > 0.0 {
                    self.stft = (self.stft - decay).max(0.0);
                } else if self.stft < 0.0 {
                    self.stft = (self.stft + decay).min(0.0);
                }
                self.integral = self.stft;
            }
            return ClosedLoopOutput {
                active: false,
                error: if measured_lambda.is_finite() && target_lambda.is_finite() {
                    measured_lambda - target_lambda
                } else {
                    0.0
                },
                stft: self.stft,
                applied_trim: self.stft,
                p_term: 0.0,
                i_term: self.integral,
                saturated_high: self.stft >= self.cfg.max_stft - 1e-6,
                saturated_low: self.stft <= -self.cfg.max_stft + 1e-6,
                gate_reasons: gate.reasons.clone(),
            };
        }

        self.invalid_since_ms = None;
        let error =
            if measured_lambda.is_finite() && target_lambda.is_finite() && target_lambda > 0.3 {
                measured_lambda - target_lambda
            } else {
                0.0
            };
        let effective_error = if error.abs() < self.cfg.deadband_lambda {
            0.0
        } else {
            error
        };

        let p_term = self.cfg.kp * effective_error;
        let mut candidate_integral = self.integral + self.cfg.ki * effective_error * dt_s;
        candidate_integral = candidate_integral.clamp(-self.cfg.max_stft, self.cfg.max_stft);
        let candidate_total = p_term + candidate_integral;

        // Conditional integration: do not push further into a saturated limit.
        let would_saturate_high = candidate_total > self.cfg.max_stft && effective_error > 0.0;
        let would_saturate_low = candidate_total < -self.cfg.max_stft && effective_error < 0.0;
        if !would_saturate_high && !would_saturate_low {
            self.integral = candidate_integral;
        }

        self.stft = (p_term + self.integral).clamp(-self.cfg.max_stft, self.cfg.max_stft);
        self.active = true;

        ClosedLoopOutput {
            active: true,
            error,
            stft: self.stft,
            applied_trim: self.stft,
            p_term,
            i_term: self.integral,
            saturated_high: self.stft >= self.cfg.max_stft - 1e-6,
            saturated_low: self.stft <= -self.cfg.max_stft + 1e-6,
            gate_reasons: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate::{evaluate, GateConfig, GateInputs};
    use ecu_types::SensorStatus;

    fn good_gate() -> GateDecision {
        evaluate(
            &GateConfig::for_closed_loop(),
            &GateInputs {
                engine_running: true,
                sync_sequential: true,
                coolant_c: 88.0,
                rpm: 2500.0,
                map_kpa: 55.0,
                tps_pct: 20.0,
                lambda: 1.0,
                target_lambda: 1.0,
                lambda_status: SensorStatus::Ok,
                lambda_ready: true,
                lambda_age_ms: 20,
                fuel_pressure_ok: true,
                oil_pressure_ok: true,
                injector_duty_pct: 30.0,
                ..GateInputs::default()
            },
        )
    }

    #[test]
    fn lean_error_adds_fuel_and_converges_with_delayed_sensor() {
        let mut cl = ClosedLoop::new(ClosedLoopConfig::default());
        let gate = good_gate();
        let mut plant_lambda = 1.10f32; // base calibration is 10% lean
        let mut sensor = 1.10f32;
        let dt = 0.02f32;
        for step in 0..1500u64 {
            // First-order wideband lag, tau 120 ms.
            sensor += dt / 0.12 * (plant_lambda - sensor);
            let out = cl.update(step * 20, sensor, 1.0, &gate);
            plant_lambda = 1.10 / (1.0 + out.applied_trim);
            assert!(out.applied_trim.abs() <= 0.25 + 1e-6);
        }
        assert!(cl.current_trim() > 0.07, "trim={}", cl.current_trim());
        assert!(cl.current_trim() < 0.13, "trim={}", cl.current_trim());
        assert!((plant_lambda - 1.0).abs() < 0.02, "lambda={plant_lambda}");
    }

    #[test]
    fn rich_error_removes_fuel() {
        let mut cl = ClosedLoop::new(ClosedLoopConfig::default());
        let gate = good_gate();
        for step in 0..500u64 {
            cl.update(step * 20, 0.90, 1.0, &gate);
        }
        assert!(cl.current_trim() < -0.05, "trim={}", cl.current_trim());
        assert!(cl.current_trim() >= -0.25);
    }

    #[test]
    fn missing_lambda_gate_freezes_then_decays_to_zero() {
        let cfg = ClosedLoopConfig {
            invalid_freeze_ms: 200,
            invalid_decay_per_s: 2.0,
            ..ClosedLoopConfig::default()
        };
        let mut cl = ClosedLoop::new(cfg);
        let good = good_gate();
        for step in 0..200u64 {
            cl.update(step * 20, 1.10, 1.0, &good);
        }
        let held = cl.current_trim();
        assert!(held > 0.05);
        let blocked = GateDecision::blocked(vec![GateReason {
            code: "LAMBDA_STALE".into(),
            message: "test".into(),
        }]);
        cl.update(4000, 1.10, 1.0, &blocked);
        assert!(!cl.is_active());
        assert!((cl.current_trim() - held).abs() < 1e-6);
        for step in 1..200u64 {
            cl.update(4000 + step * 20, 1.10, 1.0, &blocked);
        }
        assert!(cl.current_trim().abs() < 0.01, "trim={}", cl.current_trim());
        assert!(!cl.is_active());
    }

    #[test]
    fn integrator_never_winds_beyond_limit() {
        let mut cl = ClosedLoop::new(ClosedLoopConfig::default());
        let gate = good_gate();
        for step in 0..10_000u64 {
            cl.update(step * 20, 1.50, 1.0, &gate);
        }
        assert!(cl.current_trim() <= 0.25 + 1e-6);
        let out = cl.update(300_000, 1.50, 1.0, &gate);
        assert!(out.saturated_high);
        assert!(out.i_term <= 0.25 + 1e-6);
        assert!(out.stft <= 0.25 + 1e-6);
    }

    #[test]
    fn deadband_prevents_chatter() {
        let mut cl = ClosedLoop::new(ClosedLoopConfig::default());
        let gate = good_gate();
        for step in 0..100u64 {
            cl.update(step * 20, 1.001, 1.0, &gate);
        }
        assert!(cl.current_trim().abs() < 0.01);
    }

    #[test]
    fn independent_banks_do_not_share_integrator() {
        let gate = good_gate();
        let mut bank1 = ClosedLoop::new(ClosedLoopConfig::default());
        let mut bank2 = ClosedLoop::new(ClosedLoopConfig::default());
        for step in 0..300u64 {
            bank1.update(step * 20, 1.10, 1.0, &gate);
            bank2.update(step * 20, 0.90, 1.0, &gate);
        }
        assert!(bank1.current_trim() > 0.03);
        assert!(bank2.current_trim() < -0.03);
    }
}
