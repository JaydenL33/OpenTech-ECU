//! Foundational types for the OpenTech ECU.
//!
//! This crate deliberately contains no control logic. It defines unit-tagged
//! quantities, engine metadata, sensor-quality annotations and the telemetry
//! schema shared by the simulator, protocol and tuning application.
//!
//! Safety rule: a value's type never implies that it is trustworthy. Trust is
//! carried separately in [`SensorStatus`] and [`SensorSample`]. Control code
//! must check trust before using a value.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod engine;
pub mod provenance;
pub mod sensor;
pub mod telemetry;
pub mod units;

pub use engine::*;
pub use provenance::*;
pub use sensor::*;
pub use telemetry::*;
pub use units::*;

/// Wire/schema version for calibration and telemetry payloads.
pub const SCHEMA_VERSION: u16 = 1;

/// Software identity included in logs so evidence can be tied to a build.
pub const SOFTWARE_VERSION: &str = env!("CARGO_PKG_VERSION");
