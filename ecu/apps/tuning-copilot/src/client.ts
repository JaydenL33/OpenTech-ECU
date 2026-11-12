/** Read-only client for the Rust OpenTech-ECU studio service. */
export const RUST_API = process.env.OPENTECH_API ?? "http://127.0.0.1:8765";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 30_000);
  try {
    const response = await fetch(`${RUST_API}${path}`, {
      ...init,
      signal: controller.signal,
      headers: { "content-type": "application/json", ...(init?.headers ?? {}) },
    });
    if (!response.ok) {
      const text = await response.text().catch(() => "");
      throw new Error(`Rust service ${path} failed: ${response.status} ${text.slice(0, 300)}`);
    }
    return (await response.json()) as T;
  } finally {
    clearTimeout(timeout);
  }
}

interface RawFrame {
  rpm: number;
  map_kpa: number;
  tps_pct: number;
  lambda_measured: number;
  lambda_target: number;
  lambda_ready: boolean;
  stft: number;
  ltft: number;
  ignition_advance_deg: number;
  injector_pw_ms: number;
  injector_duty_pct: number;
  clt_c: number;
  oil_pressure_kpa: number;
  fuel_pressure_kpa: number;
  sync: string;
  engine_state: string;
  protection: string;
  active_faults: string[];
  scenario_time_ms: number;
  closed_loop: boolean;
}

export async function readTelemetry() {
  const status = await request<{
    running: boolean;
    scenario: { id: string; name: string };
    snapshot: Record<string, unknown>;
    calibration: { id: string; revision: number; checksum: string };
    frames: number;
    last_frame: RawFrame | null;
  }>("/api/sim/status");
  const f = status.last_frame;
  return {
    running: status.running,
    scenario: status.scenario.name,
    scenario_id: status.scenario.id,
    virtual_time_s: f ? f.scenario_time_ms / 1000 : 0,
    rpm: f?.rpm ?? 0,
    map_kpa: f?.map_kpa ?? 0,
    tps_pct: f?.tps_pct ?? 0,
    lambda: f?.lambda_ready ? f.lambda_measured : null,
    target_lambda: f?.lambda_target ?? null,
    lambda_ready: f?.lambda_ready ?? false,
    stft_percent: (f?.stft ?? 0) * 100,
    ltft_percent: (f?.ltft ?? 0) * 100,
    ignition_advance_deg: f?.ignition_advance_deg ?? 0,
    injection_pw_ms: f?.injector_pw_ms ?? 0,
    injector_duty_percent: f?.injector_duty_pct ?? 0,
    coolant_c: f?.clt_c ?? 0,
    oil_pressure_kpa: f?.oil_pressure_kpa ?? 0,
    fuel_pressure_kpa: f?.fuel_pressure_kpa ?? 0,
    sync: f?.sync ?? "unknown",
    engine_state: f?.engine_state ?? "unknown",
    protection: f?.protection ?? "unknown",
    active_faults: f?.active_faults ?? [],
    closed_loop: f?.closed_loop ?? false,
    calibration: status.calibration,
  };
}

export async function readFaults() {
  const data = await request<{
    active: {
      dtc: unknown;
      code: string;
      title: string;
      latched: boolean;
      condition_active: boolean;
      occurrences: number;
      freeze_frame: Record<string, number>;
    }[];
    state: string;
    fuel_cut: boolean;
    spark_cut: boolean;
    definitions: { code: string; title: string }[];
  }>("/api/faults");
  return {
    state: data.state,
    fuel_cut: data.fuel_cut,
    spark_cut: data.spark_cut,
    active: data.active.map((f) => ({
      code: f.code,
      title: f.title,
      latched: f.latched,
      condition_active: f.condition_active,
      occurrences: f.occurrences,
      freeze_frame: {
        rpm: f.freeze_frame.rpm,
        map_kpa: f.freeze_frame.map_kpa,
        lambda: f.freeze_frame.lambda,
        target_lambda: f.freeze_frame.target_lambda,
        coolant_c: f.freeze_frame.coolant_c,
        oil_pressure_kpa: f.freeze_frame.oil_pressure_kpa,
        fuel_pressure_kpa: f.freeze_frame.fuel_pressure_kpa,
        injector_duty_percent: f.freeze_frame.injector_duty_pct,
      },
    })),
    registry: data.definitions,
  };
}

