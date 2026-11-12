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
    /// Injector channel for `InjectionStart`; coil channel for `DwellStart` and `Spark`.
    pub output_channel: u8,
    pub kind: EventKind,
    pub angle_deg: f32,
    pub duration_deg: f32,
    pub duration_ms: f32,
}

/// One cylinder's compression TDC and simulated output channels (1-based).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CylinderSlot {
    pub cylinder: u8,
    pub tdc_deg: f32,
    pub injector_channel: u8,
    pub coil_channel: u8,
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
                output_channel: 0,
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
    InvalidCylinderAngle,
    InvalidOutputChannel,
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
            ScheduleError::InvalidCylinderAngle => {
                "cylinder TDC angle must be finite and in 0..720"
            }
            ScheduleError::InvalidOutputChannel => {
                "output channels must each be a permutation of 1..=cylinders"
            }
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
    let mut slots = [CylinderSlot {
        cylinder: 0,
        tdc_deg: 0.0,
        injector_channel: 0,
        coil_channel: 0,
    }; MAX_CYLINDERS];
    let spacing = 720.0 / n as f32;
    for (i, &c) in firing_order.iter().enumerate() {
        slots[i] = CylinderSlot {
            cylinder: c,
            tdc_deg: i as f32 * spacing,
            injector_channel: c,
            coil_channel: c,
        };
    }
    build_sequential_schedule(&slots[..n], req)
}

