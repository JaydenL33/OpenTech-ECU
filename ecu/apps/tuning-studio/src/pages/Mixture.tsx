import { useMemo, useState } from "react";
import { api } from "../api";
import { useApp } from "../context";
import type { MixtureMapReport } from "../types";
import { Panel, Badge, Stat } from "../components/ui";

const statusTone: Record<string, "good" | "warn" | "danger" | "neutral" | "info"> = {
  proposed: "good",
  on_target: "info",
  insufficient_samples: "neutral",
  high_variance: "warn",
  low_confidence: "warn",
  wot_excluded: "warn",
  no_samples: "neutral",
};

export function MixturePage() {
  const app = useApp();
  const [report, setReport] = useState<MixtureMapReport | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [allowWot, setAllowWot] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [undoAvailable, setUndoAvailable] = useState(false);

  const takeReport = (r: MixtureMapReport) => {
    setReport(r);
    setSelected(new Set((r.proposal?.cells ?? []).map((c) => `${c.row},${c.col}`)));
    setUndoAvailable(false);
    setError(null);
  };

  const runSweep = async () => {
    setBusy("Running deterministic dyno sweep and learning gates…");
    setError(null);
    try {
      const result = await api.mixtureAnalyze({ min_samples: 12, max_cell_correction: 0.05, allow_wot: allowWot });
      takeReport(result.report);
      setMessage(
        `Analysis complete: ${result.report.samples_used} accepted / ${result.report.samples_rejected} rejected samples.`,
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(null);
    }
  };

  const analyzeCurrent = async () => {
    setBusy("Analyzing the current session log…");
    setError(null);
    try {
      const result = await api.mixtureAnalyzeCurrent({ min_samples: 20, max_cell_correction: 0.05, allow_wot: allowWot });
      takeReport(result.report);
      setMessage("Current log analyzed. Coverage reflects the running scenario.");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(null);
    }
  };

  const apply = async () => {
    if (!report?.proposal) return;
    if (selected.size === 0) {
      setError("Select at least one proposed cell.");
      return;
    }
    if (
      !window.confirm(
        `Apply ${selected.size} fuel-cell corrections as a new sealed revision?\n\nThis changes the virtual calibration only. The maps remain unvalidated; no physical outputs exist.`,
      )
    ) {
      return;
    }
    setBusy("Pausing engine and applying selected cells…");
    setError(null);
    try {
      if (app.running) await app.pause();
      const cells = [...selected].map((key) => key.split(",").map(Number) as [number, number]);
      const result = await api.mixtureApply(report.proposal, cells);
      app.setCalibration(result.calibration, result.preflight);
      setMessage(
        `Applied ${result.applied_cells} cells. Revision ${result.calibration.revision}, checksum ${result.calibration.checksum.slice(0, 12)}… Max VE delta ${result.diff.max_abs_delta.toFixed(2)} points.`,
      );
      setUndoAvailable(true);
      setReport(null);
      await app.refreshStatus();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(null);
    }
  };

  const undo = async () => {
    setBusy("Rolling back to the previous sealed calibration…");
    try {
      if (app.running) await app.pause();
      const result = await api.undo();
      app.setCalibration(result.calibration, result.preflight);
      setMessage(`Restored revision ${result.calibration.revision}.`);
      setUndoAvailable(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(null);
    }
  };

  const proposals = report?.proposal?.cells ?? [];
  const coverage = useMemo(() => {
    if (!report) return { rows: 0, cols: 0, max: 1 };
    const rows = Math.max(...report.cells.map((c) => c.row)) + 1;
    const cols = Math.max(...report.cells.map((c) => c.col)) + 1;
    return { rows, cols, max: Math.max(1, ...report.cells.map((c) => c.samples_used)) };
  }, [report]);

  return (
    <div className="page">
      <Panel
        title="Offline Mixture Map"
        subtitle="Lambda-based fuel calibration with shared learning gates, coverage, confidence, bounded proposals and human approval"
        actions={
          <div className="button-row">
            <button className="button primary" onClick={runSweep} disabled={!!busy}>
              Run guided sweep
            </button>
            <button className="button ghost" onClick={analyzeCurrent} disabled={!!busy}>
              Analyze current log
            </button>
            {undoAvailable && (
              <button className="button ghost" onClick={undo} disabled={!!busy}>
                Undo applied revision
              </button>
            )}
          </div>
        }
      >
        <div className="alert warn">
          The Mixture Map proposes <strong>fuel VE changes only</strong>. It cannot propose ignition timing,
          MBT, or knock margin. WOT/high-load cells are excluded unless you explicitly opt in, and every
          proposal still requires review and approval.
        </div>
        <label className="checkbox">
          <input type="checkbox" checked={allowWot} onChange={(e) => setAllowWot(e.target.checked)} />
          Include high-load / WOT cells (explicit trust-boundary override; still approval-gated)
        </label>
        {busy && <div className="inline-status">{busy}</div>}
        {error && <div className="alert danger">{error}</div>}
        {message && <div className="alert info">{message}</div>}
        {!report && !busy && (
          <div className="empty-state">
            <p>No analysis loaded.</p>
            <p className="muted">
              “Run guided sweep” executes a deterministic warm dyno sweep in Rust, logs it, applies the
              steady-state gates, and returns cell proposals. It does not modify the calibration.
            </p>
          </div>
        )}
        {report && (
          <>
            <div className="stat-grid four">
              <Stat label="Samples accepted" value={report.samples_used} />
              <Stat label="Samples rejected" value={report.samples_rejected} tone="warn" />
              <Stat label="Proposed cells" value={proposals.length} tone={proposals.length ? "good" : ""} />
              <Stat label="Bank" value={report.bank} />
              <Stat label="Coverage" value={`${report.cells.filter((c) => c.samples_used > 0).length}/${report.cells.length}`} />
              <Stat label="Log checksum" value={report.source_log_checksum.slice(0, 12) + "…"} />
              <Stat label="Calibration" value={`rev ${report.calibration_revision}`} />
              <Stat label="Ignition" value={report.ignition_untouched ? "untouched" : "CHANGED"} tone="good" />
            </div>

            {report.hard_faults.length > 0 && (
              <div className="alert danger">
                <strong>Hard fault — no proposal issued.</strong> {report.hard_faults.join(", ")}.
                This is an abstention, not a pass. A stuck wideband cannot be used to learn fuel.
              </div>
            )}

            <div className="mixture-columns">
              <div>
                <h3>Gate rejections</h3>
                <div className="gate-chips">
                  {Object.entries(report.gate_reason_counts)
                    .sort((a, b) => b[1] - a[1])
                    .map(([code, count]) => (
                      <span className="gate-chip" key={code}>
                        {code} <b>{count}</b>
                      </span>
                    ))}
                </div>
                <h3>Cell coverage</h3>
                <div className="coverage" style={{ gridTemplateColumns: `repeat(${coverage.cols}, 22px)` }}>
                  {report.cells.map((c) => (
                    <div
                      key={`${c.row}-${c.col}`}
                      className="coverage-cell"
                      title={`${c.map_kpa.toFixed(0)} kPa / ${c.rpm.toFixed(0)} rpm — ${c.samples_used} samples, ${c.status}`}
                      style={{
                        background: c.samples_used
                          ? `hsla(210, 70%, ${28 + 30 * (c.samples_used / coverage.max)}%, 1)`
                          : "rgba(255,255,255,0.04)",
                      }}
                    />
                  ))}
                </div>
                <p className="hint">Unvisited cells remain empty. No interpolation or fabricated coverage is added.</p>
              </div>
              <div>
                <h3>Proposal preview</h3>
                {proposals.length === 0 ? (
                  <p className="muted">No cells met sample, dwell, variance and confidence requirements.</p>
                ) : (
                  <div className="proposal-list">
                    {proposals.map((c) => {
                      const key = `${c.row},${c.col}`;
                      return (
                        <label className="proposal-row" key={key}>
                          <input
                            type="checkbox"
                            checked={selected.has(key)}
                            onChange={(e) => {
                              const next = new Set(selected);
                              if (e.target.checked) next.add(key);
                              else next.delete(key);
                              setSelected(next);
                            }}
                          />
                          <span className="cell-pos">
                            {c.map_kpa.toFixed(0)} kPa · {c.rpm.toFixed(0)} rpm
                          </span>
                          <span>
                            VE {c.current_ve.toFixed(2)} → <b>{c.proposed_ve.toFixed(2)}</b>
                          </span>
                          <span className={c.correction_fraction >= 0 ? "pos" : "neg"}>
                            {c.correction_fraction >= 0 ? "+" : ""}
                            {(c.correction_fraction * 100).toFixed(1)}%
                          </span>
                          <span className="muted">n={c.samples} · conf {c.confidence.toFixed(2)}</span>
                          <span className="muted">λ {c.mean_lambda.toFixed(3)} / {c.target_lambda.toFixed(3)}</span>
                        </label>
                      );
                    })}
                  </div>
                )}
                <div className="button-row">
                  <button className="button primary" onClick={apply} disabled={!!busy || proposals.length === 0}>
                    Apply selected ({selected.size})
                  </button>
                  <button
                    className="button ghost"
                    onClick={() =>
                      setSelected(
                        selected.size === proposals.length
                          ? new Set()
                          : new Set(proposals.map((c) => `${c.row},${c.col}`)),
                      )
                    }
                  >
                    {selected.size === proposals.length ? "Clear selection" : "Select all"}
                  </button>
                </div>
              </div>
            </div>

            <h3>All cells</h3>
            <div className="table-scroll">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Map</th><th>RPM</th><th>Seen</th><th>Used</th><th>Dwell s</th><th>λ mean</th>
                    <th>Target</th><th>Trim</th><th>Conf.</th><th>Status</th><th>Correction</th>
                  </tr>
                </thead>
                <tbody>
                  {report.cells.filter((c) => c.samples_seen > 0).map((c) => (
                    <tr key={`${c.row}-${c.col}-data`}>
                      <td>{c.map_kpa.toFixed(0)}</td>
                      <td>{c.rpm.toFixed(0)}</td>
                      <td>{c.samples_seen}</td>
                      <td>{c.samples_used}</td>
                      <td>{(c.dwell_ms / 1000).toFixed(1)}</td>
                      <td>{c.mean_lambda?.toFixed(3) ?? "—"}</td>
                      <td>{c.mean_target_lambda?.toFixed(3) ?? "—"}</td>
                      <td>{c.mean_total_trim !== null ? `${(c.mean_total_trim * 100).toFixed(1)}%` : "—"}</td>
                      <td>{c.confidence.toFixed(2)}</td>
                      <td><Badge tone={statusTone[c.status] ?? "neutral"}>{c.status.replaceAll("_", " ")}</Badge></td>
                      <td>{c.status === "proposed" ? `${(c.correction_fraction * 100).toFixed(1)}%` : "—"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </>
        )}
      </Panel>
    </div>
  );
}
