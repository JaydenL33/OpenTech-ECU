//! Headless OpenTech-ECU virtual engine CLI.
//!
//! Examples:
//!   cargo run -p ecu-sim-cli -- scenarios
//!   cargo run -p ecu-sim-cli -- run --scenario mixture_sweep --out runs/sweep
//!   cargo run -p ecu-sim-cli -- analyze --log runs/sweep/log.json --cal runs/sweep/calibration.json --out runs/sweep/report.json
//!   cargo run -p ecu-sim-cli -- apply --cal ... --report ... --out ... --approve

#![forbid(unsafe_code)]

use std::fs::{self, File};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use ecu_calibration::preflight::preflight;
use ecu_calibration::wizard::{generate_starter, WizardRequest};
use ecu_calibration::Calibration;
use ecu_lambda::mixture_map::{analyze_log, apply_proposal, MixtureMapOptions, MixtureMapReport};
use ecu_simulator::scenario::{builtin_scenarios, by_id};
use ecu_simulator::SimEngine;

#[derive(Parser, Debug)]
#[command(
    name = "opentech-ecu-sim",
    about = "Deterministic OpenTech-ECU virtual ECU, Mixture Map and calibration CLI",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// List built-in deterministic scenarios.
    Scenarios,
    /// Run a scenario and write a log, CSV and calibration snapshot.
    Run {
        #[arg(long, default_value = "idle_warmup")]
        scenario: String,
        #[arg(long, default_value = "runs/latest")]
        out: PathBuf,
        /// Optional calibration JSON. A starter simulator calibration is created
        /// when omitted.
        #[arg(long)]
        cal: Option<PathBuf>,
        /// Override the deterministic scenario seed (decimal u64).
        #[arg(long)]
        seed: Option<u64>,
        /// Run twice and compare serialized telemetry, output and input traces.
        #[arg(long)]
        check_determinism: bool,
    },
    /// Analyze a log with the offline Mixture Map. Emits a proposal; never applies.
    Analyze {
        #[arg(long)]
        log: PathBuf,
        #[arg(long)]
        cal: PathBuf,
        #[arg(long, default_value = "runs/mixture-report.json")]
        out: PathBuf,
        #[arg(long, default_value_t = 12)]
        min_samples: u32,
        #[arg(long, default_value_t = 0.05)]
        max_cell_correction: f32,
        /// Explicitly allow high-load/WOT cells. Still requires human approval.
        #[arg(long)]
        allow_wot: bool,
    },
    /// Apply a reviewed proposal to a calibration and produce a new sealed revision.
    Apply {
        #[arg(long)]
        cal: PathBuf,
        #[arg(long)]
        report: PathBuf,
        #[arg(long)]
        out: PathBuf,
        /// Restrict application to cells formatted row,col separated by ';'.
        #[arg(long)]
        cells: Option<String>,
        /// Required acknowledgements: the proposal is a starting estimate, not an
        /// engine-safe tune, and no hardware outputs exist in this build.
        #[arg(long)]
        approve: bool,
    },
    /// Verify calibration schema, checksum and preflight status.
    Verify {
        #[arg(long)]
        cal: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Scenarios => list_scenarios(),
        Command::Run {
            scenario,
            out,
            cal,
            seed,
            check_determinism,
        } => run_scenario(&scenario, &out, cal.as_deref(), seed, check_determinism),
        Command::Analyze {
            log,
            cal,
            out,
            min_samples,
            max_cell_correction,
            allow_wot,
        } => analyze(
            &log,
            &cal,
            &out,
            min_samples,
            max_cell_correction,
            allow_wot,
        ),
        Command::Apply {
            cal,
            report,
            out,
            cells,
            approve,
        } => apply(&cal, &report, &out, cells.as_deref(), approve),
        Command::Verify { cal } => verify(&cal),
    }
}

fn list_scenarios() -> Result<()> {
    for s in builtin_scenarios() {
        println!(
            "{:<18} {:>5.1}s  {}\n                  {}",
            s.id,
            s.duration_seconds(),
            s.name,
            s.description
        );
    }
    Ok(())
}

