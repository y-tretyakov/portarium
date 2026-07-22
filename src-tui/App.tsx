import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useRenderer, useTerminalDimensions } from "@opentui/react";
import type { KeyEvent } from "@opentui/core";
import { DEV_PORTS, getFramework, getServiceName, isDevPort, timeAgo } from "./frameworks";
import { killProcess, restartProcess, scanPorts } from "./scanner";
import type { NavPage, PortEvent, PortFilter, PortInfo } from "./types";

const SCAN_INTERVAL_MS = 2500;
const PAGES: NavPage[] = ["ports", "dashboard", "services", "logs"];
const FILTERS: PortFilter[] = ["all", "dev", "other"];

function portKey(p: PortInfo): string {
  return `${p.port}:${p.pid}`;
}

function truncate(s: string, n: number): string {
  if (s.length <= n) return s;
  return s.slice(0, Math.max(0, n - 1)) + "…";
}

function pad(s: string, w: number): string {
  const t = truncate(s, w);
  return t + " ".repeat(Math.max(0, w - t.length));
}

export default function App() {
  const renderer = useRenderer();
  const { width, height } = useTerminalDimensions();

  const [page, setPage] = useState<NavPage>("ports");
  const [filter, setFilter] = useState<PortFilter>("all");
  const [ports, setPorts] = useState<PortInfo[]>([]);
  const [events, setEvents] = useState<PortEvent[]>([]);
  const [selected, setSelected] = useState(0);
  const [searchMode, setSearchMode] = useState(false);
  const [search, setSearch] = useState("");
  const [toast, setToast] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const firstScan = useRef(true);

  const showToast = useCallback((msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 2500);
  }, []);

  const refresh = useCallback(() => {
    try {
      const next = scanPorts();
      setPorts((prevPorts) => {
        if (!firstScan.current) {
          const prevMap = new Map(prevPorts.map((p) => [p.port, p]));
          const nextMap = new Map(next.map((p) => [p.port, p]));
          const newEvents: PortEvent[] = [];

          for (const p of next) {
            if (!prevMap.has(p.port)) {
              newEvents.push({
                port: p.port,
                pid: p.pid,
                process_name: p.process_name,
                framework: getFramework(p.port),
                event_type: "started",
                timestamp: Date.now(),
              });
            }
          }
          for (const [port, old] of prevMap) {
            if (!nextMap.has(port)) {
              newEvents.push({
                port,
                pid: old.pid,
                process_name: old.process_name,
                framework: getFramework(port),
                event_type: "stopped",
                timestamp: Date.now(),
              });
            }
          }

          if (newEvents.length > 0) {
            setEvents((ev) => [...newEvents, ...ev].slice(0, 200));
          }
        }
        firstScan.current = false;
        return next;
      });
      setError(null);
      setLoading(false);
    } catch (e) {
      setError(String(e));
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
    const id = setInterval(refresh, SCAN_INTERVAL_MS);
    return () => clearInterval(id);
  }, [refresh]);

  const filteredPorts = useMemo(() => {
    let list = ports;
    if (filter === "dev") list = list.filter((p) => isDevPort(p.port));
    else if (filter === "other") list = list.filter((p) => !isDevPort(p.port));

    const q = search.toLowerCase().trim();
    if (!q) return list;
    return list.filter((p) => {
      const dev = DEV_PORTS[p.port];
      return (
        String(p.port).includes(q) ||
        p.process_name.toLowerCase().includes(q) ||
        (p.project_name?.toLowerCase().includes(q) ?? false) ||
        (dev?.label.toLowerCase().includes(q) ?? false)
      );
    });
  }, [ports, filter, search]);

  useEffect(() => {
    if (selected >= filteredPorts.length) {
      setSelected(Math.max(0, filteredPorts.length - 1));
    }
  }, [filteredPorts.length, selected]);

  const selectedPort = filteredPorts[selected] ?? null;

  const handleKill = useCallback(() => {
    if (!selectedPort) return;
    try {
      killProcess(selectedPort.pid);
      showToast(`Killed ${selectedPort.process_name} on :${selectedPort.port}`);
      setTimeout(refresh, 300);
    } catch (e) {
      showToast(`Kill failed: ${e}`);
    }
  }, [selectedPort, refresh, showToast]);

  const handleRestart = useCallback(() => {
    if (!selectedPort?.start_cmd || !selectedPort.project_path) {
      showToast("No start command / project path for restart");
      return;
    }
    try {
      restartProcess(selectedPort.pid, selectedPort.start_cmd, selectedPort.project_path);
      showToast(`Restarting ${selectedPort.project_name ?? selectedPort.process_name}…`);
      setTimeout(refresh, 500);
    } catch (e) {
      showToast(`Restart failed: ${e}`);
    }
  }, [selectedPort, refresh, showToast]);

  const handleKillAll = useCallback(() => {
    if (filteredPorts.length === 0) return;
    let n = 0;
    for (const p of filteredPorts) {
      try {
        killProcess(p.pid);
        n++;
      } catch {
        // continue
      }
    }
    showToast(`Killed ${n} process${n === 1 ? "" : "es"}`);
    setTimeout(refresh, 300);
  }, [filteredPorts, refresh, showToast]);

  const handleKeyRef = useRef<((key: KeyEvent) => void) | null>(null);

  handleKeyRef.current = (key: KeyEvent) => {
    if (searchMode) {
      if (key.name === "escape" || key.name === "return") {
        setSearchMode(false);
        return;
      }
      if (key.name === "backspace") {
        setSearch((s) => s.slice(0, -1));
        return;
      }
      if (key.sequence && key.sequence.length === 1 && !key.ctrl && !key.meta) {
        setSearch((s) => s + key.sequence);
      }
      return;
    }

    if (key.ctrl && key.name === "c") {
      renderer.destroy();
      return;
    }

    if (key.name === "1") {
      setPage("ports");
      return;
    }
    if (key.name === "2") {
      setPage("dashboard");
      return;
    }
    if (key.name === "3") {
      setPage("services");
      return;
    }
    if (key.name === "4") {
      setPage("logs");
      return;
    }

    if (key.name === "q") {
      renderer.destroy();
      return;
    }

    if (key.name === "tab") {
      setPage((p) => {
        const idx = PAGES.indexOf(p);
        const next = key.shift
          ? (idx - 1 + PAGES.length) % PAGES.length
          : (idx + 1) % PAGES.length;
        return PAGES[next];
      });
      return;
    }

    if (key.name === "f") {
      setFilter((f) => FILTERS[(FILTERS.indexOf(f) + 1) % FILTERS.length]);
      setSelected(0);
      return;
    }

    if (key.name === "u") {
      refresh();
      showToast("Refreshed");
      return;
    }

    if (key.name === "escape") {
      setSearch("");
      return;
    }

    if (key.name === "slash" || key.sequence === "/") {
      setSearchMode(true);
      return;
    }

    if (key.name === "up") {
      setSelected((i) => Math.max(0, i - 1));
      return;
    }
    if (key.name === "down" || key.name === "j") {
      setSelected((i) => Math.min(Math.max(0, filteredPorts.length - 1), i + 1));
      return;
    }

    if (key.name === "k") {
      if (key.shift) handleKillAll();
      else handleKill();
      return;
    }
    if (key.name === "r") {
      handleRestart();
      return;
    }
  };

  useEffect(() => {
    const handler = (key: KeyEvent) => handleKeyRef.current?.(key);
    renderer.keyInput.on("keypress", handler);
    return () => { renderer.keyInput.off("keypress", handler); };
  }, [renderer]);

  const fwCount = useMemo(
    () => new Set(ports.map((p) => DEV_PORTS[p.port]?.label).filter(Boolean)).size,
    [ports],
  );

  const contentHeight = Math.max(5, height - 16);

  return (
    <box
      width={width}
      height={height}
      flexDirection="column"
      backgroundColor="#0a0a14"
      padding={1}
      gap={0}
    >
      {/* Header */}
      <box flexDirection="row" marginBottom={1}>
        <box flexDirection="column" flexGrow={1}>
          <text fg="#7c6fff">$$$$$$$\   $$$$$$\  $$$$$$$\ $$$$$$$$\  $$$$$$\  $$$$$$$\  $$$$$$\ $$\   $$\ $$\      $$\</text>
          <text fg="#7c6fff">$$  __$$\ $$  __$$\ $$  __$$\\__$$  __|$$  __$$\ $$  __$$\ \_$$  _|$$ |  $$ |$$$\    $$$ |</text>
          <text fg="#7c6fff">$$ |  $$ |$$ /  $$ |$$ |  $$ |  $$ |   $$ /  $$ |$$ |  $$ |  $$ |  $$ |  $$ |$$$$\  $$$$ |</text>
          <text fg="#7c6fff">$$$$$$$  |$$ |  $$ |$$$$$$$  |  $$ |   $$$$$$$$ |$$$$$$$  |  $$ |  $$ |  $$ |$$\$$\$$ $$ |</text>
          <text fg="#7c6fff">{'$$  ____/ $$ |  $$ |$$  __$$<   $$ |   $$  __$$ |$$  __$$<   $$ |  $$ |  $$ |$$ \\$$$  $$ |'}</text>
          <text fg="#7c6fff">$$ |      $$ |  $$ |$$ |  $$ |  $$ |   $$ |  $$ |$$ |  $$ |  $$ |  $$ |  $$ |$$ |\$  /$$ |</text>
          <text fg="#7c6fff">$$ |       $$$$$$  |$$ |  $$ |  $$ |   $$ |  $$ |$$ |  $$ |$$$$$$\ \$$$$$$  |$$ | \_/ $$ |</text>
          <text fg="#7c6fff">\__|       \______/ \__|  \__|  \__|   \__|  \__|\__|  \__|\______| \______/ \__|     \__|</text>
        </box>
        <box flexDirection="column" justifyContent="center" marginLeft={2}>
          <text>
            <span fg="#22c55e">{ports.length}</span>
            <span fg="#4a4a6a"> ports</span>
          </text>
          <text>
            <span fg="#7c6fff">{fwCount}</span>
            <span fg="#4a4a6a"> frameworks</span>
          </text>
        </box>
      </box>

      {/* Nav tabs */}
      <box flexDirection="row" gap={1} height={1} marginBottom={1}>
        {PAGES.map((p, i) => {
          const active = page === p;
          const label = p.charAt(0).toUpperCase() + p.slice(1);
          return (
            <text key={p}>
              {active ? (
                <b fg="#0a0a14" bg="#7c6fff">
                  {` ${i + 1}:${label} `}
                </b>
              ) : (
                <span fg="#8b8ba3">{` ${i + 1}:${label} `}</span>
              )}
            </text>
          );
        })}
        <box flexGrow={1} />
        <text>
          <span fg="#4a4a6a">filter:</span>
          <span fg="#a5b4fc"> {filter}</span>
          {search ? (
            <>
              <span fg="#4a4a6a"> · /</span>
              <span fg="#fbbf24">{search}</span>
            </>
          ) : null}
          {searchMode ? <span fg="#fbbf24">_</span> : null}
        </text>
      </box>

      {/* Body */}
      <box
        flexGrow={1}
        flexDirection="column"
        borderStyle="rounded"
        borderColor="#2a2a45"
        padding={1}
        backgroundColor="#0d0d1a"
      >
        {error ? (
          <text fg="#ef4444">Error: {error}</text>
        ) : loading ? (
          <text fg="#6e7681">Scanning ports…</text>
        ) : page === "ports" ? (
          <PortsView
            ports={filteredPorts}
            selected={selected}
            height={contentHeight}
            width={Math.max(40, width - 6)}
          />
        ) : page === "dashboard" ? (
          <DashboardView ports={ports} events={events} fwCount={fwCount} />
        ) : page === "services" ? (
          <ServicesView ports={ports} height={contentHeight} />
        ) : (
          <LogsView events={events} height={contentHeight} />
        )}
      </box>

      {/* Footer */}
      <box flexDirection="row" height={1} marginTop={1} justifyContent="space-between">
        <text>
          {toast ? (
            <span fg="#22c55e">{toast}</span>
          ) : searchMode ? (
            <span fg="#fbbf24">Search: type to filter · Enter/Esc to exit</span>
          ) : (
            <span fg="#3a3a55">
              ↑↓/j nav · k kill · K kill-all · r restart · / search · f filter · Tab page · u
              refresh · q quit
            </span>
          )}
        </text>
        {selectedPort && page === "ports" ? (
          <text>
            <span fg="#4a4a6a">sel </span>
            <span fg={DEV_PORTS[selectedPort.port]?.color ?? "#7c6fff"}>
              :{selectedPort.port}
            </span>
            <span fg="#6e7681"> pid {selectedPort.pid}</span>
          </text>
        ) : null}
      </box>
    </box>
  );
}

