# OpenTech-ECU formulas and assumptions

This document records the equations implemented by the Rust code. If code and
this document disagree, the code is the implementation but the discrepancy is a
defect that must be resolved before any hardware use.

Units are explicit. Angles are crank degrees unless stated. Four-stroke cycle =
720°. Pressure is absolute kPa unless labelled gauge. Temperature is °C in
telemetry and K in gas equations. Fuel mass is per cylinder per 720°.

## 1. Speed-density air mass

Implementation: `ecu/crates/ecu-fuel/src/lib.rs::air_mass_mg`

```
rho_air [kg/m³] = MAP [Pa] / (R_air * IAT [K]),  R_air = 287.05 J/(kg K)
Vd_cyl [m³]     = displacement_cc * 1e-6 / cylinders
m_air [kg]      = rho_air * Vd_cyl * (VE_percent / 100)
m_air [mg]      = m_air [kg] * 1e6
```

Assumptions:
- ideal gas;
- one intake event per cylinder per 720°;
- VE includes breathing, not fuel;
- no manifold wall-wetting or backflow model in v0.1;
- pressure and temperature sensors are trusted only when their status and age
  are valid.

## 2. Fuel mass and injector pulse

Implementation: `ecu/crates/ecu-fuel/src/lib.rs::calculate`

```
m_fuel [kg]      = m_air [kg] / (stoich_AFR * target_lambda)
pressure_factor  = sqrt(P_actual / P_rated)          if pressure compensation on
flow_g_s         = rated_flow_cc_min * density_g_cc / 60 * pressure_factor
PW_uncorrected   = m_fuel * 1000 / flow_g_s + dead_time(V)
PW               = max(PW_uncorrected, min_pw, dead_time)
delivered_fuel   = max(PW - dead_time, 0) * flow_g_s
duty             = PW / (120000 / RPM)                (720° cycle period)
```

Trims are explicit fractional inputs: `m_fuel *= (1 + STFT + LTFT)`.
Duty above the injector's configured maximum returns an inhibited decision, not
a silent clamp. Dead time is interpolated from the configured voltage curve.

Known limitations:
- fuel density is a calibration input; gasoline ≈0.745 g/cc, E85 ≈0.785 g/cc;
- stoichiometric AFR is fuel-specific and must not be hard-coded globally;
- differential-pressure regulators and returnless systems require review;
- wall wetting, injector nonlinearity near minimum pulse and staged injection
  are not fully modelled in v0.1.

## 3. Lambda

```
lambda = AFR_actual / stoich_AFR
```

`ecu-types` requires the caller to supply `stoich_AFR`; there is no implicit
gasoline constant in the conversion API. A wideband sensor measures oxygen
concentration in the exhaust, not fuel flow directly and never ignition safety.

The simulator models:
- transport delay (`exhaust_delay_ms`);
- first-order sensor response (`wideband_tau_ms`);
- heater readiness;
- noise (seeded);
- stuck-lean/rich, not-ready, exhaust-leak and misfire traps.

## 4. Offline Mixture Map correction

Implementation: `ecu/crates/ecu-lambda/src/mixture_map.rs`

For accepted steady samples in one cell:

```
mean_lambda, mean_target, mean_trim (STFT+LTFT) from time-aligned samples
raw_factor      = (mean_lambda / mean_target) * (1 + mean_trim)
bounded_factor  = clamp(raw_factor, 1 - total_bound, 1 + total_bound)
cell_factor     = clamp(bounded_factor, 1 - per_cell_bound, 1 + per_cell_bound)
VE_new          = clamp(VE_old * cell_factor, table_min, table_max)
correction      = VE_new / VE_old - 1
```

The trim term is required to avoid double-correcting when STFT/LTFT were active
when the log was recorded. `bounded_factor` and `cell_factor` are independent
limits; defaults are 20% lifetime and 5% per apply.

Acceptance rules (defaults, configurable but visible):
- sequential sync, engine running, coolant 70–110 °C;
- wideband ready, status Ok, age ≤300 ms, lambda 0.70–1.30;
- no fuel cut, overrun, misfire, exhaust leak, knock, overboost or critical DTC;
- no RPM/MAP/TPS transient beyond the configured rates;
- injector duty ≤92%;
- fuel pressure ≥200 kPa and oil pressure ≥40 kPa in the simulator analysis;
- high-load/WOT cells excluded unless explicitly opted in;
- per-cell minimum samples and dwell, variance and confidence thresholds;
- no interpolation into unvisited cells;
- a repeated constant lambda across a wide operating range triggers
  `LAMBDA_STUCK_SUSPECTED` and forces abstention.

