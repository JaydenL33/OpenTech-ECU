#![forbid(unsafe_code)]

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Context;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use ecu_calibration::preflight::{preflight, PreflightReport};
use ecu_calibration::vehicle::{builtin_profiles, by_id as vehicle_by_id};
use ecu_calibration::wizard::{generate_starter, WizardRequest};
use ecu_calibration::{now_unix_ms, Calibration};
use ecu_harness::{validate_manifest, BenchHarnessManifest};
use ecu_lambda::mixture_map::{
    analyze_log, apply_proposal, ApplyResult, MixtureMapOptions, MixtureMapReport, VeProposal,
};
use ecu_protection::ActiveFault;
use ecu_protocol::{negotiate, Hello, HelloAck, JsonEnvelope, ServerEvent, PROTOCOL_VERSION};
use ecu_simulator::scenario::{builtin_scenarios, by_id, Scenario};
use ecu_simulator::SimEngine;
use ecu_types::{Aspiration, EngineConfig, FuelConfig, InjectorConfig, LogFile};

#[derive(Parser, Debug)]
#[command(
    name = "opentech-studio-service",
    version,
    about = "OpenTech-ECU tuning service"
)]
struct Args {
    #[arg(long, default_value_t = 8765)]
    port: u16,
    #[arg(long)]
    cal: Option<PathBuf>,
    #[arg(long, default_value = "idle_warmup")]
    scenario: String,
    /// Static UI directory. Defaults to the Vite build in the workspace.
    #[arg(long)]
    static_dir: Option<PathBuf>,
    /// Do not auto-loop scenarios when they reach their scripted end.
    #[arg(long)]
    no_loop: bool,
}

#[derive(Clone)]
struct AppState {
    inner: Arc<Mutex<Session>>,
    tx: broadcast::Sender<String>,
}

struct Session {
    calibration: Calibration,
    scenario: Scenario,
    engine: SimEngine,
    running: bool,
    auto_loop: bool,
    history: Vec<Calibration>,
    frame_cursor: usize,
    seq: u64,
    last_report: Option<MixtureMapReport>,
    last_analysis_log: Option<LogFile>,
    replay: Option<ReplayState>,
}

struct ReplayState {
    frames: Vec<ecu_types::TelemetryFrame>,
    index: usize,
}

#[derive(Clone, Debug, Serialize)]
struct ApiError {
    code: String,
    message: String,
}