function PortsView({
  ports,
  selected,
  height,
  width,
}: {
  ports: PortInfo[];
  selected: number;
  height: number;
  width: number;
}) {
  if (ports.length === 0) {
    return (
      <box flexDirection="column" gap={1}>
        <text fg="#6e7681">No ports in use</text>
        <text fg="#3a3a55">Start a server and it will appear here</text>
      </box>
    );
  }

  const visible = Math.max(3, height - 2);
  let start = Math.max(0, selected - Math.floor(visible / 2));
  if (start + visible > ports.length) start = Math.max(0, ports.length - visible);
  const slice = ports.slice(start, start + visible);

  const portW = 6;
  const fwW = 10;
  const svcW = Math.max(10, Math.min(22, Math.floor(width * 0.28)));
  const procW = Math.max(10, Math.min(28, Math.floor(width * 0.32)));

  return (
    <box flexDirection="column" gap={0}>
      <text>
        <span fg="#4a4a6a">
          <b>
            {pad("PORT", portW)} {pad("FRAMEWORK", fwW)} {pad("SERVICE", svcW)}{" "}
            {pad("PROCESS", procW)} PID
          </b>
        </span>
      </text>
      {slice.map((p, i) => {
        const idx = start + i;
        const active = idx === selected;
        const dev = DEV_PORTS[p.port];
        const color = dev?.color ?? "#8b8ba3";
        const fw = pad(dev?.label ?? p.protocol, fwW);
        const svc = pad(truncate(getServiceName(p), svcW), svcW);
        const proc = pad(truncate(p.process_name, procW), procW);
        const line = `${pad(`:${p.port}`, portW)} ${fw} ${svc} ${proc} ${p.pid}`;

        return (
          <text key={portKey(p)}>
            {active ? (
              <b fg="#ffffff" bg="#2a2450">
                ▸ {line}
              </b>
            ) : (
              <span fg={color}>
                {"  "}
                {line}
              </span>
            )}
          </text>
        );
      })}
      {ports.length > visible ? (
        <text fg="#3a3a55">
          {"  "}
          {start + 1}–{start + slice.length} of {ports.length}
        </text>
      ) : null}
    </box>
  );
}

