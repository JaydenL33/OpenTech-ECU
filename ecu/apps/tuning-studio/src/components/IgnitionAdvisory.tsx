import { useState } from "react";
import { api } from "../api";
import type { IgnitionRecommendationResponse, TimingReferenceInput } from "../types";
import { Badge, Panel, Stat } from "./ui";

const statusTone = {
  insufficient_evidence: "danger",
  simulation_only: "warn",
  verified_reference: "info",
} as const;

const statusText = {
  insufficient_evidence: "Insufficient evidence — no number offered",
  simulation_only: "Simulation only",
  verified_reference: "Caller-asserted verified reference",
} as const;

const num = (v: string, fallback: number) => {
  const n = Number(v);
  return Number.isFinite(n) ? n : fallback;
};

/**
 * Advisory ignition recommendation. Deliberately has no lambda, trim or log
 * input and no apply button: a wideband cannot establish safe spark advance,
 * and the result is never written to the ignition table.
 */
export function IgnitionAdvisory() {
  const [rpm, setRpm] = useState("4000");
  const [map, setMap] = useState("100");
  const [fuel, setFuel] = useState("98 RON");
  const [haveRef, setHaveRef] = useState(false);
  const [refAdvance, setRefAdvance] = useState("");
  const [refRpm, setRefRpm] = useState("");
  const [refMap, setRefMap] = useState("");
  const [refCr, setRefCr] = useState("");
  const [refFuel, setRefFuel] = useState("");
  const [refAsp, setRefAsp] = useState("naturally_aspirated");
  const [citation, setCitation] = useState("");
  const [licence, setLicence] = useState("");
  const [verifiedBy, setVerifiedBy] = useState("");
  const [verifiedOn, setVerifiedOn] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<IgnitionRecommendationResponse | null>(null);

  const run = async () => {
    setBusy(true);
    setError(null);
    try {
      const numericInputs = [rpm, map, ...(haveRef ? [refAdvance, refRpm, refMap, refCr] : [])];
      if (numericInputs.some((v) => v.trim() === "" || !Number.isFinite(Number(v)))) {
        throw new Error("All numeric fields must be valid numbers.");
      }
      let reference: TimingReferenceInput | null = null;
      if (haveRef) {
        reference = {
          provenance: {
            source: "verified_reference",
            citation,
            licence,
            verified_by: verifiedBy,
            verification_date: verifiedOn,
            engine_specific_verified: true,
            notes: "Entered by the operator in the tuning UI; not independently checked.",
          },
          advance_deg: num(refAdvance, NaN),
          rpm: num(refRpm, NaN),
          map_kpa: num(refMap, NaN),
          compression_ratio: num(refCr, NaN),
          fuel: refFuel,
          aspiration: refAsp,
        };
      }
      setResult(
        await api.ignitionRecommend({
          target_rpm: num(rpm, NaN),
          target_map_kpa: num(map, NaN),
          engine_fuel: fuel,
          reference,
        }),
      );
    } catch (e) {
      setResult(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const rec = result?.recommendation;
  return (
    <Panel
      title="Ignition advisory (separate from lambda)"
      subtitle="Evidence-gated and read-only. Never written to the ignition table or any output."
      actions={
        <button className="button primary" onClick={run} disabled={busy}>
          Get advisory
        </button>
      }
    >
      <div className="alert warn">
        Lambda and fuel trims are <strong>not</strong> evidence of safe spark advance, so none is used here.
        Without a licensed, engine-specific timing reference this returns <em>no number</em>. A reference you
        enter is your own assertion — the service cannot verify it — and any figure still needs timing-light
        and knock verification on a supervised dyno.
      </div>
      <div className="setup-grid">
        <label>
          Target RPM
          <input value={rpm} onChange={(e) => setRpm(e.target.value)} inputMode="decimal" />
        </label>
        <label>
          Target MAP (kPa)
          <input value={map} onChange={(e) => setMap(e.target.value)} inputMode="decimal" />
        </label>
        <label>
          Engine fuel
          <input value={fuel} onChange={(e) => setFuel(e.target.value)} />
        </label>
      </div>
      <label className="checkbox">
        <input type="checkbox" checked={haveRef} onChange={(e) => setHaveRef(e.target.checked)} />I have a licensed,
        engine-specific timing reference to enter
      </label>
      {haveRef && (
        <div className="setup-grid">
          <label>
            Reference advance (° BTDC)
            <input value={refAdvance} onChange={(e) => setRefAdvance(e.target.value)} inputMode="decimal" />
          </label>
          <label>
            Reference RPM
            <input value={refRpm} onChange={(e) => setRefRpm(e.target.value)} inputMode="decimal" />
          </label>
          <label>
            Reference MAP (kPa)
            <input value={refMap} onChange={(e) => setRefMap(e.target.value)} inputMode="decimal" />
          </label>
          <label>
            Reference compression ratio
            <input value={refCr} onChange={(e) => setRefCr(e.target.value)} inputMode="decimal" />
          </label>
          <label>
            Reference fuel
            <input value={refFuel} onChange={(e) => setRefFuel(e.target.value)} />
          </label>
          <label>
            Reference aspiration
            <select value={refAsp} onChange={(e) => setRefAsp(e.target.value)}>
              <option value="naturally_aspirated">naturally_aspirated</option>
              <option value="turbocharged">turbocharged</option>
              <option value="supercharged">supercharged</option>
            </select>
          </label>
          <label>
            Citation
            <input value={citation} onChange={(e) => setCitation(e.target.value)} />
          </label>
          <label>
            Licence / permission
            <input value={licence} onChange={(e) => setLicence(e.target.value)} />
          </label>
          <label>
            Verified by
            <input value={verifiedBy} onChange={(e) => setVerifiedBy(e.target.value)} />
          </label>
          <label>
            Verification date
            <input value={verifiedOn} onChange={(e) => setVerifiedOn(e.target.value)} placeholder="YYYY-MM-DD" />
          </label>
        </div>
      )}
      {error && <div className="alert danger">{error}</div>}
      {result && rec && (
        <div>
          <p>
            <Badge tone={statusTone[rec.status]}>{statusText[rec.status]}</Badge>{" "}
            <Badge tone="neutral">advisory only</Badge> <Badge tone="neutral">no lambda input</Badge>
          </p>
          <div className="stat-grid four">
            <Stat
              label="Proposed advance"
              value={rec.proposed_advance_deg === null ? "—" : rec.proposed_advance_deg.toFixed(1)}
              unit={rec.proposed_advance_deg === null ? undefined : "°"}
            />
            <Stat
              label="Review bounds"
              value={
                rec.lower_bound_deg === null || rec.upper_bound_deg === null
                  ? "—"
                  : `${rec.lower_bound_deg.toFixed(1)} … ${rec.upper_bound_deg.toFixed(1)}`
              }
              unit={rec.lower_bound_deg === null ? undefined : "°"}
            />
            <Stat label="Confidence" value={rec.confidence.toFixed(2)} />
            <Stat label="Table policy" value={result.calibration_ignition_policy ?? "unknown"} />
          </div>
          {result.attestation_notes.length > 0 && (
            <div className="alert warn">
              {result.attestation_notes.map((n) => (
                <div key={n}>{n}</div>
              ))}
            </div>
          )}
          {rec.reasons.length > 0 && (
            <>
              <h3>Why</h3>
              <ul>
                {rec.reasons.map((r) => (
                  <li key={r}>{r}</li>
                ))}
              </ul>
            </>
          )}
          <h3>Required validation before any use</h3>
          <ul>
            {rec.required_validation.map((r) => (
              <li key={r}>{r}</li>
            ))}
          </ul>
          {result.reference_provenance && (
            <p className="muted">
              Reference provenance: {result.reference_provenance.source}
              {result.reference_provenance.citation ? ` — ${result.reference_provenance.citation}` : ""}
            </p>
          )}
          <p className="muted">{result.notice}</p>
        </div>
      )}
    </Panel>
  );
}