impl ApiError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
    fn bad(message: impl Into<String>) -> Self {
        Self::new("bad_request", message)
    }
    fn conflict(message: impl Into<String>) -> Self {
        Self::new("conflict", message)
    }
    fn internal(message: impl Into<String>) -> Self {
        Self::new("internal", message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.code.as_str() {
            "bad_request" => StatusCode::BAD_REQUEST,
            "conflict" => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(self)).into_response()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "opentech_studio_service=info,tower_http=warn".into()),
        )
        .init();
    let args = Args::parse();

    let calibration = match &args.cal {
        Some(path) => {
            let bytes = std::fs::read(path)
                .with_context(|| format!("reading calibration {}", path.display()))?;
            let cal: Calibration = serde_json::from_slice(&bytes).context("parsing calibration")?;
            if !cal.verify() {
                anyhow::bail!("calibration checksum is invalid");
            }
            cal
        }
        None => generate_starter(&WizardRequest::default_inline4(), 1)
            .map_err(|e| anyhow::anyhow!("{e}"))?,
    };
    let scenario = by_id(&args.scenario)
        .ok_or_else(|| anyhow::anyhow!("unknown scenario '{}'", args.scenario))?;
    let engine = SimEngine::new(calibration.clone(), scenario.clone())
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let history = vec![calibration.clone()];
    let (tx, _) = broadcast::channel(512);
    let state = AppState {
        inner: Arc::new(Mutex::new(Session {
            calibration,
            scenario,
            engine,
            running: true,
            auto_loop: !args.no_loop,
            history,
            frame_cursor: 0,
            seq: 0,
            last_report: None,
            last_analysis_log: None,
            replay: None,
        })),
        tx: tx.clone(),
    };

    let bg_state = state.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_millis(20));
        loop {
            tick.tick().await;
            let messages = {
                let Ok(mut session) = bg_state.inner.lock() else {
                    continue;
                };
                if let Some(replay) = &mut session.replay {
                    if replay.index >= replay.frames.len() {
                        session.replay = None;
                        session.running = false;
                        Vec::new()
                    } else {
                        let frame = replay.frames[replay.index].clone();
                        replay.index += 1;
                        session.seq += 1;
                        let event = ServerEvent::Telemetry {
                            seq: session.seq,
                            calibration_checksum: session.calibration.checksum.clone(),
                            frame,
                        };
                        JsonEnvelope::new(session.seq, event)
                            .to_json()
                            .ok()
                            .into_iter()
                            .collect()
                    }
                } else if !session.running {
                    Vec::new()
                } else {
                    session.engine.advance_ms(20);
                    let mut messages = Vec::new();
                    while session.frame_cursor < session.engine.frames().len() {
                        let frame = session.engine.frames()[session.frame_cursor].clone();
                        session.frame_cursor += 1;
                        session.seq += 1;
                        let event = ServerEvent::Telemetry {
                            seq: session.seq,
                            calibration_checksum: session.calibration.checksum.clone(),
                            frame,
                        };
                        if let Ok(json) = JsonEnvelope::new(session.seq, event).to_json() {
                            messages.push(json);
                        }
                    }
                    if session.engine.is_finished() {
                        if session.auto_loop {
                            if let Ok(mut next) = SimEngine::new(
                                session.calibration.clone(),
                                session.scenario.clone(),
                            ) {
                                next.advance_ms(20);
                                session.engine = next;
                                session.frame_cursor = 0;
                            }
                        } else {
                            session.running = false;
                        }
                    }
                    messages
                }
            };
            for m in messages {
                let _ = bg_state.tx.send(m);
            }
        }
    });

    let static_dir = args.static_dir.unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/tuning-studio/dist")
    });
    let index = static_dir.join("index.html");
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/scenarios", get(scenarios))
        .route("/api/calibration", get(get_calibration))
        .route("/api/calibration/table", post(edit_table))
        .route("/api/calibration/resample", post(resample_axis))
        .route("/api/calibration/history", get(calibration_history))
        .route("/api/calibration/undo", post(undo_calibration))
        .route("/api/preflight", get(get_preflight))
        .route("/api/setup/form", get(setup_form))
        .route("/api/setup/generate", post(setup_generate))
        .route("/api/vehicles", get(vehicles))
        .route("/api/setup/evo-simulation", post(evo_simulation))
        .route("/api/sim/start", post(sim_start))
        .route("/api/sim/pause", post(sim_pause))
        .route("/api/sim/resume", post(sim_resume))
        .route("/api/sim/stop", post(sim_stop))
        .route("/api/sim/status", get(sim_status))
        .route("/api/sim/control", post(sim_control))
        .route("/api/log", get(get_log))
        .route("/api/log/summary", get(log_summary))
        .route("/api/log.csv", get(get_log_csv))
        .route("/api/mixture/analyze", post(mixture_analyze))
        .route(
            "/api/mixture/analyze-current",
            post(mixture_analyze_current),
        )
        .route("/api/mixture/apply", post(mixture_apply))
        .route("/api/replay/start", post(replay_start))
        .route("/api/replay/stop", post(replay_stop))
        .route("/api/faults", get(get_faults))
        .route("/api/faults/clear", post(clear_faults))
        .route("/api/harness", get(get_harness))
        .route("/api/ws", get(ws_handler))
        .layer(CorsLayer::permissive())
        .fallback_service(ServeDir::new(&static_dir).not_found_service(ServeFile::new(&index)))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    tracing::info!(
        "OpenTech-ECU studio service on http://{addr} (protocol v{PROTOCOL_VERSION}); UI {}",
        static_dir.display()
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

const BENCH_HARNESS_JSON: &str =
    include_str!("../../../../config/harness/opentech-bench-harness.json");

async fn get_harness() -> Result<Json<serde_json::Value>, ApiError> {
    let manifest: BenchHarnessManifest = serde_json::from_str(BENCH_HARNESS_JSON)
        .map_err(|e| ApiError::internal(format!("bench harness manifest: {e}")))?;
    let issues = validate_manifest(&manifest);
    Ok(Json(json!({
        "manifest": manifest,
        "issues": issues,
        "valid": issues.is_empty(),
        "source_note": "OpenTech bench schematic export. UNREVIEWED; board-local connectors only; not an OEM harness."
    })))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "opentech-ecu-studio",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol_version": PROTOCOL_VERSION,
        "physical_outputs": false,
        "interlock": "PHYSICAL_OUTPUT_INTERLOCK_ENGAGED",
        "note": "Virtual ECU only. No injector or ignition hardware path exists."
    }))
}

