import { useMemo, useRef, useState, type ChangeEvent } from "react";
import { useApp } from "../context";
import { Panel, Gauge, Badge } from "../components/ui";
import { CHANNELS, CHANNEL_BY_KEY, readChannel } from "../channels";

interface DesignerLayout {
  version: 1;
  name: string;
  columns: number;
  rows: number;
  /** Row-major slot list. An empty slot is null. */
  cells: (string | null)[];
}

const DEFAULT_LAYOUT: DesignerLayout = {
  version: 1,
  name: "My dashboard",
  columns: 4,
  rows: 3,
  cells: [
    "rpm",
    "map_kpa",
    "lambda_measured",
    "ignition_advance_deg",
    "injector_duty_pct",
    "clt_c",
    null,
    null,
    null,
    null,
    null,
    null,
  ],
};

const GROUPS = Array.from(new Set(CHANNELS.map((channel) => channel.group)));

function parseLayout(text: string): DesignerLayout {
  const parsed: unknown = JSON.parse(text);
  if (!parsed || typeof parsed !== "object") throw new Error("layout must be a JSON object");
  const record = parsed as Record<string, unknown>;
  if (record.version !== 1) throw new Error("unsupported layout version");
  const columns = Number(record.columns);
  const rows = Number(record.rows);
  if (!Number.isInteger(columns) || columns < 1 || columns > 8) throw new Error("columns must be 1-8");
  if (!Number.isInteger(rows) || rows < 1 || rows > 8) throw new Error("rows must be 1-8");
  if (!Array.isArray(record.cells)) throw new Error("cells must be an array");
  if (record.cells.length !== columns * rows) {
    throw new Error(`expected ${columns * rows} slots, found ${record.cells.length}`);
  }
  const cells = record.cells.map((cell, index) => {
    if (cell === null) return null;
    if (typeof cell !== "string") throw new Error(`slot ${index + 1} is not a channel key`);
    const channel = CHANNEL_BY_KEY[cell];
    if (!channel || channel.source !== "telemetry") {
      throw new Error(`slot ${index + 1} uses unknown channel “${cell}”`);
    }
    return cell;
  });
  const name = typeof record.name === "string" && record.name.trim() ? record.name.trim() : "Imported dashboard";
  return { version: 1, name, columns, rows, cells };
}

function safeFileName(name: string): string {
  const base = name.trim().toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "");
  return `${base || "dashboard"}.json`;
}

