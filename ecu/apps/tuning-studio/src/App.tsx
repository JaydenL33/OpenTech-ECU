import { useEffect, useMemo, useState } from "react";
import { api } from "./api";
import { AppContext, type AppContextValue } from "./context";
import { useLiveTelemetry } from "./hooks/useLiveTelemetry";
import type { Calibration, PreflightReport, Scenario } from "./types";
import { DashboardPage } from "./pages/Dashboard";
import { DashboardDesignerPage } from "./pages/DashboardDesigner";
import { CylindersPage } from "./pages/Cylinders";
import { MapPage } from "./pages/MapPage";
import { MixturePage } from "./pages/Mixture";
import { SetupPage } from "./pages/Setup";
import { DiagnosticsPage } from "./pages/Diagnostics";
import { CalibrationPage } from "./pages/Calibration";
import { LogsPage } from "./pages/Logs";
import { LogViewerPage } from "./pages/LogViewer";
import { CopilotPage } from "./pages/Copilot";
import { HarnessPage } from "./pages/Harness";
import { WarningsPage } from "./pages/Warnings";
import { ChannelsPage } from "./pages/Channels";
import { WarningBanner } from "./components/WarningBanner";
import { Badge } from "./components/ui";

type PageId =
  | "dashboard"
  | "designer"
  | "cylinders"
  | "fuel"
  | "target"
  | "ignition"
  | "mixture"
  | "setup"
  | "harness"
  | "diagnostics"
  | "calibration"
  | "logs"
  | "logviewer"
  | "warnings"
  | "channels"
  | "copilot";

const NAV: { id: PageId; label: string; group: string }[] = [
  { id: "dashboard", label: "Dashboard", group: "Live" },
  { id: "designer", label: "Dashboard designer", group: "Live" },
  { id: "cylinders", label: "Per-cylinder", group: "Live" },
  { id: "fuel", label: "Fuel map", group: "Maps" },
  { id: "target", label: "Target λ", group: "Maps" },
  { id: "ignition", label: "Ignition", group: "Maps" },
  { id: "mixture", label: "Mixture Map", group: "Calibrate" },
  { id: "setup", label: "Setup", group: "Calibrate" },
  { id: "harness", label: "Harness", group: "Hardware" },
  { id: "diagnostics", label: "Diagnostics", group: "Service" },
  { id: "calibration", label: "Calibration", group: "Service" },
  { id: "logs", label: "Logs", group: "Service" },
  { id: "logviewer", label: "Log viewer", group: "Service" },
  { id: "warnings", label: "Warning limits", group: "Service" },
  { id: "channels", label: "Channels & sensors", group: "Service" },
  { id: "copilot", label: "Copilot", group: "Assist" },
];

const PAGE_META: Record<PageId, { title: string; subtitle: string }> = {
  dashboard: { title: "Live engine overview", subtitle: "Virtual ECU telemetry, protection state and deterministic simulation evidence." },
  designer: { title: "Dashboard designer", subtitle: "Arrange gauges into a layout, then save or load it as JSON." },
  cylinders: { title: "Per-cylinder view", subtitle: "Per-cylinder channels for 1 to 8 cylinders. Changing the count affects this view only." },
  fuel: { title: "Volumetric efficiency", subtitle: "Load × RPM fuel table. Edit locally, then seal an auditable revision." },
  target: { title: "Target lambda", subtitle: "Desired mixture by load and RPM, with explicit fuel stoichiometry." },
  ignition: { title: "Ignition timing", subtitle: "Simulation-only surface. Lambda provides no evidence of safe spark advance." },
  mixture: { title: "Automated fuel calibration", subtitle: "Gate, bound, review and approve every fuel-cell correction." },
  setup: { title: "Engine configuration", subtitle: "Wizard-generated profiles, vehicle reference data and preflight validation." },
  harness: { title: "Harness & I/O", subtitle: "Bench connector map, test points and physical-use limits." },
  diagnostics: { title: "Diagnostics", subtitle: "DTCs, freeze frames, latching and explicit clear actions." },
  calibration: { title: "Calibration audit", subtitle: "Immutable snapshots, checksums, preflight evidence and rollback." },
  logs: { title: "Evidence & logs", subtitle: "Recorded telemetry, event trail, CSV export and replay." },
  logviewer: { title: "Log viewer", subtitle: "Load the current session, plot channels, read values at a cursor and export CSV." },
  warnings: { title: "Channel warning limits", subtitle: "Per-channel display thresholds. Not a control loop; they do not change ECU protection." },
  channels: { title: "Channels & sensors", subtitle: "Read-only list of channels and sensors, with missing sources made explicit." },
  copilot: { title: "Tuning Copilot", subtitle: "Read-only structured analysis with a hard human-approval boundary." },
};