export async function readCalibrationSummary() {
  const data = await request<{
    calibration: {
      id: string;
      revision: number;
      checksum: string;
      engine: {
        name: string;
        cylinders: number;
        displacement_cc: number;
        firing_order: number[];
        aspiration: string;
        rev_limit: number;
      };
      ignition_policy: { kind: string; warning?: string; reason?: string };
      tables: Record<
        string,
        {
          name: string;
          unit: string;
          axes: { name: string; unit: string }[];
          values: number[];
          provenance: { source: string; engine_specific_verified: boolean; citation: string };
        }
      >;
      notes: string[];
    };
    preflight: {
      summary: string;
      virtual_run_allowed: boolean;
      physical_export_allowed: boolean;
      error_count: number;
      warning_count: number;
      checks: { id: string; title: string; status: string; detail: string }[];
    };
  }>("/api/calibration");
  const c = data.calibration;
  return {
    id: c.id,
    revision: c.revision,
    checksum: c.checksum,
    engine: c.engine,
    ignition_policy: c.ignition_policy,
    tables: Object.fromEntries(
      Object.entries(c.tables).map(([key, t]) => [
        key,
        {
          name: t.name,
          unit: t.unit,
          rank: t.axes.length,
          cells: t.values.length,
          provenance_source: t.provenance.source,
          engine_specific_verified: t.provenance.engine_specific_verified,
        },
      ]),
    ),
    preflight: {
      summary: data.preflight.summary,
      virtual_run_allowed: data.preflight.virtual_run_allowed,
      physical_export_allowed: data.preflight.physical_export_allowed,
      error_count: data.preflight.error_count,
      warning_count: data.preflight.warning_count,
      non_passing_checks: data.preflight.checks
        .filter((ch) => ch.status === "fail" || ch.status === "warn")
        .map((ch) => ({ id: ch.id, title: ch.title, status: ch.status, detail: ch.detail })),
    },
    notes: c.notes,
  };
}

export async function runMixtureAnalysis(allowWot: boolean, minSamples: number) {
  const data = await request<{
    report: {
      samples_used: number;
      samples_rejected: number;
      calibration_revision: number;
      calibration_checksum: string;
      bank: number;
      hard_faults: string[];
      ignition_untouched: boolean;
      gate_reason_counts: Record<string, number>;
      proposal: null | {
        proposal_id: string;
        requires_approval: boolean;
        ignition_effect: string;
        cells: {
          rpm: number;
          map_kpa: number;
          current_ve: number;
          proposed_ve: number;
          correction_fraction: number;
          confidence: number;
          samples: number;
        }[];
      };
    };
  }>("/api/mixture/analyze", {
    method: "POST",
    body: JSON.stringify({ allow_wot: allowWot, min_samples: minSamples, max_cell_correction: 0.05 }),
  });
  const r = data.report;
  return {
    samples_used: r.samples_used,
    samples_rejected: r.samples_rejected,
    bank: r.bank,
    calibration_revision: r.calibration_revision,
    calibration_checksum: r.calibration_checksum,
    hard_faults: r.hard_faults,
    ignition_untouched: r.ignition_untouched,
    gate_reasons: r.gate_reason_counts,
    proposal: r.proposal
      ? {
          proposal_id: r.proposal.proposal_id,
          requires_human_approval: r.proposal.requires_approval,
          ignition_effect: r.proposal.ignition_effect,
          cells: r.proposal.cells.slice(0, 30).map((cell) => ({
            rpm: cell.rpm,
            map_kpa: cell.map_kpa,
            current_ve: cell.current_ve,
            proposed_ve: cell.proposed_ve,
            correction_percent: cell.correction_fraction * 100,
            confidence: cell.confidence,
            samples: cell.samples,
          })),
        }
      : null,
  };
}

export async function readLogSummary() {
  const log = await request<{
    metadata: {
      scenario_name: string;
      seed: number;
      calibration_revision: number;
      calibration_checksum: string;
      sample_hz: number;
    };
    frames: RawFrame[];
    events: { timestamp_us: number; severity: string; source: string; code: string; message: string }[];
  }>("/api/log");
  const frames = log.frames;
  const last = frames.slice(-5).map((f) => ({
    t_s: f.scenario_time_ms / 1000,
    rpm: f.rpm,
    map_kpa: f.map_kpa,
    lambda: f.lambda_ready ? f.lambda_measured : null,
    protection: f.protection,
    faults: f.active_faults,
  }));
  const duration_s = frames.length ? (frames[frames.length - 1].scenario_time_ms - frames[0].scenario_time_ms) / 1000 : 0;
  return {
    scenario: log.metadata.scenario_name,
    seed: log.metadata.seed,
    sample_hz: log.metadata.sample_hz,
    frames: frames.length,
    duration_s,
    calibration_revision: log.metadata.calibration_revision,
    calibration_checksum: log.metadata.calibration_checksum,
    recent_events: log.events.slice(-20).map((e) => ({
      t_s: e.timestamp_us / 1_000_000,
      severity: e.severity,
      source: e.source,
      code: e.code,
      message: e.message,
    })),
    latest_frames: last,
  };
}
