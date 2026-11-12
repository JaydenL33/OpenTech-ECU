import type {
  Calibration,
  LogFile,
  MixtureMapReport,
  PreflightReport,
  Scenario,
  SessionSummary,
  SetupForm,
  SimSnapshot,
  ActiveFault,
} from "./types";

const configuredBase = (import.meta as { env?: Record<string, string> }).env?.VITE_API_BASE;
export const API_BASE = configuredBase ?? "";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers: {
      "content-type": "application/json",
      ...(init?.headers ?? {}),
    },
  });
  if (!response.ok) {
    let message = `${response.status} ${response.statusText}`;
    try {
      const body = (await response.json()) as { message?: string; code?: string };
      message = body.message ?? message;
    } catch {
      // The body was not JSON; keep the HTTP status.
    }
    throw new Error(message);
  }
  return (await response.json()) as T;
}

export const api = {
  health: () =>
    request<{ status: string; protocol_version: number; physical_outputs: boolean }>("/api/health"),

  calibration: () =>
    request<{
      calibration: Calibration;
      preflight: PreflightReport;
      revision: number;
      checksum: string;
    }>("/api/calibration"),

  scenarios: () => request<{ scenarios: Scenario[] }>("/api/scenarios"),

  status: () =>
    request<{
      running: boolean;
      scenario: Scenario;
      snapshot: SimSnapshot;
      preflight: PreflightReport;
      frames: number;
      last_frame: import("./types").TelemetryFrame | null;
    }>("/api/sim/status"),

  start: (scenario_id: string, auto_loop = false) =>
    request<unknown>("/api/sim/start", {
      method: "POST",
      body: JSON.stringify({ scenario_id, auto_loop }),
    }),

  pause: () => request<unknown>("/api/sim/pause", { method: "POST" }),
  resume: () => request<unknown>("/api/sim/resume", { method: "POST" }),
  stop: () => request<unknown>("/api/sim/stop", { method: "POST" }),

  saveTable: (table_id: string, values: number[]) =>
    request<{ calibration: Calibration; preflight: PreflightReport }>(
      "/api/calibration/table",
      { method: "POST", body: JSON.stringify({ table_id, values }) },
    ),

  resampleAxis: (table_id: string, axis_index: number, points: number) =>
    request<{ calibration: Calibration; preflight: PreflightReport }>(
      "/api/calibration/resample",
      { method: "POST", body: JSON.stringify({ table_id, axis_index, points }) },
    ),

  history: () =>
    request<{
      revisions: {
        id: string;
        revision: number;
        checksum: string;
        parent: string | null;
        created_unix_ms: number;
        current: boolean;
      }[];
    }>("/api/calibration/history"),

  undo: () =>
    request<{ calibration: Calibration; preflight: PreflightReport }>(
      "/api/calibration/undo",
      { method: "POST" },
    ),

  preflight: () => request<PreflightReport>("/api/preflight"),

  setupForm: () => request<SetupForm>("/api/setup/form"),
  vehicles: () =>
    request<{
      vehicles: import("./types").VehicleProfile[];
      notice: string;
    }>("/api/vehicles"),

  loadEvoSimulation: () =>
    request<{
      calibration: Calibration;
      preflight: PreflightReport;
      warnings: { code: string; message: string }[];
      message: string;
      physical_ready: boolean;
    }>("/api/setup/evo-simulation", { method: "POST" }),

  setupGenerate: (form: SetupForm) =>
    request<{ calibration: Calibration; preflight: PreflightReport; warnings: string[] }>(
      "/api/setup/generate",
      { method: "POST", body: JSON.stringify(form) },
    ),

  mixtureAnalyze: (opts: { min_samples?: number; max_cell_correction?: number; allow_wot?: boolean; scenario_id?: string }) =>
    request<{ report: MixtureMapReport; log: unknown }>("/api/mixture/analyze", {
      method: "POST",
      body: JSON.stringify(opts),
    }),

  mixtureAnalyzeCurrent: (opts: { min_samples?: number; max_cell_correction?: number; allow_wot?: boolean }) =>
    request<{ report: MixtureMapReport }>("/api/mixture/analyze-current", {
      method: "POST",
      body: JSON.stringify(opts),
    }),

  mixtureApply: (
    proposal: import("./types").VeProposal,
    selected: [number, number][],
  ) =>
    request<{
      applied_cells: number;
      diff: { changes: unknown[]; max_abs_delta: number };
      undo_revision: number;
      calibration: Calibration;
      preflight: PreflightReport;
    }>("/api/mixture/apply", {
      method: "POST",
      body: JSON.stringify({ proposal, selected, approve: true }),
    }),

  faults: () =>
    request<{
      active: ActiveFault[];
      state: string;
      fuel_cut: boolean;
      spark_cut: boolean;
      definitions: { code: string; title: string }[];
    }>("/api/faults"),

  clearFaults: () => request<unknown>("/api/faults/clear", { method: "POST" }),

  log: () => request<LogFile>("/api/log"),
  replay: (source: "current" | "analysis" = "current") =>
    request<{ replaying: boolean; frames: number; source: string }>("/api/replay/start", {
      method: "POST",
      body: JSON.stringify({ source }),
    }),
  replayStop: () => request<unknown>("/api/replay/stop", { method: "POST" }),
  logSummary: () => request<SessionSummary>("/api/log/summary"),

  harness: () =>
    request<{
      manifest: import("./types").HarnessManifest;
      issues: { code: string; detail: string }[];
      valid: boolean;
      source_note: string;
    }>("/api/harness"),

  copilot: (message: string, history: { role: string; content: string }[]) =>
    request<{
      reply: string;
      findings: string[];
      proposedFuelCells: unknown[];
      approvalRequired: boolean;
      riskLevel: string;
      nextActions: string[];
      toolCalls: { name: string; summary: string }[];
      mode: string;
    }>(
      `${(import.meta as { env?: Record<string, string> }).env?.VITE_COPILOT_URL ?? "http://127.0.0.1:8787"}/api/copilot`,
      { method: "POST", body: JSON.stringify({ message, history }) },
    ),
};

export function csvUrl(): string {
  return `${API_BASE}/api/log.csv`;
}

export function wsUrl(): string {
  if (API_BASE) {
    return API_BASE.replace(/^http/, "ws") + "/api/ws?name=tuning-studio";
  }
  const protocol = window.location.protocol === "https:" ? "wss" : "ws";
  return `${protocol}://${window.location.host}/api/ws?name=tuning-studio`;
}
