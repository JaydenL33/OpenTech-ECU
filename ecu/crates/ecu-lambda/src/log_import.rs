//! Import of external datalogs (CSV) for offline lambda analysis.
//!
//! Real logs rarely carry every field the analyser's gate wants. This module
//! never fills a gap silently: each field that is not in the file is either
//! (a) refused, (b) taken from an *explicit* import option the human must set,
//! or (c) defaulted conservatively — and every such decision is returned in
//! [`ImportReport::assumptions`] so it can be shown at review time and kept as
//! provenance. Nothing here touches a calibration or an ECU.
//!
//! Hard refusals (the log cannot be used rather than "used with a warning"):
//! - no coolant column (the learning gate needs it);
//! - no closed-loop flag unless `assume_open_loop` is set, and closed-loop
//!   rows without trim columns (the trim would be silently taken as zero);
//! - time that goes backwards;
//! - a calibration whose VE does not match the VE the ECU logged, or, when the
//!   log has no VE column, no explicit `confirm_recorded_with_loaded_calibration`.
//!
//! The last point matters most: a VE correction is a *ratio against the table
//! that was running*. Applying it to a different table is wrong by the
//! difference between the tables.

use std::collections::BTreeMap;
use std::string::{String, ToString};
use std::vec::Vec;

use ecu_calibration::Calibration;
use ecu_types::{
    EngineState, LogFile, LogMetadata, ProtectionState, SensorStatus, SyncState, TelemetryFrame,
    WidebandState,
};
use serde::{Deserialize, Serialize};

use crate::mixture_map::VE_TABLE_KEY;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImportOptions {
    /// Human-readable origin (file name), recorded as the source scenario.
    #[serde(default)]
    pub source_name: String,
    /// Stoichiometric AFR used to convert an AFR column to lambda. 14.7 is
    /// gasoline; it is wrong for ethanol blends. Used only if the log has AFR
    /// columns rather than lambda columns.
    #[serde(default = "default_stoich")]
    pub stoich_afr: f32,
    /// Exhaust transport delay applied to every row unless the log has an
    /// `exhaust_delay_ms` column. Engine/sensor-position specific; the default
    /// is the repository's generic frame default, not a measured value.
    #[serde(default = "default_delay")]
    pub exhaust_delay_ms: u16,
    #[serde(default = "default_bank")]
    pub bank: u8,
    /// Required when the log has no closed-loop flag: asserts the engine was
    /// in open loop (no lambda trim) for the whole file.
    #[serde(default)]
    pub assume_open_loop: bool,
    /// Required when the log has no sync column. Without it nothing can pass
    /// the sequential-sync gate.
    #[serde(default)]
    pub assume_sequential_sync: bool,
    /// Required when the log has no fuel/oil pressure columns. Substitutes
    /// healthy-pressure values so the pressure gates do not reject every row;
    /// the substitution is listed as an assumption.
    #[serde(default)]
    pub assume_pressures_ok: bool,
    /// Required when the log has no `ve` column: the human asserts the log was
    /// recorded with the VE table currently loaded.
    #[serde(default)]
    pub confirm_recorded_with_loaded_calibration: bool,
    /// Largest tolerated 75th-percentile |logged VE / loaded-table VE - 1|.
    /// A percentile rather than the median so that a table edited in only the
    /// cells the log visits is still caught (the median over all rows missed a
    /// 5 % edit by a hair in the round-7 HTTP test). Must stay above the
    /// rounding of integer-VE logs (about 1 % at VE 50). Unvalidated guess.
    #[serde(default = "default_ve_mismatch")]
    pub max_ve_mismatch: f32,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
}

