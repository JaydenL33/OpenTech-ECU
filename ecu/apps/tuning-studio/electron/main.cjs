// OpenTech-ECU Electron desktop shell.
// The Rust studio service owns every ECU operation. This shell only launches it
// and presents the local UI; there is no ECU logic or hardware access here.
const { app, BrowserWindow, dialog, shell } = require("electron");
const { spawn } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const PORT = Number(process.env.OPENTECH_PORT || 8765);
let service = null;
let mainWindow = null;

function candidateBinaries() {
  const exe = process.platform === "win32" ? "studio-service.exe" : "studio-service";
  const packaged = path.join(process.resourcesPath || "", "service", exe);
  const dev = path.resolve(__dirname, "../../../target/release", exe);
  const devDebug = path.resolve(__dirname, "../../../target/debug", exe);
  return [process.env.OPENTECH_SERVICE_BIN, packaged, dev, devDebug].filter(Boolean);
}

async function waitForHealth(timeoutMs = 15000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(`http://127.0.0.1:${PORT}/api/health`);
      if (response.ok) return true;
    } catch {
      // service still starting
    }
    await new Promise((resolve) => setTimeout(resolve, 300));
  }
  return false;
}

async function startService() {
  if (await waitForHealth(500)) return true;
  const binary = candidateBinaries().find((p) => fs.existsSync(p));
  if (!binary) {
    dialog.showErrorBox(
      "OpenTech service not found",
      "Build the Rust service first:\n\n  cd ecu && cargo build --release -p studio-service\n\n" +
        "Or set OPENTECH_SERVICE_BIN to the studio-service executable.",
    );
    return false;
  }
  service = spawn(binary, ["--port", String(PORT), "--no-loop"], {
    stdio: "ignore",
    windowsHide: true,
  });
  service.on("exit", (code) => {
    if (mainWindow && !app.isQuitting) {
      console.error(`OpenTech service exited with code ${code}`);
    }
  });
  if (!(await waitForHealth())) {
    dialog.showErrorBox("OpenTech service failed", "The Rust service did not become healthy.");
    return false;
  }
  return true;
}

async function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1480,
    height: 980,
    minWidth: 1100,
    minHeight: 720,
    backgroundColor: "#080b0f",
    title: "OpenTech-ECU Tuning Studio",
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
  });
  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url);
    return { action: "deny" };
  });
  await mainWindow.loadURL(process.env.OPENTECH_UI_URL || `http://127.0.0.1:${PORT}/`);
}

app.whenReady().then(async () => {
  if (!(await startService())) {
    app.quit();
    return;
  }
  await createWindow();
  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});

app.on("before-quit", () => {
  app.isQuitting = true;
  if (service) {
    service.kill();
    service = null;
  }
});
