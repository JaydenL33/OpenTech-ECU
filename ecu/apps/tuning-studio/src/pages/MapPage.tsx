import { useState } from "react";
import { useApp } from "../context";
import { Panel, Badge } from "../components/ui";
import { MapEditor } from "../components/MapEditor";
import { MapSetsBar } from "../components/MapSetsBar";
import { Surface3D } from "../components/Surface3D";
import { updateMapSetTable, useMapSets } from "../mapSets";

const MAP_SET_TABLE_IDS = ["ve", "ignition"];

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
  const mapSets = useMapSets();
  const activeSet = mapSets.sets.find((set) => set.id === mapSets.activeId) ?? null;
  const mapSetView = MAP_SET_TABLE_IDS.includes(tableId);
  const viewValues = activeSet?.tables[tableId];

  if (!app.calibration) return <div className="page"><p className="muted">Loading calibration…</p></div>;
  if (!table) return <div className="page"><div className="alert danger">Required table “{tableId}” is missing. Run preflight.</div></div>;

  const estimated = table.provenance.source !== "verified_reference" || !table.provenance.engine_specific_verified;
  const viewTable = activeSet && viewValues && viewValues.length === table.values.length
    ? { ...table, values: viewValues }
    : table;

  return (
    <div className="page">
      {mapSetView && <MapSetsBar tableIds={MAP_SET_TABLE_IDS} />}
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
          {mapSetView && (
            <Badge tone={activeSet ? "info" : "neutral"}>
              {activeSet ? `set view · ${activeSet.name}` : "service calibration view"}
            </Badge>
          )}
          <span>{table.axes.length}D · {table.values.length} cells · rev {table.revision}</span>
          <span className="mono">{table.description}</span>
        </div>
        <div className={`map-layout ${view}`}>
          {view !== "surface" && (
            <MapEditor
              key={`${table.id}-${mapSetView ? activeSet?.id ?? "service" : "service"}`}
              table={viewTable}
              kind={kind}
              onValuesChange={
                mapSetView && activeSet
                  ? (values) => updateMapSetTable(activeSet.id, tableId, values)
                  : undefined
              }
              onSaved={() => app.refreshStatus()}
            />
          )}
          {view !== "grid" && <Surface3D table={viewTable} kind={kind} />}
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
