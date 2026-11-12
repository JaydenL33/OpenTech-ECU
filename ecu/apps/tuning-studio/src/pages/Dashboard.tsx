import { useEffect, useState } from "react";
import { useApp } from "../context";
import { Panel, Gauge, Stat, Badge, SummaryCard, SummaryRow, MoreLink } from "../components/ui";
import { TraceChart, MiniTrace } from "../components/charts";

export function DashboardPage() {
  const app = useApp();
  const frame = app.live.frame;
  const history = app.live.history ?? [];
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
  const ignitionPolicy = app.calibration?.ignition_policy.kind.replaceAll("_", " ") ?? "loading";

  return (
    <div className="page dashboard">
      <div className="summary-grid">
        <SummaryCard
          icon={<span className="summary-glyph">λ</span>}
          title="Engine & trigger"
          subtitle={frame ? `${frame.engine_state} · ${frame.sync} sync` : "Waiting for telemetry"}
          action={<MoreLink onClick={() => app.setPage("diagnostics")}>Open diagnostics</MoreLink>}
        >
          <SummaryRow label="Engine speed" value={frame ? `${frame.rpm.toFixed(0)} rpm` : "—"} />
          <SummaryRow label="Protection" value={<Badge tone={protectionTone}>{frame?.protection ?? "—"}</Badge>} />
          <SummaryRow label="Active DTCs" value={frame?.active_faults?.length ?? 0} tone={(frame?.active_faults?.length ?? 0) > 0 ? "warn" : "good"} />
          <SummaryRow label="Crank angle" value={frame ? `${frame.cycle_angle_deg.toFixed(1)}°` : "—"} />
          <SummaryRow label="Scenario" value={app.scenarios.find((s) => s.id === scenarioId)?.name ?? "—"} />
        </SummaryCard>

        <SummaryCard
          icon={<span className="summary-glyph">F</span>}
          title="Fuel & lambda"
          subtitle="Closed-loop state and injector delivery"
          action={<MoreLink onClick={() => app.setPage("mixture")}>Review Mixture Map</MoreLink>}
        >
          <SummaryRow label="Measured λ" value={frame?.lambda_ready ? frame.lambda_measured.toFixed(3) : "not ready"} tone={lambdaTone === "warn" ? "warn" : ""} />
          <SummaryRow label="Target λ" value={frame ? frame.lambda_target.toFixed(3) : "—"} />
          <SummaryRow label="STFT / LTFT" value={frame ? `${(frame.stft * 100).toFixed(1)}% / ${(frame.ltft * 100).toFixed(1)}%` : "—"} />
          <SummaryRow label="Injector duty" value={frame ? `${frame.injector_duty_pct.toFixed(0)}%` : "—"} tone={(frame?.injector_duty_pct ?? 0) > 90 ? "warn" : ""} />
          <SummaryRow label="Closed loop" value={frame?.closed_loop ? "active" : "off"} />
        </SummaryCard>

        <SummaryCard
          icon={<span className="summary-glyph">C</span>}
          title="Calibration"
          subtitle={app.calibration ? `${app.calibration.id} · revision ${app.calibration.revision}` : "Loading…"}
          action={<MoreLink onClick={() => app.setPage("calibration")}>Open calibration audit</MoreLink>}
        >
          <SummaryRow label="Engine" value={app.calibration?.engine.name ?? "—"} />
          <SummaryRow label="Checksum" value={<span className="mono">{app.calibration?.checksum.slice(0, 12) ?? "—"}…</span>} />
          <SummaryRow label="Ignition policy" value={ignitionPolicy} tone={app.calibration?.ignition_policy.kind === "verified_reference" ? "good" : "warn"} />
          <SummaryRow label="Virtual preflight" value={app.preflight?.virtual_run_allowed ? "allowed" : "blocked"} tone={app.preflight?.virtual_run_allowed ? "good" : "danger"} />
          <SummaryRow label="Physical export" value="interlocked" tone="danger" />
        </SummaryCard>
      </div>

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
        <Gauge label="Engine speed" value={frame?.rpm ?? null} min={0} max={8000} unit="rpm" decimals={0} color="#1d1d1d" />
        <Gauge label="Manifold pressure" value={frame?.map_kpa ?? null} min={0} max={260} unit="kPa" color="#2f6f9f" decimals={0} />
        <Gauge label="Measured lambda" value={frame?.lambda_ready ? frame.lambda_measured : null} min={0.6} max={1.3} unit="λ" color="#d19a00" decimals={2} warning={lambdaTone === "warn"} />
        <Gauge label="Target lambda" value={frame?.lambda_target ?? null} min={0.6} max={1.3} unit="λ" color="#7a5aa6" decimals={2} />
        <Gauge label="Spark advance" value={frame?.ignition_advance_deg ?? null} min={-10} max={45} unit="°" color="#c05a2a" decimals={1} />
        <Gauge label="Injector duty" value={frame?.injector_duty_pct ?? null} min={0} max={100} unit="%" color="#4b7f52" decimals={0} warning={(frame?.injector_duty_pct ?? 0) > 90} />
      </div>

      <div className="dashboard-grid">
        <Panel title="Live traces" subtitle="RPM · MAP · lambda vs target · advance, last 300 frames" className="wide">
          <div className="chart-block">
            <TraceChart
              frames={history}
              series={[
                { key: "rpm", label: "RPM", color: "#1d1d1d" },
                { key: "map_kpa", label: "MAP kPa", color: "#2f6f9f" },
              ]}
            />
          </div>
          <div className="chart-block">
            <TraceChart
              frames={history}
              series={[
                { key: "lambda_measured", label: "measured λ", color: "#d19a00" },
                { key: "lambda_target", label: "target λ", color: "#7a5aa6" },
                { key: "stft", label: "STFT fraction", color: "#c05a2a" },
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
          <MiniTrace frames={history} accessor={(f) => f.lambda_measured} color="#d19a00" />
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