function DashboardView({
  ports,
  events,
  fwCount,
}: {
  ports: PortInfo[];
  events: PortEvent[];
  fwCount: number;
}) {
  const recent = events.slice(0, 6);
  const devPorts = ports.filter((p) => isDevPort(p.port));

  return (
    <box flexDirection="column" gap={1}>
      <text>
        <b fg="#e2e8f0">Dashboard</b>
      </text>
      <box flexDirection="row" gap={2}>
        <Stat label="Active Ports" value={String(ports.length)} color="#22c55e" />
        <Stat label="Dev Servers" value={String(devPorts.length)} color="#7c6fff" />
        <Stat label="Frameworks" value={String(fwCount)} color="#61dafb" />
        <Stat label="Events" value={String(events.length)} color="#eab308" />
      </box>

      <text>
        <b fg="#8b8ba3">Active services</b>
      </text>
      {ports.slice(0, 8).map((p) => {
        const dev = DEV_PORTS[p.port];
        return (
          <text key={portKey(p)}>
            <span fg={dev?.color ?? "#7c6fff"}>:{p.port}</span>
            <span fg="#4a4a6a"> · </span>
            <span fg="#c8c8d8">{getServiceName(p)}</span>
            <span fg="#4a4a6a"> · </span>
            <span fg="#6e7681">{p.process_name}</span>
          </text>
        );
      })}

      <text>
        <b fg="#8b8ba3">Recent events</b>
      </text>
      {recent.length === 0 ? (
        <text fg="#3a3a55">No events yet</text>
      ) : (
        recent.map((ev, i) => (
          <text key={`${ev.timestamp}-${i}`}>
            <span fg={ev.event_type === "started" ? "#22c55e" : "#ef4444"}>
              {ev.event_type === "started" ? "●" : "○"}
            </span>
            <span fg="#a5b4fc"> :{ev.port}</span>
            <span fg="#6e7681"> {ev.process_name}</span>
            <span fg="#4a4a6a"> {timeAgo(ev.timestamp)}</span>
          </text>
        ))
      )}
    </box>
  );
}

