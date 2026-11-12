import { useApp } from "../context";
import { Badge, Panel } from "../components/ui";
import { ALL_CHANNELS, formatChannelValue, readChannel } from "../channels";
import type { TelemetryFrame } from "../types";

export function ChannelsPage() {
  const app = useApp();
  const frame = app.live.frame;
  const groups = Array.from(new Set(ALL_CHANNELS.map((channel) => channel.group)));
  const connected = ALL_CHANNELS.filter((channel) => channel.source === "telemetry").length;
  const missing = ALL_CHANNELS.length - connected;

  return (
    <div className="page">
      <Panel
        title="Channels & sensors"
        subtitle="Read-only table of the channels the service exposes · units, source and last value"
        actions={
          <div className="status-strip">
            <Badge tone="good">{connected} service channels</Badge>
            <Badge tone="neutral">{missing} not connected</Badge>
          </div>
        }
      >
        <div className="alert info">
          <strong>BENCH PROTOTYPE · SIMULATOR DATA.</strong> This table is read-only. Every connected value
          comes from the virtual ECU telemetry frame, not from vehicle hardware. Channels with no source are
          marked “not connected”; none are synthesised.
        </div>
        <div className="status-strip channel-strip">
          <Badge tone={app.live.connected ? "good" : "danger"}>
            websocket: {app.live.connected ? "connected" : "disconnected"}
          </Badge>
          <Badge tone={frame ? "info" : "neutral"}>
            last frame: {frame ? `#${frame.seq} · ${(frame.timestamp_us / 1_000_000).toFixed(2)} s` : "none"}
          </Badge>
          <span className="muted">Values refresh with the live telemetry stream; this page never writes to the service.</span>
        </div>
        <div className="table-scroll channel-table-wrap">
          <table className="data-table channel-table">
            <thead>
              <tr>
                <th>Channel</th>
                <th>Unit</th>
                <th>Source</th>
                <th>Last value</th>
                <th>State</th>
              </tr>
            </thead>
            <tbody>
              {groups.map((group) => (
                <ChannelGroup
                  key={group}
                  group={group}
                  channels={ALL_CHANNELS.filter((channel) => channel.group === group)}
                  frame={frame}
                />
              ))}
            </tbody>
          </table>
        </div>
        <p className="hint">
          The service reports engine-wide injector pulse width and duty only. Per-cylinder injector, trim and
          exhaust-temperature channels are listed so the missing source is explicit. Read-only: there are no
          inputs, writes or hardware configuration controls on this screen.
        </p>
      </Panel>
    </div>
  );
}

function ChannelGroup({
  group,
  channels,
  frame,
}: {
  group: string;
  channels: typeof ALL_CHANNELS;
  frame: TelemetryFrame | null;
}) {
  return (
    <>
      <tr className="channel-group-row">
        <td colSpan={5}>{group}</td>
      </tr>
      {channels.map((channel) => {
        const value = readChannel(channel, frame);
        const notConnected = channel.source === "not_connected";
        return (
          <tr key={channel.key} className={notConnected ? "channel-row-missing" : ""}>
            <td>
              <span className="channel-label">{channel.label}</span>
              <span className="channel-key mono">{channel.key}</span>
            </td>
            <td>{channel.unit}</td>
            <td>
              {notConnected ? (
                <Badge tone="warn">not connected</Badge>
              ) : (
                <Badge tone="info">simulator telemetry</Badge>
              )}
            </td>
            <td>
              {notConnected ? (
                <span className="missing-source">not connected</span>
              ) : value === null ? (
                <span className="muted">no value in frame</span>
              ) : (
                `${formatChannelValue(channel, value)} ${channel.unit}`
              )}
            </td>
            <td>
              {notConnected ? (
                <span className="muted">no source</span>
              ) : value === null ? (
                <span className="muted">unavailable / not ready</span>
              ) : (
                <span className="channel-live">reporting</span>
              )}
            </td>
          </tr>
        );
      })}
    </>
  );
}
