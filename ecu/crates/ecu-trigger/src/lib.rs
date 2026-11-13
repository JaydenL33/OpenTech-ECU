//! Missing-tooth crank trigger decoder and sync state machine.
//!
//! Safety properties:
//! - angles are only reported as `Sequential` after a valid gap **and** cam
//!   phase;
//! - a gap that does not match the configured missing-tooth count causes sync
//!   loss, not a guessed position;
//! - a missing signal times out to `Lost`;
//! - all state is fixed-size and deterministic; no allocation and no unsafe.
//!
//! Physical base offset is a calibration value. `sync_offset_deg` is the angle
//! assigned to the first tooth after the gap; it is **not** evidence of TDC.
//! A timing light is mandatory before physical use.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

use core::fmt;
use serde::{Deserialize, Serialize};

pub const MAX_TEETH: u16 = 120;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MissingToothConfig {
    pub teeth: u16,
    pub missing: u16,
    pub cam_required: bool,
    pub sync_offset_deg: f32,
    pub timeout_ms: u16,
}

impl Default for MissingToothConfig {
    fn default() -> Self {
        Self {
            teeth: 36,
            missing: 1,
            cam_required: true,
            sync_offset_deg: 0.0,
            timeout_ms: 250,
        }
    }
}

impl MissingToothConfig {
    pub fn validate(&self) -> Result<(), TriggerConfigError> {
        if !(4..=MAX_TEETH).contains(&self.teeth) {
            return Err(TriggerConfigError::InvalidToothCount);
        }
        if self.missing == 0 || self.missing >= self.teeth {
            return Err(TriggerConfigError::InvalidMissingCount);
        }
        if !self.sync_offset_deg.is_finite() || !(0.0..360.0).contains(&self.sync_offset_deg) {
            return Err(TriggerConfigError::InvalidOffset);
        }
        if self.timeout_ms < 10 {
            return Err(TriggerConfigError::InvalidTimeout);
        }
        Ok(())
    }

    pub fn tooth_spacing_deg(&self) -> f32 {
        360.0 / self.teeth as f32
    }