async fn scenarios() -> Json<serde_json::Value> {
    Json(json!({ "scenarios": builtin_scenarios() }))
}

async fn get_calibration(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    Ok(Json(json!({
        "calibration": session.calibration,
        "preflight": preflight(&session.calibration),
        "revision": session.calibration.revision,
        "checksum": session.calibration.checksum,
    })))
}

async fn get_preflight(State(state): State<AppState>) -> Result<Json<PreflightReport>, ApiError> {
    let session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    Ok(Json(preflight(&session.calibration)))
}

async fn calibration_history(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    let revisions: Vec<_> = session
        .history
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "revision": c.revision,
                "checksum": c.checksum,
                "parent": c.parent,
                "created_unix_ms": c.created_unix_ms,
                "current": c.checksum == session.calibration.checksum,
            })
        })
        .collect();
    Ok(Json(json!({ "revisions": revisions })))
}

#[derive(Deserialize)]
struct TableEdit {
    table_id: String,
    values: Vec<f32>,
}

async fn edit_table(
    State(state): State<AppState>,
    Json(body): Json<TableEdit>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    if session.running {
        return Err(ApiError::conflict(
            "pause the virtual engine before editing maps so changes are auditable",
        ));
    }
    let next = session
        .calibration
        .with_table_values(&body.table_id, body.values, now_unix_ms())
        .map_err(|e| ApiError::bad(e.to_string()))?;
    let pre = preflight(&next);
    if !pre.virtual_run_allowed {
        return Err(ApiError::bad(pre.summary));
    }
    // The running/paused engine owns a snapshot. Rebuild it so a resumed run
    // actually uses the newly sealed map instead of stale table data.
    let new_engine = SimEngine::new(next.clone(), session.scenario.clone())
        .map_err(|e| ApiError::internal(e.to_string()))?;
    session.calibration = next;
    session.engine = new_engine;
    session.frame_cursor = 0;
    let snapshot = session.calibration.clone();
    session.history.push(snapshot);
    Ok(Json(json!({
        "calibration": session.calibration,
        "preflight": pre,
        "revision": session.calibration.revision,
    })))
}

#[derive(Deserialize)]
struct ResampleRequest {
    table_id: String,
    axis_index: usize,
    points: usize,
}

