import { useState } from "react";
import { api } from "../api";
import { Panel, Badge } from "../components/ui";

interface Message {
  role: "user" | "assistant" | "system";
  content: string;
  toolCalls?: { name: string; summary: string }[];
  findings?: string[];
  nextActions?: string[];
  approvalRequired?: boolean;
  riskLevel?: string;
}

const SUGGESTIONS = [
  "Summarize the current telemetry and calibration status.",
  "Run a guided Mixture Map analysis and explain the proposed corrections.",
  "Which learning gates rejected samples, and why?",
  "Explain the active DTCs and what evidence would be needed to clear them.",
];

export function CopilotPage() {
  const [messages, setMessages] = useState<Message[]>([
    {
      role: "system",
      content:
        "The Tuning Copilot is read-only by default. It can inspect logs, run the audited Mixture Map analysis and explain results. It cannot apply fuel changes, alter ignition, disable safety gates or actuate hardware. Any calibration application requires your explicit approval in the Mixture Map screen.",
    },
  ]);
  const [input, setInput] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const send = async (text: string) => {
    if (!text.trim() || busy) return;
    const next: Message[] = [...messages, { role: "user", content: text }];
    setMessages(next);
    setInput("");
    setBusy(true);
    setError(null);
    try {
      const triage = next.filter((m) => m.role !== "system").slice(-12).map((m) => ({ role: m.role, content: m.content }));
      const result = await api.copilot(text, triage);
      setMessages((m) => [
        ...m,
        {
          role: "assistant",
          content: result.reply,
          toolCalls: result.toolCalls,
          findings: result.findings,
          nextActions: result.nextActions,
          approvalRequired: result.approvalRequired,
          riskLevel: result.riskLevel,
        },
      ]);
    } catch (e) {
      setError(
        e instanceof Error
          ? `${e.message} — start the Mastra copilot service on port 8787 and set its DeepSeek API key, or use the offline tool mode.`
          : String(e),
      );
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="page">
      <Panel
        title="Tuning Copilot"
        subtitle="Mastra + structured tools · read-only analysis · human approval required for any calibration change"
        actions={<Badge tone="warn">LLM output is untrusted input</Badge>}
      >
        <div className="alert info">
          The copilot is an analyst, not an authority. It cannot select ignition timing, certify a tune, or
          bypass the Rust service's checksum, staleness, gate and approval checks. Tool results and every prompt
          are auditable. Never treat an AI recommendation as physical safety evidence.
        </div>
        <div className="chat">
          {messages.map((m, i) => (
            <article className={`chat-message ${m.role}`} key={i}>
              <header>{m.role === "assistant" ? "Copilot" : m.role === "user" ? "You" : "Safety boundary"}</header>
              <p>{m.content}</p>
              {m.riskLevel && (
                <div className="chat-meta">
                  <Badge tone={m.riskLevel === "blocked" ? "danger" : m.riskLevel === "caution" ? "warn" : "info"}>
                    risk: {m.riskLevel}
                  </Badge>
                  {m.approvalRequired && <Badge tone="warn">human approval required</Badge>}
                </div>
              )}
              {m.findings && m.findings.length > 0 && (
                <ul className="chat-findings">
                  {m.findings.map((f, j) => <li key={j}>{f}</li>)}
                </ul>
              )}
              {m.nextActions && m.nextActions.length > 0 && (
                <div className="chat-next"><strong>Next</strong> {m.nextActions.join(" · ")}</div>
              )}
              {m.toolCalls && m.toolCalls.length > 0 && (
                <div className="tool-calls">
                  {m.toolCalls.map((t, j) => (
                    <div key={j}><Badge tone="info">{t.name}</Badge> {t.summary}</div>
                  ))}
                </div>
              )}
            </article>
          ))}
          {busy && <div className="chat-message assistant"><header>Copilot</header><p className="muted">Working through the audited tools…</p></div>}
          {error && <div className="alert danger">{error}</div>}
        </div>
        <div className="suggestions">
          {SUGGESTIONS.map((s) => (
            <button key={s} className="button ghost" onClick={() => send(s)} disabled={busy}>{s}</button>
          ))}
        </div>
        <form
          className="chat-input"
          onSubmit={(e) => {
            e.preventDefault();
            send(input);
          }}
        >
          <input
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Ask about the log, gates, fuel corrections or diagnostics…"
            disabled={busy}
          />
          <button className="button primary" type="submit" disabled={busy || !input.trim()}>Send</button>
        </form>
        <p className="hint">
          No API key? The Mastra service also exposes a deterministic offline mode that exercises the same
          structured tools without an LLM call. See docs/LLM-COPILOT.md.
        </p>
      </Panel>
    </div>
  );
}
