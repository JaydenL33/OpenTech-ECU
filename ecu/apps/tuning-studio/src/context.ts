import { createContext, useContext } from "react";
import type { Calibration, PreflightReport, Scenario } from "./types";
import type { LiveState } from "./hooks/useLiveTelemetry";

export interface AppContextValue {
  calibration: Calibration | null;
  preflight: PreflightReport | null;
  scenarios: Scenario[];
  live: LiveState;
  running: boolean;
  busy: boolean;
  error: string | null;
  setError: (value: string | null) => void;
  setCalibration: (cal: Calibration, preflight?: PreflightReport) => void;
  reloadCalibration: () => Promise<void>;
  refreshStatus: () => Promise<void>;
  startScenario: (id: string) => Promise<void>;
  pause: () => Promise<void>;
  resume: () => Promise<void>;
}

export const AppContext = createContext<AppContextValue | null>(null);

export function useApp(): AppContextValue {
  const value = useContext(AppContext);
  if (!value) throw new Error("useApp outside AppContext");
  return value;
}