async fn resample_axis(
    State(state): State<AppState>,
    Json(body): Json<ResampleRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    if session.running {
        return Err(ApiError::conflict("pause before resampling an axis"));
    }
    if !(4..=32).contains(&body.points) {
        return Err(ApiError::bad("axis point count must be 4..=32"));
    }
    let table = session
        .calibration
        .table(&body.table_id)
        .map_err(|e| ApiError::bad(e.to_string()))?
        .clone();
    let axis = table
        .axes
        .get(body.axis_index)
        .ok_or_else(|| ApiError::bad("axis index out of range"))?;
    let first = *axis
        .breakpoints
        .first()
        .ok_or_else(|| ApiError::bad("empty axis"))?;
    let last = *axis
        .breakpoints
        .last()
        .ok_or_else(|| ApiError::bad("empty axis"))?;
    let mut points = Vec::with_capacity(body.points);
    for i in 0..body.points {
        let t = i as f32 / (body.points - 1) as f32;
        points.push(first + (last - first) * t);
    }
    let resampled = table
        .resample_axis(body.axis_index, points.clone())
        .map_err(|e| ApiError::bad(e.to_string()))?;
    let mut next = session
        .calibration
        .fork(session.calibration.revision + 1, now_unix_ms());
    next.tables.insert(body.table_id.clone(), resampled);
    // VE and target-lambda must stay axis-aligned. Resampling one without the
    // other would silently break cell attribution, so keep the pair aligned.
    let paired = match body.table_id.as_str() {
        "ve" => Some("target_lambda"),
        "target_lambda" => Some("ve"),
        _ => None,
    };
    if let Some(paired_key) = paired {
        if let Some(paired_table) = next.tables.get(paired_key).cloned() {
            if paired_table.rank() == 2 && paired_table.axes.get(body.axis_index).is_some() {
                let aligned = paired_table
                    .resample_axis(body.axis_index, points.clone())
                    .map_err(|e| ApiError::bad(e.to_string()))?;
                next.tables.insert(paired_key.to_string(), aligned);
                next.notes.push(format!(
                    "Table {} resampled in lockstep to keep VE/target-lambda cells aligned.",
                    paired_key
                ));
            }
        }
    }
    next.notes.push(format!(
        "Axis {} of table {} resampled to {} points (interpolated editing aid, not new evidence).",
        body.axis_index, body.table_id, body.points
    ));
    next.seal();
    let pre = preflight(&next);
    if !pre.virtual_run_allowed {
        return Err(ApiError::bad(pre.summary));
    }
    session.calibration = next;
    let snapshot = session.calibration.clone();
    session.history.push(snapshot);
    session.engine = SimEngine::new(session.calibration.clone(), session.scenario.clone())
        .map_err(|e| ApiError::internal(e.to_string()))?;
    session.frame_cursor = 0;
    Ok(Json(json!({
        "calibration": session.calibration,
        "preflight": pre,
    })))
}

async fn undo_calibration(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    if session.running {
        return Err(ApiError::conflict("pause before rolling back calibration"));
    }
    if session.history.len() < 2 {
        return Err(ApiError::bad("no earlier calibration revision to restore"));
    }
    session.history.pop();
    let restored = session
        .history
        .last()
        .cloned()
        .ok_or_else(|| ApiError::internal("history empty"))?;
    session.calibration = restored;
    session.engine = SimEngine::new(session.calibration.clone(), session.scenario.clone())
        .map_err(|e| ApiError::internal(e.to_string()))?;
    session.frame_cursor = 0;
    Ok(Json(json!({
        "calibration": session.calibration,
        "preflight": preflight(&session.calibration),
    })))
}

#[derive(Deserialize)]
struct StartRequest {
    scenario_id: String,
    #[serde(default)]
    auto_loop: bool,
}

async fn sim_start(
    State(state): State<AppState>,
    Json(body): Json<StartRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let scenario = by_id(&body.scenario_id)
        .ok_or_else(|| ApiError::bad(format!("unknown scenario '{}'", body.scenario_id)))?;
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    let mut engine = SimEngine::new(session.calibration.clone(), scenario.clone())
        .map_err(|e| ApiError::bad(e.to_string()))?;
    engine.advance_ms(20);
    session.engine = engine;
    session.scenario = scenario;
    session.running = true;
    session.auto_loop = body.auto_loop;
    session.frame_cursor = 0;
    session.replay = None;
    Ok(Json(json!({
        "running": true,
        "scenario": session.scenario,
        "calibration_checksum": session.calibration.checksum,
    })))
}

async fn sim_pause(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    session.running = false;
    Ok(Json(json!({ "running": false })))
}

