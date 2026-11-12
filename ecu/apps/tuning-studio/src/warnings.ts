import { useSyncExternalStore } from "react";
import type { TelemetryFrame } from "./types";
import { CHANNEL_BY_KEY, CHANNELS, readChannel } from "./channels";

export interface ChannelLimits {
  low: number | null;
  high: number | null;
}

export type WarningLimits = Record<string, ChannelLimits>;

export interface ActiveWarning {
  key: string;
  label: string;
  unit: string;
  value: number;
  decimals: number;
  limit: "low" | "high";
  threshold: number;
}

const STORAGE_KEY = "opentech.warningLimits.v1";
const EMPTY: WarningLimits = {};

function load(): WarningLimits {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return EMPTY;
    const parsed: unknown = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") return EMPTY;
    const limits: WarningLimits = {};
    for (const [key, value] of Object.entries(parsed as Record<string, unknown>)) {
      const channel = CHANNEL_BY_KEY[key];
      if (!channel || channel.source !== "telemetry") continue;
      if (!value || typeof value !== "object") continue;
      const record = value as Record<string, unknown>;
      const low = typeof record.low === "number" && Number.isFinite(record.low) ? record.low : null;
      const high = typeof record.high === "number" && Number.isFinite(record.high) ? record.high : null;
      if (low !== null || high !== null) limits[key] = { low, high };
    }
    return limits;
  } catch {
    return EMPTY;
  }
}

let state: WarningLimits = load();
const listeners = new Set<() => void>();

function emit(next: WarningLimits) {
  state = next;
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
  } catch {
    // Browser storage may be disabled; limits still work for this session.
  }
  listeners.forEach((listener) => listener());
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function snapshot(): WarningLimits {
  return state;
}

export function useWarningLimits(): WarningLimits {
  return useSyncExternalStore(subscribe, snapshot, snapshot);
}

export function setChannelLimit(key: string, side: "low" | "high", value: number | null) {
  const channel = CHANNEL_BY_KEY[key];
  if (!channel || channel.source !== "telemetry") return;
  const current = state[key] ?? { low: null, high: null };
  const next: WarningLimits = { ...state, [key]: { ...current, [side]: value } };
  if (next[key].low === null && next[key].high === null) delete next[key];
  emit(next);
}

export function clearWarningLimits() {
  emit(EMPTY);
}

export function evaluateWarnings(frame: TelemetryFrame | null, limits: WarningLimits): ActiveWarning[] {
  if (!frame) return [];
  const active: ActiveWarning[] = [];
  for (const channel of CHANNELS) {
    const bound = limits[channel.key];
    if (!bound) continue;
    const value = readChannel(channel, frame);
    if (value === null) continue;
    if (bound.high !== null && value >= bound.high) {
      active.push({
        key: channel.key,
        label: channel.label,
        unit: channel.unit,
        value,
        decimals: channel.decimals,
        limit: "high",
        threshold: bound.high,
      });
    }
    if (bound.low !== null && value <= bound.low) {
      active.push({
        key: channel.key,
        label: channel.label,
        unit: channel.unit,
        value,
        decimals: channel.decimals,
        limit: "low",
        threshold: bound.low,
      });
    }
  }
  return active;
}
