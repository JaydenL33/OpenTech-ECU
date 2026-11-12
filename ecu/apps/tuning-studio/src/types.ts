export type SensorStatus =
  | "ok"
  | "stale"
  | "not_ready"
  | "saturated"
  | "invalid"
  | "fault";

export interface Provenance {
  source: string;
  citation: string;
  licence: string;
  verified_by: string;
  verification_date: string;
  engine_specific_verified: boolean;
  notes: string;
}

export interface Axis {
  name: string;
  unit: string;
  breakpoints: number[];
}

export interface Table {
  id: string;
  name: string;
  unit: string;
  description: string;
  axes: Axis[];
  values: number[];
  provenance: Provenance;
  revision: number;
  value_min: number;
  value_max: number;
  decimals: number;
}

export interface EngineConfig {
  name: string;
  displacement_cc: number;
  cylinders: number;
  firing_order: number[];
  compression_ratio: number;
  aspiration: "naturally_aspirated" | "turbocharged" | "supercharged";
  rev_limit: number;
  idle_rpm: number;
  trigger: {
    pattern: { missing_tooth?: { teeth: number; missing: number } } | string;
    cam_required: boolean;
    sync_offset_deg: number;
    tooth_timeout_ms: number;
  };
  injectors: {
    id: string;
    flow_cc_min: number;
    rated_pressure_kpa: number;
    dead_time_ms: number;
    dead_time_curve: [number, number][];
    min_pw_ms: number;
    max_duty: number;
    cylinders: number[];
  }[];
  cylinder_banks: number[];
}

export interface Calibration {
  schema_version: number;
  id: string;
  revision: number;
  parent: string | null;
  created_unix_ms: number;
  engine: EngineConfig;
  fuel: {
    stoich_afr: number;
    fuel_density_g_cc: number;
    lower_heating_value_mj_kg: number;
    pressure_reference_kpa: number;
    load_strategy: unknown;
    injection_mode: string;
    pressure_compensation: boolean;
  };
  sensors: Record<string, unknown>;
  io: { assignments: unknown[]; notes: string };
  tables: Record<string, Table>;
  ignition_policy:
    | { kind: "synthetic_simulation_only"; warning: string; table_key: string }
    | {
        kind: "verified_reference";
        citation: string;
        licence: string;
        verified_by: string;
        timing_light_required: boolean;
      }
    | { kind: "insufficient_evidence"; reason: string };
  notes: string[];
  checksum: string;
}

export type ProtectionState = "normal" | "warning" | "derated" | "limp" | "cut";
export type EngineState = "off" | "cranking" | "running" | "stalled" | "cut";
export type SyncState = "unsynced" | "crank_synced" | "sequential" | "lost";

export interface WidebandState {
  ready: boolean;
  heater_on: boolean;
  pumping_current_ma: number;
  sensor_temperature_c: number;
  status: SensorStatus;
  response_tau_ms: number;
  transport_delay_ms: number;
  bank: number;
  controller_id: string;
}

export interface TelemetryFrame {
  seq: number;
  timestamp_us: number;
  rpm: number;
  map_kpa: number;
  tps_pct: number;
  iat_c: number;
  clt_c: number;
  oil_pressure_kpa: number;
  fuel_pressure_kpa: number;
  battery_v: number;
  lambda_measured: number;
  lambda_target: number;
  lambda_status: SensorStatus;
  lambda_ready: boolean;
  lambda_age_ms: number;
  lambda_bank: number;
  exhaust_delay_ms: number;
  wideband: WidebandState;
  ve: number;
  ve_row: number;
  ve_col: number;
  ignition_advance_deg: number;
  dwell_ms: number;
  injector_pw_ms: number;
  injector_duty_pct: number;
  stft: number;
  ltft: number;
  closed_loop: boolean;
  sync: SyncState;
  cycle_angle_deg: number;
  engine_state: EngineState;
  protection: ProtectionState;
  active_faults: string[];
  fuel_cut: boolean;
  spark_cut: boolean;
  knock_level: number;
  boost_kpa: number;
  speed_kph: number;
  gear: number;
  ethanol_pct: number;
  starter_active: boolean;
  throttle_commanded_pct: number;
  dyno_load_nm: number;
  scenario_time_ms: number;
  log_note: string;
}

export interface PreflightCheck {
  id: string;
  title: string;
  status: "pass" | "warn" | "fail" | "not_applicable";
  detail: string;
  blocks_physical: boolean;
}

export interface PreflightReport {
  checks: PreflightCheck[];
  error_count: number;
  warning_count: number;
  hardware_candidate: boolean;
  physical_export_allowed: boolean;
  virtual_run_allowed: boolean;
  summary: string;
}

export interface Scenario {
  id: string;
  name: string;
  description: string;
  duration_ms: number;
  seed: number;
  notes: string[];
}

export interface SimSnapshot {
  scenario_id: string;
  scenario_name: string;
  time_ms: number;
  rpm: number;
  map_kpa: number;
  lambda: number;
  target_lambda: number;
  sync: string;
  engine_state: string;
  protection_state: string;
  calibration_revision: number;
  sample_count: number;
}