async fn sim_resume(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    if session.engine.is_finished() {
        let mut next = SimEngine::new(session.calibration.clone(), session.scenario.clone())
            .map_err(|e| ApiError::internal(e.to_string()))?;
        next.advance_ms(20);
        session.engine = next;
        session.frame_cursor = 0;
    }
    session.running = true;
    Ok(Json(json!({ "running": true })))
}

async fn sim_stop(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    session.running = false;
    session.engine = SimEngine::new(session.calibration.clone(), session.scenario.clone())
        .map_err(|e| ApiError::internal(e.to_string()))?;
    session.frame_cursor = 0;
    Ok(Json(json!({ "running": false, "reset": true })))
}

async fn sim_status(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    Ok(Json(json!({
        "running": session.running,
        "scenario": session.scenario,
        "snapshot": session.engine.snapshot(),
        "preflight": preflight(&session.calibration),
        "calibration": {
            "id": session.calibration.id,
            "revision": session.calibration.revision,
            "checksum": session.calibration.checksum,
        },
        "frames": session.engine.frames().len(),
        "last_frame": session.engine.last_frame(),
    })))
}

#[derive(Deserialize)]
struct ControlRequest {
    throttle_pct: f32,
    load_nm: f32,
}

async fn sim_control(
    State(state): State<AppState>,
    Json(body): Json<ControlRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    if !(0.0..=100.0).contains(&body.throttle_pct) || !(0.0..=500.0).contains(&body.load_nm) {
        return Err(ApiError::bad("throttle 0..100 and load 0..500 required"));
    }
    // Manual controls are not enabled in v0.1: scenario scripts are the only
    // deterministic driver. Returning conflict is safer than silently ignoring.
    let _ = (&mut session, body);
    Err(ApiError::conflict(
        "manual controls are not implemented; start a deterministic scenario instead",
    ))
}

#[derive(Deserialize)]
struct ReplayRequest {
    #[serde(default)]
    source: Option<String>,
}

async fn replay_start(
    State(state): State<AppState>,
    Json(body): Json<ReplayRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    let log = match body.source.as_deref() {
        Some("analysis") => session
            .last_analysis_log
            .clone()
            .ok_or_else(|| ApiError::bad("no analyzed log available"))?,
        _ => session.engine.log_file(),
    };
    if log.frames.is_empty() {
        return Err(ApiError::bad("log has no frames to replay"));
    }
    session.running = false;
    session.replay = Some(ReplayState {
        frames: log.frames,
        index: 0,
    });
    Ok(Json(json!({
        "replaying": true,
        "frames": session.replay.as_ref().map(|r| r.frames.len()).unwrap_or(0),
        "source": body.source.unwrap_or_else(|| "current".into()),
    })))
}

async fn replay_stop(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    session.replay = None;
    session.running = false;
    Ok(Json(json!({ "replaying": false })))
}

async fn get_log(State(state): State<AppState>) -> Result<Json<LogFile>, ApiError> {
    let session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    Ok(Json(session.engine.log_file()))
}

async fn log_summary(
    State(state): State<AppState>,
) -> Result<Json<ecu_diagnostics::SessionSummary>, ApiError> {
    let session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    Ok(Json(ecu_diagnostics::session_summary(
        session.engine.frames(),
    )))
}

async fn get_log_csv(State(state): State<AppState>) -> Result<Response, ApiError> {
    let session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    let csv = session.engine.log_file().to_csv();
    Ok(([(header::CONTENT_TYPE, "text/csv; charset=utf-8")], csv).into_response())
}

#[derive(Deserialize, Default)]
struct AnalyzeOptions {
    #[serde(default)]
    scenario_id: Option<String>,
    #[serde(default)]
    min_samples: Option<u32>,
    #[serde(default)]
    max_cell_correction: Option<f32>,
    #[serde(default)]
    allow_wot: bool,
}

