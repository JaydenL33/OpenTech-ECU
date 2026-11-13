import { useEffect, useState } from "react";
import { api } from "../api";
import { useApp } from "../context";
import { Badge, Panel } from "../components/ui";
import { AuditTrail } from "../components/AuditTrail";

export function CalibrationPage() {
  const app = useApp();
  const [history, setHistory] = useState<{ revision: number; checksum: string; id: string; current: boolean; created_unix_ms: number }[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const loadHistory = () =>
    api.history().then((r) => setHistory(r.revisions)).catch((e) => setError(String(e)));

  useEffect(() => {
    loadHistory();
  }, []);

  const undo = async () => {
    setBusy(true);
    try {
      if (app.running) await app.pause();
      const result = await api.undo();
      app.setCalibration(result.calibration, result.preflight);
      await loadHistory();
      await app.refreshStatus();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const download = () => {
    if (!app.calibration) return;
    const blob = new Blob([JSON.stringify(app.calibration, null, 2)], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${app.calibration.id}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  if (!app.calibration) return <div className="page"><p className="muted">Loading calibration…</p></div>;
  const cal = app.calibration;
  const pre = app.preflight;

  return (
    <div className="page">
      <Panel
        title="Calibration & preflight"
        subtitle="Immutable snapshots, checksummed revisions and explicit rollback"
        actions={
          <div className="button-row">
            <button className="button ghost" onClick={download}>Export JSON</button>
            <button className="button ghost" onClick={undo} disabled={busy || history.length < 2}>Rollback</button>
          </div>
        }
      >
        {error && <div className="alert danger">{error}</div>}
        <div className="cal-summary">
          <div><span>Calibration ID</span><b className="mono">{cal.id}</b></div>
          <div><span>Revision</span><b>{cal.revision}</b></div>
          <div><span>Schema</span><b>{cal.schema_version}</b></div>
          <div><span>Checksum</span><b className="mono">{cal.checksum.slice(0, 24)}…</b></div>
          <div><span>Engine</span><b>{cal.engine.name} · {cal.engine.cylinders} cyl · {cal.engine.displacement_cc.toFixed(0)} cc</b></div>
          <div><span>Firing order</span><b>{cal.engine.firing_order.join("-")}</b></div>
          <div><span>Rev limit</span><b>{cal.engine.rev_limit} rpm</b></div>
          <div><span>Ignition policy</span><b>{cal.ignition_policy.kind.replaceAll("_", " ")}</b></div>
        </div>
        {pre && (
          <div className="preflight-list">
            {pre.checks.map((check) => (
              <div className={`preflight-check ${check.status}`} key={check.id}>
                <Badge tone={check.status === "fail" ? "danger" : check.status === "warn" ? "warn" : check.status === "pass" ? "good" : "neutral"}>
                  {check.status}
                </Badge>
                <strong>{check.title}</strong>
                <span>{check.detail}</span>
                {check.blocks_physical && check.status === "fail" && <em>blocks physical</em>}
              </div>
            ))}
          </div>
        )}
        <div className="alert danger">
          Physical export is {pre?.physical_export_allowed ? "technically eligible but still unreviewed" : "blocked"}. No physical
          injector or ignition output path exists in this build. Passing preflight is not an engine-safety approval.
        </div>
      </Panel>
      <Panel title="Revision history" subtitle="Applied Mixture Map changes and setup revisions">
        <div className="table-scroll">
          <table className="data-table">
            <thead><tr><th>Revision</th><th>ID</th><th>Checksum</th><th>Created</th><th>State</th></tr></thead>
            <tbody>
              {history.map((h) => (
                <tr key={h.checksum}>
                  <td>{h.revision}</td>
                  <td className="mono">{h.id}</td>
                  <td className="mono">{h.checksum.slice(0, 20)}…</td>
                  <td>{h.created_unix_ms ? new Date(h.created_unix_ms).toLocaleString() : "deterministic seed"}</td>
                  <td>{h.current ? <Badge tone="good">current</Badge> : <Badge tone="neutral">snapshot</Badge>}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <p className="hint">Rollback requires the engine to be paused. The current snapshot is discarded from the active history, not silently merged.</p>
      </Panel>
      <AuditTrail refreshKey={cal.checksum} />
      <Panel title="Safety notes recorded with this calibration">
        <ul className="notes">
          {cal.notes.map((n, i) => <li key={i}>{n}</li>)}
        </ul>
      </Panel>
    </div>
  );
}
