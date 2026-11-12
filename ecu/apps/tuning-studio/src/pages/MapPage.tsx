import { useState } from "react";
import { useApp } from "../context";
import { Panel, Badge } from "../components/ui";
import { MapEditor } from "../components/MapEditor";
import { Surface3D } from "../components/Surface3D";

export function MapPage({
  tableId,
  kind,
  title,
  subtitle,
}: {
  tableId: string;
  kind: "fuel" | "target" | "ignition";
  title: string;
  subtitle: string;
}) {
  const app = useApp();
  const table = app.calibration?.tables[tableId];
  const [view, setView] = useState<"grid" | "surface" | "split">("split");

  if (!app.calibration) return <div className="page"><p className="muted">Loading calibration…</p></div>;
  if (!table) return <div className="page"><div className="alert danger">Required table “{tableId}” is missing. Run preflight.</div></div>;

  const estimated = table.provenance.source !== "verified_reference" || !table.provenance.engine_specific_verified;

  return (
    <div className="page">
      <Panel
        title={title}
        subtitle={subtitle}
        actions={
          <div className="view-toggle">
            {(["grid", "surface", "split"] as const).map((v) => (
              <button key={v} className={`button ${view === v ? "primary" : "ghost"}`} onClick={() => setView(v)}>
                {v}
              </button>
            ))}
          </div>
        }
      >
        {kind === "ignition" && (
          <div className="alert danger">
            <strong>SIMULATION-ONLY IGNITION TABLE.</strong> This is not MBT, not a knock limit, and not an
            engine-safe timing map. Lambda is never used to estimate ignition advance. Physical ignition export
            is blocked by preflight; a licensed engine-specific reference plus timing-light verification and
            knock instrumentation are required.
          </div>
        )}
        {estimated && kind !== "ignition" && (
          <div className="alert warn">
            <strong>UNVALIDATED STARTING ESTIMATE.</strong> This table is generated from engineering assumptions,
            not measured engine data. It must be reviewed on a dyno with verified sensors and fuel hardware.
          </div>
        )}
        <div className="table-meta">
          <Badge tone={estimated ? "warn" : "good"}>{table.provenance.source.replaceAll("_", " ")}</Badge>
          <span>{table.axes.length}D · {table.values.length} cells · rev {table.revision}</span>
          <span className="mono">{table.description}</span>
        </div>
        <div className={`map-layout ${view}`}>
          {view !== "surface" && (
            <MapEditor table={table} kind={kind} onSaved={() => app.refreshStatus()} />
          )}
          {view !== "grid" && <Surface3D table={table} kind={kind} />}
        </div>
        <div className="provenance">
          <div>
            <span>Source</span> {table.provenance.citation || "not recorded"}
          </div>
          <div>
            <span>Licence</span> {table.provenance.licence || "not recorded"}
          </div>
          <div>
            <span>Verified by</span> {table.provenance.verified_by || "UNVERIFIED"}
          </div>
          <div>
            <span>Notes</span> {table.provenance.notes}
          </div>
        </div>
      </Panel>
    </div>
  );
}
