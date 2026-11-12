import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import { useApp } from "../context";
import type { HarnessManifest } from "../types";
import { Panel, Badge, SummaryCard, SummaryRow, MoreLink } from "../components/ui";

function BenchIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M4 8h16M6 8v10m12-10v10M8 13h8M9 8V6h6v2" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function CarIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M4 15h16m-2 0v2m-12-2v2M6 15l1.5-5h9L18 15M8 10V8h8v2" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
      <circle cx="8" cy="17" r="1.4" fill="currentColor" />
      <circle cx="16" cy="17" r="1.4" fill="currentColor" />
    </svg>
  );
}

function TestIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M5 4h9l5 5v11H5zM14 4v6h6M8 15l2 2 4-5" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function HarnessPage() {
  const app = useApp();
  const [manifest, setManifest] = useState<HarnessManifest | null>(null);
  const [issues, setIssues] = useState<{ code: string; detail: string }[]>([]);
  const [sourceNote, setSourceNote] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string>("power");

  useEffect(() => {
    api
      .harness()
      .then((result) => {
        setManifest(result.manifest);
        setIssues(result.issues);
        setSourceNote(result.source_note);
        setSelected(result.manifest.subsystems[0]?.id ?? "power");
      })
      .catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, []);

  const subsystem = useMemo(
    () => manifest?.subsystems.find((s) => s.id === selected),
    [manifest, selected],
  );
  const connectors = useMemo(
    () => manifest?.connectors.filter((c) => subsystem?.connectors.includes(c.reference)) ?? [],
    [manifest, subsystem],
  );

  return (
    <div className="page harness-page">
      {app.error && <div className="alert danger">{app.error}</div>}
      {error && <div className="alert danger">{error}</div>}

      <div className="summary-grid">
        <SummaryCard
          icon={<BenchIcon />}
          title="Bench harness"
          subtitle={manifest ? `${manifest.id} · ${manifest.connectors.length} connectors · ${manifest.connectors.reduce((n, c) => n + c.pins.length, 0)} pins` : "Loading schematic export…"}
          action={<MoreLink onClick={() => setSelected("injectors")}>Review injector sinks</MoreLink>}
        >
          <SummaryRow label="Status" value={<Badge tone="danger">{manifest?.status ?? "UNREVIEWED"}</Badge>} />
          <SummaryRow label="Physical use" value={manifest?.physical_use ?? "PROHIBITED"} tone="danger" />
          <SummaryRow label="Schematic ERC" value="0 violations · unreviewed" />
          <SummaryRow label="PCB DRC" value="14 violations · 499 unconnected" tone="danger" />
          <SummaryRow label="Provenance" value={manifest?.source.design ?? "hardware/kicad/evo8-bench-ecu"} />
        </SummaryCard>

        <SummaryCard
          icon={<CarIcon />}
          title="Reference target"
          subtitle="Mitsubishi Lancer Evolution VIII GSR"
          action={<MoreLink onClick={() => app.setPage("setup")}>Open vehicle profile</MoreLink>}
        >
          <SummaryRow label="Market / chassis" value="JDM CT9A · 4G63T · 6MT" />
          <SummaryRow label="Compatibility" value={<Badge tone="danger">UNVERIFIED</Badge>} />
          <SummaryRow label="ECU / connector" value="Not identified" tone="warn" />
          <SummaryRow label="Trigger / injectors" value="Not scoped / not measured" tone="warn" />
          <SummaryRow label="Vehicle use" value="BLOCKED — Gate 0 only" tone="danger" />
        </SummaryCard>

        <SummaryCard
          icon={<TestIcon />}
          title="Bench test readiness"
          subtitle="Unpowered and powered test plan"
          action={<MoreLink onClick={() => app.setPage("diagnostics")}>Open diagnostics</MoreLink>}
        >
          <SummaryRow label="Test status" value={<Badge tone="danger">NOT EXECUTED</Badge>} />
          <SummaryRow label="Emergency stop" value="External NC contacts" />
          <SummaryRow label="Injector loads" value="Dummy loads only" tone="warn" />
          <SummaryRow label="Ignition outputs" value="Logic-level only" tone="warn" />
          <SummaryRow label="Gate 1" value="Blocked — no reviewed hardware" tone="danger" />
        </SummaryCard>
      </div>

      {issues.length > 0 && (
        <div className="alert danger">
          <strong>Manifest validation failed.</strong>
          <ul>{issues.map((i) => <li key={`${i.code}-${i.detail}`}>{i.code}: {i.detail}</li>)}</ul>
        </div>
      )}

      <Panel
        title="Bench subsystems"
        subtitle="The connector names below are board-local. They are not Mitsubishi or any OEM cavity numbers."
        actions={<Badge tone="warn">UNREVIEWED SCHEMATIC EXPORT</Badge>}
      >
        <div className="harness-tabs" role="tablist" aria-label="Harness subsystems">
          {manifest?.subsystems.map((s) => (
            <button
              key={s.id}
              role="tab"
              aria-selected={selected === s.id}
              className={`harness-tab ${selected === s.id ? "active" : ""}`}
              onClick={() => setSelected(s.id)}
            >
              {s.name}
            </button>
          ))}
        </div>
        {subsystem && (
          <>
            <p className="harness-description">{subsystem.description}</p>
            <div className="connector-grid">
              {connectors.map((connector) => (
                <section className="connector-card" key={connector.reference}>
                  <header>
                    <div>
                      <h3>
                        {connector.reference} <span>{connector.label}</span>
                      </h3>
                      <p className="mono">{connector.mpn || "no MPN"}</p>
                    </div>
                    <Badge tone="neutral">{connector.pins.length} pins</Badge>
                  </header>
                  <div className="table-scroll">
                    <table className="data-table compact harness-pin-table">
                      <thead>
                        <tr>
                          <th>Pin</th>
                          <th>Net</th>
                          <th>Function / constraint</th>
                        </tr>
                      </thead>
                      <tbody>
                        {connector.pins.map((pin) => (
                          <tr key={pin.pin}>
                            <td className="mono">{pin.pin}</td>
                            <td className="mono">{pin.net}</td>
                            <td>{pin.function || "—"}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </section>
              ))}
            </div>
          </>
        )}
      </Panel>

      <Panel
        title="Test points"
        subtitle="Scope and measurement points from the bench schematic. Nothing here has been measured yet."
        actions={<Badge tone="danger">NOT EXECUTED</Badge>}
      >
        <div className="table-scroll">
          <table className="data-table compact">
            <thead>
              <tr>
                <th>Reference</th>
                <th>Net</th>
                <th>Purpose / measurement</th>
              </tr>
            </thead>
            <tbody>
              {manifest?.test_points.map((tp) => (
                <tr key={tp.reference}>
                  <td className="mono">{tp.reference}</td>
                  <td className="mono">{tp.net}</td>
                  <td>{tp.purpose}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Panel>

      <Panel title="Provenance & limits" subtitle="What this page does and does not prove">
        <div className="harness-provenance">
          <div>
            <h3>Source</h3>
            <p>{sourceNote || "OpenTech bench schematic export."}</p>
            <p className="mono">
              {manifest?.source.files.map((f) => `${f.path} · ${f.sha256.slice(0, 12)}…`).join("  |  ")}
            </p>
          </div>
          <div>
            <h3>What this page proves</h3>
            <p>
              The generated manifest matches the schematic's own connector and test-point exports. It is a
              documentation view, not a wiring approval or a measurement result.
            </p>
            <p><Badge tone="danger">UNREVIEWED</Badge></p>
          </div>
          <div>
            <h3>Do not</h3>
            <ul className="notes">
              <li>Do not use these pins to build a vehicle adapter harness.</li>
              <li>Do not assume the Evo VIII trigger, injector or ignition topology.</li>
              <li>Do not fabricate the PCB: DRC fails and 499 connections are unrouted.</li>
              <li>Do not treat this page as bench evidence; no test has been executed.</li>
            </ul>
          </div>
        </div>
      </Panel>
    </div>
  );
}