export function DashboardDesignerPage() {
  const app = useApp();
  const [layout, setLayout] = useState<DesignerLayout>(DEFAULT_LAYOUT);
  const [selectedIndex, setSelectedIndex] = useState<number | null>(0);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  const frame = app.live.frame;

  const setCell = (index: number, key: string | null) => {
    setLayout((current) => {
      const cells = current.cells.slice();
      cells[index] = key;
      return { ...current, cells };
    });
    setStatus(null);
  };

  const resize = (columns: number, rows: number) => {
    setLayout((current) => {
      const cells: (string | null)[] = Array.from({ length: columns * rows }, () => null);
      for (let row = 0; row < Math.min(rows, current.rows); row += 1) {
        for (let col = 0; col < Math.min(columns, current.columns); col += 1) {
          cells[row * columns + col] = current.cells[row * current.columns + col] ?? null;
        }
      }
      return { ...current, columns, rows, cells };
    });
    setSelectedIndex(null);
    setStatus(null);
  };

  const selectedChannel = selectedIndex !== null ? layout.cells[selectedIndex] : null;

  const save = () => {
    const blob = new Blob([JSON.stringify(layout, null, 2)], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = safeFileName(layout.name);
    anchor.click();
    URL.revokeObjectURL(url);
    setStatus(`Downloaded ${safeFileName(layout.name)}. The layout stays in this browser session only.`);
    setError(null);
  };

  const load = async (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;
    try {
      const parsed = parseLayout(await file.text());
      setLayout(parsed);
      setSelectedIndex(null);
      setStatus(`Loaded ${file.name}. Nothing was uploaded to the service.`);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setStatus(null);
    }
  };

  const gaugeCount = layout.cells.filter(Boolean).length;
  const groupOptions = useMemo(() => GROUPS.map((group) => ({
    group,
    channels: CHANNELS.filter((channel) => channel.group === group),
  })), []);

  return (
    <div className="page">
      <Panel
        title="Dashboard designer"
        subtitle="Place simulator gauges on a grid, then download or reload the layout as a JSON file"
        actions={
          <div className="designer-toolbar">
            <input
              aria-label="Dashboard name"
              value={layout.name}
              onChange={(event) => setLayout((current) => ({ ...current, name: event.target.value }))}
            />
            <label className="button ghost file-button">
              Load JSON
              <input ref={fileRef} type="file" accept="application/json,.json" onChange={load} />
            </label>
            <button className="button primary" onClick={save}>Save JSON</button>
          </div>
        }
      >
        <div className="alert info">
          <strong>BENCH PROTOTYPE · SIMULATOR DATA.</strong> These gauges display the virtual ECU telemetry
          stream only. No physical sensor or output is connected. Layouts are saved as local JSON downloads
          and are never stored on a server.
        </div>
        <div className="designer-toolbar designer-toolbar-secondary">
          <label>
            Columns
            <select value={layout.columns} onChange={(event) => resize(Number(event.target.value), layout.rows)}>
              {[1, 2, 3, 4, 5, 6, 7, 8].map((value) => <option key={value} value={value}>{value}</option>)}
            </select>
          </label>
          <label>
            Rows
            <select value={layout.rows} onChange={(event) => resize(layout.columns, Number(event.target.value))}>
              {[1, 2, 3, 4, 5, 6, 7, 8].map((value) => <option key={value} value={value}>{value}</option>)}
            </select>
          </label>
          <Badge tone="neutral">{gaugeCount} / {layout.cells.length} gauges</Badge>
          <button
            className="button ghost"
            onClick={() => {
              setLayout({ ...DEFAULT_LAYOUT, name: layout.name });
              setSelectedIndex(0);
              setStatus(null);
              setError(null);
            }}
          >
            Reset to default
          </button>
          {status && <span className="inline-status">{status}</span>}
          {error && <span className="inline-error">{error}</span>}
        </div>
        <div
          className="designer-grid"
          style={{ gridTemplateColumns: `repeat(${layout.columns}, minmax(0, 1fr))` }}
        >
          {layout.cells.map((key, index) => {
            const channel = key ? CHANNEL_BY_KEY[key] : null;
            return (
              <div
                className={`designer-slot ${channel ? "filled" : "empty"} ${selectedIndex === index ? "selected" : ""}`}
                key={index}
              >
                {channel ? (
                  <>
                    <button className="slot-select" onClick={() => setSelectedIndex(index)} title="Select this gauge">
                      <Gauge
                        label={channel.label}
                        value={readChannel(channel, frame)}
                        min={channel.min}
                        max={channel.max}
                        unit={channel.unit}
                        decimals={channel.decimals}
                      />
                    </button>
                    <button
                      className="slot-remove"
                      title={`Remove ${channel.label}`}
                      onClick={() => setCell(index, null)}
                    >
                      ×
                    </button>
                  </>
                ) : (
                  <button className="designer-empty" onClick={() => setSelectedIndex(index)}>
                    <span>+</span>
                    Empty slot
                  </button>
                )}
              </div>
            );
          })}
        </div>
        <div className="designer-inspector">
          <div>
            <span className="stat-label">Selected slot</span>
            <b>{selectedIndex === null ? "none" : `row ${Math.floor(selectedIndex / layout.columns) + 1}, column ${(selectedIndex % layout.columns) + 1}`}</b>
          </div>
          <label>
            Channel
            <select
              value={selectedChannel ?? ""}
              disabled={selectedIndex === null}
              onChange={(event) => {
                if (selectedIndex === null) return;
                setCell(selectedIndex, event.target.value || null);
              }}
            >
              <option value="">Empty slot</option>
              {groupOptions.map(({ group, channels }) => (
                <optgroup key={group} label={group}>
                  {channels.map((channel) => (
                    <option key={channel.key} value={channel.key}>
                      {channel.label} ({channel.unit})
                    </option>
                  ))}
                </optgroup>
              ))}
            </select>
          </label>
          <p className="hint designer-hint">
            Click a slot, choose a channel from the service telemetry frame, then save the JSON. Loading a
            layout validates every channel against the current channel list.
          </p>
        </div>
      </Panel>
    </div>
  );
}
