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
  const light = 66 + 26 * t;
  const sat = 38 + 34 * t;
  return `hsl(${hue} ${sat}% ${light}%)`;
}

export function MapEditor({
  table,
  kind,
  onSaved,
  initialValues,
  onValuesChange,
}: {
  table: Table;
  kind: "fuel" | "target" | "ignition";
  onSaved?: () => void;
  /** View values supplied by a named map set. */
  initialValues?: number[];
  /** Called for every local edit so a map-set draft can follow the view. */
  onValuesChange?: (values: number[]) => void;
}) {
  const app = useApp();
  const [values, setValues] = useState<number[]>(() => (initialValues ? initialValues.slice() : table.values));
  const [selected, setSelected] = useState({ row: 0, col: 0 });
  const [anchor, setAnchor] = useState({ row: 0, col: 0 });
  const [entry, setEntry] = useState<{ mode: "value" | "percent"; text: string } | null>(null);
  const [dirty, setDirty] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [rpmPoints, setRpmPoints] = useState(table.axes[1]?.breakpoints.length ?? 16);
  const [loadPoints, setLoadPoints] = useState(table.axes[0]?.breakpoints.length ?? 16);
  const undo = useRef<number[][]>([]);
  const redo = useRef<number[][]>([]);
  const clipboard = useRef<number[][] | null>(null);
  const gridRef = useRef<HTMLDivElement>(null);

  const rank = table.axes.length;
  const rows = table.axes[0]?.breakpoints.length ?? 0;
  const cols = table.axes[1]?.breakpoints.length ?? 0;

  useEffect(() => {
    setValues(initialValues ? initialValues.slice() : table.values);
    setDirty(false);
    setSelected({ row: 0, col: 0 });
    setAnchor({ row: 0, col: 0 });
    setEntry(null);
    undo.current = [];
    redo.current = [];
  }, [table.id, table.revision]);

  useEffect(() => {
    onValuesChange?.(values);
  }, [values]);

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

  const top = Math.min(anchor.row, selected.row), bottom = Math.max(anchor.row, selected.row);
  const left = Math.min(anchor.col, selected.col), right = Math.max(anchor.col, selected.col);
  const bounded = (value: number) => Number(Math.max(table.value_min, Math.min(table.value_max, value)).toFixed(table.decimals));
  const rangeEdit = (fn: (r: number, c: number) => number) => {
    const next = values.slice();
    for (let r = top; r <= bottom; r++) for (let c = left; c <= right; c++) next[r * cols + c] = bounded(fn(r, c));
    commit(next);
  };
  const onKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if ((event.target as HTMLElement).closest("input, textarea, select, [contenteditable=true]")) return;
    const movement: Record<string, [number, number]> = {
      ArrowUp: [-1, 0],
      ArrowDown: [1, 0],
      ArrowLeft: [0, -1],
      ArrowRight: [0, 1],
    };
    const ctrl = event.ctrlKey || event.metaKey;
    if (movement[event.key]) {
      event.preventDefault();
      const [dr, dc] = movement[event.key];
      if (ctrl && !event.shiftKey) {
        setAnchor({ row: anchor.row + (dr && anchor.row !== selected.row ? Math.sign(selected.row - anchor.row) : 0), col: anchor.col + (dc && anchor.col !== selected.col ? Math.sign(selected.col - anchor.col) : 0) });
        return;
      }
      const next = { row: Math.max(0, Math.min(rows - 1, selected.row + dr)), col: Math.max(0, Math.min(cols - 1, selected.col + dc)) };
      if (ctrl && event.shiftKey) setCell(next.row, next.col, selectedValue);
      setSelected(next);
      if (!event.shiftKey || ctrl) setAnchor(next);
    } else if (event.key === "Home" || event.key === "End") {
      event.preventDefault();
      const next = { ...selected, col: event.key === "Home" ? 0 : cols - 1 };
      setSelected(next); setAnchor(next);
    } else if (event.key === " ") {
      event.preventDefault();
      const frame = app.live.frame;
      if (ctrl) setAnchor(selected);
      else if (table.id === "ve" && frame && frame.ve_row >= 0 && frame.ve_row < rows && frame.ve_col >= 0 && frame.ve_col < cols) {
        const next = { row: frame.ve_row, col: frame.ve_col }; setSelected(next); setAnchor(next);
      }
    } else if (ctrl && event.key.toLowerCase() === "a") {
      event.preventDefault(); setAnchor({ row: 0, col: 0 }); setSelected({ row: rows - 1, col: cols - 1 });
    } else if (event.key === "PageUp" || event.key === "PageDown") {
      event.preventDefault(); setCell(selected.row, selected.col, bounded(selectedValue + (event.key === "PageUp" ? 1 : -1) * step * (ctrl ? 1 : event.shiftKey ? 100 : 10)));
    } else if (!ctrl && /^[0-9]$/.test(event.key)) {
      event.preventDefault(); setEntry({ mode: "value", text: event.key });
    } else if (!ctrl && event.key.toLowerCase() === "p") {
      event.preventDefault(); setEntry({ mode: "percent", text: "" });
    } else if (!ctrl && event.key.toLowerCase() === "l") {
      event.preventDefault();
      rangeEdit((r, c) => right > left
        ? values[r * cols + left] + (values[r * cols + right] - values[r * cols + left]) * (c - left) / (right - left)
        : bottom > top ? values[top * cols + c] + (values[bottom * cols + c] - values[top * cols + c]) * (r - top) / (bottom - top) : selectedValue);
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
      event.preventDefault();
      clipboard.current = Array.from({ length: bottom - top + 1 }, (_, r) => values.slice((top + r) * cols + left, (top + r) * cols + right + 1));
    } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "v") {
      if (clipboard.current !== null) {
        event.preventDefault();
        const next = values.slice();
        clipboard.current.forEach((line, r) => line.forEach((value, c) => {
          if (selected.row + r < rows && selected.col + c < cols) next[(selected.row + r) * cols + selected.col + c] = bounded(value);
        }));
        commit(next);
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
      {entry && <label> {entry.mode === "percent" ? "Percentage change" : "Cell value"}
        <input autoFocus value={entry.text} onChange={e => setEntry({ ...entry, text: e.target.value })} onKeyDown={e => {
          e.stopPropagation();
          if (e.key === "Escape") { setEntry(null); gridRef.current?.focus(); }
          if (e.key === "Enter" && entry.text.trim() && Number.isFinite(Number(entry.text))) {
            if (entry.mode === "percent") rangeEdit((r, c) => values[r * cols + c] * (1 + Number(entry.text) / 100));
            else setCell(selected.row, selected.col, Number(entry.text));
            setEntry(null); gridRef.current?.focus();
          }
        }} /></label>}
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
                  const active = table.id === "ve" && app.live.frame?.ve_row === row && app.live.frame?.ve_col === col;
                  return (
                    <td key={rpm}>
                      <button
                        className={`map-cell ${row >= top && row <= bottom && col >= left && col <= right ? "selected" : ""} ${active ? "active-cell" : ""}`}
                        style={{ background: cellColor(value, table, kind) }}
                        onClick={() => {
                          setSelected({ row, col });
                          setAnchor({ row, col });
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
        Arrows/Home/End move · Shift+Arrow selects · Ctrl+Arrow shrinks · Ctrl+Space collapses · Ctrl+A all · PgUp/Dn adjusts (Ctrl small, Shift large) · digits enter, Enter commits, Escape cancels · P percent · L linearise rows (single column vertically) · Ctrl+C/V rectangle · Ctrl+Shift+Arrow copies adjacent · Space live · Ctrl+Z undo. Selection {bottom - top + 1} × {right - left + 1}.
        The blue live marker is available for the VE table only; other tables have no table-specific live indices. Applying pauses the virtual engine
        and seals a new checksummed revision.
      </p>
    </div>
  );
}
