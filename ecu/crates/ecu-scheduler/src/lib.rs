//! Deterministic 720-degree injection and ignition event scheduling.
//!
//! This module computes *what* should happen and at which crank angle. It does
//! not claim hardware timing precision. On a physical MCU, compare/capture
//! peripherals must execute these events; async tasks are not a timing
//! guarantee.
//!
//! Event angles are degrees in the 0..720 cycle, where 0 is compression TDC for
//! cylinder 1. Each cylinder's compression TDC follows the firing order spaced
//! by 720/cylinders degrees.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

use core::fmt;
use serde::{Deserialize, Serialize};

pub const MAX_CYLINDERS: usize = 12;
pub const MAX_EVENTS: usize = MAX_CYLINDERS * 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    InjectionStart,
    DwellStart,
    Spark,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub cylinder: u8,
    pub kind: EventKind,
    pub angle_deg: f32,
    pub duration_deg: f32,
    pub duration_ms: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EventSchedule {
    events: [ScheduledEvent; MAX_EVENTS],
    len: usize,
}

impl EventSchedule {
    pub fn empty() -> Self {
        Self {
            events: [ScheduledEvent {
                cylinder: 0,
                kind: EventKind::Spark,
                angle_deg: 0.0,
                duration_deg: 0.0,
                duration_ms: 0.0,
            }; MAX_EVENTS],
            len: 0,
        }
    }

