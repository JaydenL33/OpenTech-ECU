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
import { WarningsPage } from "./pages/Warnings";
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
  | "diagnostics"
  | "calibration"
  | "logs"
  | "logviewer"
  | "warnings"
  | "copilot";

const NAV: { id: PageId; label: string; group: string }[] = [
  { id: "dashboard", label: "Dashboard", group: "Live" },
  { id: "designer", label: "Dashboard designer", group: "Live" },
  { id: "cylinders", label: "Per-cylinder", group: "Live" },
  { id: "fuel", label: "Fuel / VE map", group: "Maps" },
  { id: "target", label: "Target lambda", group: "Maps" },
  { id: "ignition", label: "Ignition timing", group: "Maps" },
  { id: "mixture", label: "Mixture Map", group: "Calibrate" },
  { id: "setup", label: "Engine setup", group: "Calibrate" },
  { id: "diagnostics", label: "Diagnostics", group: "Service" },
  { id: "calibration", label: "Calibration audit", group: "Service" },
  { id: "logs", label: "Logs & evidence", group: "Service" },
  { id: "logviewer", label: "Log viewer", group: "Service" },
  { id: "warnings", label: "Warning limits", group: "Service" },
  { id: "copilot", label: "Tuning Copilot", group: "Assist" },
];

export default function App() {
  const live = useLiveTelemetry();
  const [page, setPage] = useState<PageId>("dashboard");
  const [calibration, setCalibrationState] = useState<Calibration | null>(null);
  const [preflight, setPreflight] = useState<PreflightReport | null>(null);
  const [scenarios, setScenarios] = useState<Scenario[]>([]);
  const [running, setRunning] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

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
      setError,
      setCalibration,
      reloadCalibration,
      refreshStatus,
      startScenario,
      pause,
      resume,
    }),
    [calibration, preflight, scenarios, live, running, busy, error],
  );

  const groups = ["Live", "Maps", "Calibrate", "Service", "Assist"];

  return (
    <AppContext.Provider value={context}>
      <div className="app-shell">
        <aside className="sidebar">
          <div className="brand">
            <div className="brand-mark">OE</div>
            <div>
              <strong>OpenTech</strong>
              <span>ECU Tuning Studio</span>
            </div>
          </div>
          <nav>
            {groups.map((group) => (
              <div className="nav-group" key={group}>
                <span className="nav-title">{group}</span>
                {NAV.filter((n) => n.group === group).map((item) => (
                  <button
                    key={item.id}
                    className={`nav-item ${page === item.id ? "active" : ""}`}
                    onClick={() => setPage(item.id)}
                  >
                    <span className={`nav-dot dot-${item.group.toLowerCase()}`} />
                    {item.label}
                  </button>
                ))}
              </div>
            ))}
          </nav>
          <div className="sidebar-footer">
            <Badge tone="danger">VIRTUAL ONLY</Badge>
            <p>Physical output interlock engaged. No injector or ignition hardware path exists in v0.1.</p>
            <p className="mono">protocol {live.protocol ?? "—"} · {live.serverVersion ?? ""}</p>
          </div>
        </aside>
        <main className="main">
          <header className="topbar">
            <div>
              <h1>{
                page === "designer" ? "Dashboard designer"
                : page === "cylinders" ? "Per-cylinder view"
                : page === "fuel" ? "Volumetric efficiency"
                : page === "target" ? "Target lambda"
                : page === "ignition" ? "Ignition timing"
                : page === "mixture" ? "Automated fuel calibration"
                : page === "setup" ? "Engine configuration"
                : page === "diagnostics" ? "Diagnostics"
                : page === "calibration" ? "Calibration audit"
                : page === "logs" ? "Evidence & logs"
                : page === "logviewer" ? "Log viewer"
                : page === "warnings" ? "Channel warning limits"
                : page === "copilot" ? "Tuning Copilot"
                : "Live engine overview"
              }</h1>
              <p>
                Rust virtual ECU · 4-cylinder simulation · {calibration ? `${calibration.engine.name}` : "loading"}
              </p>
            </div>
            <div className="topbar-status">
              <Badge tone="warn">BENCH PROTOTYPE · SIMULATOR</Badge>
              <span className={`live-dot ${live.connected ? "on" : "off"}`} />
              {live.connected ? "connected" : "reconnecting"}
              <span className={`run-dot ${running ? "on" : "off"}`} />
              {running ? "running" : "paused"}
              {calibration && <span className="mono">rev {calibration.revision}</span>}
            </div>
          </header>
          {error && <div className="global-error"><span>{error}</span><button onClick={() => setError(null)}>dismiss</button></div>}
          <WarningBanner />
          {page === "dashboard" && <DashboardPage />}
          {page === "designer" && <DashboardDesignerPage />}
          {page === "cylinders" && <CylindersPage />}
          {page === "fuel" && <MapPage tableId="ve" kind="fuel" title="Volumetric efficiency" subtitle="Load × RPM fuel table. Edit locally, then seal an auditable revision." />}
          {page === "target" && <MapPage tableId="target_lambda" kind="target" title="Target lambda" subtitle="Desired mixture by load and RPM. Fuel-specific only through explicit stoichiometric AFR." />}
          {page === "ignition" && <MapPage tableId="ignition" kind="ignition" title="Ignition timing" subtitle="Simulation-only surface. Lambda provides no evidence of safe spark advance." />}
          {page === "mixture" && <MixturePage />}
          {page === "setup" && <SetupPage />}
          {page === "diagnostics" && <DiagnosticsPage />}
          {page === "calibration" && <CalibrationPage />}
          {page === "logs" && <LogsPage />}
          {page === "logviewer" && <LogViewerPage />}
          {page === "warnings" && <WarningsPage />}
          {page === "copilot" && <CopilotPage />}
          <footer className="app-footer">
            OpenTech-ECU is an independent Rust platform based on Speeduino. Speeduino C++ is retained as the
            GPL-2.0 behavioural oracle. Not affiliated with Mitsubishi, Haltech or Link. Not engine-safe.
          </footer>
        </main>
      </div>
    </AppContext.Provider>
  );
}