fn default_stoich() -> f32 {
    14.7
}
fn default_delay() -> u16 {
    60
}
fn default_bank() -> u8 {
    1
}
fn default_ve_mismatch() -> f32 {
    0.02
}
fn default_max_rows() -> usize {
    500_000
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            source_name: String::new(),
            stoich_afr: default_stoich(),
            exhaust_delay_ms: default_delay(),
            bank: default_bank(),
            assume_open_loop: false,
            assume_sequential_sync: false,
            assume_pressures_ok: false,
            confirm_recorded_with_loaded_calibration: false,
            max_ve_mismatch: default_ve_mismatch(),
            max_rows: default_max_rows(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VeIdentityCheck {
    pub compared_rows: u32,
    pub median_abs_rel_diff: f32,
    pub p75_abs_rel_diff: f32,
    pub max_abs_rel_diff: f32,
    pub tolerance: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImportReport {
    pub source_name: String,
    pub delimiter: String,
    pub rows_read: usize,
    pub rows_imported: usize,
    pub rows_skipped: usize,
    /// First few skipped lines (1-based file line numbers) with the reason.
    pub skipped_examples: Vec<String>,
    pub median_sample_interval_ms: f32,
    /// canonical field -> source column header
    pub columns_mapped: BTreeMap<String, String>,
    /// Source headers that matched nothing and were ignored.
    pub columns_ignored: Vec<String>,
    /// Every non-trivial decision taken on the user's behalf. Never empty
    /// information: if nothing was assumed this is an empty list.
    pub assumptions: Vec<String>,
    pub warnings: Vec<String>,
    /// `None` when the log carried no VE column to compare.
    pub ve_identity: Option<VeIdentityCheck>,
    pub content_sha256: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportedLog {
    pub log: LogFile,
    pub report: ImportReport,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImportError {
    Empty,
    NoHeader,
    MissingColumn(&'static str),
    OpenLoopNotStated,
    ClosedLoopWithoutTrim,
    NoUsableRows(String),
    TimeNotMonotonic { line: usize },
    TooManyRows(usize),
    InvalidOption(String),
    CalibrationIdentityUnconfirmed,
    CalibrationMismatch(VeIdentityCheck),
    MissingTable(String),
}

impl core::fmt::Display for ImportError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => write!(f, "log is empty"),
            Self::NoHeader => write!(f, "no header row found"),
            Self::MissingColumn(c) => write!(f, "required column '{c}' not found in the log"),
            Self::OpenLoopNotStated => write!(
                f,
                "the log has no closed-loop flag; set assume_open_loop only if the engine was in \
                 open loop (no lambda trim active) for the whole file"
            ),
            Self::ClosedLoopWithoutTrim => write!(
                f,
                "the log marks rows as closed loop but has no stft/ltft columns; the correction \
                 cannot be separated from the running trim"
            ),
            Self::NoUsableRows(why) => write!(f, "no usable rows: {why}"),
            Self::TimeNotMonotonic { line } => {
                write!(
                    f,
                    "time goes backwards at line {line}; refusing to guess an order"
                )
            }
            Self::TooManyRows(n) => write!(f, "more than {n} rows"),
            Self::InvalidOption(s) => write!(f, "invalid import option: {s}"),
            Self::CalibrationIdentityUnconfirmed => write!(
                f,
                "the log has no VE column, so it cannot be checked against the loaded \
                 calibration; confirm explicitly that it was recorded with this exact VE table"
            ),
            Self::CalibrationMismatch(c) => write!(
                f,
                "logged VE differs from the loaded VE table (p75 |diff| {:.1} %, median {:.1} %, \
                 max {:.1} % over {} rows, tolerance {:.1} %): the log was recorded with a \
                 different table, so a correction computed against the loaded table would be wrong",
                c.p75_abs_rel_diff * 100.0,
                c.median_abs_rel_diff * 100.0,
                c.max_abs_rel_diff * 100.0,
                c.compared_rows,
                c.tolerance * 100.0
            ),
            Self::MissingTable(t) => write!(f, "calibration has no '{t}' table"),
        }
    }
}

impl std::error::Error for ImportError {}

/// Canonical field names and the (normalised) header aliases that map to them.
/// Normalisation: lowercase, keep only ASCII alphanumerics.
const ALIASES: &[(&str, &[&str])] = &[
    ("time_ms", &["timems", "timestampms", "millis"]),
    ("time_s", &["time", "times", "seconds", "sec", "secs"]),
    ("rpm", &["rpm", "enginerpm", "enginespeed"]),
    ("map_kpa", &["mapkpa", "map"]),
    ("tps_pct", &["tpspct", "tps", "throttle", "throttlepos"]),
    ("clt_c", &["cltc", "clt", "coolant", "coolanttemp", "ect"]),
    ("iat_c", &["iatc", "iat", "mat"]),
    (
        "battery_v",
        &["batteryv", "battery", "batt", "vbat", "voltage"],
    ),
    ("lambda", &["lambda", "lambdameasured", "wblambda"]),
    ("afr", &["afr", "afrmeasured", "wbafr"]),
    ("lambda_target", &["lambdatarget", "targetlambda"]),
    ("afr_target", &["afrtarget", "targetafr", "afrtgt"]),
    ("stft", &["stft"]),
    ("ltft", &["ltft"]),
    (
        "closed_loop",
        &["closedloop", "closedloopactive", "egoactive"],
    ),
    ("fuel_cut", &["fuelcut", "dfco"]),
    ("spark_cut", &["sparkcut"]),
    ("sync", &["sync", "syncstate"]),
    ("fuel_kpa", &["fuelkpa", "fuelpressurekpa"]),
    ("oil_kpa", &["oilkpa", "oilpressurekpa"]),
    (
        "accel_enrich_pct",
        &["accelenrichpct", "accelenrich", "aepct"],
    ),
    ("misfire", &["misfire"]),
    ("faults", &["faults", "activefaults"]),
    ("ve", &["ve", "ve1", "vetable"]),
    ("exhaust_delay_ms", &["exhaustdelayms"]),
    ("lambda_status", &["lambdastatus", "wbstatus"]),
];

fn normalise(h: &str) -> String {
    h.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn split_line(line: &str, delim: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    cur.push('"');
                    chars.next();
                } else {
                    quoted = false;
                }
            } else {
                cur.push(c);
            }
        } else if c == '"' {
            quoted = true;
        } else if c == delim {
            out.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(c);
        }
    }
    out.push(cur.trim().to_string());
    out
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "t" => Some(true),
        "0" | "false" | "no" | "off" | "f" | "" => Some(false),
        _ => None,
    }
}

