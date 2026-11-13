import { useEffect, useRef, useState } from "react";
import type { Table } from "../types";

export function Surface3D({ table, kind }: { table: Table; kind: "fuel" | "target" | "ignition" }) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const [yaw, setYaw] = useState(-0.65);
  const [pitch, setPitch] = useState(0.72);
  const drag = useRef<{ x: number; y: number; yaw: number; pitch: number } | null>(null);
  const rows = table.axes[0]?.breakpoints.length ?? 0;
  const cols = table.axes[1]?.breakpoints.length ?? 0;

  useEffect(() => {
    const el = canvas.current;
    if (!el) return;
    const ctx = el.getContext("2d");
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const width = el.clientWidth;
    const height = el.clientHeight;
    el.width = width * dpr;
    el.height = height * dpr;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, height);

    if (!rows || !cols || !table.values.length) return;
    const values = table.values;
    const min = Math.min(...values);
    const max = Math.max(...values);
    const range = Math.max(1e-6, max - min);
    const scale = Math.min(width, height) * 0.62;
    const cx = width / 2;
    const cy = height / 2 + height * 0.12;

    const project = (row: number, col: number, value: number): [number, number, number] => {
      const x = (col / Math.max(1, cols - 1) - 0.5) * 1.5;
      const y = (row / Math.max(1, rows - 1) - 0.5) * 1.2;
      const z = ((value - min) / range - 0.42) * 0.65;
      const x1 = x * Math.cos(yaw) - y * Math.sin(yaw);
      const y1 = x * Math.sin(yaw) + y * Math.cos(yaw);
      const y2 = y1 * Math.cos(pitch) - z * Math.sin(pitch);
      const z2 = y1 * Math.sin(pitch) + z * Math.cos(pitch);
      return [cx + x1 * scale, cy - z2 * scale, y2];
    };

    const hue1 = kind === "fuel" ? 200 : kind === "target" ? 270 : 24;
    const hue2 = kind === "fuel" ? 46 : kind === "target" ? 18 : 52;
    const line = (a: number[], b: number[]) => {
      const [x1, y1] = project(a[0], a[1], a[2]);
      const [x2, y2] = project(b[0], b[1], b[2]);
      const t = (((a[2] + b[2]) / 2) - min) / range;
      ctx.strokeStyle = `hsla(${hue1 + (hue2 - hue1) * t} 62% 48% / 0.78)`;
      ctx.lineWidth = 1.15;
      ctx.beginPath();
      ctx.moveTo(x1, y1);
      ctx.lineTo(x2, y2);
      ctx.stroke();
    };

    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < cols - 1; c++) {
        line([r, c, values[r * cols + c]], [r, c + 1, values[r * cols + c + 1]]);
      }
    }
    for (let c = 0; c < cols; c++) {
      for (let r = 0; r < rows - 1; r++) {
        line([r, c, values[r * cols + c]], [r + 1, c, values[(r + 1) * cols + c]]);
      }
    }

    ctx.fillStyle = "#6d6d6d";
    ctx.font = "11px ui-monospace, monospace";
    ctx.fillText(`${table.axes[0].name} [rows]`, 14, 20);
    ctx.fillText(`${table.axes[1].name} [cols]`, width - 110, height - 30);
    ctx.fillText(`${min.toFixed(table.decimals)} … ${max.toFixed(table.decimals)} ${table.unit}`, 14, height - 14);
  }, [table, valuesKey(table), yaw, pitch, kind, rows, cols]);

  return (
    <div className="surface-wrap">
      <button onClick={() => { setYaw(-0.65); setPitch(0.72); }}>Reset camera</button>
      {!table.values.length && <p>No surface data available.</p>}
      <canvas
        ref={canvas}
        className="surface"
        onPointerDown={(e) => {
          drag.current = { x: e.clientX, y: e.clientY, yaw, pitch };
          (e.target as HTMLCanvasElement).setPointerCapture(e.pointerId);
        }}
        onPointerMove={(e) => {
          if (!drag.current) return;
          setYaw(drag.current.yaw + (e.clientX - drag.current.x) * 0.008);
          setPitch(Math.max(0.12, Math.min(1.4, drag.current.pitch + (e.clientY - drag.current.y) * 0.006)));
        }}
        onPointerUp={() => {
          drag.current = null;
        }}
      />
      <div className="surface-hint">Drag to rotate · {rows}×{cols} grid · data from sealed calibration</div>
    </div>
  );
}

function valuesKey(table: Table): string {
  return `${table.id}:${table.revision}:${table.values.length}`;
}
