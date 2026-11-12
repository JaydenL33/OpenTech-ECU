import { Fragment, useEffect, useState } from "react";
import { useApp } from "../context";
import { Panel, Badge } from "../components/ui";
import { CHANNELS, formatChannelValue, readChannel } from "../channels";
import { clearWarningLimits, evaluateWarnings, setChannelLimit, useWarningLimits } from "../warnings";

function LimitInput({
  value,
  onCommit,
}: {
  value: number | null;
  onCommit: (value: number | null) => void;
}) {
  const [text, setText] = useState(value === null ? "" : String(value));

  useEffect(() => {
    setText(value === null ? "" : String(value));
  }, [value]);

  return (
    <input
      type="number"
      step="any"
      value={text}
      placeholder="—"
      onChange={(event) => setText(event.target.value)}
      onBlur={() => {
        const trimmed = text.trim();
        if (!trimmed) {
          onCommit(null);
          return;
        }
        const parsed = Number(trimmed);
        if (Number.isFinite(parsed)) onCommit(parsed);
        else setText(value === null ? "" : String(value));
      }}
      onKeyDown={(event) => {
        if (event.key === "Enter") event.currentTarget.blur();
      }}
    />
  );
}

export function WarningsPage() {
  const app = useApp();
  const limits = useWarningLimits();
  const active = evaluateWarnings(app.live.frame, limits);
  const groups = Array.from(new Set(CHANNELS.map((channel) => channel.group)));

  return (
    <div className="page">
      <Panel
        title="Channel warning limits"
        subtitle="High and low limits per channel · stored in this browser only"
        actions={
          <button className="button ghost" onClick={clearWarningLimits} disabled={Object.keys(limits).length === 0}>
            Clear all limits
          </button>
        }
      >
        <div className="alert info">
          <strong>BENCH PROTOTYPE · SIMULATOR DATA.</strong> Warning limits are a local visual aid. They are
          stored in this browser's localStorage, are never sent to the service, and do not configure ECU
          protection, fuel cut or any physical output.
        </div>
        {active.length > 0 ? (
          <div className="alert danger warning-summary">
            <strong>{active.length} limit{active.length === 1 ? "" : "s"} crossed in the current frame:</strong>
            <div className="warning-list">
              {active.map((warning) => (
                <span key={`${warning.key}-${warning.limit}`}>
                  {warning.label}: {warning.value.toFixed(warning.decimals)} {warning.unit}{" "}
                  ({warning.limit} limit {warning.threshold.toFixed(warning.decimals)} {warning.unit})
                </span>
              ))}
            </div>
          </div>
        ) : (
          <div className="alert warn">
            No configured channel limit is currently crossed. Limits only evaluate when a live simulator frame
            is available.
          </div>
        )}
        <div className="table-scroll warning-table-wrap">
          <table className="data-table warning-table">
            <thead>
              <tr>
                <th>Channel</th>
                <th>Unit</th>
                <th>Current value</th>
                <th>Low limit</th>
                <th>High limit</th>
                <th>State</th>
              </tr>
            </thead>
            <tbody>
              {groups.map((group) => (
                <Fragment key={group}>
                  <tr className="warning-group-row">
                    <td colSpan={6}>{group}</td>
                  </tr>
                  {CHANNELS.filter((channel) => channel.group === group).map((channel) => {
                    const bound = limits[channel.key];
                    const low = bound?.low ?? null;
                    const high = bound?.high ?? null;
                    const value = readChannel(channel, app.live.frame);
                    const invalid = low !== null && high !== null && low >= high;
                    const crossed = active.some((warning) => warning.key === channel.key);
                    return (
                      <tr key={channel.key} className={crossed ? "warning-row-crossed" : ""}>
                        <td>{channel.label}</td>
                        <td>{channel.unit}</td>
                        <td>{formatChannelValue(channel, value)}{value !== null ? ` ${channel.unit}` : ""}</td>
                        <td>
                          <LimitInput
                            value={low}
                            onCommit={(next) => setChannelLimit(channel.key, "low", next)}
                          />
                        </td>
                        <td>
                          <LimitInput
                            value={high}
                            onCommit={(next) => setChannelLimit(channel.key, "high", next)}
                          />
                        </td>
                        <td>
                          {invalid ? (
                            <Badge tone="danger">low ≥ high</Badge>
                          ) : crossed ? (
                            <Badge tone="danger">limit crossed</Badge>
                          ) : low === null && high === null ? (
                            <Badge tone="neutral">no limits</Badge>
                          ) : (
                            <Badge tone="good">within limits</Badge>
                          )}
                        </td>
                      </tr>
                    );
                  })}
                </Fragment>
              ))}
            </tbody>
          </table>
        </div>
        <p className="hint">
          Limits are compared with the latest telemetry frame at the UI refresh rate; they are not a control
          loop and may lag real events. “Low ≥ high” marks a configuration mistake and still evaluates both
          bounds. Clear a field to remove that bound.
        </p>
      </Panel>
    </div>
  );
}
