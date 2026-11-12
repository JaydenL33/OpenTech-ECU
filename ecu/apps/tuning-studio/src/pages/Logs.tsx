import { useState } from "react";
import { api, csvUrl } from "../api";
import { Panel, Badge, Stat } from "../components/ui";
import type { LogFile, SessionSummary } from "../types";

export function LogsPage() {
  const [log, setLog] = useState<LogFile | null>(null);
  const [summary, setSummary] = useState<SessionSummary | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [replayMessage, setReplayMessage] = useState<string | null>(null);

  const load = async () => {
    setBusy(true);
    try {
      const [l, s] = await Promise.all([api.log(), api.logSummary()]);
      setLog(l);
      setSummary(s);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="page">
      <Panel
        title="Recorded session log"
        subtitle="Deterministic telemetry evidence with calibration checksum and event trail"
        actions={
          <div className="button-row">
            <button className="button primary" onClick={load} disabled={busy}>{busy ? "Loading…" : "Load current log"}</button>
            <button
              className="button ghost"
              onClick={async () => {
                try {
                  const r = await api.replay("current");
                  setReplayMessage(`Replaying ${r.frames} frames through the live WebSocket. Open Dashboard to watch.`);
                } catch (e) {
                  setError(e instanceof Error ? e.message : String(e));
                }
              }}
            >
              Replay current log
            </button>
            <a className="button ghost" href={csvUrl()} download="opentech-session.csv">Download CSV</a>
          </div>
        }
      >
        {error && <div className="alert danger">{error}</div>}
        {replayMessage && <div className="alert info">{replayMessage}</div>}
        {!log && <div className="empty-state"><p>No log loaded. The virtual ECU records at 50 Hz.</p></div>}
        {log && summary && (
          <>
            <div className="stat-grid four">
              <Stat label="Scenario" value={log.metadata.scenario_name} />
              <Stat label="Frames" value={summary.samples} />
              <Stat label="Duration" value={(summary.duration_ms / 1000).toFixed(1)} unit="s" />
              <Stat label="Mean RPM" value={summary.mean_rpm.toFixed(0)} />
              <Stat label="Max RPM" value={summary.max_rpm.toFixed(0)} />
              <Stat label="Max coolant" value={summary.max_coolant_c.toFixed(1)} unit="°C" />
              <Stat label="Lambda samples" value={summary.lambda_samples} />
              <Stat label="Protection cuts" value={summary.protection_cut_samples} tone={summary.protection_cut_samples ? "warn" : ""} />
            </div>
            <div className="log-meta">
              <span>Seed <b>{log.metadata.seed}</b></span>
              <span>Calibration <b className="mono">{log.metadata.calibration_id}</b> rev <b>{log.metadata.calibration_revision}</b></span>
              <span>Checksum <b className="mono">{log.metadata.calibration_checksum.slice(0, 16)}…</b></span>
              <span>Sample rate <b>{log.metadata.sample_hz} Hz</b></span>
            </div>
            <div className="mixture-columns">
              <div>
                <h3>Events</h3>
                <div className="event-list">
                  {log.events.slice(-40).reverse().map((e, i) => (
                    <div className="event-row" key={`${e.timestamp_us}-${i}`}>
                      <span className="mono">{(e.timestamp_us / 1_000_000).toFixed(2)}s</span>
                      <Badge tone={e.severity === "error" ? "danger" : e.severity === "warning" ? "warn" : "neutral"}>{e.severity}</Badge>
                      <span className="mono">{e.code}</span>
                      <span>{e.message}</span>
                    </div>
                  ))}
                  {log.events.length === 0 && <p className="muted">No events recorded.</p>}
                </div>
              </div>
              <div>
                <h3>Recent frames</h3>
                <div className="table-scroll">
                  <table className="data-table compact">
                    <thead><tr><th>t</th><th>RPM</th><th>MAP</th><th>λ</th><th>STFT</th><th>Adv</th><th>PW</th><th>Faults</th></tr></thead>
                    <tbody>
                      {log.frames.slice(-12).reverse().map((f) => (
                        <tr key={f.seq}>
                          <td>{(f.scenario_time_ms / 1000).toFixed(2)}</td>
                          <td>{f.rpm.toFixed(0)}</td>
                          <td>{f.map_kpa.toFixed(0)}</td>
                          <td>{f.lambda_ready ? f.lambda_measured.toFixed(3) : "—"}</td>
                          <td>{(f.stft * 100).toFixed(1)}%</td>
                          <td>{f.ignition_advance_deg.toFixed(1)}</td>
                          <td>{f.injector_pw_ms.toFixed(2)}</td>
                          <td>{f.active_faults.join(",") || "—"}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          </>
        )}
        <p className="hint">Logs are deterministic for the same scenario, seed and calibration. CSV includes the full frame schema; events and freeze frames remain in JSON/Diagnostics.</p>
      </Panel>
    </div>
  );
}