Confidence (0..1):

```
sample   = min(1, used / min_samples)
variance = max(0, 1 - stddev / max_stddev)
dwell    = min(1, dwell_ms / min_dwell_ms)
confidence = 0.45*sample + 0.35*variance + 0.20*dwell
```

This is a coverage/quality score, not a probability that the tune is safe.

## 5. Short-term closed loop (STFT)

Implementation: `ecu/crates/ecu-lambda/src/closed_loop.rs`

```
error = lambda_measured - target_lambda          (lean positive → add fuel)
P     = kp * error
I    += ki * error * dt
STFT  = clamp(P + I, -max_stft, +max_stft)
```

- deadband suppresses tiny oscillation;
- conditional integration prevents windup at a limit;
- a blocked gate stops adaptation;
- a short invalid period freezes trim; a prolonged invalid period decays it to
  zero so a failed sensor cannot hold a large correction;
- WOT/high-load adaptation is blocked unless explicitly authorised;
- physical real-time adaptation is deferred pending independent validation.

## 6. Long-term trim (LTFT)

Implementation: `ecu/crates/ecu-lambda/src/long_term.rs`

LTFT learns toward an active, gated STFT after sustained cell dwell:

```
step = clamp((STFT - LTFT_cell) * gain, -max_step, +max_step)
LTFT_cell = clamp(LTFT_cell + step, -max_trim, +max_trim)
```

STFT, LTFT and base VE are separate structures. Merging selected learned cells
into base VE is explicit and reviewable; it does not happen during a live run.

## 7. Trigger and timing

Implementation: `ecu/crates/ecu-trigger/src/lib.rs`, `ecu-scheduler/src/lib.rs`

- Missing-tooth gap identifies crank position; cam phase is required for
  sequential operation.
- A gap is accepted only when its interval ratio matches the configured missing
  tooth count; after the first sync, the number of teeth between gaps is also
  checked. A dropped tooth causes sync loss rather than a guessed angle.
- RPM from gap-to-gap is `60_000_000 / rev_us`.
- Sequence timeout forces `Lost`; the first resumed edge establishes a fresh
  baseline and the first gap re-syncs.
- Event angles are 0..720 with compression TDC of cylinder 1 at 0; firing order
  determines each cylinder's TDC.
- Spark angle = `TDC - advance`; dwell start = `spark - dwell_deg`.
- **Sync offset is not proof of TDC.** A timing light is mandatory before
  physical ignition use.

## 8. Protection thresholds (simulator placeholders)

Implementation: `ecu/crates/ecu-protection/src/lib.rs`.
The numeric thresholds are engineering placeholders for the virtual ECU, not
validated 4G63 limits. They must be reviewed against the actual engine, sensors,
fuel system and operating conditions.

| Condition | Default action | Latch |
|---|---|---|
| crank sync lost | cut fuel + spark | no, fast clear |
| coolant ≥110 °C / ≥118 °C | derate / cut | cut latches |
| oil <60 kPa above 1200 rpm | cut | yes |
| fuel pressure <180 kPa | cut fuel | no |
| lambda >1.12 under ≥90 kPa | derate | no |
| lambda >1.22 under ≥90 kPa | cut fuel | yes |
| boost gauge >95 / >120 kPa | derate / cut | cut latches |
| battery <10 V / <8 V | derate / cut | cut latches |
| calibration checksum failure | cut all | yes |
| watchdog failure | cut all | yes |
| injector duty >95% | derate | no |
| knock ≥1.5 / ≥4.0 | retard / derate | no |

## 9. Ignition evidence

Implementation: `ecu/crates/ecu-ignition/src/lib.rs`.
There is no lambda input to any ignition function. A recommendation requires a
licensed, engine-specific verified reference; without one the API returns
`INSUFFICIENT EVIDENCE`. Synthetic simulator surfaces are marked as such and
cannot be exported for physical use. Lambda does not determine MBT, knock limit
or safe spark advance.

## 10. Determinism

The simulator uses a fixed 1 ms internal step and a SplitMix64 PRNG seeded by
the scenario. Same scenario + same calibration + same seed produces identical
telemetry frames; CI checks this with `--check-determinism`.

## References and audit links

- `AUDIT/ENGINE_CONTROL_SAFETY.md`
- `AUDIT/REQUIREMENTS_TRACEABILITY.md`
- `AUDIT/TEST_EVIDENCE.md` (audit snapshot; the README verification note has the current test status)
- Speeduino C++ reference tree (GPL-2.0) at `speeduino/`, retained for
  behavioural comparison; not copied implicitly into these equations.