export default function App() {
  const live = useLiveTelemetry();
  const [page, setPage] = useState<PageId>("dashboard");
  const [calibration, setCalibrationState] = useState<Calibration | null>(null);
  const [preflight, setPreflight] = useState<PreflightReport | null>(null);
  const [scenarios, setScenarios] = useState<Scenario[]>([]);
  const [running, setRunning] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [query, setQuery] = useState("");

  const refreshStatus = async () => {
    try {
      const status = await api.status();
      setRunning(status.running);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  const reloadCalibration = async () => {
    const result = await api.calibration();
    setCalibrationState(result.calibration);
    setPreflight(result.preflight);
  };

  const setCalibration = (cal: Calibration, pre?: PreflightReport) => {
    setCalibrationState(cal);
    if (pre) setPreflight(pre);
  };

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const [cal, sc, st] = await Promise.all([api.calibration(), api.scenarios(), api.status()]);
        if (cancelled) return;
        setCalibrationState(cal.calibration);
        setPreflight(cal.preflight);
        setScenarios(sc.scenarios);
        setRunning(st.running);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    })();
    const id = window.setInterval(() => {
      api.status().then((s) => !cancelled && setRunning(s.running)).catch(() => undefined);
    }, 3000);
    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, []);

  const startScenario = async (id: string) => {
    setBusy(true);
    setError(null);
    try {
      await api.start(id, false);
      setRunning(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const pause = async () => {
    setBusy(true);
    try {
      await api.pause();
      setRunning(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const resume = async () => {
    setBusy(true);
    try {
      await api.resume();
      setRunning(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const context = useMemo<AppContextValue>(
    () => ({
      calibration,
      preflight,
      scenarios,
      live,
      running,
      busy,
      error,
      page,
      setPage,
      setError,
      setCalibration,
      reloadCalibration,
      refreshStatus,
      startScenario,
      pause,
      resume,
    }),
    [calibration, preflight, scenarios, live, running, busy, error, page],
  );

  const meta = PAGE_META[page];
  const group = NAV.find((n) => n.id === page)?.group ?? "Tuning Studio";
  const visibleNav = query.trim()
    ? NAV.filter((n) => `${n.label} ${n.group}`.toLowerCase().includes(query.trim().toLowerCase()))
    : NAV;

  return (
    <AppContext.Provider value={context}>
      <div className="app-shell">
        <header className="masthead">
          <div className="masthead-inner">
            <button className="brand" onClick={() => setPage("dashboard")} aria-label="OpenTech dashboard">
              <span className="brand-mark">OT</span>
              <span className="brand-text">
                <strong>OpenTech</strong>
                <span>ECU Tuning Studio</span>
              </span>
            </button>
            <div className="masthead-tools">
              <span className={`header-live ${live.connected ? "on" : "off"}`}>
                <i />
                {live.connected ? "Service connected" : "Reconnecting"}
              </span>
              <span className="header-sep" />
              <span className={`header-run ${running ? "on" : "off"}`}>{running ? "Engine running" : "Engine paused"}</span>
              {calibration && <span className="header-rev">rev {calibration.revision}</span>}
              <Badge tone="danger">Virtual only</Badge>
            </div>
          </div>
        </header>

        <div className="nav-band">
          <div className="nav-band-inner">
            <nav className="primary-nav" aria-label="Tuning studio sections">
              {visibleNav.map((item) => (
                <button
                  key={item.id}
                  className={`nav-tab ${page === item.id ? "active" : ""}`}
                  onClick={() => setPage(item.id)}
                >
                  {item.label}
                </button>
              ))}
            </nav>
            <label className="nav-search">
              <svg viewBox="0 0 20 20" aria-hidden="true">
                <circle cx="8.5" cy="8.5" r="5.5" fill="none" stroke="currentColor" strokeWidth="1.8" />
                <path d="m12.8 12.8 4 4" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
              </svg>
              <input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search pages"
                aria-label="Search pages"
              />
            </label>
          </div>
        </div>

        <main className="main">
          <div className="content">
            <div className="page-head">
              <div>
                <div className="breadcrumb">
                  Tuning Studio <span>/</span> {group} <span>/</span> {meta.title}
                </div>
                <h1>{meta.title}</h1>
                <p>{meta.subtitle}</p>
              </div>
              <div className="page-head-status">
                <span className={`status-pill ${running ? "live" : ""}`}>{running ? "RUNNING" : "PAUSED"}</span>
                {preflight && (
                  <span className={`status-pill ${preflight.virtual_run_allowed ? "ok" : "bad"}`}>
                    {preflight.virtual_run_allowed ? "VIRTUAL PREFLIGHT OK" : "PREFLIGHT BLOCKED"}
                  </span>
                )}
                <span className="status-pill muted-pill">GATE 0</span>
              </div>
            </div>

            {error && (
              <div className="global-error">
                <span>{error}</span>
                <button onClick={() => setError(null)}>dismiss</button>
              </div>
            )}

            <WarningBanner />

            {page === "dashboard" && <DashboardPage />}
            {page === "designer" && <DashboardDesignerPage />}
            {page === "cylinders" && <CylindersPage />}
            {page === "fuel" && <MapPage tableId="ve" kind="fuel" title="Volumetric efficiency" subtitle="Load × RPM fuel table. Edit locally, then seal an auditable revision." />}
            {page === "target" && <MapPage tableId="target_lambda" kind="target" title="Target lambda" subtitle="Desired mixture by load and RPM. Fuel-specific only through explicit stoichiometric AFR." />}
            {page === "ignition" && <MapPage tableId="ignition" kind="ignition" title="Ignition timing" subtitle="Simulation-only surface. Lambda provides no evidence of safe spark advance." />}
            {page === "mixture" && <MixturePage />}
            {page === "setup" && <SetupPage />}
            {page === "harness" && <HarnessPage />}
            {page === "diagnostics" && <DiagnosticsPage />}
            {page === "calibration" && <CalibrationPage />}
            {page === "logs" && <LogsPage />}
            {page === "logviewer" && <LogViewerPage />}
            {page === "warnings" && <WarningsPage />}
            {page === "channels" && <ChannelsPage />}
            {page === "copilot" && <CopilotPage />}

            <footer className="app-footer">
              <strong>OpenTech-ECU</strong> is an independent Rust platform based on Speeduino. Speeduino C++ is
              retained as the GPL-2.0 behavioural oracle. Not affiliated with Mitsubishi, Haltech, Link or any OEM. Bench data only;
              no vehicle or engine validation. Not engine-safe.
            </footer>
          </div>
        </main>
      </div>
    </AppContext.Provider>
  );
}
