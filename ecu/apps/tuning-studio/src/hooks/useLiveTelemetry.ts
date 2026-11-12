import { useEffect, useRef, useState } from "react";
import type { TelemetryFrame } from "../types";
import { wsUrl } from "../api";

export interface LiveState {
  frame: TelemetryFrame | null;
  history: TelemetryFrame[];
  connected: boolean;
  protocol: number | null;
  serverVersion: string | null;
  error: string | null;
}

const EMPTY: LiveState = {
  frame: null,
  history: [],
  connected: false,
  protocol: null,
  serverVersion: null,
  error: null,
};

/** One shared WebSocket. Frames arrive at 50 Hz; React updates at 10 Hz. */
export function useLiveTelemetry(): LiveState {
  const [state, setState] = useState<LiveState>(EMPTY);
  const pending = useRef<TelemetryFrame[]>([]);
  const latest = useRef<TelemetryFrame | null>(null);
  const history = useRef<TelemetryFrame[]>([]);
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    let closed = false;
    let reconnect: number | undefined;
    let retry = 0;

    const connect = () => {
      if (closed) return;
      const ws = new WebSocket(wsUrl());
      wsRef.current = ws;
      ws.onopen = () => {
        retry = 0;
        setState((s) => ({ ...s, connected: true, error: null }));
      };
      ws.onmessage = (message) => {
        try {
          const envelope = JSON.parse(message.data as string) as {
            protocol: number;
            event: { type: string; [key: string]: unknown };
          };
          if (envelope.event.type === "hello") {
            setState((s) => ({
              ...s,
              protocol: envelope.protocol,
              serverVersion: String(
                (envelope.event as { server_version?: string }).server_version ?? "",
              ),
            }));
          } else if (envelope.event.type === "telemetry") {
            const frame = (envelope.event as unknown as { frame: TelemetryFrame }).frame;
            if (!frame) return;
            pending.current.push(frame);
            latest.current = frame;
            history.current.push(frame);
            if (history.current.length > 600) history.current.splice(0, history.current.length - 600);
          }
        } catch {
          setState((s) => ({ ...s, error: "Malformed telemetry envelope" }));
        }
      };
      ws.onerror = () => setState((s) => ({ ...s, error: "WebSocket error" }));
      ws.onclose = () => {
        setState((s) => ({ ...s, connected: false }));
        if (!closed) {
          retry = Math.min(retry + 1, 5);
          reconnect = window.setTimeout(connect, 500 * retry);
        }
      };
    };
    connect();

    const flush = window.setInterval(() => {
      if (latest.current) {
        setState((s) => ({
          ...s,
          frame: latest.current,
          history: [...history.current],
        }));
        pending.current = [];
      }
    }, 100);

    return () => {
      closed = true;
      window.clearInterval(flush);
      if (reconnect) window.clearTimeout(reconnect);
      wsRef.current?.close();
    };
  }, []);

  return state;
}
