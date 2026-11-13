import { useEffect, useState } from "react";
import { api, type AuditEntry, type StoreSummary, type StoredRevision } from "../api";
import { Badge, Panel } from "./ui";

const short = (c: string | null) => (c ? `${c.slice(0, 12)}…` : "—");
const when = (ms: number) => (ms ? new Date(ms).toLocaleString() : "—");

/**
 * Read-only view of the durable calibration store. It cannot apply, undo or
 * edit anything; it only displays what the service has recorded on disk.
 */
export function AuditTrail({ refreshKey }: { refreshKey?: unknown }) {
  const [store, setStore] = useState<StoreSummary | null>(null);
  const [revisions, setRevisions] = useState<StoredRevision[]>([]);
  const [entries, setEntries] = useState<AuditEntry[]>([]);
  const [verify, setVerify] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = () =>
    Promise.all([api.revisions(), api.audit()])
      .then(([r, a]) => {
        setStore(r.store);
        setRevisions(r.revisions);
        setEntries([...a.entries].reverse());
        setError(null);
      })
      .catch((e) => setError(e instanceof Error ? e.message : String(e)));

  useEffect(() => {
    load();
  }, [refreshKey]);

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
          <button className="button ghost" onClick={load}>Refresh</button>
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
      <div className="table-scroll">
        <table className="data-table">
          <thead>
            <tr><th>#</th><th>Time</th><th>Action</th><th>Outcome</th><th>Actor</th><th>Approval</th><th>Before</th><th>After</th><th>Detail</th></tr>
          </thead>
          <tbody>
            {entries.map((e) => (
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
    </Panel>
  );
}