pub fn build_sequential_schedule(
    slots: &[CylinderSlot],
    req: &ScheduleRequest,
) -> Result<EventSchedule, ScheduleError> {
    let n = slots.len();
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
    for s in slots {
        let c = s.cylinder;
        let ci = c as usize;
        if c == 0 || ci > n || seen[ci] {
            return Err(ScheduleError::InvalidFiringOrder);
        }
        seen[ci] = true;
    }
    if slots
        .iter()
        .any(|s| !s.tdc_deg.is_finite() || !(0.0..720.0).contains(&s.tdc_deg))
    {
        return Err(ScheduleError::InvalidCylinderAngle);
    }
    let mut injector_seen = [false; MAX_CYLINDERS + 1];
    let mut coil_seen = [false; MAX_CYLINDERS + 1];
    for s in slots {
        for (channel, seen) in [
            (s.injector_channel, &mut injector_seen),
            (s.coil_channel, &mut coil_seen),
        ] {
            let ci = channel as usize;
            if channel == 0 || ci > n || seen[ci] {
                return Err(ScheduleError::InvalidOutputChannel);
            }
            seen[ci] = true;
        }
    }

    let deg_per_ms = req.rpm * 0.006;
    let pw_deg = req.pulse_width_ms * deg_per_ms;
    let dwell_deg = req.dwell_ms * deg_per_ms;
    if pw_deg > 720.0 || dwell_deg > 720.0 {
        return Err(ScheduleError::PulseLongerThanCycle);
    }

    let mut sched = EventSchedule::empty();
    for slot in slots {
        let tdc = slot.tdc_deg;
        let cylinder = slot.cylinder;
        if !req.fuel_cut {
            let end = wrap_angle(tdc - req.injection_end_btdc_deg);
            let start = wrap_angle(end - pw_deg);
            sched.push(ScheduledEvent {
                cylinder,
                output_channel: slot.injector_channel,
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
                output_channel: slot.coil_channel,
                kind: EventKind::DwellStart,
                angle_deg: dwell_start,
                duration_deg: dwell_deg,
                duration_ms: req.dwell_ms,
            });
            sched.push(ScheduledEvent {
                cylinder,
                output_channel: slot.coil_channel,
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

    const ORDERS: [&[u8]; 8] = [
        &[1],
        &[1, 2],
        &[1, 2, 3],
        &[1, 3, 4, 2],
        &[1, 2, 4, 5, 3],
        &[1, 5, 3, 6, 2, 4],
        &[1, 3, 5, 7, 2, 4, 6],
        &[1, 8, 4, 3, 6, 5, 7, 2],
    ];

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

    #[test]
    fn firing_intervals_are_even_for_one_to_eight_cylinders() {
        for order in ORDERS {
            let s = build_schedule(order, &req()).unwrap();
            let n = order.len();
            if n == 1 {
                assert_eq!(
                    s.events()
                        .iter()
                        .find(|e| e.kind == EventKind::Spark)
                        .unwrap()
                        .angle_deg,
                    700.0
                );
                continue;
            }
            for kind in [
                EventKind::InjectionStart,
                EventKind::DwellStart,
                EventKind::Spark,
            ] {
                let angle = |cylinder| {
                    s.events()
                        .iter()
                        .find(|e| e.cylinder == cylinder && e.kind == kind)
                        .unwrap()
                        .angle_deg
                };
                for i in 0..n {
                    let interval = (angle(order[(i + 1) % n]) - angle(order[i])).rem_euclid(720.0);
                    assert!(
                        (interval - 720.0 / n as f32).abs() < 1e-3,
                        "n={n} kind={kind:?} interval={interval}"
                    );
                }
            }
        }
    }

    #[test]
    fn each_cylinder_has_one_injection_dwell_and_spark() {
        for order in ORDERS {
            let s = build_schedule(order, &req()).unwrap();
            assert_eq!(s.len(), 3 * order.len());
            for &cylinder in order {
                for kind in [
                    EventKind::InjectionStart,
                    EventKind::DwellStart,
                    EventKind::Spark,
                ] {
                    assert_eq!(
                        s.events()
                            .iter()
                            .filter(|e| e.cylinder == cylinder && e.kind == kind)
                            .count(),
                        1
                    );
                }
            }
        }
    }

    #[test]
    fn events_carry_output_channel_of_their_cylinder() {
        for order in ORDERS {
            let s = build_schedule(order, &req()).unwrap();
            assert!(s.events().iter().all(|e| e.output_channel == e.cylinder));
        }
    }

    #[test]
    fn sequential_schedule_matches_build_schedule_for_inline4() {
        let slots = [(1, 0.0), (3, 180.0), (4, 360.0), (2, 540.0)].map(|(cylinder, tdc_deg)| {
            CylinderSlot {
                cylinder,
                tdc_deg,
                injector_channel: cylinder,
                coil_channel: cylinder,
            }
        });
        assert_eq!(
            build_sequential_schedule(&slots, &req()).unwrap().events(),
            build_schedule(&[1, 3, 4, 2], &req()).unwrap().events()
        );
        let slots = [
            CylinderSlot {
                cylinder: 1,
                tdc_deg: 0.0,
                injector_channel: 2,
                coil_channel: 1,
            },
            CylinderSlot {
                cylinder: 2,
                tdc_deg: 360.0,
                injector_channel: 1,
                coil_channel: 2,
            },
        ];
        let s = build_sequential_schedule(&slots, &req()).unwrap();
        for (kind, channel) in [(EventKind::InjectionStart, 2), (EventKind::Spark, 1)] {
            assert_eq!(
                s.events()
                    .iter()
                    .find(|e| e.cylinder == 1 && e.kind == kind)
                    .unwrap()
                    .output_channel,
                channel
            );
        }
    }

    #[test]
    fn invalid_slots_rejected() {
        let slots = [
            CylinderSlot {
                cylinder: 1,
                tdc_deg: 0.0,
                injector_channel: 1,
                coil_channel: 1,
            },
            CylinderSlot {
                cylinder: 2,
                tdc_deg: 360.0,
                injector_channel: 2,
                coil_channel: 2,
            },
        ];
        for angle in [720.0, f32::NAN] {
            let mut invalid = slots;
            invalid[0].tdc_deg = angle;
            assert_eq!(
                build_sequential_schedule(&invalid, &req()).unwrap_err(),
                ScheduleError::InvalidCylinderAngle
            );
        }
        let mut invalid = slots;
        invalid[1].injector_channel = 1;
        assert_eq!(
            build_sequential_schedule(&invalid, &req()).unwrap_err(),
            ScheduleError::InvalidOutputChannel
        );
        let mut invalid = slots;
        invalid[0].coil_channel = 0;
        assert_eq!(
            build_sequential_schedule(&invalid, &req()).unwrap_err(),
            ScheduleError::InvalidOutputChannel
        );
        let mut invalid = slots;
        invalid[1].cylinder = 1;
        assert_eq!(
            build_sequential_schedule(&invalid, &req()).unwrap_err(),
            ScheduleError::InvalidFiringOrder
        );
    }
}
