//! Lambda measurement, feedback and learning.
//!
//! Three deliberately separate workflows share one gate implementation:
//! - [`closed_loop`]: bounded short-term trim (STFT) for live target tracking;
//! - [`long_term`]: cell/bank-aware learned trim (LTFT), reviewable before merge;
//! - [`mixture_map`]: offline analysis of a log into bounded VE proposals;
//! - [`log_import`]: external CSV datalogs -> analyser input, with explicit assumptions.
//!
//! None of these modules can affect ignition timing. Lambda/AFR data is fuel
//! evidence only.

#![forbid(unsafe_code)]

pub mod closed_loop;
pub mod gate;
pub mod log_import;
pub mod long_term;
pub mod mixture_map;

pub use closed_loop::{ClosedLoop, ClosedLoopConfig, ClosedLoopOutput};
pub use gate::{evaluate, GateConfig, GateDecision, GateInputs, GateReason};
pub use log_import::{
    import_csv, ImportError, ImportOptions, ImportReport, ImportedLog, VeIdentityCheck,
};
pub use long_term::{BankTrims, LongTermTrimTable, LtftConfig, LtftUpdate};
pub use mixture_map::{
    analyze, analyze_log, apply_proposal, ApplyError, ApplyResult, CellStatus, MixtureCellReport,
    MixtureMapError, MixtureMapOptions, MixtureMapReport, ProposedVeCell, VeProposal,
};

/// Hard ceiling on any lambda-derived fuel correction in v0.1. Changes beyond
/// this require a fresh review.
pub const MAX_AUTOMATIC_FUEL_TRIM: f32 = 0.25;
