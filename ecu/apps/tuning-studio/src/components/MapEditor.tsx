import { useEffect, useRef, useState } from "react";
import { api } from "../api";
import { useApp } from "../context";
import type { Table } from "../types";
import { Badge } from "./ui";

const KIND_HUES: Record<string, [number, number]> = {
  fuel: [205, 48],
  target: [265, 18],
  ignition: [28, 48],
};

function cellColor(value: number, table: Table, kind: string): string {
  const t = Math.max(0, Math.min(1, (value - table.value_min) / (table.value_max - table.value_min || 1)));
  const [h1, h2] = KIND_HUES[kind] ?? [200, 40];
  const hue = h1 + (h2 - h1) * t;
  const light = 14 + 46 * t;
  const sat = 45 + 35 * t;
  return `hsl(${hue} ${sat}% ${light}%)`;
}

export function MapEditor({
  table,
  kind,
  onSaved,
}: {
  table: Table;
  kind: "fuel" | "target" | "ignition";
  onSaved?: () => void;
}) {
  const app = useApp();
  const [values, setValues] = useState<number[]>(table.values);
  const [selected, setSelected] = useState({ row: 0, col: 0 });
  const [dirty, setDirty] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [rpmPoints, setRpmPoints] = useState(table.axes[1]?.breakpoints.length ?? 16);
  const [loadPoints, setLoadPoints] = useState(table.axes[0]?.breakpoints.length ?? 16);
  const undo = useRef<number[][]>([]);
  const redo = useRef<number[][]>([]);
  const clipboard = useRef<number | null>(null);
  const gridRef = useRef<HTMLDivElement>(null);

  const rank = table.axes.length;
  const rows = table.axes[0]?.breakpoints.length ?? 0;
  const cols = table.axes[1]?.breakpoints.length ?? 0;

  useEffect(() => {
    setValues(table.values);
    setDirty(false);
    undo.current = [];
    redo.current = [];
  }, [table.id, table.revision, table.values]);

  if (rank !== 2) {
    return <p className="muted">This editor currently renders 2D tables. This table has rank {rank}.</p>;
  }

  const commit = (next: number[]) => {
    undo.current.push(values);
    if (undo.current.length > 100) undo.current.shift();
    redo.current = [];
    setValues(next);
    setDirty(true);
    setStatus(null);
  };

  const setCell = (row: number, col: number, value: number) => {
    if (!Number.isFinite(value)) return;
    const clamped = Math.max(table.value_min, Math.min(table.value_max, value));
    const idx = row * cols + col;
    if (idx < 0 || idx >= values.length || values[idx] === clamped) return;
    const next = values.slice();
    next[idx] = clamped;
    commit(next);
  };

  const selectedIndex = selected.row * cols + selected.col;
  const selectedValue = values[selectedIndex] ?? 0;
  const step = Math.pow(10, -Math.min(table.decimals, 2));

  const apply = async () => {
    try {
      setStatus("Pausing engine and applying revision…");
      if (app.running) await api.pause();
      const result = await api.saveTable(table.id, values);
      app.setCalibration(result.calibration, result.preflight);
      setDirty(false);
      setStatus(`Revision ${result.calibration.revision} saved and sealed.`);
      await app.refreshStatus();
      onSaved?.();
    } catch (err) {
      setStatus(err instanceof Error ? err.message : String(err));
    }
  };

  const resample = async (axisIndex: number, points: number) => {
    try {
      setStatus("Resampling axis (editing aid, not new evidence)…");
      if (app.running) await api.pause();
      const result = await api.resampleAxis(table.id, axisIndex, points);
      app.setCalibration(result.calibration, result.preflight);
      setStatus(`Axis resampled and sealed as revision ${result.calibration.revision}.`);
      await app.refreshStatus();
      onSaved?.();
    } catch (err) {
      setStatus(err instanceof Error ? err.message : String(err));
    }
  };

  const onKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if ((event.target as HTMLElement).tagName === "INPUT") return;
    const movement: Record<string, [number, number]> = {
      ArrowUp: [-1, 0],
      ArrowDown: [1, 0],
      ArrowLeft: [0, -1],
      ArrowRight: [0, 1],
    };
    if (movement[event.key]) {
      event.preventDefault();
      const [dr, dc] = movement[event.key];
      setSelected((s) => ({
        row: Math.max(0, Math.min(rows - 1, s.row + dr)),
        col: Math.max(0, Math.min(cols - 1, s.col + dc)),
      }));
    } else if (event.key === "+" || event.key === "=") {
      event.preventDefault();
      setCell(selected.row, selected.col, selectedValue + step * (event.shiftKey ? 5 : 1));
    } else if (event.key === "-" || event.key === "_") {
      event.preventDefault();
      setCell(selected.row, selected.col, selectedValue - step * (event.shiftKey ? 5 : 1));
    } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "z") {
      event.preventDefault();
      if (event.shiftKey) {
        const next = redo.current.pop();
        if (next) {
          undo.current.push(values);
          setValues(next);
          setDirty(true);
        }
      } else {
        const prev = undo.current.pop();
        if (prev) {
          redo.current.push(values);
          setValues(prev);
          setDirty(true);
        }
      }
    } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "c") {
      clipboard.current = selectedValue;
    } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "v") {
      if (clipboard.current !== null) {
        setCell(selected.row, selected.col, clipboard.current);
      }
    }
  };

  return (
    <div className="map-editor">
      <div className="map-toolbar">
        <div className="selected-cell">
          <span>{table.axes[0].name} {table.axes[0].breakpoints[selected.row]?.toFixed(0)}</span>
          <span>{table.axes[1].name} {table.axes[1].breakpoints[selected.col]?.toFixed(0)}</span>
          <label>
            value
            <input
              type="number"
              step={step}
              value={Number(selectedValue.toFixed(table.decimals))}
              onChange={(e) => setCell(selected.row, selected.col, Number(e.target.value))}
            />
          </label>
          <span className="unit">{table.unit}</span>
        </div>
        <div className="map-actions">
          {dirty && <Badge tone="warn">unsaved local edits</Badge>}
          <label className="axis-resample">
            {table.axes[1].name}
            <select value={rpmPoints} onChange={(e) => setRpmPoints(Number(e.target.value))}>
              {[8, 12, 16, 24, 32].map((n) => <option key={n} value={n}>{n}</option>)}
            </select>
            <button className="button ghost" onClick={() => resample(1, rpmPoints)}>Resample</button>
          </label>
          <label className="axis-resample">
            {table.axes[0].name}
            <select value={loadPoints} onChange={(e) => setLoadPoints(Number(e.target.value))}>
              {[8, 12, 16, 24, 32].map((n) => <option key={n} value={n}>{n}</option>)}
            </select>
            <button className="button ghost" onClick={() => resample(0, loadPoints)}>Resample</button>
          </label>
          <button
            className="button ghost"
            onClick={() => {
              const prev = undo.current.pop();
              if (prev) {
                redo.current.push(values);
                setValues(prev);
                setDirty(true);
              }
            }}
            disabled={undo.current.length === 0}
          >
            Undo
          </button>
          <button
            className="button ghost"
            onClick={() => {
              const next = redo.current.pop();
              if (next) {
                undo.current.push(values);
                setValues(next);
                setDirty(true);
              }
            }}
            disabled={redo.current.length === 0}
          >
            Redo
          </button>
          <button className="button ghost" onClick={() => { setValues(table.values); setDirty(false); undo.current = []; redo.current = []; }}>
            Revert
          </button>
          <button className="button primary" onClick={apply} disabled={!dirty}>
            Apply sealed revision
          </button>
        </div>
      </div>
      {status && <div className="inline-status">{status}</div>}
      <div className="map-scroll" tabIndex={0} onKeyDown={onKeyDown} ref={gridRef}>
        <table className="map-grid">
          <thead>
            <tr>
              <th className="axis-corner">{table.axes[0].name} \ {table.axes[1].name}</th>
              {table.axes[1].breakpoints.map((b) => (
                <th key={b}>{b.toFixed(0)}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {table.axes[0].breakpoints.map((load, row) => (
              <tr key={load}>
                <th>{load.toFixed(0)}</th>
                {table.axes[1].breakpoints.map((rpm, col) => {
                  const idx = row * cols + col;
                  const value = values[idx] ?? 0;
                  const active = app.live.frame?.ve_row === row && app.live.frame?.ve_col === col;
                  return (
                    <td key={rpm}>
                      <button
                        className={`map-cell ${selected.row === row && selected.col === col ? "selected" : ""} ${active ? "active-cell" : ""}`}
                        style={{ background: cellColor(value, table, kind) }}
                        onClick={() => {
                          setSelected({ row, col });
                          gridRef.current?.focus();
                        }}
                        title={`${load.toFixed(0)} ${table.axes[0].unit} / ${rpm.toFixed(0)} ${table.axes[1].unit}: ${value.toFixed(table.decimals)} ${table.unit}`}
                      >
                        {value.toFixed(table.decimals)}
                      </button>
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="hint">
        Arrow keys move · +/− adjusts · Shift accelerates · Ctrl/Cmd+C/V copies · Ctrl/Cmd+Z undoes.
        The outlined cell is the live operating cell from decoded RPM/MAP. Applying pauses the virtual engine
        and seals a new checksummed revision.
      </p>
    </div>
  );
}
