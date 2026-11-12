import { createServer, type IncomingMessage, type ServerResponse } from "node:http";
import { runCopilot } from "./agent.js";
import { RUST_API } from "./client.js";

const PORT = Number(process.env.COPILOT_PORT ?? 8787);
const ALLOWED_ORIGINS = new Set(
  (process.env.COPILOT_ALLOWED_ORIGINS ??
    "http://127.0.0.1:8765,http://localhost:8765,http://127.0.0.1:5173,http://localhost:5173")
    .split(",")
    .map((s) => s.trim()),
);

function cors(req: IncomingMessage, res: ServerResponse): boolean {
  const origin = req.headers.origin;
  if (origin && ALLOWED_ORIGINS.has(origin)) {
    res.setHeader("access-control-allow-origin", origin);
    res.setHeader("vary", "origin");
  }
  res.setHeader("access-control-allow-methods", "GET,POST,OPTIONS");
  res.setHeader("access-control-allow-headers", "content-type");
  return true;
}

function json(res: ServerResponse, status: number, body: unknown) {
  res.statusCode = status;
  res.setHeader("content-type", "application/json; charset=utf-8");
  res.end(JSON.stringify(body));
}

async function readBody(req: IncomingMessage): Promise<unknown> {
  const chunks: Buffer[] = [];
  let size = 0;
  for await (const chunk of req) {
    const buf = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
    size += buf.length;
    if (size > 1_000_000) throw new Error("request body too large");
    chunks.push(buf);
  }
  if (!chunks.length) return {};
  return JSON.parse(Buffer.concat(chunks).toString("utf8"));
}

const server = createServer(async (req, res) => {
  cors(req, res);
  if (req.method === "OPTIONS") {
    res.statusCode = 204;
    res.end();
    return;
  }
  if (req.method === "GET" && req.url === "/health") {
    json(res, 200, {
      status: "ok",
      service: "opentech-tuning-copilot",
      mode:
        process.env.COPILOT_ENABLE_LLM === "1" &&
        Boolean(process.env.DEEPSEEK_API_KEY || process.env.OPENAI_API_KEY || process.env.OPENROUTER_API_KEY)
          ? "llm-enabled"
          : "offline-tools",
      model: process.env.COPILOT_MODEL ?? "deepseek/deepseek-v4-pro",
      rust_api: RUST_API,
      safety: "read-only tools; no calibration writes, no ignition advice, no hardware access",
    });
    return;
  }
  if (req.method === "POST" && req.url === "/api/copilot") {
    try {
      const body = (await readBody(req)) as {
        message?: string;
        history?: { role: string; content: string }[];
      };
      if (!body.message || typeof body.message !== "string" || !body.message.trim()) {
        json(res, 400, { error: "message is required" });
        return;
      }
      if (body.message.length > 8000) {
        json(res, 400, { error: "message too long" });
        return;
      }
      const history = Array.isArray(body.history) ? body.history.slice(-20) : [];
      const result = await runCopilot(body.message, history);
      json(res, 200, {
        reply: result.answer.reply,
        findings: result.answer.findings,
        proposedFuelCells: result.answer.proposedFuelCells,
        approvalRequired: result.answer.requiresHumanApproval,
        riskLevel: result.answer.riskLevel,
        nextActions: result.answer.nextActions,
        ignitionAdvice: result.answer.ignitionAdvice,
        toolCalls: result.toolCalls,
        mode: result.mode,
        model: result.model,
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      console.error("copilot error:", message);
      json(res, 502, {
        error: message,
        hint: "Ensure the Rust studio service is running on port 8765. Offline tool mode requires no API key.",
      });
    }
    return;
  }
  json(res, 404, { error: "not found" });
});

server.listen(PORT, "127.0.0.1", () => {
  console.log(
    `OpenTech Tuning Copilot on http://127.0.0.1:${PORT} (Rust API ${RUST_API}, LLM ${
      process.env.COPILOT_ENABLE_LLM === "1" ? "enabled-if-key" : "offline-tools"
    })`,
  );
});
