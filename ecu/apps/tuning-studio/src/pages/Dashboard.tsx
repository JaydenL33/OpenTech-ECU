import { useEffect, useState } from "react";
import { api } from "../api";
import { useApp } from "../context";
import { Panel, Gauge, Stat, Badge } from "../components/ui";
import { TraceChart, MiniTrace } from "../components/charts";
import { evaluateWarnings, useWarningLimits } from "../warnings";

export function DashboardPage() {
  const app = useApp();
  const frame = app.live.frame;
  const history = app.live.history ?? [];
  const warningLimits = useWarningLimits();
  const activeWarnings = evaluateWarnings(frame, warningLimits);
  const warningFor = (key: string) => activeWarnings.some((warning) => warning.key === key);
  const [scenarioId, setScenarioId] = useState("idle_warmup");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (app.scenarios.length && !app.scenarios.some((s) => s.id === scenarioId)) {
      setScenarioId(app.scenarios[0].id);
    }
  }, [app.scenarios, scenarioId]);

  const start = async () => {
    setBusy(true);
    try {
      await app.startScenario(scenarioId);
    } finally {
      setBusy(false);
    }
  };

  const protectionTone =
    frame?.protection === "cut"
      ? "danger"
      : frame?.protection === "derated" || frame?.protection === "limp"
        ? "warn"
        : frame?.protection === "warning"
          ? "warn"
          : "good";
  const lambdaTone =
    frame && frame.lambda_ready && Math.abs(frame.lambda_measured - frame.lambda_target) > 0.08
      ? "warn"
      : "good";

  return (
    <div className="page dashboard">
      {app.error && <div className="alert danger">{app.error}</div>}
      <Panel
        title="Virtual engine"
        subtitle={
          frame
            ? `Scenario ${frame.log_note || "running"} · t=${(frame.scenario_time_ms / 1000).toFixed(1)} s · ${frame.engine_state}`
            : "Waiting for telemetry from the Rust virtual ECU"
        }
        actions={
          <div className="scenario-controls">
            <select value={scenarioId} onChange={(e) => setScenarioId(e.target.value)}>
              {app.scenarios.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.name}
                </option>
              ))}
            </select>
            <button className="button primary" onClick={start} disabled={busy}>
              {busy ? "Starting…" : "Start scenario"}
            </button>
            {app.running ? (
              <button className="button ghost" onClick={() => app.pause()}>
                Pause
              </button>
            ) : (
              <button className="button ghost" onClick={() => app.resume()}>
                Resume
              </button>
            )}
          </div>
        }
      >
        {app.scenarios.find((s) => s.id === scenarioId)?.description && (
          <p className="scenario-description">
            {app.scenarios.find((s) => s.id === scenarioId)?.description}
          </p>
        )}
        <div className="status-strip">
          <Badge tone={app.live.connected ? "good" : "danger"}>
            {app.live.connected ? "live" : "disconnected"}
          </Badge>
          <Badge tone={frame?.sync === "sequential" ? "good" : "warn"}>
            sync: {frame?.sync ?? "—"}
          </Badge>
          <Badge tone={protectionTone}>protection: {frame?.protection ?? "—"}</Badge>
          <Badge tone={lambdaTone}>
            lambda: {frame?.lambda_ready ? "valid" : "not ready"}
          </Badge>
          <Badge tone={frame?.closed_loop ? "info" : "neutral"}>
            closed loop: {frame?.closed_loop ? "active" : "off"}
          </Badge>
          <span className="mono muted">
            {app.calibration ? `${app.calibration.id} rev ${app.calibration.revision}` : ""}
          </span>
        </div>
        {frame?.fuel_cut && <div className="alert warn">Fuel cut active — no injection events scheduled.</div>}
        {frame?.spark_cut && <div className="alert warn">Spark cut active.</div>}
      </Panel>

      <div className="gauge-row">
        <Gauge label="Engine speed" value={frame?.rpm ?? null} min={0} max={8000} unit="rpm" decimals={0} warning={warningFor("rpm")} />
        <Gauge label="Manifold pressure" value={frame?.map_kpa ?? null} min={0} max={260} unit="kPa" color="#6dd3c7" decimals={0} warning={warningFor("map_kpa")} />
        <Gauge label="Measured lambda" value={frame?.lambda_ready ? frame.lambda_measured : null} min={0.6} max={1.3} unit="λ" color="#e8c46a" decimals={2} warning={warningFor("lambda_measured") || lambdaTone === "warn"} />
        <Gauge label="Target lambda" value={frame?.lambda_target ?? null} min={0.6} max={1.3} unit="λ" color="#b49cf0" decimals={2} warning={warningFor("lambda_target")} />
        <Gauge label="Spark advance" value={frame?.ignition_advance_deg ?? null} min={-10} max={45} unit="°" color="#f08a5d" decimals={1} warning={warningFor("ignition_advance_deg")} />
        <Gauge label="Injector duty" value={frame?.injector_duty_pct ?? null} min={0} max={100} unit="%" color="#89b4fa" decimals={0} warning={warningFor("injector_duty_pct") || (frame?.injector_duty_pct ?? 0) > 90} />
      </div>

      <div className="dashboard-grid">
        <Panel title="Live traces" subtitle="RPM · MAP · lambda vs target · advance last 300 frames" className="wide">
          <div className="chart-block">
            <TraceChart
              frames={history}
              series={[
                { key: "rpm", label: "RPM", color: "#6da8ff" },
                { key: "map_kpa", label: "MAP kPa", color: "#5fd1c0" },
              ]}
            />
          </div>
          <div className="chart-block">
            <TraceChart
              frames={history}
              series={[
                { key: "lambda_measured", label: "measured λ", color: "#e8c46a" },
                { key: "lambda_target", label: "target λ", color: "#b49cf0" },
                { key: "stft", label: "STFT fraction", color: "#f08a5d" },
              ]}
              fixedMin={-0.25}
              fixedMax={1.35}
            />
          </div>
        </Panel>

        <Panel title="Fuel & air" subtitle="ECU outputs and strongly-typed sensor path">
          <div className="stat-grid">
            <Stat label="Injector PW" value={(frame?.injector_pw_ms ?? 0).toFixed(3)} unit="ms" />
            <Stat label="Dwell" value={(frame?.dwell_ms ?? 0).toFixed(2)} unit="ms" />
            <Stat label="VE cell" value={frame ? `${frame.ve.toFixed(1)}%` : "—"} />
            <Stat label="VE index" value={frame ? `${frame.ve_row},${frame.ve_col}` : "—"} />
            <Stat label="STFT" value={`${((frame?.stft ?? 0) * 100).toFixed(2)}%`} tone={Math.abs(frame?.stft ?? 0) > 0.2 ? "warn" : ""} />
            <Stat label="LTFT" value={`${((frame?.ltft ?? 0) * 100).toFixed(2)}%`} />
            <Stat label="Fuel pressure" value={(frame?.fuel_pressure_kpa ?? 0).toFixed(0)} unit="kPa" />
            <Stat label="Oil pressure" value={(frame?.oil_pressure_kpa ?? 0).toFixed(0)} unit="kPa" />
          </div>
        </Panel>

        <Panel title="Thermal & electrical" subtitle="Sensor values with quality state">
          <div className="stat-grid">
            <Stat label="Coolant" value={(frame?.clt_c ?? 0).toFixed(1)} unit="°C" />
            <Stat label="Intake" value={(frame?.iat_c ?? 0).toFixed(1)} unit="°C" />
            <Stat label="Battery" value={(frame?.battery_v ?? 0).toFixed(2)} unit="V" />
            <Stat label="Boost gauge" value={(frame?.boost_kpa ?? 0).toFixed(0)} unit="kPa" />
            <Stat label="Throttle" value={(frame?.tps_pct ?? 0).toFixed(1)} unit="%" />
            <Stat label="Crank angle" value={(frame?.cycle_angle_deg ?? 0).toFixed(1)} unit="°" />
            <Stat label="Lambda age" value={String(frame?.lambda_age_ms ?? "—")} unit="ms" />
            <Stat label="Wideband" value={frame?.wideband?.controller_id ?? "—"} />
          </div>
          <MiniTrace frames={history} accessor={(f) => f.lambda_measured} color="#e8c46a" />
        </Panel>

        <Panel title="Active faults" subtitle="Protection is independent of fuel learning" className="wide">
          {frame?.active_faults?.length ? (
            <div className="fault-list">
              {frame.active_faults.map((f) => (
                <Badge key={f} tone="danger">
                  {f}
                </Badge>
              ))}
            </div>
          ) : (
            <p className="muted">No active fault codes. Freeze frames are available in Diagnostics.</p>
          )}
          <p className="hint">
            Lambda is never used as evidence of safe ignition timing. The ignition table is a synthetic
            simulator surface until a verified engine-specific reference is imported.
          </p>
        </Panel>
      </div>
    </div>
  );
}