async fn mixture_analyze(
    State(state): State<AppState>,
    Json(body): Json<AnalyzeOptions>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (cal, scenario, opts) = {
        let session = state
            .inner
            .lock()
            .map_err(|_| ApiError::internal("state poisoned"))?;
        let scenario_id = body.scenario_id.as_deref().unwrap_or("mixture_sweep");
        let scenario = by_id(scenario_id)
            .ok_or_else(|| ApiError::bad(format!("unknown scenario '{scenario_id}'")))?;
        let opts = MixtureMapOptions {
            min_samples_per_cell: body.min_samples.unwrap_or(12),
            max_cell_correction: body.max_cell_correction.unwrap_or(0.05),
            allow_wot: body.allow_wot,
            ..MixtureMapOptions::default()
        };
        (session.calibration.clone(), scenario, opts)
    };
    let (log, report) = tokio::task::spawn_blocking(move || {
        let mut engine = SimEngine::new(cal.clone(), scenario).map_err(|e| e.to_string())?;
        engine.run_to_end();
        let log = engine.log_file();
        let report = analyze_log(&cal, &log, &opts).map_err(|e| e.to_string())?;
        Ok::<_, String>((log, report))
    })
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?
    .map_err(ApiError::bad)?;
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    session.last_report = Some(report.clone());
    session.last_analysis_log = Some(log.clone());
    Ok(Json(json!({
        "report": report,
        "log": { "metadata": log.metadata, "frames": log.frames.len(), "events": log.events.len() },
    })))
}

async fn mixture_analyze_current(
    State(state): State<AppState>,
    Json(body): Json<AnalyzeOptions>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (cal, log) = {
        let session = state
            .inner
            .lock()
            .map_err(|_| ApiError::internal("state poisoned"))?;
        (session.calibration.clone(), session.engine.log_file())
    };
    let opts = MixtureMapOptions {
        min_samples_per_cell: body.min_samples.unwrap_or(20),
        max_cell_correction: body.max_cell_correction.unwrap_or(0.05),
        allow_wot: body.allow_wot,
        ..MixtureMapOptions::default()
    };
    let report = analyze_log(&cal, &log, &opts).map_err(|e| ApiError::bad(e.to_string()))?;
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    session.last_report = Some(report.clone());
    session.last_analysis_log = Some(log.clone());
    Ok(Json(json!({ "report": report })))
}

#[derive(Deserialize)]
struct ApplyRequest {
    proposal: VeProposal,
    #[serde(default)]
    selected: Option<Vec<[usize; 2]>>,
    #[serde(default)]
    approve: bool,
}

async fn mixture_apply(
    State(state): State<AppState>,
    Json(body): Json<ApplyRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !body.approve {
        return Err(ApiError::bad(
            "explicit approve=true is required; proposals are never applied automatically",
        ));
    }
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    if session.running {
        return Err(ApiError::conflict(
            "pause before applying a calibration revision",
        ));
    }
    let selected: Option<Vec<(usize, usize)>> = body
        .selected
        .map(|v| v.into_iter().map(|a| (a[0], a[1])).collect());
    let result: ApplyResult = apply_proposal(
        &session.calibration,
        &body.proposal,
        selected.as_deref(),
        now_unix_ms(),
    )
    .map_err(|e| ApiError::bad(e.to_string()))?;
    let next = result.calibration.clone();
    let pre = preflight(&next);
    session.history.push(next.clone());
    session.calibration = next;
    session.engine = SimEngine::new(session.calibration.clone(), session.scenario.clone())
        .map_err(|e| ApiError::internal(e.to_string()))?;
    session.frame_cursor = 0;
    Ok(Json(json!({
        "applied_cells": result.applied_cells,
        "diff": result.diff,
        "undo_revision": result.undo_revision,
        "calibration": session.calibration,
        "preflight": pre,
    })))
}

