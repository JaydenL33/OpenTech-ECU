//! Ideal conditioned input edges, separate from decoding and output commands.
//! No electrical waveform, capture latency or stochastic trigger jitter model.

use ecu_trigger::{MissingToothConfig, TriggerConfigError};
use serde::{Deserialize, Serialize};

pub const DEFAULT_CAM_PHASE_DEG: f32 = 690.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerInput {
    Crank,
    Cam,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TriggerEdge {
    pub time_us: u64,
    /// Plant cycle angle, including the configured tooth-zero offset.
    pub angle_deg: f32,
    pub input: TriggerInput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveformError {
    Configuration(TriggerConfigError),
    InvalidCamPhase,
    InvalidStep,
}

#[derive(Clone, Debug)]
pub struct TriggerWaveform {
    config: MissingToothConfig,
    cam_phase_deg: f32,
}

impl TriggerWaveform {
    /// Cam phase is measured from tooth zero of the first revolution in a
    /// 720-degree cycle. It is independent of crank tooth positions and remains
    /// present when `cam_required` is false (that flag is decoder policy).
    pub fn new(config: MissingToothConfig, cam_phase_deg: f32) -> Result<Self, WaveformError> {
        config.validate().map_err(WaveformError::Configuration)?;
        if !cam_phase_deg.is_finite() || !(0.0..720.0).contains(&cam_phase_deg) {
            return Err(WaveformError::InvalidCamPhase);
        }
        Ok(Self {
            config,
            cam_phase_deg,
        })
    }

    /// Chronological edges in (start, start + advance], at constant speed for
    /// this step. At most one cycle per call; zero advance produces no edges.
    /// Crossing times truncate fractional microseconds, preserving the existing
    /// simulator's input capture convention. Coincident crank precedes cam.
    pub fn edges(
        &self,
        start_angle_deg: f32,
        advance_deg: f32,
        start_us: u64,
        duration_us: u64,
    ) -> Result<Vec<TriggerEdge>, WaveformError> {
        if !start_angle_deg.is_finite()
            || !(0.0..720.0).contains(&start_angle_deg)
            || !advance_deg.is_finite()
            || !(0.0..=720.0).contains(&advance_deg)
            || duration_us == 0
            || start_us.checked_add(duration_us).is_none()
        {
            return Err(WaveformError::InvalidStep);
        }
        let mut crossings = Vec::new();
        let mut push = |angle: f32, input| {
            let angle_deg = angle.rem_euclid(720.0);
            let distance = (angle_deg - start_angle_deg).rem_euclid(720.0);
            // A full cycle includes the starting edge once, at its end.
            let distance = if distance == 0.0 { 720.0 } else { distance };
            if distance <= advance_deg {
                crossings.push((
                    distance,
                    TriggerEdge {
                        time_us: start_us
                            + (((distance / advance_deg) * duration_us as f32) as u64)
                                .min(duration_us),
                        angle_deg,
                        input,
                    },
                ));
            }
        };
        for rev in 0..2 {
            // All consecutive missing positions are absent, including the last
            // position before tooth zero. No synthetic crank at the cam edge.
            for tooth in 0..self.config.teeth - self.config.missing {
                push(
                    rev as f32 * 360.0
                        + tooth as f32 * self.config.tooth_spacing_deg()
                        + self.config.sync_offset_deg,
                    TriggerInput::Crank,
                );
            }
        }
        push(
            self.cam_phase_deg + self.config.sync_offset_deg,
            TriggerInput::Cam,
        );
        // Stable sort preserves crank-before-cam for exactly coincident edges.
        crossings.sort_by(|a, b| a.0.total_cmp(&b.0));
        Ok(crossings.into_iter().map(|(_, edge)| edge).collect())
    }
}