    /// With `sync_offset_deg == 0.0` the argument equals the decoder cycle angle.
    pub fn tooth_reference(&self, angle_from_tooth0_deg: f32) -> Option<ToothReference> {
        let angle = angle_from_tooth0_deg;
        if self.validate().is_err() || !angle.is_finite() || !(0.0..720.0).contains(&angle) {
            return None;
        }
        let spacing = self.tooth_spacing_deg();
        let mut pos = ((angle / spacing) as u16).min(2 * self.teeth - 1);
        let in_rev = pos % self.teeth;
        let first_missing = self.teeth - self.missing;
        if in_rev >= first_missing {
            pos = pos - in_rev + first_missing - 1;
        }
        Some(ToothReference {
            tooth: pos,
            offset_deg: angle - pos as f32 * spacing,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerConfigError {
    InvalidToothCount,
    InvalidMissingCount,
    InvalidOffset,
    InvalidTimeout,
    InvalidCamWindow,
}

/// Last physically present tooth at or before an angle (D6).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToothReference {
    /// cycle position 0..2*teeth; 0 = first tooth after the gap in the cam-phase revolution
    pub tooth: u16,
    /// degrees from that tooth edge to the angle
    pub offset_deg: f32,
}

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
pub enum DecodeError {
    TimestampWentBackwards,
    ZeroInterval,
    GapMismatch,
    ToothDropped,
    Timeout,
    CamWithoutSync,
    TooFewTeethBetweenGaps,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecodeEvent {
    None,
    GapDetected,
    Synchronised { sequential: bool },
    CamPhase,
    SyncLost(DecodeError),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DecoderStatus {
    pub state: SyncState,
    pub rpm: f32,
    /// Current crank angle 0..360 after sync. `None` before first valid gap.
    pub crank_angle_deg: Option<f32>,
    /// Current 0..720 cycle angle. Only available in Sequential state.
    pub cycle_angle_deg: Option<f32>,
    pub tooth_index: u16,
    pub expected_interval_us: f32,
    pub last_error: Option<DecodeError>,
    pub error_count: u32,
    pub gap_count: u64,
    pub cam_seen: bool,
    pub cam_phase_rejected: bool,
}

#[derive(Clone, Debug)]
pub struct MissingToothDecoder {
    loss_teeth: u16,
    cam_loss_revolutions: u16,
    cam_age_teeth: u32,
    last_cam_us: Option<u64>,
    cfg: MissingToothConfig,
    cam_window_deg: Option<(f32, f32)>,
    state: SyncState,
    last_tooth_us: Option<u64>,
    last_interval_us: f32,
    expected_interval_us: f32,
    last_gap_us: Option<u64>,
    rpm: f32,
    crank_angle: f32,
    cam_seen_since_gap: bool,
    cam_ever_seen: bool,
    cam_phase_known: bool,
    cam_phase_rejected: bool,
    rev_parity: u8,
    tooth_index: u16,
    gap_count: u64,
    error_count: u32,
    last_error: Option<DecodeError>,
}

impl MissingToothDecoder {
    pub fn new(cfg: MissingToothConfig) -> Result<Self, TriggerConfigError> {
        cfg.validate()?;
        Ok(Self {
            loss_teeth: cfg.missing.saturating_add(3),
            cam_loss_revolutions: 6,
            cam_age_teeth: 0,
            last_cam_us: None,
            cfg,
            cam_window_deg: None,
            state: SyncState::Unsynced,
            last_tooth_us: None,
            last_interval_us: 0.0,
            expected_interval_us: 0.0,
            last_gap_us: None,
            rpm: 0.0,
            crank_angle: cfg.sync_offset_deg,
            cam_seen_since_gap: false,
            cam_ever_seen: false,
            cam_phase_known: false,
            cam_phase_rejected: false,
            rev_parity: 0,
            tooth_index: 0,
            gap_count: 0,
            error_count: 0,
            last_error: None,
        })
    }

    /// Configure loss bounds without changing the serialized wheel config.
    /// Crank deadline is normal interval * teeth, capped by timeout_ms.
    /// Must exceed the legitimate missing-tooth gap. Cam expires after the
    /// specified revolutions of accepted edges, or timeout_ms without a cam.
    pub fn set_loss_bounds(
        &mut self,
        teeth: u16,
        cam_revolutions: u16,
    ) -> Result<(), TriggerConfigError> {
        if teeth <= self.cfg.missing + 1 || cam_revolutions < 2 {
            return Err(TriggerConfigError::InvalidTimeout);
        }
        self.loss_teeth = teeth;
        self.cam_loss_revolutions = cam_revolutions;
        Ok(())
    }

    fn expire_cam(&mut self, now: u64) {
        let limit = u32::from(self.cfg.teeth - self.cfg.missing)
            .saturating_mul(u32::from(self.cam_loss_revolutions));
        if self.cfg.cam_required
            && (self.cam_age_teeth >= limit
                || self.last_cam_us.is_some_and(|last| {
                    now.saturating_sub(last) > u64::from(self.cfg.timeout_ms) * 1000
                }))
        {
            self.cam_phase_known = false;
            self.cam_seen_since_gap = false;
            if self.state == SyncState::Sequential {
                self.state = SyncState::CrankSynced;
            }
        }
    }

    pub fn config(&self) -> MissingToothConfig {
        self.cfg
    }

    /// Set an inclusive cam window in crank degrees from the last gap.
    /// Windows must be finite, ordered, and within one revolution. With cam
    /// required, `None` prevents phase validation. Changing the window requires
    /// revalidation.
    pub fn set_cam_window(
        &mut self,
        window_deg: Option<(f32, f32)>,
    ) -> Result<(), TriggerConfigError> {
        if let Some((start, end)) = window_deg {
            if !start.is_finite()
                || !end.is_finite()
                || !(0.0..360.0).contains(&start)
                || !(start..360.0).contains(&end)
            {
                return Err(TriggerConfigError::InvalidCamWindow);
            }
        }
        self.cam_window_deg = window_deg;
        self.cam_phase_known = false;
        self.cam_seen_since_gap = false;
        if self.cfg.cam_required && self.state == SyncState::Sequential {
            self.state = SyncState::CrankSynced;
        }
        Ok(())
    }

    pub fn status(&self) -> DecoderStatus {
        DecoderStatus {
            state: self.state,
            rpm: self.rpm,
            crank_angle_deg: if matches!(self.state, SyncState::CrankSynced | SyncState::Sequential)
            {
                Some(self.crank_angle)
            } else {
                None
            },
            cycle_angle_deg: if self.state == SyncState::Sequential {
                Some(self.cycle_angle_at_last_tooth())
            } else {
                None
            },
            tooth_index: self.tooth_index,
            expected_interval_us: self.expected_interval_us,
            last_error: self.last_error,
            error_count: self.error_count,
            gap_count: self.gap_count,
            cam_seen: self.cam_ever_seen,
            cam_phase_rejected: self.cam_phase_rejected,
        }
    }

    pub fn state(&self) -> SyncState {
        self.state
    }

    pub fn rpm(&self) -> f32 {
        self.rpm
    }

    pub fn is_sequential(&self) -> bool {
        self.state == SyncState::Sequential
    }

    /// Current crank angle extrapolated from the last tooth to `now_us`.
    /// Returns `None` before a valid gap.
    pub fn crank_angle_at(&self, now_us: u64) -> Option<f32> {
        if !matches!(self.state, SyncState::CrankSynced | SyncState::Sequential) {
            return None;
        }
        let last = self.last_tooth_us?;
        let dt_us = now_us.saturating_sub(last) as f32;
        // rpm * 360/60 = rpm*6 deg/s = rpm*6e-6 deg/us.
        let advance = self.rpm * 6e-6 * dt_us;
        Some(wrap_angle(self.crank_angle + advance, 360.0))
    }

    pub fn cycle_angle_at(&self, now_us: u64) -> Option<f32> {
        if self.state != SyncState::Sequential {
            return None;
        }
        let last = self.last_tooth_us?;
        let advance = self.rpm * 6e-6 * now_us.saturating_sub(last) as f32;
        let from_gap = wrap_angle(self.crank_angle - self.cfg.sync_offset_deg, 360.0);
        // Advance before adding cycle parity to preserve sub-degree precision.
        let advanced = from_gap + self.cfg.sync_offset_deg + advance;
        Some(wrap_angle(advanced + self.rev_parity as f32 * 360.0, 720.0))
    }

    fn cycle_angle_at_last_tooth(&self) -> f32 {
        // Parity changes at tooth zero, which need not coincide with crank 0°.
        let from_gap = wrap_angle(self.crank_angle - self.cfg.sync_offset_deg, 360.0);
        wrap_angle(
            from_gap + self.cfg.sync_offset_deg + self.rev_parity as f32 * 360.0,
            720.0,
        )
    }

    /// Feed one conditioned crank tooth edge. `timestamp_us` is a monotonic
    /// hardware capture timestamp, not the control-loop time.
    pub fn on_tooth(&mut self, timestamp_us: u64) -> DecodeEvent {
        let Some(last) = self.last_tooth_us else {
            self.last_tooth_us = Some(timestamp_us);
            self.tooth_index = 0;
            if self.state == SyncState::Lost {
                self.state = SyncState::Unsynced;
            }
            return DecodeEvent::None;
        };
        if timestamp_us <= last {
            return self.fail(DecodeError::TimestampWentBackwards);
        }
        let dt = (timestamp_us - last) as f32;
        if dt < 1.0 {
            return self.fail(DecodeError::ZeroInterval);
        }
        if matches!(self.state, SyncState::CrankSynced | SyncState::Sequential)
            && self.expected_interval_us > 1.0
            && dt < self.expected_interval_us * 0.5
        {
            // Ignore short edges without poisoning timestamp or speed history.
            // Only once a gap has validated the tooth period: before sync the
            // reference may be the 2x gap interval, and rejecting against it
            // under acceleration discards every other real tooth so the gap
            // is never observed (half-rate lock).
            return DecodeEvent::None;
        }
        if matches!(self.tick(timestamp_us), DecodeEvent::SyncLost(_)) {
            self.last_tooth_us = Some(timestamp_us);
            self.state = SyncState::Unsynced;
            return DecodeEvent::SyncLost(DecodeError::Timeout);
        }
        if self.state == SyncState::Lost {
            // The first edge after a stall/too-long gap is a fresh capture
            // baseline, not a tooth interval. Consume it and let the next
            // edge establish the tooth period so the long gap cannot poison
            // `expected_interval_us` and starve short-edge rejection.
            self.last_tooth_us = Some(timestamp_us);
            self.tooth_index = 0;
            self.expected_interval_us = 0.0;
            self.state = SyncState::Unsynced;
            return DecodeEvent::None;
        }
        self.cam_age_teeth = self.cam_age_teeth.saturating_add(1);
        self.expire_cam(timestamp_us);
        self.last_tooth_us = Some(timestamp_us);

        let had_expected = self.expected_interval_us > 1.0;
        let ratio = if had_expected {
            dt / self.expected_interval_us
        } else {
            1.0
        };
        let missing_ratio_threshold = 1.45;
        let plausible_gap_max = (self.cfg.missing as f32 + 2.2).max(2.2);

        if had_expected && ratio >= missing_ratio_threshold && ratio <= plausible_gap_max {
            let estimated_missing = (round_to_i32(ratio) - 1).max(0) as u16;
            if estimated_missing != self.cfg.missing {
                return self.fail(DecodeError::GapMismatch);
            }
            // After the first synchronisation, the tooth count between gaps is
            // a second, independent check. A dropped tooth produces a gap-like
            // interval; counting prevents it from being accepted as the index
            // gap at the wrong crank angle.
            if self.gap_count > 0 {
                let expected_normal_teeth = self.cfg.teeth - self.cfg.missing - 1;
                if self.tooth_index != expected_normal_teeth {
                    return self.fail(if self.tooth_index < expected_normal_teeth {
                        DecodeError::ToothDropped
                    } else {
                        DecodeError::GapMismatch
                    });
                }
            }

            self.gap_count = self.gap_count.saturating_add(1);
            self.tooth_index = 0;
            self.crank_angle = self.cfg.sync_offset_deg;
            self.last_interval_us = dt / (self.cfg.missing as f32 + 1.0);
            self.expected_interval_us = self.last_interval_us;

            // Speed from gap-to-gap (360 crank degrees) when available; this is
            // one full revolution and rejects single-tooth jitter.
            if let Some(prev_gap) = self.last_gap_us {
                let rev_us = (timestamp_us - prev_gap) as f32;
                if rev_us > 0.0 {
                    self.rpm = (60_000_000.0 / rev_us).clamp(0.0, 20_000.0);
                }
            } else {
                // First gap: use angular distance covered by the gap.
                let gap_deg = (self.cfg.missing as f32 + 1.0) * self.cfg.tooth_spacing_deg();
                self.rpm = (60_000_000.0 * gap_deg / (360.0 * dt)).clamp(0.0, 20_000.0);
            }
            self.last_gap_us = Some(timestamp_us);

            if self.cam_seen_since_gap {
                self.rev_parity = 0;
            } else {
                self.rev_parity = self.rev_parity.wrapping_add(1) & 1;
            }
            let sequential = !self.cfg.cam_required || self.cam_phase_known;
            self.cam_seen_since_gap = false;
            self.state = if sequential {
                SyncState::Sequential
            } else {
                SyncState::CrankSynced
            };
            return DecodeEvent::Synchronised { sequential };
        }

        // Normal tooth.
        if ratio > 1.25 && ratio < missing_ratio_threshold && self.gap_count > 0 {
            // An interval noticeably longer than expected but not a full gap is
            // evidence of a dropped tooth. Do not invent an angle.
            return self.fail(DecodeError::ToothDropped);
        }
        if self.state == SyncState::Unsynced && !had_expected {
            self.expected_interval_us = dt;
        } else {
            // Track speed changes without letting a gap poison the reference.
            self.expected_interval_us = 0.75 * self.expected_interval_us + 0.25 * dt;
        }
        self.last_interval_us = dt;
        self.tooth_index = self.tooth_index.saturating_add(1);
        if matches!(self.state, SyncState::CrankSynced | SyncState::Sequential) {
            self.crank_angle = wrap_angle(self.crank_angle + self.cfg.tooth_spacing_deg(), 360.0);
        }
        DecodeEvent::None
    }

    /// Cam phase input, validated against the configured window from the last
    /// crank gap. Physical cam timing still requires independent verification.
    pub fn on_cam(&mut self, timestamp_us: u64) -> DecodeEvent {
        self.cam_ever_seen = true;
        if !self.cfg.cam_required {
            self.cam_seen_since_gap = true;
            self.cam_phase_known = true;
            if self.state == SyncState::CrankSynced {
                self.state = SyncState::Sequential;
            }
            return DecodeEvent::CamPhase;
        }

        let synced = matches!(self.state, SyncState::CrankSynced | SyncState::Sequential);
        let valid = match (self.cam_window_deg, self.last_gap_us) {
            (Some((start, end)), Some(gap)) if synced && timestamp_us >= gap => {
                // Use the decoder's own tooth-based angle, not gap time times
                // the lagged speed estimate. That estimate is off by 30-60 deg
                // while the engine accelerates, which rejected valid cam edges.
                match self.crank_angle_at(timestamp_us) {
                    Some(angle) => {
                        (start..=end).contains(&wrap_angle(angle - self.cfg.sync_offset_deg, 360.0))
                    }
                    None => false,
                }
            }
            _ => false,
        };
        self.cam_phase_rejected = !valid;
        self.cam_phase_known = valid;
        self.cam_seen_since_gap = valid;
        if valid {
            self.cam_age_teeth = 0;
            self.last_cam_us = Some(timestamp_us);
            self.rev_parity = 1;
            self.state = SyncState::Sequential;
        } else if synced {
            self.state = SyncState::CrankSynced;
        }
        DecodeEvent::CamPhase
    }

    /// Call from the control loop. Times out a stopped engine.
    pub fn tick(&mut self, now_us: u64) -> DecodeEvent {
        self.expire_cam(now_us);
        let Some(last) = self.last_tooth_us else {
            return DecodeEvent::None;
        };
        let timeout_us = self.cfg.timeout_ms as u64 * 1000;
        let timeout_us = if self.expected_interval_us > 1.0 {
            timeout_us.min((self.expected_interval_us * f32::from(self.loss_teeth)) as u64)
        } else {
            timeout_us
        };
        if now_us.saturating_sub(last) > timeout_us && self.state != SyncState::Lost {
            self.state = SyncState::Lost;
            self.rpm = 0.0;
            self.last_error = Some(DecodeError::Timeout);
            self.error_count = self.error_count.saturating_add(1);
            self.cam_seen_since_gap = false;
            self.cam_phase_known = false;
            self.expected_interval_us = 0.0;
            self.last_interval_us = 0.0;
            self.last_gap_us = None;
            // Forget the stale timestamp and prior revolution count. When capture
            // resumes after a dropout, the first edge establishes a fresh
            // baseline and the first gap is accepted as a resync, not rejected by
            // a tooth count from before the outage.
            self.last_tooth_us = None;
            self.tooth_index = 0;
            self.gap_count = 0;
            self.rev_parity = 0;
            return DecodeEvent::SyncLost(DecodeError::Timeout);
        }
        DecodeEvent::None
    }

    fn fail(&mut self, err: DecodeError) -> DecodeEvent {
        self.last_error = Some(err);
        self.error_count = self.error_count.saturating_add(1);
        self.state = SyncState::Lost;
        self.rpm = 0.0;
        self.expected_interval_us = 0.0;
        self.last_interval_us = 0.0;
        self.last_gap_us = None;
        self.cam_seen_since_gap = false;
        self.cam_phase_known = false;
        self.rev_parity = 0;
        // Force a fresh capture baseline and gap count after any sync loss.
        self.last_tooth_us = None;
        self.tooth_index = 0;
        self.gap_count = 0;
        DecodeEvent::SyncLost(err)
    }
}

#[inline]
fn wrap_angle(v: f32, modulus: f32) -> f32 {
    // f32::rem_euclid is std-only; embedded firmware needs this local form.
    let r = v % modulus;
    if r < 0.0 {
        r + modulus
    } else {
        r
    }
}

#[inline]
fn round_to_i32(v: f32) -> i32 {
    if v >= 0.0 {
        (v + 0.5) as i32
    } else {
        (v - 0.5) as i32
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DecodeError::TimestampWentBackwards => "capture timestamp went backwards",
            DecodeError::ZeroInterval => "zero tooth interval",
            DecodeError::GapMismatch => "gap does not match configured missing teeth",
            DecodeError::ToothDropped => "dropped tooth detected",
            DecodeError::Timeout => "no crank signal before timeout",
            DecodeError::CamWithoutSync => "cam phase without crank sync",
            DecodeError::TooFewTeethBetweenGaps => "too few teeth between gaps",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_extrapolation_preserves_phase_across_gap_before_next_capture() {
        let mut d = decoder();
        d.set_cam_window(Some((320.0, 340.0))).unwrap();
        // 1000 rpm, independent integer timestamps (500/3 us per degree).
        for rev in 0..2_u64 {
            for tooth in 0..35_u64 {
                let angle = rev * 360 + tooth * 10;
                let time = 1000 + angle * 500 / 3;
                d.on_tooth(time);
                if angle == 690 {
                    d.on_cam(time);
                }
            }
        }
        assert_eq!(d.status().cycle_angle_deg, Some(700.0));
        // Last capture is 700°. After 24° (4000 us), cycle phase is 4°,
        // even before the delayed next gap capture is delivered.
        let time = 1000 + 700 * 500 / 3 + 4000;
        assert!((d.cycle_angle_at(time).unwrap() - 4.0).abs() < 0.01);
        assert!((d.crank_angle_at(time).unwrap() - 4.0).abs() < 0.01);
    }

    /// Produce tooth timestamps for a 36-1 wheel. Teeth at 0..=340 step 10,
    /// gap at 350. Cam pulse at 690 degrees (before the gap at 720/0).
    fn generate(rpm: f32, revolutions: u32, cam: bool) -> Vec<(bool, u64)> {
        let mut events = Vec::new();
        let us_per_deg = 60_000_000.0 / (rpm * 360.0);
        for rev in 0..revolutions * 2 {
            for tooth in 0..36 {
                let angle = rev as f32 * 360.0 + tooth as f32 * 10.0;
                if tooth == 35 {
                    continue; // missing tooth at 350-360
                }
                events.push((false, (angle * us_per_deg) as u64 + 1000));
            }
            // A real cam pulse repeats once per 720-degree cycle. The old
            // fixture emitted it only in the first cycle, which the new
            // bounded cam-expiry policy correctly rejects after six
            // revolutions without a valid edge.
            if cam && rev % 2 == 1 {
                // Cam at 690 degrees of each 720-degree cycle (i.e. within
                // odd-numbered crank revolutions).
                let cycle = rev / 2;
                events.push((
                    true,
                    ((cycle as f32 * 720.0 + 690.0) * us_per_deg) as u64 + 1000,
                ));
            }
        }
        events.sort_by_key(|e| e.1);
        events
    }

    fn decoder() -> MissingToothDecoder {
        MissingToothDecoder::new(MissingToothConfig::default()).unwrap()
    }

    #[test]
    fn config_rejects_impossible_wheels() {
        let c = MissingToothConfig {
            teeth: 0,
            ..Default::default()
        };
        assert!(c.validate().is_err());
        let c = MissingToothConfig {
            missing: 36,
            ..Default::default()
        };
        assert!(c.validate().is_err());
    }

    #[test]
    fn syncs_and_measures_constant_speed() {
        let mut d = decoder();
        d.set_cam_window(Some((320.0, 340.0))).unwrap();
        for (cam, ts) in generate(3000.0, 3, true) {
            if cam {
                d.on_cam(ts);
            } else {
                d.on_tooth(ts);
            }
        }
        let s = d.status();
        assert_eq!(s.state, SyncState::Sequential, "{s:?}");
        assert!((s.rpm - 3000.0).abs() < 15.0, "rpm={}", s.rpm);
        assert!(s.cam_seen);
        assert!(s.cycle_angle_deg.is_some());
        assert_eq!(s.error_count, 0, "{:?}", s.last_error);
    }

    #[test]
    fn without_cam_it_stays_crank_synced() {
        let mut d = decoder();
        for (_, ts) in generate(1500.0, 3, false) {
            d.on_tooth(ts);
        }
        assert_eq!(d.state(), SyncState::CrankSynced);
        assert!(d.cycle_angle_at(20_000_000).is_none());
    }

    #[test]
    fn cam_inside_window_gives_sequential() {
        let mut d = decoder();
        d.set_cam_window(Some((320.0, 340.0))).unwrap();
        for (cam, ts) in generate(3000.0, 2, true) {
            if cam {
                d.on_cam(ts);
                assert_eq!(d.state(), SyncState::Sequential);
                assert!(!d.status().cam_phase_rejected);
            } else {
                d.on_tooth(ts);
            }
        }
        assert_eq!(d.state(), SyncState::Sequential);
    }

    #[test]
    fn cam_outside_window_is_rejected() {
        let mut d = decoder();
        d.set_cam_window(Some((10.0, 20.0))).unwrap();
        for (cam, ts) in generate(3000.0, 2, true) {
            if cam {
                d.on_cam(ts);
                assert_eq!(d.state(), SyncState::CrankSynced);
                assert!(d.status().cam_phase_rejected);
            } else {
                d.on_tooth(ts);
            }
        }
        assert_eq!(d.state(), SyncState::CrankSynced);
        assert!(d.status().cycle_angle_deg.is_none());
    }

    #[test]
    fn cam_without_window_is_rejected() {
        let mut d = decoder();
        for (cam, ts) in generate(3000.0, 2, true) {
            if cam {
                d.on_cam(ts);
                assert_eq!(d.state(), SyncState::CrankSynced);
                assert!(d.status().cam_phase_rejected);
            } else {
                d.on_tooth(ts);
            }
        }
        assert_eq!(d.state(), SyncState::CrankSynced);
    }

    #[test]
    fn cam_rejection_revokes_validated_phase() {
        let mut d = decoder();
        d.set_cam_window(Some((320.0, 340.0))).unwrap();
        for (cam, ts) in generate(3000.0, 2, true) {
            if cam {
                d.on_cam(ts);
                assert_eq!(d.state(), SyncState::Sequential);
                d.on_cam(ts + 1_000);
                assert_eq!(d.state(), SyncState::CrankSynced);
                assert!(d.status().cam_phase_rejected);
            } else {
                d.on_tooth(ts);
            }
        }
        assert_eq!(d.state(), SyncState::CrankSynced);
    }

    #[test]
    fn cam_not_required_keeps_sequential_without_window() {
        let mut d = MissingToothDecoder::new(MissingToothConfig {
            cam_required: false,
            ..Default::default()
        })
        .unwrap();
        for (cam, ts) in generate(3000.0, 2, true) {
            if cam {
                d.on_cam(ts);
            } else {
                d.on_tooth(ts);
            }
        }
        assert_eq!(d.state(), SyncState::Sequential);
        assert!(!d.status().cam_phase_rejected);
    }

    #[test]
    fn invalid_cam_windows_are_rejected() {
        let mut d = decoder();
        for window in [
            (f32::NAN, 10.0),
            (0.0, f32::INFINITY),
            (-1.0, 10.0),
            (20.0, 10.0),
            (0.0, 360.0),
        ] {
            assert_eq!(
                d.set_cam_window(Some(window)),
                Err(TriggerConfigError::InvalidCamWindow)
            );
        }
    }

    #[test]
    fn extra_missing_tooth_causes_sync_loss() {
        let mut d = decoder();
        for (cam, ts) in generate(2000.0, 3, true) {
            if cam {
                d.on_cam(ts);
            } else {
                d.on_tooth(ts);
            }
        }
        // Spacing at 2000 rpm for 10 degrees is 833 us. Feed one normal tooth,
        // then an interval representing three missing teeth (should be one).
        let last = d.last_tooth_us.unwrap();
        let spacing = 833u64;
        d.on_tooth(last + spacing);
        let ev = d.on_tooth(last + spacing * 4);
        assert!(
            matches!(ev, DecodeEvent::SyncLost(DecodeError::GapMismatch)),
            "{ev:?}"
        );
    }

    #[test]
    fn extra_tooth_between_gaps_is_rejected() {
        let mut d = decoder();
        for (cam, ts) in generate(2000.0, 3, true) {
            if cam {
                d.on_cam(ts);
            } else {
                d.on_tooth(ts);
            }
        }
        let last = d.last_tooth_us.unwrap();
        let spacing = 833u64;
        // 35 normal teeth instead of the expected 34, then the gap.
        let mut ts = last;
        for _ in 0..35 {
            ts += spacing;
            d.on_tooth(ts);
        }
        let ev = d.on_tooth(ts + 2 * spacing);
        assert!(matches!(ev, DecodeEvent::SyncLost(_)), "{ev:?}");
    }

    #[test]
    fn dropped_tooth_is_not_guessed() {
        let mut d = decoder();
        let mut evs = Vec::new();
        let mut first = true;
        for (cam, ts) in generate(2500.0, 4, true) {
            if cam {
                d.on_cam(ts);
                continue;
            }
            if first {
                first = false;
                d.on_tooth(ts);
                continue;
            }
            // Drop one normal tooth in the middle, simulating a bad sensor edge.
            if ts > 8_000 && ts < 8_500 {
                continue;
            }
            evs.push(d.on_tooth(ts));
        }
        assert!(
            evs.iter().any(|e| matches!(e, DecodeEvent::SyncLost(_)))
                || d.state() == SyncState::Lost
        );
    }

    #[test]
    fn timeout_declares_lost_sync() {
        let mut d = decoder();
        for (cam, ts) in generate(1000.0, 2, true) {
            if cam {
                d.on_cam(ts);
            } else {
                d.on_tooth(ts);
            }
        }
        let last = d.last_tooth_us.unwrap();
        assert_eq!(d.tick(last + 1_000), DecodeEvent::None);
        let ev = d.tick(last + 300_000);
        assert!(matches!(ev, DecodeEvent::SyncLost(DecodeError::Timeout)));
        assert_eq!(d.state(), SyncState::Lost);
    }

    #[test]
    fn acceleration_tracks_rpm_without_false_gap() {
        // Simulate speed rising from 900 to 2600 rpm over 6 revolutions.
        let mut events = Vec::new();
        let mut t_us = 0.0f64;
        let mut rev = 0f64;
        while rev < 12.0 {
            let rpm = 900.0 + 1700.0 * (rev / 12.0);
            let us_per_deg = 60_000_000.0 / (rpm * 360.0);
            for tooth in 0..36 {
                if tooth != 35 {
                    events.push((false, t_us + 1000.0));
                }
                t_us += 10.0 * us_per_deg;
            }
            if rev as i32 % 2 == 1 {
                events.push((true, t_us - 30.0 * us_per_deg));
            }
            rev += 1.0;
        }
        let mut d = decoder();
        for (cam, ts) in events {
            if cam {
                d.on_cam(ts as u64);
            } else {
                d.on_tooth(ts as u64);
            }
        }
        // Final RPM should be near 2600 and the decoder should not have lost sync
        // during normal acceleration.
        assert!(d.rpm() > 2300.0, "rpm={}", d.rpm());
        assert!(matches!(
            d.state(),
            SyncState::Sequential | SyncState::CrankSynced
        ));
        assert!(d.status().error_count == 0, "{:?}", d.status());
    }

    #[test]
    fn resyncs_after_a_one_second_dropout() {
        let rpm = 900.0f32;
        let us_per_deg = 60_000_000.0 / (rpm * 360.0);
        let mut events = Vec::new();
        for rev in 0..20 {
            for tooth in 0..36 {
                if tooth == 35 {
                    continue;
                }
                let ts = (rev as f32 * 360.0 + tooth as f32 * 10.0) * us_per_deg + 1000.0;
                if (6_000_000.0..7_000_000.0).contains(&ts) {
                    continue;
                }
                events.push((false, ts as u64));
            }
            let cam = (rev as f32 * 360.0 + 330.0) * us_per_deg + 1000.0;
            if !(6_000_000.0..7_000_000.0).contains(&cam) {
                events.push((true, cam as u64));
            }
        }
        let mut d = decoder();
        d.set_cam_window(Some((320.0, 340.0))).unwrap();
        for (cam, ts) in events {
            if cam {
                d.on_cam(ts);
            } else {
                d.on_tooth(ts);
            }
            d.tick(ts + 1);
        }
        let s = d.status();
        assert_eq!(s.state, SyncState::Sequential, "{s:?}");
        assert!((s.rpm - 900.0).abs() < 20.0, "{s:?}");
    }

    #[test]
    fn cycle_angle_wraps_0_to_720() {
        let mut d = decoder();
        d.set_cam_window(Some((320.0, 340.0))).unwrap();
        for (cam, ts) in generate(1000.0, 4, true) {
            if cam {
                d.on_cam(ts);
            } else {
                d.on_tooth(ts);
            }
        }
        let s = d.status();
        let a = s.cycle_angle_deg.unwrap();
        assert!((0.0..720.0).contains(&a));
    }

    #[test]
    fn tooth_reference_on_tooth_edges() {
        let cfg = MissingToothConfig::default();
        for (angle, tooth) in [
            (0.0, 0),
            (10.0, 1),
            (180.0, 18),
            (360.0, 36),
            (540.0, 54),
            (700.0, 70),
        ] {
            let reference = cfg.tooth_reference(angle).unwrap();
            assert_eq!(reference.tooth, tooth);
            assert!(reference.offset_deg.abs() < 1e-4);
        }
    }

    #[test]
    fn tooth_reference_skips_missing_positions() {
        let cfg = MissingToothConfig::default();
        for (angle, tooth, offset) in [
            (345.0, 34, 5.0),
            (350.0, 34, 10.0),
            (355.0, 34, 15.0),
            (715.0, 70, 15.0),
            (719.5, 70, 19.5),
        ] {
            let reference = cfg.tooth_reference(angle).unwrap();
            assert_eq!(reference.tooth, tooth);
            assert!((reference.offset_deg - offset).abs() < 1e-4);
        }
    }

    #[test]
    fn tooth_reference_rejects_out_of_range() {
        let cfg = MissingToothConfig::default();
        for angle in [720.0, -0.1, f32::NAN, f32::INFINITY] {
            assert_eq!(cfg.tooth_reference(angle), None);
        }
        assert_eq!(
            MissingToothConfig { teeth: 0, ..cfg }.tooth_reference(0.0),
            None
        );
    }

    fn feed(d: &mut MissingToothDecoder, events: &[(bool, u64)], offset: u64) -> u64 {
        let mut last = 0;
        for &(cam, ts) in events {
            let ts = ts + offset;
            if cam {
                d.on_cam(ts);
            } else {
                d.on_tooth(ts);
            }
            last = ts;
        }
        last
    }

    #[test]
    fn cam_loss_expires_sequential_phase_and_reacquires() {
        let rpm = 1000.0;
        let us_per_deg = 60_000_000.0 / (rpm * 360.0);
        let mut d = decoder();
        d.set_cam_window(Some((320.0, 340.0))).unwrap();
        let mut last = feed(&mut d, &generate(rpm, 3, true), 0);
        assert_eq!(d.state(), SyncState::Sequential);

        // More than the six-revolution cam-expiry bound without a cam edge.
        // generate() ends on the 340-degree tooth; the next 0-degree edge is
        // one missing-tooth gap (20 degrees) later, not one normal 10-degree
        // tooth spacing.
        let offset = last + (20.0 * us_per_deg) as u64 - 1000;
        last = feed(&mut d, &generate(rpm, 8, false), offset);
        let s = d.status();
        assert_eq!(s.state, SyncState::CrankSynced, "{s:?}");
        assert!(s.cycle_angle_deg.is_none(), "{s:?}");
        assert!(s.cam_seen, "the earlier valid cam must remain in history");

        // A valid cam in the configured window re-establishes phase.
        // generate() ends on the 340-degree tooth; the next 0-degree edge is
        // one missing-tooth gap (20 degrees) later, not one normal 10-degree
        // tooth spacing.
        let offset = last + (20.0 * us_per_deg) as u64 - 1000;
        feed(&mut d, &generate(rpm, 2, true), offset);
        let s = d.status();
        assert_eq!(s.state, SyncState::Sequential, "{s:?}");
        assert!(s.cycle_angle_deg.is_some(), "{s:?}");
    }

    #[test]
    fn crank_loss_times_out_and_recovers() {
        let rpm = 1000.0;
        let mut d = decoder();
        d.set_cam_window(Some((320.0, 340.0))).unwrap();
        let last = feed(&mut d, &generate(rpm, 2, true), 0);
        assert_eq!(d.state(), SyncState::Sequential);

        let event = d.tick(last + 300_000);
        assert!(
            matches!(event, DecodeEvent::SyncLost(DecodeError::Timeout)),
            "{event:?}"
        );
        assert_eq!(d.state(), SyncState::Lost);

        // Resume at the next gap tooth after the stall.
        let offset = last + 300_000 - 1000;
        feed(&mut d, &generate(rpm, 4, true), offset);
        let s = d.status();
        assert_eq!(s.state, SyncState::Sequential, "{s:?}");
    }

    #[test]
    fn short_edge_is_ignored_without_poisoning_sync() {
        let rpm = 1000.0;
        let us_per_deg = 60_000_000.0 / (rpm * 360.0);
        let mut d = decoder();
        d.set_cam_window(Some((320.0, 340.0))).unwrap();
        let last = feed(&mut d, &generate(rpm, 2, true), 0);
        assert_eq!(d.state(), SyncState::Sequential);
        let before = d.status();

        let short = (before.expected_interval_us * 0.1) as u64;
        let event = d.on_tooth(last + 1.max(short));
        assert!(matches!(event, DecodeEvent::None), "{event:?}");
        let after = d.status();
        assert_eq!(after.state, before.state);
        assert_eq!(after.error_count, before.error_count);
        assert_eq!(after.crank_angle_deg, before.crank_angle_deg);

        // The next real gap edge is still accepted and the engine stays synced.
        // generate() ends on the 340-degree tooth; the next 0-degree edge is
        // one missing-tooth gap (20 degrees) later, not one normal 10-degree
        // tooth spacing.
        let offset = last + (20.0 * us_per_deg) as u64 - 1000;
        feed(&mut d, &generate(rpm, 2, true), offset);
        assert_eq!(d.state(), SyncState::Sequential);
    }

    #[test]
    fn cam_outside_window_drops_sequential_but_recovers() {
        let rpm = 1000.0;
        let us_per_deg = 60_000_000.0 / (rpm * 360.0);
        let mut d = decoder();
        d.set_cam_window(Some((320.0, 340.0))).unwrap();
        let last = feed(&mut d, &generate(rpm, 2, true), 0);
        assert_eq!(d.state(), SyncState::Sequential);

        // Just past the configured 320..=340 window.
        d.on_cam(last + 1);
        let s = d.status();
        assert!(s.cam_phase_rejected, "{s:?}");
        assert_eq!(s.state, SyncState::CrankSynced, "{s:?}");
        assert!(s.cycle_angle_deg.is_none(), "{s:?}");

        // generate() ends on the 340-degree tooth; the next 0-degree edge is
        // one missing-tooth gap (20 degrees) later, not one normal 10-degree
        // tooth spacing.
        let offset = last + (20.0 * us_per_deg) as u64 - 1000;
        feed(&mut d, &generate(rpm, 2, true), offset);
        assert_eq!(d.state(), SyncState::Sequential);
    }

    #[test]
    fn restart_uses_exactly_one_baseline_with_or_without_tick() {
        for explicit_tick in [false, true] {
            let mut d = decoder();
            let last = feed(&mut d, &generate(1000.0, 2, false), 0);
            let restart = last + 300_000;
            if explicit_tick {
                assert_eq!(d.tick(restart), DecodeEvent::SyncLost(DecodeError::Timeout));
            }
            d.on_tooth(restart);
            assert_eq!(d.state(), SyncState::Unsynced);
            d.on_tooth(restart + 1667);
            assert_eq!(d.expected_interval_us, 1667.0);
            assert_eq!(d.tooth_index, 1);
        }
    }

    #[test]
    fn loss_bounds_reject_impossible_values() {
        let mut d = decoder();
        assert!(d.set_loss_bounds(1, 6).is_err());
        assert!(d.set_loss_bounds(4, 1).is_err());
        assert!(d.set_loss_bounds(6, 3).is_ok());
    }

    /// Regression: capture starting on the tooth before the gap makes the
    /// first interval the 2x gap. Under cranking acceleration the next real
    /// tooth is then slightly under 50% of that unvalidated reference. If
    /// short-edge rejection applies before sync, every other real tooth is
    /// discarded, the gap is hidden and the decoder never synchronises.
    #[test]
    fn short_edge_filter_cannot_half_rate_lock_before_sync() {
        for accel in [0.0f64, 0.002, 0.005, 0.02] {
            let mut d = decoder();
            let mut t = 1000.0f64;
            let mut period = 5000.0f64; // us per 10 degrees
            let mut tooth = 33i32; // first captured edge is tooth 34
            let mut gaps = Vec::new();
            for _ in 0..140 {
                tooth = (tooth + 1) % 36;
                t += period;
                period = (period * (1.0 - accel)).max(300.0);
                if tooth == 35 {
                    continue;
                }
                if let DecodeEvent::Synchronised { .. } = d.on_tooth(t as u64) {
                    gaps.push(tooth);
                }
            }
            assert!(
                gaps.len() >= 2,
                "accel={accel}: gaps={gaps:?} {:?}",
                d.status()
            );
            assert!(gaps.iter().all(|&g| g == 0), "accel={accel}: gaps={gaps:?}");
            assert_eq!(d.status().error_count, 0, "accel={accel}");
        }
    }
}