export interface MixtureCellReport {
  row: number;
  col: number;
  rpm: number;
  map_kpa: number;
  samples_seen: number;
  samples_used: number;
  dwell_ms: number;
  mean_lambda: number | null;
  mean_target_lambda: number | null;
  lambda_stddev: number;
  mean_stft: number;
  mean_ltft: number;
  mean_total_trim: number | null;
  current_ve: number;
  proposed_ve: number;
  correction_fraction: number;
  confidence: number;
  status:
    | "proposed"
    | "on_target"
    | "insufficient_samples"
    | "high_variance"
    | "low_confidence"
    | "wot_excluded"
    | "no_samples";
  exclusion_reasons: Record<string, number>;
}

export interface ProposedVeCell {
  row: number;
  col: number;
  rpm: number;
  map_kpa: number;
  current_ve: number;
  proposed_ve: number;
  correction_fraction: number;
  confidence: number;
  samples: number;
  mean_lambda: number;
  target_lambda: number;
  mean_trim: number;
}

export interface VeProposal {
  proposal_id: string;
  calibration_id: string;
  calibration_revision: number;
  base_checksum: string;
  table_key: string;
  cells: ProposedVeCell[];
  source_scenario: string;
  source_seed: number;
  source_log_checksum: string;
  ignition_effect: string;
  requires_approval: boolean;
  created_unix_ms: number;
}

export interface MixtureMapReport {
  source_scenario: string;
  source_seed: number;
  source_log_checksum: string;
  calibration_id: string;
  calibration_revision: number;
  calibration_checksum: string;
  bank: number;
  total_frames: number;
  bank_frames: number;
  samples_used: number;
  samples_rejected: number;
  gate_reason_counts: Record<string, number>;
  cells: MixtureCellReport[];
  proposal: VeProposal | null;
  hard_faults: string[];
  ignition_untouched: boolean;
  notes: string[];
}

export interface SessionSummary {
  samples: number;
  duration_ms: number;
  mean_rpm: number;
  max_rpm: number;
  max_coolant_c: number;
  min_oil_pressure_kpa: number;
  mean_lambda: number | null;
  lambda_samples: number;
  fault_codes: string[];
  protection_cut_samples: number;
}

export interface ActiveFault {
  dtc: string;
  code?: string;
  title?: string;
  latched: boolean;
  condition_active: boolean;
  first_seen_us: number;
  last_seen_us: number;
  occurrences: number;
  freeze_frame: {
    timestamp_us: number;
    rpm: number;
    map_kpa: number;
    tps_pct: number;
    coolant_c: number;
    oil_pressure_kpa: number;
    fuel_pressure_kpa: number;
    battery_v: number;
    lambda: number;
    target_lambda: number;
    injector_duty_pct: number;
  };
}

export interface SetupForm {
  vehicle_name: string;
  engine_name: string;
  displacement_cc: number;
  cylinders: number;
  firing_order: number[];
  compression_ratio: number;
  aspiration: string;
  rev_limit: number;
  idle_rpm: number;
  injector_flow_cc_min: number;
  injector_dead_time_ms: number;
  injector_rated_pressure_kpa: number;
  stoich_afr: number;
  fuel_density_g_cc: number;
}

export interface LogFile {
  metadata: {
    schema_version: number;
    software_version: string;
    scenario_id: string;
    scenario_name: string;
    seed: number;
    calibration_id: string;
    calibration_revision: number;
    calibration_checksum: string;
    sample_hz: number;
    notes: string[];
  };
  frames: TelemetryFrame[];
  events: {
    timestamp_us: number;
    severity: string;
    source: string;
    code: string;
    message: string;
  }[];
}

export interface VehicleProfile {
  id: string;
  make: string;
  model: string;
  variant: string;
  market: string;
  chassis: string;
  engine: string;
  transmission: string;
  aspiration: string;
  verification: string;
  ecu_part_number: string | null;
  harness_documented: boolean;
  connector_documented: boolean;
  pinout_documented: boolean;
  trigger_verified: boolean;
  injector_data_verified: boolean;
  ignition_data_verified: boolean;
  unverified: {
    id: string;
    title: string;
    blocking_reason: string;
    verification_method: string;
    gate: number;
  }[];
  factory_systems: { name: string; status: string; note: string }[];
  notes: string[];
}

export interface HarnessPin {
  pin: string;
  name: string;
  net: string;
  function: string;
  oem_assignment: string;
}

export interface HarnessConnector {
  reference: string;
  label: string;
  sheet: string;
  mpn: string;
  pins: HarnessPin[];
}

export interface HarnessSubsystem {
  id: string;
  name: string;
  sheet: string;
  description: string;
  connectors: string[];
}

export interface HarnessManifest {
  schema_version: number;
  id: string;
  title: string;
  status: string;
  physical_use: string;
  notice: string;
  source: {
    design: string;
    erc: string;
    drc: string;
    files: { path: string; sha256: string }[];
  };
  generated_by: string;
  subsystems: HarnessSubsystem[];
  connectors: HarnessConnector[];
  test_points: { reference: string; net: string; purpose: string }[];
  reference_target: { vehicle: string; compatibility: string; note: string };
}