fn parse_num(s: &str) -> Option<f32> {
    let v: f32 = s.trim().parse().ok()?;
    v.is_finite().then_some(v)
}

fn median(v: &mut [f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    let m = v.len() / 2;
    if v.len() % 2 == 1 {
        v[m]
    } else {
        (v[m - 1] + v[m]) / 2.0
    }
}

/// Parse an external CSV log into a [`LogFile`] suitable for
/// [`crate::analyze_log`], and report exactly what was assumed to do so.
pub fn import_csv(
    text: &str,
    cal: &Calibration,
    opts: &ImportOptions,
) -> Result<ImportedLog, ImportError> {
    if !(opts.stoich_afr.is_finite() && opts.stoich_afr > 5.0 && opts.stoich_afr < 25.0) {
        return Err(ImportError::InvalidOption(
            "stoich_afr must be in 5..25".into(),
        ));
    }
    if !(opts.max_ve_mismatch.is_finite()
        && opts.max_ve_mismatch > 0.0
        && opts.max_ve_mismatch < 0.5)
    {
        return Err(ImportError::InvalidOption(
            "max_ve_mismatch must be in (0, 0.5)".into(),
        ));
    }
    if text.trim().is_empty() {
        return Err(ImportError::Empty);
    }
    let ve_table = cal
        .table(VE_TABLE_KEY)
        .map_err(|_| ImportError::MissingTable(VE_TABLE_KEY.into()))?;
    let tgt_table = cal.table("target_lambda").ok();

    let mut lines = text
        .lines()
        .enumerate()
        .map(|(i, l)| (i + 1, l.trim_start_matches('\u{feff}')))
        .filter(|(_, l)| !l.trim().is_empty() && !l.trim_start().starts_with('#'));
    let (_, header_line) = lines.next().ok_or(ImportError::NoHeader)?;
    let delim = [',', '\t', ';']
        .into_iter()
        .max_by_key(|d| header_line.matches(*d).count())
        .filter(|d| header_line.contains(*d))
        .ok_or(ImportError::NoHeader)?;
    let headers = split_line(header_line, delim);

    // Map canonical -> column index. First match wins; a second column that
    // would map to the same field is reported as ignored.
    let mut col: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut mapped_headers: BTreeMap<String, String> = BTreeMap::new();
    let mut ignored = Vec::new();
    for (i, h) in headers.iter().enumerate() {
        let n = normalise(h);
        let hit = ALIASES
            .iter()
            .find(|(_, al)| al.contains(&n.as_str()))
            .map(|(k, _)| *k);
        match hit {
            Some(k) if !col.contains_key(k) => {
                col.insert(k, i);
                mapped_headers.insert(k.to_string(), h.clone());
            }
            _ => ignored.push(h.clone()),
        }
    }

    let mut assumptions: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    let time_is_ms = col.contains_key("time_ms");
    if !time_is_ms && !col.contains_key("time_s") {
        return Err(ImportError::MissingColumn("time_ms or time (seconds)"));
    }
    for (k, label) in [
        ("rpm", "rpm"),
        ("map_kpa", "map_kpa"),
        ("tps_pct", "tps_pct"),
        ("clt_c", "clt_c"),
    ] {
        if !col.contains_key(k) {
            return Err(ImportError::MissingColumn(label));
        }
    }
    if !col.contains_key("lambda") && !col.contains_key("afr") {
        return Err(ImportError::MissingColumn("lambda or afr"));
    }
    let lambda_from_afr = !col.contains_key("lambda");
    if lambda_from_afr {
        assumptions.push(format!(
            "Lambda computed from AFR using stoichiometric AFR {:.2} (option; wrong for ethanol blends)",
            opts.stoich_afr
        ));
    }
    if time_is_ms {
        assumptions.push("Time column interpreted as milliseconds".into());
    } else {
        assumptions.push("Time column interpreted as seconds".into());
    }
    assumptions.push("MAP interpreted as absolute kPa, TPS as percent, temperatures as °C".into());

    let has_loop_flag = col.contains_key("closed_loop");
    if !has_loop_flag {
        if !opts.assume_open_loop {
            return Err(ImportError::OpenLoopNotStated);
        }
        assumptions.push(
            "No closed-loop flag: all rows treated as open loop with zero trim (assume_open_loop set by user)"
                .into(),
        );
    }
    let has_trim = col.contains_key("stft") || col.contains_key("ltft");
    if has_trim {
        assumptions.push("stft/ltft interpreted as fractions (0.05 = +5 %), not percent".into());
    }

    if !col.contains_key("sync") {
        if !opts.assume_sequential_sync {
            warnings.push(
                "No sync column and assume_sequential_sync not set: the learning gate will reject every row (SYNC_NOT_SEQUENTIAL)."
                    .into(),
            );
        } else {
            assumptions
                .push("No sync column: sequential sync assumed for all rows (user option)".into());
        }
    }
    let has_pressures = col.contains_key("fuel_kpa") && col.contains_key("oil_kpa");
    if !has_pressures {
        if opts.assume_pressures_ok {
            assumptions.push(
                "Fuel/oil pressure missing: healthy values (300 / 300 kPa) substituted so those gates pass (user option); low pressure cannot be detected from this log"
                    .into(),
            );
        } else {
            warnings.push(
                "Fuel and/or oil pressure columns missing and assume_pressures_ok not set: the learning gate will reject those rows."
                    .into(),
            );
        }
    }
    if !col.contains_key("lambda_status") {
        assumptions.push(
            "No wideband status column: sensor treated as valid (Ok/ready) whenever the value is finite; heater/pump faults are invisible in this log"
                .into(),
        );
    }
    assumptions.push(
        "Wideband sample age taken as 0 ms (a logged sample is not known to be stale)".into(),
    );
    if !col.contains_key("exhaust_delay_ms") {
        assumptions.push(format!(
            "Exhaust transport delay fixed at {} ms for all rows (option; not measured)",
            opts.exhaust_delay_ms
        ));
    }
    assumptions.push(
        "Engine state derived from RPM: >=400 running, >0 cranking, else off; protection state taken as normal"
            .into(),
    );
    if !col.contains_key("fuel_cut") {
        warnings.push(
            "No fuel-cut column: DFCO/overrun relies on the analyser's TPS/RPM overrun check only."
                .into(),
        );
    }
    if !col.contains_key("misfire") && !col.contains_key("faults") {
        warnings
            .push("No misfire/fault columns: misfires are not detectable from this log.".into());
    }
    if !col.contains_key("accel_enrich_pct") {
        warnings.push(
            "No accel-enrichment column: transient enrichment is inferred from TPS/MAP/RPM rates only."
                .into(),
        );
    }
    let target_from_table = !col.contains_key("lambda_target") && !col.contains_key("afr_target");
    if target_from_table {
        if tgt_table.is_none() {
            return Err(ImportError::MissingColumn("lambda_target or afr_target"));
        }
        assumptions.push(
            "No lambda target column: target taken from the loaded calibration's target_lambda table at each row's MAP/RPM"
                .into(),
        );
    }
    let has_ve = col.contains_key("ve");

    // Row parse.
    let mut frames: Vec<TelemetryFrame> = Vec::new();
    let mut skipped = 0usize;
    let mut skipped_examples: Vec<String> = Vec::new();
    let mut rows_read = 0usize;
    let mut last_t_us: Option<u64> = None;
    let mut closed_rows = 0usize;
    let mut ve_diffs: Vec<f32> = Vec::new();
    let mut skip = |line_no: usize, why: &str, ex: &mut Vec<String>| {
        skipped += 1;
        if ex.len() < 5 {
            ex.push(format!("line {line_no}: {why}"));
        }
    };

    for (line_no, line) in lines {
        rows_read += 1;
        if rows_read > opts.max_rows {
            return Err(ImportError::TooManyRows(opts.max_rows));
        }
        let cells = split_line(line, delim);
        let get = |k: &str| -> Option<&str> {
            col.get(k).and_then(|&i| cells.get(i)).map(|s| s.as_str())
        };
        let num = |k: &str| -> Option<f32> { get(k).and_then(parse_num) };

        let Some(t_raw) = num(if time_is_ms { "time_ms" } else { "time_s" }) else {
            skip(
                line_no,
                "time is not a number (units/comment row?)",
                &mut skipped_examples,
            );
            continue;
        };
        let t_us_f = if time_is_ms {
            t_raw as f64 * 1000.0
        } else {
            t_raw as f64 * 1e6
        };
        if t_us_f < 0.0 {
            skip(line_no, "negative time", &mut skipped_examples);
            continue;
        }
        let t_us = t_us_f.round() as u64;
        let (Some(rpm), Some(map), Some(tps), Some(clt)) =
            (num("rpm"), num("map_kpa"), num("tps_pct"), num("clt_c"))
        else {
            skip(
                line_no,
                "rpm/map/tps/clt not all numeric",
                &mut skipped_examples,
            );
            continue;
        };
        let lambda = if lambda_from_afr {
            num("afr").map(|a| a / opts.stoich_afr)
        } else {
            num("lambda")
        };
        let Some(lambda) = lambda else {
            skip(line_no, "lambda/afr not numeric", &mut skipped_examples);
            continue;
        };
        if let Some(prev) = last_t_us {
            if t_us < prev {
                return Err(ImportError::TimeNotMonotonic { line: line_no });
            }
        }
        last_t_us = Some(t_us);

        let target = if let Some(v) = num("lambda_target") {
            v
        } else if let Some(a) = num("afr_target") {
            a / opts.stoich_afr
        } else if target_from_table {
            tgt_table
                .and_then(|t| t.interpolate(&[map, rpm]).ok())
                .unwrap_or(f32::NAN)
        } else {
            f32::NAN
        };

        let closed = if has_loop_flag {
            match get("closed_loop").and_then(parse_bool) {
                Some(b) => b,
                None => {
                    skip(
                        line_no,
                        "closed-loop flag unreadable",
                        &mut skipped_examples,
                    );
                    continue;
                }
            }
        } else {
            false
        };
        let stft = num("stft").unwrap_or(0.0);
        let ltft = num("ltft").unwrap_or(0.0);
        if closed {
            closed_rows += 1;
        }

        let lambda_ok = match get("lambda_status") {
            Some(s) => matches!(s.trim().to_ascii_lowercase().as_str(), "ok" | "1" | "true"),
            None => lambda.is_finite() && lambda > 0.0,
        };
        let sync = match get("sync") {
            Some(s) => {
                let l = s.trim().to_ascii_lowercase();
                if l.contains("seq") || l == "full" || l == "2" {
                    SyncState::Sequential
                } else if l.contains("crank") || l == "half" || l == "1" {
                    SyncState::CrankSynced
                } else if l.contains("lost") {
                    SyncState::Lost
                } else {
                    SyncState::Unsynced
                }
            }
            None if opts.assume_sequential_sync => SyncState::Sequential,
            None => SyncState::Unsynced,
        };
        let mut faults: Vec<String> = Vec::new();
        if get("misfire").and_then(parse_bool).unwrap_or(false) {
            faults.push("MISFIRE".into());
        }
        if let Some(f) = get("faults") {
            faults.extend(
                f.split('|')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string()),
            );
        }
        let ae = match get("accel_enrich_pct") {
            None => 0.0,
            // A present-but-unreadable value is "unknown", which the analyser
            // treats as active enrichment.
            Some(s) => parse_num(s).unwrap_or(f32::NAN),
        };
        let delay = num("exhaust_delay_ms")
            .map(|d| d.clamp(0.0, 2000.0) as u16)
            .unwrap_or(opts.exhaust_delay_ms);
        let ve_logged = num("ve");
        if let Some(v) = ve_logged {
            if rpm >= 500.0 && v > 0.0 {
                if let Ok(table_ve) = ve_table.interpolate(&[map, rpm]) {
                    if table_ve > 0.0 {
                        ve_diffs.push((v / table_ve - 1.0).abs());
                    }
                }
            }
        }

        let f = TelemetryFrame {
            seq: frames.len() as u64,
            timestamp_us: t_us,
            scenario_time_ms: t_us / 1000,
            rpm,
            map_kpa: map,
            tps_pct: tps,
            clt_c: clt,
            iat_c: num("iat_c").unwrap_or(25.0),
            battery_v: num("battery_v").unwrap_or(12.6),
            fuel_pressure_kpa: num("fuel_kpa").unwrap_or(if opts.assume_pressures_ok {
                300.0
            } else {
                0.0
            }),
            oil_pressure_kpa: num("oil_kpa").unwrap_or(if opts.assume_pressures_ok {
                300.0
            } else {
                0.0
            }),
            lambda_measured: lambda,
            lambda_target: target,
            lambda_status: if lambda_ok {
                SensorStatus::Ok
            } else {
                SensorStatus::Invalid
            },
            lambda_ready: lambda_ok,
            lambda_age_ms: 0,
            lambda_bank: opts.bank,
            exhaust_delay_ms: delay,
            wideband: WidebandState {
                ready: lambda_ok,
                status: if lambda_ok {
                    SensorStatus::Ok
                } else {
                    SensorStatus::Invalid
                },
                transport_delay_ms: delay,
                bank: opts.bank,
                controller_id: "external-csv".into(),
                ..WidebandState::not_ready(opts.bank)
            },
            ve: ve_logged.unwrap_or(0.0),
            stft,
            ltft,
            closed_loop: closed,
            sync,
            engine_state: if rpm >= 400.0 {
                EngineState::Running
            } else if rpm > 0.0 {
                EngineState::Cranking
            } else {
                EngineState::Off
            },
            protection: ProtectionState::Normal,
            fuel_cut: get("fuel_cut").and_then(parse_bool).unwrap_or(false),
            spark_cut: get("spark_cut").and_then(parse_bool).unwrap_or(false),
            active_faults: faults,
            accel_enrich_pct: ae,
            log_note: "external csv".into(),
            ..TelemetryFrame::default()
        };
        frames.push(f);
    }

    if closed_rows > 0 && !has_trim {
        return Err(ImportError::ClosedLoopWithoutTrim);
    }
    if frames.is_empty() {
        return Err(ImportError::NoUsableRows(format!(
            "{rows_read} data rows read, all skipped"
        )));
    }
    if skipped > 0 {
        warnings.push(format!(
            "{skipped} of {rows_read} data rows were skipped (see skipped_examples)."
        ));
    }

    let mut dts: Vec<f32> = frames
        .windows(2)
        .map(|w| (w[1].timestamp_us - w[0].timestamp_us) as f32 / 1000.0)
        .collect();
    let median_dt = median(&mut dts);
    if median_dt > 100.0 {
        warnings.push(format!(
            "Median sample interval is {median_dt:.0} ms: the latency-window and transient checks are much coarser than with a >=20 Hz log."
        ));
    }
    if median_dt <= 0.0 && frames.len() > 1 {
        warnings.push("Median sample interval is 0 ms (duplicate timestamps); rate checks will reject these rows.".into());
    }

    // Calibration identity.
    let ve_identity = if has_ve {
        if ve_diffs.is_empty() {
            return Err(ImportError::NoUsableRows(
                "VE column present but no row at >=500 rpm to compare".into(),
            ));
        }
        let max = ve_diffs.iter().copied().fold(0.0f32, f32::max);
        let n = ve_diffs.len() as u32;
        ve_diffs.sort_by(|a, b| a.total_cmp(b));
        let p75 = ve_diffs[((ve_diffs.len() - 1) * 3) / 4];
        let med = median(&mut ve_diffs);
        let chk = VeIdentityCheck {
            compared_rows: n,
            median_abs_rel_diff: med,
            p75_abs_rel_diff: p75,
            max_abs_rel_diff: max,
            tolerance: opts.max_ve_mismatch,
        };
        if p75 > opts.max_ve_mismatch {
            return Err(ImportError::CalibrationMismatch(chk));
        }
        assumptions.push(format!(
            "Logged VE matches the loaded VE table: p75 |diff| {:.2} % over {n} rows (tolerance {:.1} %)",
            p75 * 100.0,
            opts.max_ve_mismatch * 100.0
        ));
        Some(chk)
    } else {
        if !opts.confirm_recorded_with_loaded_calibration {
            return Err(ImportError::CalibrationIdentityUnconfirmed);
        }
        assumptions.push(
            "No VE column: the user confirmed the log was recorded with the currently loaded VE table; this is NOT verified"
                .into(),
        );
        None
    };

    let content_sha256 = ecu_calibration::hex_sha256(text.as_bytes());
    let name = if opts.source_name.trim().is_empty() {
        "external-csv".to_string()
    } else {
        format!("external-csv:{}", opts.source_name.trim())
    };
    let log = LogFile {
        metadata: LogMetadata {
            schema_version: ecu_types::SCHEMA_VERSION,
            software_version: "log_import".into(),
            scenario_id: "external_csv".into(),
            scenario_name: name.clone(),
            seed: 0,
            calibration_id: cal.id.clone(),
            calibration_revision: cal.revision,
            calibration_checksum: cal.checksum.clone(),
            started_unix_ms: 0,
            sample_hz: if median_dt > 0.0 {
                (1000.0 / median_dt).round().min(65535.0) as u16
            } else {
                0
            },
            notes: assumptions.clone(),
        },
        frames,
        events: Vec::new(),
    };
    let rows_imported = log.frames.len();
    Ok(ImportedLog {
        report: ImportReport {
            source_name: name,
            delimiter: if delim == '\t' {
                "\\t".into()
            } else {
                delim.to_string()
            },
            rows_read,
            rows_imported,
            rows_skipped: skipped,
            skipped_examples,
            median_sample_interval_ms: median_dt,
            columns_mapped: mapped_headers,
            columns_ignored: ignored,
            assumptions,
            warnings,
            ve_identity,
            content_sha256,
        },
        log,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{analyze_log, MixtureMapOptions};
    use ecu_calibration::wizard::{generate_starter, WizardRequest};

    fn cal() -> Calibration {
        generate_starter(&WizardRequest::default_inline4(), 1).unwrap()
    }

    /// Minimal open-loop log at a steady point, `lambda` constant, 20 ms rows.
    fn csv(rows: usize, lambda: f32, extra_hdr: &str, extra_val: &str) -> String {
        let mut s = format!("time_ms,rpm,map_kpa,tps_pct,clt_c,lambda,lambda_target{extra_hdr}\n");
        for i in 0..rows {
            s.push_str(&format!(
                "{},3000,60,25,90,{lambda},1.0{extra_val}\n",
                i * 20
            ));
        }
        s
    }

    fn lenient() -> ImportOptions {
        ImportOptions {
            assume_open_loop: true,
            assume_sequential_sync: true,
            assume_pressures_ok: true,
            confirm_recorded_with_loaded_calibration: true,
            exhaust_delay_ms: 0,
            ..ImportOptions::default()
        }
    }

    fn analysis_opts() -> MixtureMapOptions {
        MixtureMapOptions {
            min_samples_per_cell: 10,
            transient_holdoff_ms: 100,
            ..MixtureMapOptions::default()
        }
    }

    #[test]
    fn lean_open_loop_log_imports_and_proposes_a_ve_increase() {
        let c = cal();
        let imp = import_csv(&csv(80, 1.06, "", ""), &c, &lenient()).unwrap();
        assert_eq!(imp.report.rows_imported, 80);
        assert!(imp
            .report
            .assumptions
            .iter()
            .any(|a| a.contains("NOT verified")));
        assert!(imp.log.metadata.scenario_name.starts_with("external-csv"));
        let rep = analyze_log(&c, &imp.log, &analysis_opts()).unwrap();
        let p = rep.proposal.expect("proposal from lean steady log");
        assert!(p.cells.iter().all(|x| x.proposed_ve > x.current_ve));
        assert_eq!(p.source_scenario, imp.log.metadata.scenario_name);
        assert!(p.requires_approval);
    }

    #[test]
    fn afr_columns_are_converted_with_the_stated_stoich() {
        let c = cal();
        let mut s = String::from("Time,RPM,MAP,TPS,CLT,AFR,AFR Target\n");
        for i in 0..40 {
            s.push_str(&format!(
                "{:.3},3000,60,25,90,15.582,14.7\n",
                i as f32 * 0.02
            ));
        }
        let imp = import_csv(&s, &c, &lenient()).unwrap();
        let f = &imp.log.frames[0];
        assert!(
            (f.lambda_measured - 1.06).abs() < 1e-3,
            "{}",
            f.lambda_measured
        );
        assert!((f.lambda_target - 1.0).abs() < 1e-6);
        assert_eq!(imp.log.frames[1].timestamp_us, 20_000);
        assert!(imp.report.assumptions.iter().any(|a| a.contains("14.70")));
        // Ethanol stoich changes the conversion: the assumption is a real input.
        let e = import_csv(
            &s,
            &c,
            &ImportOptions {
                stoich_afr: 9.0,
                ..lenient()
            },
        )
        .unwrap();
        assert!((e.log.frames[0].lambda_measured - 15.582 / 9.0).abs() < 1e-3);
    }

    #[test]
    fn missing_loop_state_is_refused_unless_open_loop_is_asserted() {
        let c = cal();
        let o = ImportOptions {
            assume_open_loop: false,
            ..lenient()
        };
        assert_eq!(
            import_csv(&csv(30, 1.0, "", ""), &c, &o).unwrap_err(),
            ImportError::OpenLoopNotStated
        );
    }

    #[test]
    fn closed_loop_rows_without_trim_columns_are_refused() {
        let c = cal();
        let e = import_csv(&csv(30, 1.0, ",closed_loop", ",1"), &c, &lenient()).unwrap_err();
        assert_eq!(e, ImportError::ClosedLoopWithoutTrim);
        // With trims it imports and the trim is carried.
        let ok = import_csv(
            &csv(30, 1.0, ",closed_loop,stft,ltft", ",1,0.03,0.02"),
            &c,
            &lenient(),
        )
        .unwrap();
        assert!(ok.log.frames[0].closed_loop);
        assert!((ok.log.frames[0].stft + ok.log.frames[0].ltft - 0.05).abs() < 1e-6);
    }

    #[test]
    fn calibration_identity_must_be_checked_or_confirmed() {
        let c = cal();
        let mut o = lenient();
        o.confirm_recorded_with_loaded_calibration = false;
        assert_eq!(
            import_csv(&csv(30, 1.0, "", ""), &c, &o).unwrap_err(),
            ImportError::CalibrationIdentityUnconfirmed
        );

        // A VE column that agrees with the loaded table passes without the
        // confirmation flag and is reported as a measured check.
        let ve = c
            .table(VE_TABLE_KEY)
            .unwrap()
            .interpolate(&[60.0, 3000.0])
            .unwrap();
        let good = import_csv(
            &csv(30, 1.0, ",ve", &format!(",{ve}")),
            &c,
            &ImportOptions {
                confirm_recorded_with_loaded_calibration: false,
                ..lenient()
            },
        )
        .unwrap();
        let chk = good.report.ve_identity.expect("identity check");
        assert!(chk.median_abs_rel_diff < 1e-4);

        // 10 % different logged VE is a different table: refused even though the
        // user flag is set (a logged VE overrides the assertion).
        let bad = import_csv(
            &csv(30, 1.0, ",ve", &format!(",{}", ve * 1.10)),
            &c,
            &lenient(),
        )
        .unwrap_err();
        match bad {
            ImportError::CalibrationMismatch(c) => {
                assert!((c.median_abs_rel_diff - 0.10).abs() < 0.005)
            }
            other => panic!("expected mismatch, got {other:?}"),
        }
    }

    #[test]
    fn time_going_backwards_and_missing_coolant_are_refused() {
        let c = cal();
        let s = "time_ms,rpm,map_kpa,tps_pct,clt_c,lambda\n0,3000,60,25,90,1\n40,3000,60,25,90,1\n20,3000,60,25,90,1\n";
        assert_eq!(
            import_csv(s, &c, &lenient()).unwrap_err(),
            ImportError::TimeNotMonotonic { line: 4 }
        );
        let s = "time_ms,rpm,map_kpa,tps_pct,lambda\n0,3000,60,25,1\n";
        assert_eq!(
            import_csv(s, &c, &lenient()).unwrap_err(),
            ImportError::MissingColumn("clt_c")
        );
    }

    #[test]
    fn units_rows_and_comments_are_skipped_and_counted() {
        let c = cal();
        let mut s = String::from("# datalog from somewhere\n");
        s.push_str("time_ms\trpm\tmap_kpa\ttps_pct\tclt_c\tlambda\tlambda_target\n");
        s.push_str("ms\tRPM\tkPa\t%\tC\tlam\tlam\n");
        for i in 0..30 {
            s.push_str(&format!("{}\t3000\t60\t25\t90\t1.0\t1.0\n", i * 20));
        }
        let imp = import_csv(&s, &c, &lenient()).unwrap();
        assert_eq!(imp.report.delimiter, "\\t");
        assert_eq!(imp.report.rows_skipped, 1);
        assert_eq!(imp.report.rows_imported, 30);
        assert!(imp.report.skipped_examples[0].starts_with("line 3"));
    }

    #[test]
    fn logged_accel_enrichment_reaches_the_analyser_and_blocks_samples() {
        let c = cal();
        // Flat TPS/MAP/RPM; only the AE column shows the disturbance.
        let mut s = String::from(
            "time_ms,rpm,map_kpa,tps_pct,clt_c,lambda,lambda_target,accel_enrich_pct\n",
        );
        for i in 0..80 {
            let ae = if i == 30 { 15.0 } else { 0.0 };
            s.push_str(&format!("{},3000,60,25,90,1.06,1.0,{ae}\n", i * 20));
        }
        let imp = import_csv(&s, &c, &lenient()).unwrap();
        assert!(imp
            .report
            .warnings
            .iter()
            .all(|w| !w.contains("accel-enrichment")));
        let rep = analyze_log(&c, &imp.log, &analysis_opts()).unwrap();
        assert!(
            rep.gate_reason_counts.contains_key("WINDOW_ACCEL_ENRICH"),
            "{:?}",
            rep.gate_reason_counts
        );
        // An unreadable AE value is unknown, hence treated as active.
        let s2 = s.replace(",15\n", ",n/a\n");
        let imp2 = import_csv(&s2, &c, &lenient()).unwrap();
        assert!(imp2.log.frames[30].accel_enrich_pct.is_nan());
    }

    #[test]
    fn missing_gate_inputs_without_explicit_assumptions_reject_rows_not_pass_them() {
        let c = cal();
        let strict = ImportOptions {
            assume_open_loop: true,
            confirm_recorded_with_loaded_calibration: true,
            exhaust_delay_ms: 0,
            ..ImportOptions::default()
        };
        let imp = import_csv(&csv(60, 1.06, "", ""), &c, &strict).unwrap();
        assert!(imp
            .report
            .warnings
            .iter()
            .any(|w| w.contains("SYNC_NOT_SEQUENTIAL")));
        let rep = analyze_log(&c, &imp.log, &analysis_opts()).unwrap();
        assert_eq!(rep.samples_used, 0);
        assert!(rep.proposal.is_none());
        assert!(rep.gate_reason_counts.contains_key("SYNC_NOT_SEQUENTIAL"));
    }

    #[test]
    fn stuck_wideband_in_an_imported_log_is_still_caught() {
        let c = cal();
        // 80 identical lambda readings across a wide MAP/RPM span.
        let mut s = String::from("time_ms,rpm,map_kpa,tps_pct,clt_c,lambda,lambda_target\n");
        for i in 0..80 {
            s.push_str(&format!(
                "{},{},{},25,90,1.0123,1.0\n",
                i * 20,
                2000 + i * 30,
                40 + i
            ));
        }
        let imp = import_csv(&s, &c, &lenient()).unwrap();
        let rep = analyze_log(&c, &imp.log, &analysis_opts()).unwrap();
        assert!(rep
            .hard_faults
            .contains(&"LAMBDA_STUCK_SUSPECTED".to_string()));
        assert!(rep.proposal.is_none());
    }
}
