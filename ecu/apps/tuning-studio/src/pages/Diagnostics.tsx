import { useEffect, useState } from "react";
import { api } from "../api";
import { useApp } from "../context";
import { Badge, Panel } from "../components/ui";
import type { ActiveFault } from "../types";

export function DiagnosticsPage() {
  const app = useApp();
  const [faults, setFaults] = useState<ActiveFault[]>([]);
  const [state, setState] = useState("—");
  const [definitions, setDefinitions] = useState<{ code: string; title: string }[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const load = async () => {
    try {
      const data = await api.faults();
      setFaults(data.active);
      setState(data.state);
      setDefinitions(data.definitions);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  useEffect(() => {
    load();
    const id = window.setInterval(load, 2000);
    return () => window.clearInterval(id);
  }, []);

  const clear = async () => {
    setBusy(true);
    try {
      await api.clearFaults();
      await load();
      await app.refreshStatus();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="page">
      <Panel
        title="Diagnostics & protection"
        subtitle="DTCs, freeze frames, latching and explicit clear actions"
        actions={
          <div className="button-row">
            <button className="button ghost" onClick={load}>Refresh</button>
            <button className="button primary" onClick={clear} disabled={busy}>Clear cleared-condition latches</button>
          </div>
        }
      >
        <div className="status-strip">
          <Badge tone={state === "cut" ? "danger" : state === "normal" ? "good" : "warn"}>{state}</Badge>
          <Badge tone={app.live.frame?.fuel_cut ? "danger" : "good"}>fuel cut: {app.live.frame?.fuel_cut ? "yes" : "no"}</Badge>
          <Badge tone={app.live.frame?.spark_cut ? "danger" : "good"}>spark cut: {app.live.frame?.spark_cut ? "yes" : "no"}</Badge>
          <span className="muted">Latched faults require the physical condition to clear and an explicit operator action.</span>
        </div>
        {error && <div className="alert danger">{error}</div>}
        {faults.length === 0 ? (
          <div className="empty-state"><p>No active faults in the virtual ECU.</p></div>
        ) : (
          <div className="fault-cards">
            {faults.map((f) => {
              const def = definitions.find((d) => d.code === (f.code ?? String(f.dtc).toUpperCase()));
              return (
                <article className="fault-card" key={`${String(f.dtc)}-${f.first_seen_us}`}>
                  <header>
                    <div>
                      <h3>{def?.title ?? String(f.dtc).replaceAll("_", " ")}</h3>
                      <span className="mono">{f.code ?? def?.code ?? String(f.dtc)}</span>
                    </div>
                    <div>
                      <Badge tone={f.latched ? "danger" : "warn"}>{f.latched ? "latched" : "non-latched"}</Badge>
                      <Badge tone={f.condition_active ? "danger" : "good"}>{f.condition_active ? "active" : "condition cleared"}</Badge>
                    </div>
                  </header>
                  <div className="freeze-frame">
                    <span>RPM <b>{f.freeze_frame.rpm.toFixed(0)}</b></span>
                    <span>MAP <b>{f.freeze_frame.map_kpa.toFixed(0)}</b> kPa</span>
                    <span>λ <b>{f.freeze_frame.lambda.toFixed(3)}</b></span>
                    <span>target <b>{f.freeze_frame.target_lambda.toFixed(3)}</b></span>
                    <span>coolant <b>{f.freeze_frame.coolant_c.toFixed(1)}</b> °C</span>
                    <span>oil <b>{f.freeze_frame.oil_pressure_kpa.toFixed(0)}</b> kPa</span>
                    <span>fuel <b>{f.freeze_frame.fuel_pressure_kpa.toFixed(0)}</b> kPa</span>
                    <span>duty <b>{f.freeze_frame.injector_duty_pct.toFixed(1)}</b> %</span>
                    <span>occurrences <b>{f.occurrences}</b></span>
                  </div>
                </article>
              );
            })}
          </div>
        )}
        <h3>DTC registry</h3>
        <div className="table-scroll">
          <table className="data-table">
            <thead><tr><th>Code</th><th>Description</th></tr></thead>
            <tbody>
              {definitions.map((d) => <tr key={d.code}><td className="mono">{d.code}</td><td>{d.title}</td></tr>)}
            </tbody>
          </table>
        </div>
        <p className="hint">
          Diagnostic thresholds in this build are simulator placeholders, not Mitsubishi Evo VIII engine limits.
          Independent review is required before any physical protection strategy is used.
        </p>
      </Panel>
    </div>
  );
}
