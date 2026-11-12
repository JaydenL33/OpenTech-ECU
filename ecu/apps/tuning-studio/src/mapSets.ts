import { useSyncExternalStore } from "react";

export interface MapSet {
  id: string;
  name: string;
  /** Table key -> values. These are browser drafts, never service state. */
  tables: Record<string, number[]>;
  updatedUnixMs: number;
}

export interface MapSetsState {
  sets: MapSet[];
  activeId: string | null;
}

const STORAGE_KEY = "opentech.mapSets.v1";
const EMPTY: MapSetsState = { sets: [], activeId: null };

function isNumberArray(value: unknown): value is number[] {
  return Array.isArray(value) && value.every((item) => typeof item === "number" && Number.isFinite(item));
}

function load(): MapSetsState {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return EMPTY;
    const parsed: unknown = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") return EMPTY;
    const record = parsed as Record<string, unknown>;
    if (!Array.isArray(record.sets)) return EMPTY;
    const sets: MapSet[] = [];
    for (const entry of record.sets) {
      if (!entry || typeof entry !== "object") continue;
      const item = entry as Record<string, unknown>;
      if (typeof item.id !== "string" || typeof item.name !== "string") continue;
      if (!item.tables || typeof item.tables !== "object") continue;
      const tables: Record<string, number[]> = {};
      for (const [key, value] of Object.entries(item.tables as Record<string, unknown>)) {
        if (isNumberArray(value)) tables[key] = value.slice();
      }
      sets.push({
        id: item.id,
        name: item.name,
        tables,
        updatedUnixMs: typeof item.updatedUnixMs === "number" ? item.updatedUnixMs : 0,
      });
    }
    const activeId = typeof record.activeId === "string" && sets.some((set) => set.id === record.activeId)
      ? record.activeId
      : null;
    return { sets, activeId };
  } catch {
    return EMPTY;
  }
}

let state: MapSetsState = load();
const listeners = new Set<() => void>();

function emit(next: MapSetsState) {
  state = next;
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
  } catch {
    // Browser storage may be disabled; the sets still work for this session.
  }
  listeners.forEach((listener) => listener());
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function snapshot(): MapSetsState {
  return state;
}

export function useMapSets(): MapSetsState {
  return useSyncExternalStore(subscribe, snapshot, snapshot);
}

export function setActiveMapSet(id: string | null) {
  emit({ ...state, activeId: id && state.sets.some((set) => set.id === id) ? id : null });
}

export function createMapSet(name: string, tables: Record<string, number[]>): string {
  const id = globalThis.crypto?.randomUUID?.() ?? `mapset-${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const next: MapSet = {
    id,
    name: name.trim() || `Map set ${state.sets.length + 1}`,
    tables: Object.fromEntries(Object.entries(tables).map(([key, values]) => [key, values.slice()])),
    updatedUnixMs: Date.now(),
  };
  emit({ sets: [...state.sets, next], activeId: id });
  return id;
}

export function renameMapSet(id: string, name: string) {
  const trimmed = name.trim();
  if (!trimmed) return;
  emit({
    ...state,
    sets: state.sets.map((set) => (set.id === id ? { ...set, name: trimmed, updatedUnixMs: Date.now() } : set)),
  });
}

export function updateMapSetTable(id: string, tableId: string, values: number[]) {
  const target = state.sets.find((set) => set.id === id);
  if (!target) return;
  const existing = target.tables[tableId];
  if (
    existing
    && existing.length === values.length
    && existing.every((value, index) => value === values[index])
  ) {
    return;
  }
  emit({
    ...state,
    sets: state.sets.map((set) => (
      set.id === id
        ? { ...set, tables: { ...set.tables, [tableId]: values.slice() }, updatedUnixMs: Date.now() }
        : set
    )),
  });
}

export function deleteMapSet(id: string) {
  emit({
    sets: state.sets.filter((set) => set.id !== id),
    activeId: state.activeId === id ? null : state.activeId,
  });
}
