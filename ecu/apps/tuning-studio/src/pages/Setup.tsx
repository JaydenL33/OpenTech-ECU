import { useEffect, useState } from "react";
import { api } from "../api";
import { useApp } from "../context";
import type { SetupForm, VehicleProfile } from "../types";
import { Panel, Badge } from "../components/ui";

export function SetupPage() {
  const app = useApp();
  const [form, setForm] = useState<SetupForm | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [evoBusy, setEvoBusy] = useState(false);
  const [evoMessage, setEvoMessage] = useState<string | null>(null);
  const [vehicles, setVehicles] = useState<VehicleProfile[]>([]);
  const [selectedVehicle, setSelectedVehicle] = useState<string>("mitsubishi-evo8-jdm-gsr-6mt");

  useEffect(() => {
    api.setupForm().then(setForm).catch((e) => setError(String(e)));
    api.vehicles().then((r) => setVehicles(r.vehicles)).catch(() => undefined);
  }, []);

  const vehicle = vehicles.find((v) => v.id === selectedVehicle);

  const loadEvoSimulation = async () => {
    if (!window.confirm("Load the Evo VIII GSR SIMULATION-ONLY template? Its pinout, trigger, injector and ignition data are unverified. It cannot be used for a physical car.")) return;
    setEvoBusy(true);
    setError(null);
    try {
      const result = await api.loadEvoSimulation();
      app.setCalibration(result.calibration, result.preflight);
      setEvoMessage(result.message);
      await app.refreshStatus();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setEvoBusy(false);
    }
  };

  if (!form) {
    return <div className="page"><Panel title="Engine setup wizard">{error ? <div className="alert danger">{error}</div> : <p className="muted">Loading defaults…</p>}</Panel></div>;
  }

  const set = <K extends keyof SetupForm>(key: K, value: SetupForm[K]) =>
    setForm((f) => (f ? { ...f, [key]: value } : f));

  const generate = async () => {
    if (!window.confirm("Replace the current calibration with a newly generated starter calibration? All generated maps are unvalidated estimates.")) return;
    setBusy(true);
    setError(null);
    try {
      const result = await api.setupGenerate(form);
      app.setCalibration(result.calibration, result.preflight);
      setMessage(`Generated calibration ${result.calibration.id} revision ${result.calibration.revision}. Virtual engine paused; physical use remains blocked.`);
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
        title="Engine setup wizard"
        subtitle="Configurable engine profile → versioned calibration → preflight validation"
        actions={<Badge tone="warn">generated maps are UNVALIDATED</Badge>}
      >
        <div className="alert danger">
          <strong>Mitsubishi Evo VIII GSR is the primary reference vehicle, but its exact ECU pinout, trigger
          pattern and connector families are not yet verified.</strong> Do not connect this software to the car
          or a fabricated board. The Evo profile is deliberately non-operational until the AUDIT verification
          matrix is completed against the physical vehicle.
        </div>
        <div className="reference-vehicle">
          <div>
            <h3>Reference vehicle</h3>
            <p>Vehicle → manufacturer → model → variant → engine → transmission</p>
            <select value={selectedVehicle} onChange={(e) => setSelectedVehicle(e.target.value)}>
              {vehicles.map((v) => (
                <option key={v.id} value={v.id}>
                  {v.make} → {v.model} → {v.market} {v.variant} → {v.engine} → {v.transmission}
                </option>
              ))}
            </select>
            {vehicle && (
              <p className="vehicle-detail">
                {vehicle.chassis} · {vehicle.aspiration} · verification: <b>{vehicle.verification}</b>
              </p>
            )}
          </div>
          <div className="button-row">
            <Badge tone="danger">GATE 0 ONLY</Badge>
            <button className="button ghost" onClick={loadEvoSimulation} disabled={evoBusy}>
              {evoBusy ? "Loading…" : "Load simulation profile"}
            </button>
          </div>
        </div>
        {vehicle && (
          <div className="alert warn">
            <strong>Blocking unknowns ({vehicle.unverified.length}):</strong>{" "}
            {vehicle.unverified.slice(0, 6).map((u) => u.id).join(", ")}
            {vehicle.unverified.length > 6 ? "…" : ""}. Pinout documented:{" "}
            {vehicle.pinout_documented ? "yes" : "NO"}; trigger verified:{" "}
            {vehicle.trigger_verified ? "yes" : "NO"}; injector data verified:{" "}
            {vehicle.injector_data_verified ? "yes" : "NO"}. See AUDIT/ECU_PINOUT_VERIFICATION.md.
          </div>
        )}
        {evoMessage && <div className="alert info">{evoMessage}</div>}
        <div className="setup-grid">
          <label>Vehicle name<input value={form.vehicle_name} onChange={(e) => set("vehicle_name", e.target.value)} /></label>
          <label>Engine name<input value={form.engine_name} onChange={(e) => set("engine_name", e.target.value)} /></label>
          <label>Displacement (cc)<input type="number" value={form.displacement_cc} onChange={(e) => set("displacement_cc", Number(e.target.value))} /></label>
          <label>Cylinders<input type="number" min={1} max={12} value={form.cylinders} onChange={(e) => set("cylinders", Number(e.target.value))} /></label>
          <label>Firing order<input value={form.firing_order.join(",")} onChange={(e) => set("firing_order", e.target.value.split(",").map((v) => Number(v.trim())).filter(Number.isFinite))} /></label>
          <label>Compression ratio<input type="number" step={0.1} value={form.compression_ratio} onChange={(e) => set("compression_ratio", Number(e.target.value))} /></label>
          <label>Aspiration
            <select value={form.aspiration} onChange={(e) => set("aspiration", e.target.value)}>
              <option value="naturally_aspirated">Naturally aspirated</option>
              <option value="turbocharged">Turbocharged</option>
              <option value="supercharged">Supercharged</option>
            </select>
          </label>
          <label>Rev limit (rpm)<input type="number" value={form.rev_limit} onChange={(e) => set("rev_limit", Number(e.target.value))} /></label>
          <label>Idle target (rpm)<input type="number" value={form.idle_rpm} onChange={(e) => set("idle_rpm", Number(e.target.value))} /></label>
          <label>Injector flow (cc/min)<input type="number" value={form.injector_flow_cc_min} onChange={(e) => set("injector_flow_cc_min", Number(e.target.value))} /></label>
          <label>Injector dead time (ms @13.8 V)<input type="number" step={0.01} value={form.injector_dead_time_ms} onChange={(e) => set("injector_dead_time_ms", Number(e.target.value))} /></label>
          <label>Rated pressure (kPa)<input type="number" value={form.injector_rated_pressure_kpa} onChange={(e) => set("injector_rated_pressure_kpa", Number(e.target.value))} /></label>
          <label>Stoichiometric AFR<input type="number" step={0.1} value={form.stoich_afr} onChange={(e) => set("stoich_afr", Number(e.target.value))} /></label>
          <label>Fuel density (g/cc)<input type="number" step={0.001} value={form.fuel_density_g_cc} onChange={(e) => set("fuel_density_g_cc", Number(e.target.value))} /></label>
        </div>
        <div className="button-row">
          <button className="button primary" onClick={generate} disabled={busy}>
            {busy ? "Generating…" : "Generate starter calibration"}
          </button>
        </div>
        {message && <div className="alert info">{message}</div>}
        {error && <div className="alert danger">{error}</div>}
        {app.preflight && (
          <div className="preflight-summary">
            <Badge tone={app.preflight.virtual_run_allowed ? "good" : "danger"}>
              virtual run: {app.preflight.virtual_run_allowed ? "allowed" : "blocked"}
            </Badge>
            <Badge tone="danger">physical export: blocked</Badge>
            <span>{app.preflight.summary}</span>
          </div>
        )}
        <p className="hint">
          The wizard generates estimated VE and target-lambda tables, a synthetic simulator-only ignition
          surface, dwell data and a sealed checksummed document. It does not create a running-ready tune.
          Injector characterisation and sensor transfer functions must be verified independently.
        </p>
      </Panel>
    </div>
  );
}
