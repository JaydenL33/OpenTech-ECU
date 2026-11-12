import { useMemo } from "react";
import type { TelemetryFrame } from "../types";

export interface Series {
  key: keyof TelemetryFrame;
  label: string;
  color: string;
}

export function TraceChart({
  frames,
  series,
  height = 170,
  fixedMin,
  fixedMax,
}: {
  frames: TelemetryFrame[];
  series: Series[];
  height?: number;
  fixedMin?: number;
  fixedMax?: number;
}) {
  const width = 900;
  const pad = { left: 42, right: 12, top: 12, bottom: 22 };
  const data = frames.slice(-300);
  const { min, max } = useMemo(() => {
    if (fixedMin !== undefined && fixedMax !== undefined) return { min: fixedMin, max: fixedMax };
    let lo = Infinity;
    let hi = -Infinity;
    for (const f of data) {
      for (const s of series) {
        const v = f[s.key] as number;
        if (Number.isFinite(v)) {
          lo = Math.min(lo, v);
          hi = Math.max(hi, v);
        }
      }
    }
    if (!Number.isFinite(lo)) return { min: 0, max: 1 };
    if (hi - lo < 1e-6) hi = lo + 1;
    return { min: lo, max: hi };
  }, [data, series, fixedMin, fixedMax]);

  const x = (i: number) =>
    pad.left + (i / Math.max(1, data.length - 1)) * (width - pad.left - pad.right);
  const y = (value: number) =>
    pad.top + (1 - (value - min) / (max - min)) * (height - pad.top - pad.bottom);

  return (
    <div className="trace">
      <svg viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="none">
        {[0, 0.25, 0.5, 0.75, 1].map((t) => (
          <line
            key={t}
            x1={pad.left}
            x2={width - pad.right}
            y1={pad.top + t * (height - pad.top - pad.bottom)}
            y2={pad.top + t * (height - pad.top - pad.bottom)}
            stroke="#ececec"
          />
        ))}
        <text x="4" y={pad.top + 5} fill="var(--muted)" fontSize="10">
          {max.toFixed(1)}
        </text>
        <text x="4" y={height - pad.bottom} fill="var(--muted)" fontSize="10">
          {min.toFixed(1)}
        </text>
        {series.map((s) => {
          const points = data
            .map((f, i) => {
              const v = f[s.key] as number;
              return Number.isFinite(v) ? `${x(i)},${y(v)}` : null;
            })
            .filter(Boolean)
            .join(" ");
          return (
            <polyline
              key={String(s.key)}
              points={points}
              fill="none"
              stroke={s.color}
              strokeWidth="1.8"
              vectorEffect="non-scaling-stroke"
            />
          );
        })}
      </svg>
      <div className="legend">
        {series.map((s) => (
          <span key={String(s.key)}>
            <i style={{ background: s.color }} />
            {s.label}
          </span>
        ))}
      </div>
    </div>
  );
}

export function MiniTrace({
  frames,
  accessor,
  color,
  height = 48,
}: {
  frames: TelemetryFrame[];
  accessor: (frame: TelemetryFrame) => number;
  color: string;
  height?: number;
}) {
  const data = frames.slice(-120);
  const values = data.map(accessor).filter((v) => Number.isFinite(v));
  const min = Math.min(...values, 0);
  const max = Math.max(...values, min + 0.001);
  const points = data
    .map((f, i) => {
      const v = accessor(f);
      const x = (i / Math.max(1, data.length - 1)) * 200;
      const y = height - ((v - min) / (max - min)) * (height - 4) - 2;
      return `${x},${y}`;
    })
    .join(" ");
  return (
    <svg className="mini-trace" viewBox={`0 0 200 ${height}`} preserveAspectRatio="none">
      <polyline points={points} fill="none" stroke={color} strokeWidth="1.5" vectorEffect="non-scaling-stroke" />
    </svg>
  );
}
