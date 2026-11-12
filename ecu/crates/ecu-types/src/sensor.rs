use alloc::string::String;

use serde::{Deserialize, Serialize};

/// Quality of a sensor reading. A numeric value with any status other than
/// `Ok` is retained for diagnostics but must not drive fuel or spark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorStatus {
    Ok,
    Stale,
    NotReady,
    Saturated,
    Invalid,
    Fault,
}

impl SensorStatus {
    pub fn is_usable(self) -> bool {
        matches!(self, SensorStatus::Ok)
    }

    pub fn is_ready_but_untrusted(self) -> bool {
        matches!(self, SensorStatus::NotReady | SensorStatus::Stale)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SensorSample {
    pub value: f32,
    pub status: SensorStatus,
    pub timestamp_us: u64,
    /// Age at the time the sample was consumed, filled by the consumer.
    pub age_ms: u32,
}

impl SensorSample {
    pub fn valid(value: f32, timestamp_us: u64) -> Self {
        Self {
            value,
            status: SensorStatus::Ok,
            timestamp_us,
            age_ms: 0,
        }
    }

    pub fn invalid(timestamp_us: u64, status: SensorStatus) -> Self {
        Self {
            value: f32::NAN,
            status,
            timestamp_us,
            age_ms: 0,
        }
    }

    /// Trust requires both a good status, a finite value and a bounded age.
    pub fn usable(&self, max_age_ms: u32) -> bool {
        self.status.is_usable() && self.value.is_finite() && self.age_ms <= max_age_ms
    }

    pub fn value_or(&self, fallback: f32) -> f32 {
        if self.value.is_finite() {
            self.value
        } else {
            fallback
        }
    }
}

impl Default for SensorSample {
    fn default() -> Self {
        Self::invalid(0, SensorStatus::Invalid)
    }
}

/// Wideband controller state is separate from the lambda number so a plausible
/// but not-yet-ready reading cannot be mistaken for a measurement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WidebandState {
    pub ready: bool,
    pub heater_on: bool,
    pub pumping_current_ma: f32,
    pub sensor_temperature_c: f32,
    pub status: SensorStatus,
    /// First-order response time reported/assumed for the controller.
    pub response_tau_ms: u16,
    /// Exhaust transport delay, used for offline time alignment.
    pub transport_delay_ms: u16,
    pub bank: u8,
    pub controller_id: String,
}

impl WidebandState {
    pub fn not_ready(bank: u8) -> Self {
        Self {
            ready: false,
            heater_on: false,
            pumping_current_ma: 0.0,
            sensor_temperature_c: 20.0,
            status: SensorStatus::NotReady,
            response_tau_ms: 80,
            transport_delay_ms: 60,
            bank,
            controller_id: "virtual-lsu49".into(),
        }
    }

    pub fn healthy(bank: u8) -> Self {
        Self {
            ready: true,
            heater_on: true,
            pumping_current_ma: 0.0,
            sensor_temperature_c: 780.0,
            status: SensorStatus::Ok,
            response_tau_ms: 80,
            transport_delay_ms: 60,
            bank,
            controller_id: "virtual-lsu49".into(),
        }
    }
}
