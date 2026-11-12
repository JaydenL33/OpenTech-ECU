import { useMemo, useRef, useState } from "react";
import { api } from "../api";
import { Panel, Badge } from "../components/ui";
import type { LogFile, TelemetryFrame } from "../types";
import { CHANNELS, CHANNEL_BY_KEY, readChannel, type ChannelDefinition } from "../channels";

const WIDTH = 1000;
const PAD = { left: 158, right: 22, top: 14, bottom: 28 };
const LANE_HEIGHT = 84;
const INNER_PAD = 7;
const COLORS = ["#6da8ff", "#5fd1c0", "#e8c46a", "#f08a5d", "#b49cf0", "#64d39a", "#ff8b8b", "#89b4fa"];

const DEFAULT_CHANNELS = ["rpm", "map_kpa", "lambda_measured", "lambda_target", "ignition_advance_deg"];

interface ViewRange {
  start: number;
  end: number;
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

function timeSeconds(frame: TelemetryFrame | undefined): string {
  return frame ? (frame.scenario_time_ms / 1000).toFixed(2) : "—";
}

export function LogViewerPage() {
  const [log, setLog] = useState<LogFile | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [selected, setSelected] = useState<string[]>(DEFAULT_CHANNELS);
  const [range, setRange] = useState<ViewRange>({ start: 0, end: 0 });
  const [cursor, setCursor] = useState<number | null>(null);
  const [dragging, setDragging] = useState(false);
  const svgRef = useRef<SVGSVGElement>(null);
  const dragRef = useRef<{ x: number; range: ViewRange } | null>(null);

  const selectedChannels = useMemo(
    () => CHANNELS.filter((channel) => selected.includes(channel.key)),
    [selected],
  );

  const load = async () => {
    setBusy(true);
    try {
      const data = await api.log();
      setLog(data);
      setRange({ start: 0, end: Math.max(0, data.frames.length - 1) });
      setCursor(data.frames.length ? data.frames.length - 1 : null);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const total = log?.frames.length ?? 0;
  const span = Math.max(1, range.end - range.start);
  const laneCount = Math.max(1, selectedChannels.length);
  const chartHeight = PAD.top + laneCount * LANE_HEIGHT + PAD.bottom;
  const plotWidth = WIDTH - PAD.left - PAD.right;

  const sampled = useMemo(() => {
    if (!log || total === 0) return [];
    const step = Math.max(1, Math.ceil((span + 1) / 1800));
    const rows: { frame: TelemetryFrame; index: number }[] = [];
    for (let index = range.start; index <= range.end; index += step) {
      const frame = log.frames[index];
      if (frame) rows.push({ frame, index });
    }
    if (rows.length === 0 || rows[rows.length - 1].index !== range.end) {
      const frame = log.frames[range.end];
      if (frame) rows.push({ frame, index: range.end });
    }
    return rows;
  }, [log, range, span, total]);

  const xForIndex = (index: number) => PAD.left + ((index - range.start) / span) * plotWidth;

  const laneTop = (lane: number) => PAD.top + lane * LANE_HEIGHT;
  const yForValue = (channel: ChannelDefinition, lane: number, value: number) => {
    const t = clamp((value - channel.min) / (channel.max - channel.min || 1), 0, 1);
    return laneTop(lane) + INNER_PAD + (1 - t) * (LANE_HEIGHT - INNER_PAD * 2);
  };

  const pointerFraction = (clientX: number): number | null => {
    const svg = svgRef.current;
    if (!svg) return null;
    const rect = svg.getBoundingClientRect();
    if (rect.width === 0) return null;
    const svgX = ((clientX - rect.left) / rect.width) * WIDTH;
    const fraction = (svgX - PAD.left) / plotWidth;
    if (fraction < 0 || fraction > 1) return null;
    return fraction;
  };

  const indexAt = (clientX: number): number | null => {
    const fraction = pointerFraction(clientX);
    if (fraction === null) return null;
    return Math.round(range.start + fraction * span);
  };

  const zoom = (factor: number, anchor: number) => {
    if (!log || total <= 1) return;
    const currentSpan = range.end - range.start;
    const minSpan = Math.min(20, total - 1);
    const nextSpan = clamp(Math.round(currentSpan * factor), minSpan, total - 1);
    const fraction = (anchor - range.start) / Math.max(1, currentSpan);
    const nextStart = clamp(
      Math.round(anchor - fraction * nextSpan),
      0,
      Math.max(0, total - 1 - nextSpan),
    );
    setRange({ start: nextStart, end: nextStart + nextSpan });
  };

  const resetZoom = () => {
    if (!log || total === 0) return;
    setRange({ start: 0, end: total - 1 });
  };

  const toggleChannel = (key: string) => {
    setSelected((current) =>
      current.includes(key) ? current.filter((item) => item !== key) : [...current, key],
    );
  };

  const exportCsv = () => {
    if (!log || total === 0) return;
    const channels = selectedChannels.length ? selectedChannels : [CHANNELS[0]];
    const header = ["time_s", ...channels.map((channel) => `${channel.label}_${channel.unit}`)];
    const lines = [header.join(",")];
    for (let index = range.start; index <= range.end; index += 1) {
      const frame = log.frames[index];
      if (!frame) continue;
      const row = [
        (frame.scenario_time_ms / 1000).toFixed(3),
        ...channels.map((channel) => {
          const value = readChannel(channel, frame);
          return value === null ? "" : value.toFixed(channel.decimals);
        }),
      ];
      lines.push(row.join(","));
    }
    const blob = new Blob([lines.join("\n")], { type: "text/csv;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `opentech-log-visible-${range.start}-${range.end}.csv`;
    anchor.click();
    URL.revokeObjectURL(url);
  };

  const cursorFrame = cursor !== null && cursor >= range.start && cursor <= range.end
    ? log?.frames[cursor]
    : undefined;

  return (
    <div className="page">
      <Panel
        title="Log viewer"
        subtitle="Simulator log traces from GET /api/log · cursor readout · drag to pan · wheel to zoom"
        actions={
          <div className="button-row">
            <button className="button primary" onClick={load} disabled={busy}>
              {busy ? "Loading…" : "Load current log"}
            </button>
            <button className="button ghost" onClick={exportCsv} disabled={!log || total === 0}>
              Export visible CSV
            </button>
          </div>
        }
      >
        <div className="alert info">
          <strong>BENCH PROTOTYPE · SIMULATOR DATA.</strong> This viewer reads the virtual ECU's recorded
          session log. It is not connected to vehicle hardware, and the traces are not engine evidence.
        </div>
        {error && <div className="alert danger">{error}</div>}
        {!log && !error && (
          <div className="empty-state">
            <p>No log loaded. Load the current session recorded by the virtual ECU.</p>
          </div>
        )}
        {log && total === 0 && (
          <div className="empty-state"><p>The current log has no frames yet. Start a scenario, then load again.</p></div>
        )}
        {log && total > 0 && (
          <>
            <div className="log-meta">
              <span>Scenario <b>{log.metadata.scenario_name}</b></span>
              <span>Frames <b>{total}</b></span>
              <span>Sample rate <b>{log.metadata.sample_hz} Hz</b></span>
              <span>Calibration <b className="mono">{log.metadata.calibration_id}</b> rev <b>{log.metadata.calibration_revision}</b></span>
              <span>Seed <b>{log.metadata.seed}</b></span>
            </div>
            <div className="log-viewer-layout">
              <div className="channel-picker">
                <div className="channel-picker-header">
                  <h3>Channels</h3>
                  <div className="button-row">
                    <button className="button ghost" onClick={() => setSelected(CHANNELS.map((channel) => channel.key))}>All</button>
                    <button className="button ghost" onClick={() => setSelected([])}>None</button>
                  </div>
                </div>
                <div className="channel-list">
                  {CHANNELS.map((channel) => (
                    <label key={channel.key} className="channel-check">
                      <input
                        type="checkbox"
                        checked={selected.includes(channel.key)}
                        onChange={() => toggleChannel(channel.key)}
                      />
                      <span>{channel.label}</span>
                      <em>{channel.unit}</em>
                    </label>
                  ))}
                </div>
              </div>
              <div className="log-trace-column">
                <div className="trace-toolbar">
                  <div className="view-toggle">
                    <button className="button ghost" onClick={() => zoom(0.75, cursor ?? Math.round((range.start + range.end) / 2))}>Zoom in</button>
                    <button className="button ghost" onClick={() => zoom(1.35, cursor ?? Math.round((range.start + range.end) / 2))}>Zoom out</button>
                    <button className="button ghost" onClick={resetZoom}>Reset zoom</button>
                  </div>
                  <Badge tone="neutral">
                    frames {range.start}–{range.end} of {total} · {timeSeconds(log.frames[range.start])}–{timeSeconds(log.frames[range.end])} s
                  </Badge>
                </div>
                <div className="log-chart-wrap">
                  <svg
                    ref={svgRef}
                    className={`log-chart ${dragging ? "dragging" : ""}`}
                    viewBox={`0 0 ${WIDTH} ${chartHeight}`}
                    preserveAspectRatio="none"
                    onWheel={(event) => {
                      event.preventDefault();
                      const fraction = pointerFraction(event.clientX);
                      const anchor = fraction === null
                        ? Math.round((range.start + range.end) / 2)
                        : Math.round(range.start + fraction * span);
                      zoom(event.deltaY > 0 ? 1.35 : 0.75, anchor);
                    }}
                    onPointerDown={(event) => {
                      const fraction = pointerFraction(event.clientX);
                      if (fraction === null) return;
                      dragRef.current = { x: event.clientX, range: { ...range } };
                      setDragging(true);
                      event.currentTarget.setPointerCapture(event.pointerId);
                    }}
                    onPointerMove={(event) => {
                      const drag = dragRef.current;
                      if (drag) {
                        const svg = svgRef.current;
                        if (!svg) return;
                        const rect = svg.getBoundingClientRect();
                        const dxSvg = ((event.clientX - drag.x) / Math.max(1, rect.width)) * WIDTH;
                        const shift = Math.round((-dxSvg / plotWidth) * (drag.range.end - drag.range.start));
                        const nextStart = clamp(
                          drag.range.start + shift,
                          0,
                          Math.max(0, total - 1 - (drag.range.end - drag.range.start)),
                        );
                        setRange({
                          start: nextStart,
                          end: nextStart + (drag.range.end - drag.range.start),
                        });
                        return;
                      }
                      const index = indexAt(event.clientX);
                      if (index !== null) setCursor(index);
                    }}
                    onPointerUp={(event) => {
                      dragRef.current = null;
                      setDragging(false);
                      event.currentTarget.releasePointerCapture(event.pointerId);
                    }}
                    onPointerCancel={() => {
                      dragRef.current = null;
                      setDragging(false);
                    }}
                    onPointerLeave={() => {
                      if (!dragRef.current) setCursor(null);
                    }}
                  >
                    {selectedChannels.map((channel, lane) => {
                      const top = laneTop(lane);
                      const points = sampled
                        .map(({ frame, index }) => {
                          const value = readChannel(channel, frame);
                          if (value === null) return null;
                          return `${xForIndex(index).toFixed(1)},${yForValue(channel, lane, value).toFixed(1)}`;
                        })
                        .filter((point): point is string => point !== null)
                        .join(" ");
                      return (
                        <g key={channel.key}>
                          <rect
                            x={PAD.left}
                            y={top + 1}
                            width={plotWidth}
                            height={LANE_HEIGHT - 2}
                            fill={lane % 2 === 0 ? "rgba(255,255,255,0.018)" : "rgba(255,255,255,0.006)"}
                          />
                          <line x1={PAD.left} x2={WIDTH - PAD.right} y1={top + INNER_PAD} y2={top + INNER_PAD} stroke="rgba(255,255,255,0.06)" />
                          <line x1={PAD.left} x2={WIDTH - PAD.right} y1={top + LANE_HEIGHT - INNER_PAD} y2={top + LANE_HEIGHT - INNER_PAD} stroke="rgba(255,255,255,0.06)" />
                          <text x={10} y={top + 19} fill={COLORS[lane % COLORS.length]} fontSize="12" fontWeight="600">
                            {channel.label}
                          </text>
                          <text x={10} y={top + 34} fill="var(--muted)" fontSize="10">
                            {channel.unit} · {channel.min}…{channel.max}
                          </text>
                          <polyline
                            points={points}
                            fill="none"
                            stroke={COLORS[lane % COLORS.length]}
                            strokeWidth="1.6"
                            vectorEffect="non-scaling-stroke"
                          />
                        </g>
                      );
                    })}
                    {selectedChannels.length === 0 && (
                      <text x={PAD.left + 20} y={PAD.top + 40} fill="var(--muted)" fontSize="13">
                        Select at least one channel to draw a trace.
                      </text>
                    )}
                    {cursorFrame && (
                      <g>
                        <line
                          x1={xForIndex(cursor ?? range.start)}
                          x2={xForIndex(cursor ?? range.start)}
                          y1={PAD.top}
                          y2={PAD.top + laneCount * LANE_HEIGHT}
                          stroke="rgba(255,255,255,0.72)"
                          strokeWidth="1"
                          strokeDasharray="3 3"
                          vectorEffect="non-scaling-stroke"
                        />
                        {selectedChannels.map((channel, lane) => {
                          const value = readChannel(channel, cursorFrame);
                          if (value === null) return null;
                          return (
                            <circle
                              key={channel.key}
                              cx={xForIndex(cursor ?? range.start)}
                              cy={yForValue(channel, lane, value)}
                              r="3"
                              fill={COLORS[lane % COLORS.length]}
                            />
                          );
                        })}
                      </g>
                    )}
                    <text x={PAD.left} y={chartHeight - 9} fill="var(--muted)" fontSize="10">
                      {timeSeconds(log.frames[range.start])} s
                    </text>
                    <text x={PAD.left + plotWidth / 2} y={chartHeight - 9} textAnchor="middle" fill="var(--muted)" fontSize="10">
                      {timeSeconds(log.frames[Math.round((range.start + range.end) / 2)])} s
                    </text>
                    <text x={WIDTH - PAD.right} y={chartHeight - 9} textAnchor="end" fill="var(--muted)" fontSize="10">
                      {timeSeconds(log.frames[range.end])} s
                    </text>
                  </svg>
                </div>
                <div className="log-readout">
                  {cursorFrame ? (
                    <>
                      <div className="readout-time">
                        <span className="stat-label">Cursor</span>
                        <b>{(cursorFrame.scenario_time_ms / 1000).toFixed(3)} s</b>
                        <span className="muted">frame {cursor}</span>
                      </div>
                      <div className="readout-channels">
                        {selectedChannels.map((channel, lane) => (
                          <span key={channel.key}>
                            <i style={{ background: COLORS[lane % COLORS.length] }} />
                            {channel.label}
                            <b>{(() => {
                              const value = readChannel(channel, cursorFrame);
                              return value === null ? "—" : `${value.toFixed(channel.decimals)} ${channel.unit}`;
                            })()}</b>
                          </span>
                        ))}
                      </div>
                    </>
                  ) : (
                    <p className="muted">Move the pointer over the chart for a cursor readout, drag to pan, or use the mouse wheel to zoom.</p>
                  )}
                </div>
              </div>
            </div>
            <p className="hint">
              The CSV export contains the {selectedChannels.length} selected channel{selectedChannels.length === 1 ? "" : "s"} for the
              visible frame window only. Log files are deterministic for the same scenario, seed and calibration.
            </p>
          </>
        )}
      </Panel>
    </div>
  );
}
