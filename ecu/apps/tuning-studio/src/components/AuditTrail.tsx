import { useCallback, useEffect, useRef, useState } from "react";
import { api, type AuditEntry, type StoreSummary, type StoredRevision } from "../api";
import { Badge, Panel } from "./ui";

const short = (c: string | null) => (c ? `${c.slice(0, 12)}…` : "—");
/** Audit entries fetched per request. The panel never loads the whole log at once. */
export const AUDIT_PAGE_SIZE = 100;

const isAbort = (e: unknown) => e instanceof DOMException && e.name === "AbortError";
const when = (ms: number) => (ms ? new Date(ms).toLocaleString() : "—");

/**
 * Read-only view of the durable calibration store. It cannot apply, undo or
 * edit anything; it only displays what the service has recorded on disk.
 */
export function AuditTrail({ refreshKey }: { refreshKey?: unknown }) {
  const [store, setStore] = useState<StoreSummary | null>(null);
  const [revisions, setRevisions] = useState<StoredRevision[]>([]);
  // Loaded audit window, oldest-first: entries[0].seq === windowStart and the
  // window extends to the newest entry seen at the last refresh.
  const [entries, setEntries] = useState<AuditEntry[]>([]);
  const [total, setTotal] = useState(0);
  const [verify, setVerify] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  // Only the most recent request may update state: starting a new one aborts
  // the previous one, and every completion re-checks it is still the latest.
  const inflight = useRef<AbortController | null>(null);

  const begin = () => {
    inflight.current?.abort();
    const ctrl = new AbortController();
    inflight.current = ctrl;
    setLoading(true);
    return ctrl;
  };
  const finish = (ctrl: AbortController) => {
    if (inflight.current === ctrl) {
      inflight.current = null;
      setLoading(false);
    }
  };

  /** Newest page + revisions. Resets the window to the most recent entries. */
  const refresh = useCallback(async () => {
    const ctrl = begin();
    try {
      // limit=0 is a count-only probe, so only the newest page is transferred.
      const probe = await api.auditPage(0, 0, ctrl.signal);
      const start = Math.max(0, probe.total - AUDIT_PAGE_SIZE);
      const [r, a] = await Promise.all([
        api.revisions(ctrl.signal),
        api.auditPage(start, AUDIT_PAGE_SIZE, ctrl.signal),
      ]);
      if (ctrl.signal.aborted) return;
      setStore(r.store);
      setRevisions(r.revisions);
      setEntries(a.entries);
      setTotal(a.total);
      setError(null);
    } catch (e) {
      if (isAbort(e) || ctrl.signal.aborted) return;
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      finish(ctrl);
    }
  }, []);

  /** Fetch the next older page and prepend it. Keeps already-loaded rows. */
  const loadOlder = async () => {
    const first = entries[0]?.seq ?? 0;
    if (first <= 0) return;
    const start = Math.max(0, first - AUDIT_PAGE_SIZE);
    const ctrl = begin();
    try {
      const a = await api.auditPage(start, first - start, ctrl.signal);
      if (ctrl.signal.aborted) return;
      // Append-only log: older entries never change, but dedupe by seq anyway.
      setEntries((cur) => {
        const have = new Set(cur.map((e) => e.seq));
        return [...a.entries.filter((e) => !have.has(e.seq)), ...cur];
      });
      setTotal(a.total);
      setError(null);
    } catch (e) {
      if (isAbort(e) || ctrl.signal.aborted) return;
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      finish(ctrl);
    }
  };

  useEffect(() => {
    void refresh();
  }, [refresh, refreshKey]);
  useEffect(() => () => inflight.current?.abort(), []);

  const older = entries.length > 0 ? entries[0].seq : 0;
  const newestFirst = [...entries].reverse();

  const runVerify = async () => {
    try {
      const r = await api.verifyStore();
      setVerify(
        r.ok
          ? `Verified: ${r.report?.revisions} revisions, ${r.report?.audit_entries} audit entries.`
          : `FAILED: ${r.error}`,
      );
    } catch (e) {
      setVerify(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <Panel
      title="Durable revision & audit trail"
      subtitle="Read-only. Local append-only evidence log — not a signed certification."
      actions={
        <div className="button-row">
          <button className="button ghost" onClick={() => void refresh()}>Refresh</button>
          <button className="button ghost" onClick={runVerify} disabled={!store?.persistent}>Verify on disk</button>
        </div>
      }
    >
      {error && <div className="alert danger">{error}</div>}
      {store && !store.persistent && <div className="alert warn">{store.note}</div>}
      {store?.persistent && (
        <p className="hint">
          Store: <span className="mono">{store.directory}</span> · {store.revisions} revisions · {store.audit_entries} audit entries.{" "}
          {store.identity_note}
        </p>
      )}
      {store?.warnings?.map((w) => <div className="alert warn" key={w}>{w}</div>)}
      {verify && <div className={`alert ${verify.startsWith("FAILED") ? "danger" : "info"}`}>{verify}</div>}
      <div className="table-scroll">
        <table className="data-table">
          <thead>
            <tr><th>#</th><th>Rev</th><th>Checksum</th><th>Source</th><th>Actor</th><th>Stored</th><th>State</th></tr>
          </thead>
          <tbody>
            {revisions.map((r) => (
              <tr key={r.checksum}>
                <td>{r.seq}</td>
                <td>{r.revision}</td>
                <td className="mono" title={r.checksum}>{short(r.checksum)}</td>
                <td>{r.source}</td>
                <td>{r.actor.label}{r.actor.supplied ? "" : " (default)"}</td>
                <td>{when(r.stored_unix_ms)}</td>
                <td>
                  {r.current ? <Badge tone="good">current</Badge> : r.in_active_lineage ? <Badge tone="neutral">lineage</Badge> : <Badge tone="warn">rolled back</Badge>}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="hint" aria-live="polite">
        Audit log: showing {entries.length} of {total} entries (newest first)
        {older > 0 ? ` — ${older} older not loaded.` : total > 0 ? " — all loaded." : "."}
        {loading ? " Loading…" : ""}
      </p>
      <div className="table-scroll">
        <table className="data-table">
          <thead>
            <tr><th>#</th><th>Time</th><th>Action</th><th>Outcome</th><th>Actor</th><th>Approval</th><th>Before</th><th>After</th><th>Detail</th></tr>
          </thead>
          <tbody>
            {newestFirst.map((e) => (
              <tr key={e.seq}>
                <td>{e.seq}</td>
                <td>{when(e.timestamp_unix_ms)}</td>
                <td>{e.action}</td>
                <td><Badge tone={e.outcome === "applied" ? "good" : e.outcome === "rejected" ? "danger" : "neutral"}>{e.outcome}</Badge></td>
                <td>{e.actor.label}{e.actor.supplied ? "" : " (default)"}</td>
                <td title={e.approval.mechanism}>{e.approval.required ? (e.approval.granted ? "approved" : "not approved") : "n/a"}</td>
                <td className="mono" title={e.before_checksum ?? ""}>{short(e.before_checksum)}</td>
                <td className="mono" title={e.after_checksum ?? ""}>{short(e.after_checksum)}</td>
                <td>{e.detail}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {older > 0 && (
        <div className="button-row">
          <button className="button ghost" onClick={() => void loadOlder()} disabled={loading}>
            Load {Math.min(AUDIT_PAGE_SIZE, older)} older entries ({older} remaining)
          </button>
        </div>
      )}
    </Panel>
  );
}