function Stat({ label, value, color }: { label: string; value: string; color: string }) {
  return (
    <box
      borderStyle="rounded"
      borderColor="#2a2a45"
      paddingLeft={1}
      paddingRight={1}
      flexDirection="column"
      minWidth={14}
    >
      <text>
        <b fg={color}>{value}</b>
      </text>
      <text fg="#6e7681">{label}</text>
    </box>
  );
}

function ServicesView({ ports, height }: { ports: PortInfo[]; height: number }) {
  const groups = useMemo(() => {
    const map = new Map<string, { name: string; color: string; ports: PortInfo[] }>();
    for (const p of ports) {
      const dev = DEV_PORTS[p.port];
      const key = p.project_name ?? dev?.label ?? p.process_name;
      const g = map.get(key);
      if (g) g.ports.push(p);
      else {
        map.set(key, {
          name: key,
          color: dev?.color ?? "#7c6fff",
          ports: [p],
        });
      }
    }
    return [...map.values()].sort((a, b) => b.ports.length - a.ports.length);
  }, [ports]);

  if (groups.length === 0) {
    return <text fg="#6e7681">No services running</text>;
  }

  const max = Math.max(3, height - 2);
  return (
    <box flexDirection="column" gap={1}>
      <text>
        <b fg="#e2e8f0">Services</b>
        <span fg="#4a4a6a">
          {" "}
          · {groups.length} group{groups.length === 1 ? "" : "s"}
        </span>
      </text>
      {groups.slice(0, max).map((g) => (
        <box key={g.name} flexDirection="column">
          <text>
            <b fg={g.color}>{g.name}</b>
            <span fg="#4a4a6a">
              {" "}
              · {g.ports.length} port{g.ports.length === 1 ? "" : "s"}
            </span>
          </text>
          <text fg="#6e7681">
            {"  "}
            {g.ports.map((p) => `:${p.port}`).join("  ")}
          </text>
        </box>
      ))}
    </box>
  );
}

function LogsView({ events, height }: { events: PortEvent[]; height: number }) {
  if (events.length === 0) {
    return (
      <box flexDirection="column" gap={1}>
        <text fg="#6e7681">No events yet</text>
        <text fg="#3a3a55">Port start/stop events appear as they are detected</text>
      </box>
    );
  }

  const max = Math.max(3, height - 2);
  return (
    <box flexDirection="column" gap={0}>
      <text>
        <b fg="#e2e8f0">Event Logs</b>
        <span fg="#4a4a6a"> · {events.length} recorded</span>
      </text>
      {events.slice(0, max).map((ev, i) => (
        <text key={`${ev.timestamp}-${i}`}>
          <span fg={ev.event_type === "started" ? "#22c55e" : "#ef4444"}>
            {ev.event_type.padEnd(8)}
          </span>
          <span fg="#a5b4fc">:{String(ev.port).padEnd(6)}</span>
          <span fg="#c8c8d8">{truncate(ev.process_name, 24).padEnd(24)}</span>
          <span fg="#4a4a6a">
            {ev.framework ? `${ev.framework} · ` : ""}
            {new Date(ev.timestamp).toLocaleTimeString()}
          </span>
        </text>
      ))}
    </box>
  );
}