fn load_cal(path: &Path) -> Result<Calibration> {
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let cal: Calibration =
        serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))?;
    if !cal.verify() {
        bail!(
            "calibration checksum verification failed for {}",
            path.display()
        );
    }
    let issues = cal.validate();
    if let Some(e) = issues.iter().find(|i| i.is_error()) {
        bail!("calibration validation failed: {}: {}", e.code, e.message);
    }
    Ok(cal)
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(path).with_context(|| format!("creating {}", path.display()))?;
    serde_json::to_writer_pretty(file, value)?;
    Ok(())
}

fn run_scenario(
    scenario_id: &str,
    out: &Path,
    cal_path: Option<&Path>,
    seed: Option<u64>,
    check_determinism: bool,
) -> Result<()> {
    let mut scenario =
        by_id(scenario_id).with_context(|| format!("unknown scenario '{scenario_id}'"))?;
    if let Some(seed) = seed {
        scenario.seed = seed;
    }
    let cal = match cal_path {
        Some(p) => load_cal(p)?,
        None => generate_starter(&WizardRequest::default_inline4(), 1)
            .map_err(|e| anyhow::anyhow!("{e}"))?,
    };
    let report = preflight(&cal);
    if !report.virtual_run_allowed {
        bail!("preflight blocks virtual run: {}", report.summary);
    }
    fs::create_dir_all(out)?;
    if cal_path.is_none() {
        write_json(&out.join("calibration.json"), &cal)?;
    }
    let mut engine = SimEngine::new(cal, scenario.clone()).map_err(|e| anyhow::anyhow!("{e}"))?;
    engine.run_to_end();
    let log = engine.log_file();
    let output_trace = engine.simulated_output_trace();
    let trigger_trace = engine.simulated_trigger_trace();
    write_json(&out.join("log.json"), &log)?;
    write_json(&out.join("outputs.json"), &output_trace)?;
    write_json(&out.join("triggers.json"), &trigger_trace)?;
    fs::write(out.join("log.csv"), log.to_csv())?;
    let summary = session_summary(&log.frames);
    write_json(&out.join("summary.json"), &summary)?;

    if check_determinism {
        let mut engine2 = SimEngine::new(engine.calibration.clone(), scenario)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        engine2.run_to_end();
        if serde_json::to_vec(&log)? != serde_json::to_vec(&engine2.log_file())?
            || serde_json::to_vec(&output_trace)?
                != serde_json::to_vec(&engine2.simulated_output_trace())?
            || serde_json::to_vec(&trigger_trace)?
                != serde_json::to_vec(&engine2.simulated_trigger_trace())?
        {
            bail!("determinism check FAILED: repeated telemetry, output or input trace differs");
        }
        println!(
            "determinism: PASS ({} frames, output and input traces byte-identical)",
            log.frames.len()
        );
    }

    println!("scenario: {}", scenario_id);
    println!("frames:   {}", summary.samples);
    println!("duration: {:.1}s", summary.duration_ms as f32 / 1000.0);
    println!(
        "rpm mean/max: {:.0} / {:.0}",
        summary.mean_rpm, summary.max_rpm
    );
    println!("coolant max: {:.1} C", summary.max_coolant_c);
    println!(
        "lambda mean: {} ({} samples)",
        summary
            .mean_lambda
            .map(|v| format!("{v:.3}"))
            .unwrap_or_else(|| "n/a".into()),
        summary.lambda_samples
    );
    println!("faults:   {:?}", summary.fault_codes);
    println!(
        "outputs:  {} emitted, {} retained, {} dropped (simulation only)",
        output_trace.events_emitted,
        output_trace.outputs.len(),
        output_trace.events_dropped
    );
    println!("wrote:    {}", out.display());
    println!(
        "inputs:   {} emitted, {} retained, {} dropped (simulation only)",
        trigger_trace.events_emitted,
        trigger_trace.inputs.len(),
        trigger_trace.events_dropped
    );
    Ok(())
}