    pub fn events(&self) -> &[ScheduledEvent] {
        &self.events[..self.len]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn push(&mut self, e: ScheduledEvent) {
        if self.len < MAX_EVENTS {
            self.events[self.len] = e;
            self.len += 1;
        }
    }

    fn sort(&mut self) {
        // Bounded insertion sort; deterministic and allocation-free.
        for i in 1..self.len {
            let mut j = i;
            while j > 0
                && (self.events[j].angle_deg < self.events[j - 1].angle_deg
                    || (self.events[j].angle_deg == self.events[j - 1].angle_deg
                        && self.events[j].cylinder < self.events[j - 1].cylinder))
            {
                self.events.swap(j, j - 1);
                j -= 1;
            }
        }
    }

    /// Iterate events crossed when crank advances from `prev` to `now`.
    /// Angles wrap at 720. Used by the simulator and by scheduler tests.
    pub fn events_crossed(
        &self,
        prev_deg: f32,
        now_deg: f32,
    ) -> impl Iterator<Item = &ScheduledEvent> {
        let p = wrap_angle(prev_deg);
        let n = wrap_angle(now_deg);
        self.events().iter().filter(move |e| {
            let a = wrap_angle(e.angle_deg);
            if p <= n {
                a > p && a <= n
            } else {
                a > p || a <= n
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleError {
    NoCylinders,
    TooManyCylinders,
    InvalidFiringOrder,
    InvalidRpm,
    InvalidPulseWidth,
    InvalidDwell,
    InvalidAdvance,
    InvalidInjectionTiming,
    PulseLongerThanCycle,
    TooManyEvents,
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ScheduleError::NoCylinders => "no cylinders configured",
            ScheduleError::TooManyCylinders => "more than 12 cylinders not supported",
            ScheduleError::InvalidFiringOrder => "firing order is not a permutation",
            ScheduleError::InvalidRpm => "invalid RPM",
            ScheduleError::InvalidPulseWidth => "invalid injection pulse width",
            ScheduleError::InvalidDwell => "invalid dwell",
            ScheduleError::InvalidAdvance => "invalid spark advance",
            ScheduleError::InvalidInjectionTiming => "invalid injection-end angle",
            ScheduleError::PulseLongerThanCycle => "pulse longer than a complete cycle",
            ScheduleError::TooManyEvents => "event buffer overflow",
        };
        f.write_str(s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScheduleRequest {
    pub rpm: f32,
    pub pulse_width_ms: f32,
    pub dwell_ms: f32,
    pub advance_deg: f32,
    /// Injection ends this many degrees before compression TDC. 240 deg places
    /// the end in the intake stroke for a conventional four-stroke.
    pub injection_end_btdc_deg: f32,
    pub spark_cut: bool,
    pub fuel_cut: bool,
}

impl Default for ScheduleRequest {
    fn default() -> Self {
        Self {
            rpm: 900.0,
            pulse_width_ms: 3.0,
            dwell_ms: 3.0,
            advance_deg: 10.0,
            injection_end_btdc_deg: 240.0,
            spark_cut: false,
            fuel_cut: false,
        }
    }
}

pub fn build_schedule(
    firing_order: &[u8],
    req: &ScheduleRequest,
) -> Result<EventSchedule, ScheduleError> {
    let n = firing_order.len();
    if n == 0 {
        return Err(ScheduleError::NoCylinders);
    }
    if n > MAX_CYLINDERS {
        return Err(ScheduleError::TooManyCylinders);
    }
    if !(1.0..=20_000.0).contains(&req.rpm) || !req.rpm.is_finite() {
        return Err(ScheduleError::InvalidRpm);
    }
    if !(0.0..=50.0).contains(&req.pulse_width_ms) || !req.pulse_width_ms.is_finite() {
        return Err(ScheduleError::InvalidPulseWidth);
    }
    if !(0.1..=12.0).contains(&req.dwell_ms) || !req.dwell_ms.is_finite() {
        return Err(ScheduleError::InvalidDwell);
    }
    if !(-10.0..=55.0).contains(&req.advance_deg) || !req.advance_deg.is_finite() {
        return Err(ScheduleError::InvalidAdvance);
    }
    if !(5.0..=355.0).contains(&req.injection_end_btdc_deg)
        || !req.injection_end_btdc_deg.is_finite()
    {
        return Err(ScheduleError::InvalidInjectionTiming);
    }
    let mut seen = [false; MAX_CYLINDERS + 1];
    for &c in firing_order {
        let ci = c as usize;
        if c == 0 || ci > n || seen[ci] {
            return Err(ScheduleError::InvalidFiringOrder);
        }
        seen[ci] = true;
    }

    let deg_per_ms = req.rpm * 0.006;
    let pw_deg = req.pulse_width_ms * deg_per_ms;
    let dwell_deg = req.dwell_ms * deg_per_ms;
    if pw_deg > 720.0 || dwell_deg > 720.0 {
        return Err(ScheduleError::PulseLongerThanCycle);
    }

    let mut sched = EventSchedule::empty();
    let spacing = 720.0 / n as f32;
    for (i, &cylinder) in firing_order.iter().enumerate() {
        let tdc = i as f32 * spacing;
        if !req.fuel_cut {
            let end = wrap_angle(tdc - req.injection_end_btdc_deg);
            let start = wrap_angle(end - pw_deg);
            sched.push(ScheduledEvent {
                cylinder,
                kind: EventKind::InjectionStart,
                angle_deg: start,
                duration_deg: pw_deg,
                duration_ms: req.pulse_width_ms,
            });
        }
        if !req.spark_cut {
            let spark = wrap_angle(tdc - req.advance_deg);
            let dwell_start = wrap_angle(spark - dwell_deg);
            sched.push(ScheduledEvent {
                cylinder,
                kind: EventKind::DwellStart,
                angle_deg: dwell_start,
                duration_deg: dwell_deg,
                duration_ms: req.dwell_ms,
            });
            sched.push(ScheduledEvent {
                cylinder,
                kind: EventKind::Spark,
                angle_deg: spark,
                duration_deg: 0.0,
                duration_ms: 0.0,
            });
        }
    }
    if sched.len > MAX_EVENTS {
        return Err(ScheduleError::TooManyEvents);
    }
    sched.sort();
    Ok(sched)
}

/// Compression TDC angle for one-based cylinder number, given firing order.
pub fn compression_tdc_deg(firing_order: &[u8], cylinder: u8) -> Option<f32> {
    firing_order
        .iter()
        .position(|&c| c == cylinder)
        .map(|i| i as f32 * 720.0 / firing_order.len() as f32)
}

#[inline]
fn wrap_angle(v: f32) -> f32 {
    // f32::rem_euclid is std-only; needed for no_std firmware.
    let r = v % 720.0;
    if r < 0.0 {
        r + 720.0
    } else {
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req() -> ScheduleRequest {
        ScheduleRequest {
            rpm: 3000.0,
            pulse_width_ms: 8.0,
            dwell_ms: 3.0,
            advance_deg: 20.0,
            injection_end_btdc_deg: 240.0,
            spark_cut: false,
            fuel_cut: false,
        }
    }

    #[test]
    fn four_cylinder_schedule_has_three_events_per_cylinder_sorted() {
        let s = build_schedule(&[1, 3, 4, 2], &req()).unwrap();
        assert_eq!(s.len(), 12);
        assert!(s
            .events()
            .windows(2)
            .all(|w| w[0].angle_deg <= w[1].angle_deg));
        for cyl in 1..=4 {
            let sparks: Vec<_> = s
                .events()
                .iter()
                .filter(|e| e.cylinder == cyl && e.kind == EventKind::Spark)
                .collect();
            assert_eq!(sparks.len(), 1);
            let tdc = compression_tdc_deg(&[1, 3, 4, 2], cyl).unwrap();
            let expected = (tdc - 20.0).rem_euclid(720.0);
            assert!((sparks[0].angle_deg - expected).abs() < 1e-3);
        }
    }

    #[test]
    fn dwell_starts_before_spark_by_angle() {
        let s = build_schedule(&[1, 3, 4, 2], &req()).unwrap();
        let dwell = s
            .events()
            .iter()
            .find(|e| e.cylinder == 1 && e.kind == EventKind::DwellStart)
            .unwrap();
        let spark = s
            .events()
            .iter()
            .find(|e| e.cylinder == 1 && e.kind == EventKind::Spark)
            .unwrap();
        let delta = (spark.angle_deg - dwell.angle_deg).rem_euclid(720.0);
        // 3 ms at 3000 rpm = 54 degrees.
        assert!((delta - 54.0).abs() < 0.01, "delta={delta}");
    }

    #[test]
    fn injection_and_spark_can_be_cut_independently() {
        let mut r = req();
        r.spark_cut = true;
        let s = build_schedule(&[1, 3, 4, 2], &r).unwrap();
        assert!(s
            .events()
            .iter()
            .all(|e| e.kind != EventKind::Spark && e.kind != EventKind::DwellStart));
        assert!(s
            .events()
            .iter()
            .any(|e| e.kind == EventKind::InjectionStart));
        r.spark_cut = false;
        r.fuel_cut = true;
        let s = build_schedule(&[1, 3, 4, 2], &r).unwrap();
        assert!(s
            .events()
            .iter()
            .all(|e| e.kind != EventKind::InjectionStart));
        assert!(s.events().iter().any(|e| e.kind == EventKind::Spark));
    }

    #[test]
    fn invalid_firing_order_rejected() {
        assert_eq!(
            build_schedule(&[1, 1, 4, 2], &req()).unwrap_err(),
            ScheduleError::InvalidFiringOrder
        );
        assert_eq!(
            build_schedule(&[], &req()).unwrap_err(),
            ScheduleError::NoCylinders
        );
    }

    #[test]
    fn wrap_crossing_is_detected_once() {
        let mut r = req();
        r.advance_deg = 5.0;
        let s = build_schedule(&[1, 3, 4, 2], &r).unwrap();
        let spark = s
            .events()
            .iter()
            .find(|e| e.cylinder == 1 && e.kind == EventKind::Spark)
            .unwrap();
        assert!((spark.angle_deg - 715.0).abs() < 1e-3);
        let crossed: Vec<_> = s.events_crossed(710.0, 0.0).collect();
        assert!(crossed
            .iter()
            .any(|e| e.kind == EventKind::Spark && e.cylinder == 1));
        assert_eq!(s.events_crossed(700.0, 710.0).count(), 0);
    }

    #[test]
    fn twelve_cylinders_fit_event_buffer() {
        let order: Vec<u8> = (1..=12).collect();
        let mut r = req();
        r.rpm = 6000.0;
        let s = build_schedule(&order, &r).unwrap();
        assert_eq!(s.len(), 36);
    }
}
