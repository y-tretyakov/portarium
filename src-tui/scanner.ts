import { spawn, spawnSync } from "node:child_process";
import { existsSync, readFileSync, readlinkSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import type { PortInfo } from "./types";

const PROJECT_MARKERS = [
  "package.json",
  "Cargo.toml",
  "go.mod",
  "pyproject.toml",
  "requirements.txt",
  "pom.xml",
  "build.gradle",
  ".git",
];

function sleepMs(ms: number): void {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
}

function run(cmd: string, args: string[]): string {
  const result = spawnSync(cmd, args, {
    encoding: "utf8",
    maxBuffer: 10 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  return result.stdout ?? "";
}

function findProjectRoot(start: string | null): string | null {
  if (!start) return null;
  let dir = start;
  for (let i = 0; i < 6; i++) {
    for (const marker of PROJECT_MARKERS) {
      if (existsSync(join(dir, marker))) return dir;
    }
    const parent = dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return null;
}

function extractProjectName(path: string | null): string | null {
  if (!path) return null;
  const name = basename(path);
  return name || null;
}

function getProcessMetaUnix(pid: number): {
  process_name: string | null;
  project_path: string | null;
  start_cmd: string | null;
} {
  let process_name: string | null = null;
  let project_path: string | null = null;
  let start_cmd: string | null = null;

  try {
    const cmdline = readFileSync(`/proc/${pid}/cmdline`, "utf8");
    if (cmdline) {
      start_cmd = cmdline.replace(/\0/g, " ").trim() || null;
    }
  } catch {
    // ignore
  }

  try {
    const cwd = readlinkSync(`/proc/${pid}/cwd`);
    project_path = findProjectRoot(cwd);
  } catch {
    // ignore
  }

  try {
    const comm = readFileSync(`/proc/${pid}/comm`, "utf8").trim();
    if (comm) process_name = comm;
  } catch {
    // ignore
  }

  if (!project_path && process.platform === "darwin") {
    try {
      const out = run("lsof", ["-p", String(pid), "-a", "-d", "cwd", "-Fn"]);
      const line = out.split("\n").find((l) => l.startsWith("n") && l.length > 1);
      if (line) {
        project_path = findProjectRoot(line.slice(1));
      }
    } catch {
      // ignore
    }
  }

  if (!process_name) {
    try {
      const out = run("ps", ["-p", String(pid), "-o", "comm="]);
      const name = out.trim();
      if (name) process_name = name;
    } catch {
      // ignore
    }
  }

  if (!start_cmd) {
    try {
      const out = run("ps", ["-p", String(pid), "-o", "args="]);
      const args = out.trim();
      if (args) start_cmd = args;
    } catch {
      // ignore
    }
  }

  return { process_name, project_path, start_cmd };
}

function scanUnix(): PortInfo[] {
  const stdout = run("lsof", ["-iTCP", "-sTCP:LISTEN", "-n", "-P"]);
  const ports: PortInfo[] = [];
  const seen = new Set<string>();

  for (const line of stdout.split("\n").slice(1)) {
    const parts = line.trim().split(/\s+/);
    if (parts.length < 9) continue;

    let process_name = parts[0];
    const pid = Number(parts[1]);
    if (!Number.isFinite(pid) || pid <= 0) continue;

    // NAME may be "*:3000", "127.0.0.1:3000", or "127.0.0.1:3000 (LISTEN)"
    const addrPart =
      parts.find((p) => /:\d+$/.test(p)) ??
      parts.find((p) => p.includes(":") && /\d/.test(p));
    if (!addrPart) continue;
    const portMatch = addrPart.match(/:(\d+)\s*$/);
    const port = portMatch ? Number(portMatch[1]) : NaN;
    if (!Number.isFinite(port)) continue;

    const key = `${port}:${pid}`;
    if (seen.has(key)) continue;
    seen.add(key);

    const meta = getProcessMetaUnix(pid);
    if (meta.process_name) process_name = meta.process_name;

    ports.push({
      port,
      pid,
      process_name,
      project_path: meta.project_path,
      project_name: extractProjectName(meta.project_path),
      protocol: "TCP",
      start_cmd: meta.start_cmd,
    });
  }

  ports.sort((a, b) => a.port - b.port);
  return ports;
}

function scanWindows(): PortInfo[] {
  const stdout = run("netstat", ["-ano"]);
  const ports: PortInfo[] = [];
  const seen = new Set<string>();

  for (const line of stdout.split("\n")) {
    if (!line.includes("LISTENING")) continue;
    const parts = line.trim().split(/\s+/);
    if (parts.length < 5) continue;

    const local = parts[1];
    const portStr = local.includes(":") ? local.split(":").pop() : null;
    const port = portStr ? Number(portStr) : NaN;
    const pid = Number(parts[parts.length - 1]);
    if (!Number.isFinite(port) || !Number.isFinite(pid) || pid <= 0) continue;

    const key = `${port}:${pid}`;
    if (seen.has(key)) continue;
    seen.add(key);

    let process_name = `PID ${pid}`;
    let start_cmd: string | null = null;
    try {
      const out = run("tasklist", ["/FI", `PID eq ${pid}`, "/FO", "CSV", "/NH"]);
      const m = out.match(/"([^"]+)","(\d+)"/);
      if (m) process_name = m[1];
    } catch {
      // ignore
    }

    try {
      const out = run("wmic", [
        "process",
        "where",
        `ProcessId=${pid}`,
        "get",
        "CommandLine",
        "/value",
      ]);
      const cmdLine = out
        .split("\n")
        .find((l) => l.startsWith("CommandLine="))
        ?.slice("CommandLine=".length)
        .trim();
      if (cmdLine) start_cmd = cmdLine;
    } catch {
      // ignore
    }

    ports.push({
      port,
      pid,
      process_name,
      project_path: null,
      project_name: null,
      protocol: "TCP",
      start_cmd,
    });
  }

  ports.sort((a, b) => a.port - b.port);
  return ports;
}

/** Scan listening TCP ports (mirrors the Rust scanner logic). */
export function scanPorts(): PortInfo[] {
  if (process.platform === "win32") return scanWindows();
  return scanUnix();
}

export function killProcess(pid: number): void {
  if (process.platform === "win32") {
    const result = spawnSync("taskkill", ["/PID", String(pid), "/F"], {
      encoding: "utf8",
    });
    if (result.status !== 0) {
      throw new Error(result.stderr || `Failed to kill PID ${pid}`);
    }
    return;
  }

  try {
    process.kill(pid, "SIGTERM");
  } catch (e) {
    throw new Error(`Failed to kill PID ${pid}: ${e}`);
  }

  const deadline = Date.now() + 2000;
  while (Date.now() < deadline) {
    try {
      process.kill(pid, 0);
      sleepMs(50);
    } catch {
      return;
    }
  }

  try {
    process.kill(pid, "SIGKILL");
  } catch {
    // already dead
  }
}

export function restartProcess(pid: number, cmd: string, cwd: string): void {
  killProcess(pid);
  sleepMs(800);

  if (process.platform === "win32") {
    const child = spawn("cmd", ["/C", "start", "cmd", "/K", cmd], {
      cwd,
      detached: true,
      stdio: "ignore",
    });
    child.unref();
    return;
  }

  const parts = cmd.split(/\s+/).filter(Boolean);
  const program = parts[0];
  if (!program) throw new Error("empty command");
  const args = parts.slice(1);

  const child = spawn(program, args, {
    cwd,
    detached: true,
    stdio: "ignore",
  });
  child.unref();
}
