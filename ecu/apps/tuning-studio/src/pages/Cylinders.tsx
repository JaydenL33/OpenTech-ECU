import { useEffect, useState } from "react";
import { useApp } from "../context";
import { Badge, Panel, Stat } from "../components/ui";

const STORAGE_KEY = "opentech.cylinders.view.v1";
const MISSING = "no per-cylinder data from the service yet";

function readCount(): number | null {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const value = Number(raw);
    return Number.isInteger(value) && value >= 1 && value <= 8 ? value : null;
  } catch {
    return null;
  }
}

export function CylindersPage() {
  const app = useApp();
  const frame = app.live.frame;
  const [count, setCount] = useState<number>(() => readCount() ?? 4);
  const [stored, setStored] = useState<boolean>(() => readCount() !== null);

  useEffect(() => {
    if (!stored && app.calibration) {
      setCount(Math.max(1, Math.min(8, app.calibration.engine.cylinders)));
    }
  }, [app.calibration, stored]);

  const updateCount = (value: number) => {
    const next = Math.max(1, Math.min(8, value));
    setCount(next);
    setStored(true);
    try {
      window.localStorage.setItem(STORAGE_KEY, String(next));
    } catch {
      // Browser storage can be unavailable; the view still works for this session.
    }
  };

  const firingOrder = app.calibration?.engine.firing_order ?? [];
  const cylinders = Array.from({ length: count }, (_, index) => index + 1);

  return (
    <div className="page">
      <Panel
        title="Per-cylinder view"
        subtitle={`${count} cylinder${count === 1 ? "" : "s"} · one row per cylinder · view setting only`}
        actions={
          <label className="cylinder-count">
            Cylinders
            <select value={count} onChange={(event) => updateCount(Number(event.target.value))}>
              {[1, 2, 3, 4, 5, 6, 7, 8].map((value) => (
                <option key={value} value={value}>{value}</option>
              ))}
            </select>
          </label>
        }
      >
        <div className="alert info">
          <strong>BENCH PROTOTYPE · SIMULATOR DATA.</strong> The current service telemetry frame reports
          engine-wide injector channels only. It has no per-cylinder injector, trim or exhaust-temperature
          source, so every cylinder row below is explicitly marked as missing data. This panel is not
          connected to physical injectors or ignition hardware.
        </div>
        <div className="status-strip cylinder-strip">
          <Badge tone="warn">per-cylinder source: not connected</Badge>
          <Badge tone="neutral">view: {count} cylinders</Badge>
          {app.calibration && (
            <Badge tone="info">calibration: {app.calibration.engine.cylinders} cylinders configured</Badge>
          )}
          {firingOrder.length > 0 && (
            <span className="mono muted">firing order {firingOrder.join("-")}</span>
          )}
        </div>
        <div className="table-scroll">
          <table className="data-table cylinder-table">
            <thead>
              <tr>
                <th>Cylinder</th>
                <th>Firing order position</th>
                <th>Injector pulse width</th>
                <th>Fuel trim</th>
                <th>Exhaust temperature</th>
                <th>Source state</th>
              </tr>
            </thead>
            <tbody>
              {cylinders.map((cylinder) => {
                const position = firingOrder.indexOf(cylinder);
                return (
                  <tr key={cylinder} className="cylinder-row">
                    <td className="cylinder-number">Cyl {cylinder}</td>
                    <td>{position >= 0 ? position + 1 : "—"}</td>
                    <td>—</td>
                    <td>—</td>
                    <td>—</td>
                    <td><span className="missing-source">{MISSING}</span></td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
        <p className="hint">
          Changing the cylinder count changes this view only. It does not write to the service or alter the
          calibration. The simulator's firing order is {firingOrder.length ? firingOrder.join("-") : "not loaded"}.
        </p>
      </Panel>

      <Panel
        title="Engine-wide reference"
        subtitle="Values the service does report, shared across all cylinders"
      >
        <div className="stat-grid four">
          <Stat label="Engine speed" value={frame ? frame.rpm.toFixed(0) : "—"} unit="rpm" />
          <Stat label="Injector pulse width" value={frame ? frame.injector_pw_ms.toFixed(3) : "—"} unit="ms" />
          <Stat label="Injector duty" value={frame ? frame.injector_duty_pct.toFixed(1) : "—"} unit="%" />
          <Stat label="Dwell" value={frame ? frame.dwell_ms.toFixed(2) : "—"} unit="ms" />
          <Stat label="Manifold pressure" value={frame ? frame.map_kpa.toFixed(0) : "—"} unit="kPa" />
          <Stat label="Measured lambda" value={frame?.lambda_ready ? frame.lambda_measured.toFixed(3) : "—"} unit="λ" />
          <Stat label="STFT" value={frame ? (frame.stft * 100).toFixed(1) : "—"} unit="%" />
          <Stat label="Sync" value={frame?.sync ?? "—"} />
        </div>
        <p className="hint">
          These are engine-wide or bank-wide values. Do not read them as individual cylinder measurements.
          Individual cylinder fuel trims and EGT require additional sensors and a service endpoint that does
          not exist in this build.
        </p>
      </Panel>
    </div>
  );
}