async fn get_faults(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    let faults = session.engine.protection_faults();
    let last = session.engine.last_frame();
    let fault_json: Vec<serde_json::Value> = faults
        .iter()
        .map(|f: &ActiveFault| {
            json!({
                "dtc": f.dtc,
                "code": f.dtc.code(),
                "title": f.dtc.title(),
                "latched": f.latched,
                "condition_active": f.condition_active,
                "first_seen_us": f.first_seen_us,
                "last_seen_us": f.last_seen_us,
                "occurrences": f.occurrences,
                "freeze_frame": f.freeze_frame,
            })
        })
        .collect();
    Ok(Json(json!({
        "active": fault_json,
        "state": last.map(|f| f.protection),
        "fuel_cut": last.map(|f| f.fuel_cut).unwrap_or(false),
        "spark_cut": last.map(|f| f.spark_cut).unwrap_or(false),
        "definitions": ecu_protection::Dtc::ALL.iter().map(|d| json!({
            "code": d.code(), "title": d.title()
        })).collect::<Vec<_>>(),
    })))
}

async fn clear_faults(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    let cleared = session.engine.clear_latched_faults();
    Ok(Json(
        json!({ "cleared": cleared, "faults": session.engine.protection_faults() }),
    ))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SetupForm {
    pub vehicle_name: String,
    pub engine_name: String,
    pub displacement_cc: f32,
    pub cylinders: u8,
    pub firing_order: Vec<u8>,
    pub compression_ratio: f32,
    pub aspiration: String,
    pub rev_limit: u16,
    pub idle_rpm: u16,
    pub injector_flow_cc_min: f32,
    pub injector_dead_time_ms: f32,
    pub injector_rated_pressure_kpa: f32,
    pub stoich_afr: f32,
    pub fuel_density_g_cc: f32,
}

impl Default for SetupForm {
    fn default() -> Self {
        let e = EngineConfig::default_inline4();
        Self {
            vehicle_name: "Virtual Development Vehicle".into(),
            engine_name: e.name,
            displacement_cc: e.displacement_cc,
            cylinders: e.cylinders,
            firing_order: e.firing_order,
            compression_ratio: e.compression_ratio,
            aspiration: "naturally_aspirated".into(),
            rev_limit: e.rev_limit,
            idle_rpm: e.idle_rpm,
            injector_flow_cc_min: e.injectors[0].flow_cc_min,
            injector_dead_time_ms: e.injectors[0].dead_time_ms,
            injector_rated_pressure_kpa: e.injectors[0].rated_pressure_kpa,
            stoich_afr: 14.7,
            fuel_density_g_cc: 0.745,
        }
    }
}

async fn setup_form() -> Json<SetupForm> {
    Json(SetupForm::default())
}

async fn vehicles() -> Json<serde_json::Value> {
    Json(json!({
        "vehicles": builtin_profiles(),
        "notice": "All vehicle data is UNVERIFIED. Simulation profiles are synthetic and cannot be used for physical integration."
    }))
}

async fn evo_simulation(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let profile = vehicle_by_id("mitsubishi-evo8-jdm-gsr-6mt")
        .ok_or_else(|| ApiError::internal("Evo profile missing"))?;
    let (calibration, warnings) = profile
        .simulation_calibration()
        .map_err(|e| ApiError::bad(e.to_string()))?;
    let pre = preflight(&calibration);
    if !pre.virtual_run_allowed {
        return Err(ApiError::bad(pre.summary));
    }
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    session.running = false;
    session.calibration = calibration.clone();
    session.history.push(calibration.clone());
    session.engine = SimEngine::new(calibration.clone(), session.scenario.clone())
        .map_err(|e| ApiError::internal(e.to_string()))?;
    session.frame_cursor = 0;
    Ok(Json(json!({
        "profile": profile,
        "calibration": calibration,
        "preflight": pre,
        "warnings": warnings,
        "physical_ready": false,
        "message": "Evo VIII simulation profile loaded. Pinout, connector, trigger and injector data remain UNVERIFIED; physical outputs are blocked."
    })))
}

async fn setup_generate(
    State(state): State<AppState>,
    Json(form): Json<SetupForm>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if form.cylinders == 0 || form.firing_order.is_empty() {
        return Err(ApiError::bad(
            "cylinder count and firing order are required",
        ));
    }
    let mut engine = EngineConfig::default_inline4();
    engine.name = form.engine_name;
    engine.displacement_cc = form.displacement_cc;
    engine.cylinders = form.cylinders;
    engine.firing_order = form.firing_order;
    engine.cylinder_banks = vec![1; form.cylinders as usize];
    engine.compression_ratio = form.compression_ratio;
    engine.rev_limit = form.rev_limit;
    engine.idle_rpm = form.idle_rpm;
    engine.aspiration = match form.aspiration.as_str() {
        "turbocharged" => Aspiration::Turbocharged,
        "supercharged" => Aspiration::Supercharged,
        _ => Aspiration::NaturallyAspirated,
    };
    let mut injector = InjectorConfig {
        id: "wizard-injector".into(),
        flow_cc_min: form.injector_flow_cc_min,
        rated_pressure_kpa: form.injector_rated_pressure_kpa,
        dead_time_ms: form.injector_dead_time_ms,
        dead_time_curve: vec![
            (8.0, form.injector_dead_time_ms * 1.8),
            (12.0, form.injector_dead_time_ms * 1.2),
            (13.8, form.injector_dead_time_ms),
            (15.0, form.injector_dead_time_ms * 0.9),
        ],
        min_pw_ms: 0.5,
        max_duty: 0.9,
        cylinders: (1..=form.cylinders).collect(),
    };
    injector.rated_pressure_kpa = form.injector_rated_pressure_kpa;
    engine.injectors = vec![injector];
    engine.cylinder_banks = vec![1; form.cylinders as usize];
    let fuel = FuelConfig {
        stoich_afr: form.stoich_afr,
        fuel_density_g_cc: form.fuel_density_g_cc,
        ..FuelConfig::default()
    };
    let mut req = WizardRequest::default_inline4();
    req.engine = engine;
    req.fuel = fuel;
    req.vehicle_name = form.vehicle_name;
    let result = generate_starter(&req, 1).map_err(|e| ApiError::bad(e.to_string()))?;
    let pre = preflight(&result);
    let mut session = state
        .inner
        .lock()
        .map_err(|_| ApiError::internal("state poisoned"))?;
    session.running = false;
    session.calibration = result.clone();
    session.history.push(result.clone());
    let scenario = by_id("mixture_sweep").unwrap_or_else(|| session.scenario.clone());
    session.scenario = scenario;
    session.engine = SimEngine::new(result, session.scenario.clone())
        .map_err(|e| ApiError::internal(e.to_string()))?;
    session.frame_cursor = 0;
    Ok(Json(json!({
        "calibration": session.calibration,
        "preflight": pre,
        "warnings": [
            "ALL GENERATED FUEL MAPS ARE UNVALIDATED STARTING ESTIMATES.",
            "Ignition table is synthetic and simulator-only. Physical outputs remain blocked."
        ]
    })))
}

#[derive(Deserialize)]
struct WsQuery {
    #[serde(default)]
    name: Option<String>,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(q): Query<WsQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_loop(socket, state, q.name.unwrap_or_else(|| "ui".into())))
}

async fn ws_loop(mut socket: WebSocket, state: AppState, name: String) {
    let ack: HelloAck = negotiate(
        &Hello {
            protocol_version: PROTOCOL_VERSION,
            client_name: name,
            client_version: "0.1".into(),
        },
        env!("CARGO_PKG_VERSION"),
    );
    if let Ok(json) = JsonEnvelope::new(0, ServerEvent::Hello(ack)).to_json() {
        if socket.send(Message::Text(json.into())).await.is_err() {
            return;
        }
    }
    let mut rx = state.tx.subscribe();
    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(text) => {
                        if socket.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        let _ = socket.send(Message::Text(
                            json!({"protocol": PROTOCOL_VERSION, "type": "warning", "message": format!("lagged {n} frames")}).to_string().into()
                        )).await;
                    }
                    Err(_) => break,
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(p))) => {
                        if socket.send(Message::Pong(p)).await.is_err() { break; }
                    }
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }
}