fn analyze(
    log_path: &Path,
    cal_path: &Path,
    out: &Path,
    min_samples: u32,
    max_cell_correction: f32,
    allow_wot: bool,
) -> Result<()> {
    let bytes = fs::read(log_path).with_context(|| format!("reading {}", log_path.display()))?;
    let log: ecu_types::LogFile = serde_json::from_slice(&bytes).context("parsing log JSON")?;
    let cal = load_cal(cal_path)?;
    let opts = MixtureMapOptions {
        min_samples_per_cell: min_samples,
        max_cell_correction,
        allow_wot,
        ..MixtureMapOptions::default()
    };
    let report = analyze_log(&cal, &log, &opts).map_err(|e| anyhow::anyhow!("{e}"))?;
    write_json(out, &report)?;
    println!("{}", report_headline(&report));
    if let Some(p) = &report.proposal {
        println!(
            "proposal: {} ({} cells, requires approval)",
            p.proposal_id,
            p.cells.len()
        );
        for c in p.cells.iter().take(10) {
            println!(
                "  {:>5} rpm {:>4} kPa  VE {:5.2} -> {:5.2}  ({:+.1}%, n={}, conf={:.2})",
                c.rpm,
                c.map_kpa,
                c.current_ve,
                c.proposed_ve,
                c.correction_fraction * 100.0,
                c.samples,
                c.confidence
            );
        }
    } else {
        println!("No proposal. This is an abstention, not an implicit safe result.");
    }
    println!("ignition untouched: {}", report.ignition_untouched);
    println!("wrote: {}", out.display());
    Ok(())
}

fn report_headline(report: &MixtureMapReport) -> String {
    let proposals = report.proposal.as_ref().map(|p| p.cells.len()).unwrap_or(0);
    format!(
        "samples used/rejected: {}/{}  cells proposed: {}  hard faults: {:?}\n  gates: {:?}",
        report.samples_used,
        report.samples_rejected,
        proposals,
        report.hard_faults,
        report.gate_reason_counts
    )
}

fn apply(
    cal_path: &Path,
    report_path: &Path,
    out: &Path,
    cells: Option<&str>,
    approve: bool,
) -> Result<()> {
    if !approve {
        bail!(
            "refusing to apply without --approve. This applies a proposal to a virtual \
             calibration only; it is not engine-safe validation."
        );
    }
    let cal = load_cal(cal_path)?;
    let bytes = fs::read(report_path).context("reading report")?;
    let report: MixtureMapReport = serde_json::from_slice(&bytes).context("parsing report")?;
    let proposal = report
        .proposal
        .as_ref()
        .context("report contains no proposal")?;
    let selection: Option<Vec<(usize, usize)>> = match cells {
        None => None,
        Some(s) => {
            let mut v = Vec::new();
            for part in s.split(';').filter(|p| !p.trim().is_empty()) {
                let (r, c) = part
                    .split_once(',')
                    .with_context(|| format!("bad cell '{part}', expected row,col"))?;
                v.push((r.trim().parse()?, c.trim().parse()?));
            }
            Some(v)
        }
    };
    let result = apply_proposal(
        &cal,
        proposal,
        selection.as_deref(),
        ecu_calibration::now_unix_ms(),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    write_json(out, &result.calibration)?;
    println!(
        "applied {} cells; revision {} -> {}; checksum {}",
        result.applied_cells,
        cal.revision,
        result.calibration.revision,
        &result.calibration.checksum[..16]
    );
    println!(
        "diff: {} cells, max |delta| {:.2} VE points",
        result.diff.changes.len(),
        result.diff.max_abs_delta
    );
    println!(
        "undo snapshot is the input calibration: {}",
        cal_path.display()
    );
    println!("wrote: {}", out.display());
    Ok(())
}

fn verify(cal_path: &Path) -> Result<()> {
    let cal = load_cal(cal_path)?;
    let report = preflight(&cal);
    println!("id:       {}", cal.id);
    println!("revision: {}", cal.revision);
    println!("checksum: {} (verified)", cal.checksum);
    println!("ignition: {:?}", cal.ignition_policy);
    println!("virtual run allowed: {}", report.virtual_run_allowed);
    println!(
        "physical export allowed: {}",
        report.physical_export_allowed
    );
    for c in report
        .checks
        .iter()
        .filter(|c| c.status != ecu_calibration::CheckStatus::Pass)
    {
        println!("  [{:?}] {}: {}", c.status, c.title, c.detail);
    }
    Ok(())
}

fn session_summary(frames: &[ecu_types::TelemetryFrame]) -> ecu_diagnostics::SessionSummary {
    ecu_diagnostics::session_summary(frames)
}
