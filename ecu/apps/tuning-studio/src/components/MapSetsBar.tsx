import { useEffect, useState } from "react";
import { useApp } from "../context";
import { useMapSets, createMapSet, deleteMapSet, renameMapSet, setActiveMapSet } from "../mapSets";
import { Badge } from "./ui";

export function MapSetsBar({ tableIds }: { tableIds: string[] }) {
  const app = useApp();
  const maps = useMapSets();
  const [name, setName] = useState("");
  const [rename, setRename] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const active = maps.sets.find((set) => set.id === maps.activeId) ?? null;

  useEffect(() => {
    setRename(active?.name ?? "");
  }, [active?.id, active?.name]);

  const create = () => {
    const tables: Record<string, number[]> = {};
    for (const tableId of tableIds) {
      const table = app.calibration?.tables[tableId];
      if (!table) continue;
      const viewValues = active?.tables[tableId];
      tables[tableId] = (viewValues ?? table.values).slice();
    }
    if (Object.keys(tables).length === 0) {
      setMessage("The calibration has not loaded yet; no set was created.");
      return;
    }
    const resolvedName = name.trim() || `Map set ${maps.sets.length + 1}`;
    createMapSet(resolvedName, tables);
    setName("");
    setMessage(`Created “${resolvedName}” from the current view. It is stored in this browser only.`);
  };

  return (
    <section className="map-sets-bar">
      <div className="map-sets-row">
        <Badge tone={active ? "info" : "neutral"}>{active ? "view-only set" : "service view"}</Badge>
        <label>
          Map set
          <select
            value={maps.activeId ?? ""}
            onChange={(event) => {
              setActiveMapSet(event.target.value || null);
              setMessage(null);
            }}
          >
            <option value="">Service calibration</option>
            {maps.sets.map((set) => (
              <option key={set.id} value={set.id}>{set.name}</option>
            ))}
          </select>
        </label>
        {active && (
          <label>
            Rename
            <input
              value={rename}
              onChange={(event) => setRename(event.target.value)}
              onBlur={() => {
                if (!active) return;
                if (rename.trim()) renameMapSet(active.id, rename);
                else setRename(active.name);
              }}
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  event.currentTarget.blur();
                }
              }}
              aria-label="Map set name"
            />
          </label>
        )}
        <label>
          New set
          <input
            value={name}
            placeholder="Name"
            onChange={(event) => setName(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") create();
            }}
          />
        </label>
        <button className="button primary" onClick={create}>Create from current view</button>
        {active && (
          <button
            className="button ghost"
            onClick={() => {
              const deleted = active.name;
              deleteMapSet(active.id);
              setMessage(`Deleted “${deleted}”. The service calibration was not changed.`);
            }}
          >
            Delete set
          </button>
        )}
      </div>
      {message && <p className="map-sets-message">{message}</p>}
      <p className="hint">
        Map sets are named browser-only views of the fuel and ignition tables. Switching a set changes this
        screen only; it makes no service call. “Apply sealed revision” in the map editor remains the only
        control that writes a table to the service. Simulator calibration only — not engine-safe.
      </p>
    </section>
  );
}
