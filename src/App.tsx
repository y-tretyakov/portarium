import { useEffect, useState, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";
import PortMap from "./PortMap";

interface PortInfo {
  port: number;
  pid: number;
  process_name: string;
  project_name: string | null;
  project_path: string | null;
  protocol: string;
  start_cmd: string | null;
}

interface PortEvent {
  port: number;
  pid: number;
  process_name: string;
  framework: string | null;
  event_type: string;
  timestamp: number;
}

interface TrafficSample {
  connections: number;
  timestamp: number;
}

const DEV_PORTS: Record<number, { label: string; color: string; icon: string }> = {
  3000: { label: "React", color: "#61dafb", icon: "⚛" },
  3001: { label: "React", color: "#61dafb", icon: "⚛" },
  4000: { label: "Node", color: "#68a063", icon: "⬢" },
  4200: { label: "Angular", color: "#dd0031", icon: "△" },
  5173: { label: "Vite", color: "#646cff", icon: "⚡" },
  5174: { label: "Vite", color: "#646cff", icon: "⚡" },
  8000: { label: "Django", color: "#2bbc8a", icon: "🐍" },
  8080: { label: "HTTP", color: "#f0a500", icon: "🌐" },
  8888: { label: "Jupyter", color: "#f37626", icon: "📓" },
  5432: { label: "Postgres", color: "#336791", icon: "🐘" },
  3306: { label: "MySQL", color: "#4479a1", icon: "🐬" },
  6379: { label: "Redis", color: "#dc382d", icon: "◆" },
  27017: { label: "Mongo", color: "#4db33d", icon: "🍃" },
  9000: { label: "PHP", color: "#8892bf", icon: "🐘" },
  1420: { label: "Tauri", color: "#ffc131", icon: "🦀" },
  22:   { label: "SSH", color: "#6e7681", icon: "🔒" },
  443:  { label: "HTTPS", color: "#22c55e", icon: "🔐" },
  80:   { label: "HTTP", color: "#f0a500", icon: "🌐" },
};

type NavPage = "dashboard" | "ports" | "traffic" | "map" | "services" | "logs" | "settings";

function getServiceName(port: PortInfo): string {
  const dev = DEV_PORTS[port.port];
  if (port.project_name) return port.project_name;
  if (dev) return `${dev.label} Server`;
  return port.process_name;
}

function getStatus(port: PortInfo): { label: string; cls: string } {
  const dev = DEV_PORTS[port.port];
  if (dev && [5432, 3306, 6379, 27017].includes(port.port)) {
    return { label: "ACTIVE", cls: "status-active" };
  }
  if (dev) return { label: "ACTIVE", cls: "status-active" };
  return { label: "LISTENING", cls: "status-listening" };
}

function timeAgo(ts: number): string {
  const diff = Math.floor((Date.now() - ts) / 1000);
  if (diff < 5) return "just now";
  if (diff < 60) return `${diff}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
}

/* ── Sparkline mini-chart ── */
function Sparkline({ data, color, width = 64, height = 20 }: {
  data: number[]; color: string; width?: number; height?: number;
}) {
  if (data.length < 2) {
    return <div className="sparkline-empty" style={{ width, height }} />;
  }
  const max = Math.max(...data, 1);
  const pts = data.map((v, i) => {
    const x = (i / (data.length - 1)) * width;
    const y = height - (v / max) * (height - 2);
    return `${x},${y}`;
  }).join(" ");

  return (
    <svg width={width} height={height} className="sparkline">
      <defs>
        <linearGradient id={`sg-${color.replace("#","")}`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={color} stopOpacity="0.3" />
          <stop offset="100%" stopColor={color} stopOpacity="0.02" />
        </linearGradient>
      </defs>
      <polygon
        points={`0,${height} ${pts} ${width},${height}`}
        fill={`url(#sg-${color.replace("#","")})`}
      />
      <polyline
        points={pts}
        fill="none"
        stroke={color}
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export default function App() {
  const [ports, setPorts] = useState<PortInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [killing, setKilling] = useState<Set<number>>(new Set());
  const [restarting, setRestarting] = useState<Set<number>>(new Set());
  const [killedPorts, setKilledPorts] = useState<Map<number, PortInfo>>(new Map());
  const [search, setSearch] = useState("");
  const [toast, setToast] = useState<string | null>(null);
  const [page, setPage] = useState<NavPage>("ports");
  const [portFilter, setPortFilter] = useState<"all" | "dev" | "other">("all");
  const [events, setEvents] = useState<PortEvent[]>([]);
  const [traffic, setTraffic] = useState<Record<number, TrafficSample[]>>({});
  const [firstSeen, setFirstSeen] = useState<Record<number, number>>({});

  const fetchPorts = useCallback(async () => {
    try {
      const result = await invoke<PortInfo[]>("get_ports");
      setPorts(result);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchEvents = useCallback(async () => {
    try {
      const ev = await invoke<PortEvent[]>("get_port_events");
      setEvents(ev);
    } catch {}
  }, []);

  const fetchTraffic = useCallback(async () => {
    try {
      const t = await invoke<Record<number, TrafficSample[]>>("get_port_traffic");
      setTraffic(t);
    } catch {}
  }, []);

  useEffect(() => {
    fetchPorts();
    fetchEvents();
    fetchTraffic();

    // Poll traffic samples every 4s (samples accumulate even without port changes)
    const trafficTimer = setInterval(fetchTraffic, 4000);

    let unlisten1: (() => void) | null = null;
    let unlisten2: (() => void) | null = null;

    import("@tauri-apps/api/event").then(({ listen }) => {
      listen<PortInfo[]>("ports-updated", (event) => {
        setPorts(event.payload);
        setLoading(false);
        setKilledPorts((prev) => {
          const livePorts = new Set(event.payload.map((p: PortInfo) => p.port));
          const next = new Map(prev);
          for (const port of next.keys()) {
            if (livePorts.has(port)) next.delete(port);
          }
          return next;
        });
        // Refresh traffic data when ports update
        fetchTraffic();
      }).then((fn) => { unlisten1 = fn; });

      listen<PortEvent[]>("port-events", (event) => {
        setEvents((prev) => [...event.payload, ...prev].slice(0, 200));
        // Track first seen
        for (const ev of event.payload) {
          if (ev.event_type === "started") {
            setFirstSeen((prev) => ({ ...prev, [ev.port]: ev.timestamp }));
          }
        }
      }).then((fn) => { unlisten2 = fn; });
    });

    return () => {
      clearInterval(trafficTimer);
      if (unlisten1) unlisten1();
      if (unlisten2) unlisten2();
    };
  }, [fetchPorts, fetchEvents, fetchTraffic]);

  const handleKill = async (pid: number) => {
    const port = ports.find((p) => p.pid === pid);
    if (!port) return;
    setKilling((prev) => new Set(prev).add(pid));
    try {
      await invoke("kill_process", { pid });
      showToast(`Killed ${port.process_name} on :${port.port}`);
      setPorts((prev) => prev.filter((p) => p.pid !== pid));
      if (port.start_cmd && port.project_path) {
        setKilledPorts((prev) => new Map(prev).set(port.port, port));
      }
    } catch {
      showToast(`Failed to kill PID ${pid}`);
    } finally {
      setKilling((prev) => { const n = new Set(prev); n.delete(pid); return n; });
    }
  };

  const handleRestart = async (pid: number, cmd: string, cwd: string) => {
    const port = ports.find((p) => p.pid === pid)
      ?? [...killedPorts.values()].find((p) => p.pid === pid);
    if (!port) return;
    setRestarting((prev) => new Set(prev).add(pid));
    try {
      await invoke("restart_process", { pid, cmd, cwd });
      showToast(`Restarting ${port.project_name ?? port.process_name}…`);
      setKilledPorts((prev) => { const n = new Map(prev); n.delete(port.port); return n; });
    } catch (e) {
      showToast(`Failed to restart: ${e}`);
    } finally {
      setRestarting((prev) => { const n = new Set(prev); n.delete(pid); return n; });
    }
  };

  const showToast = (msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 3000);
  };

  const filteredPorts = useMemo(() => {
    let list = ports;
    // Filter by tab
    if (portFilter === "dev") list = list.filter((p) => DEV_PORTS[p.port]);
    else if (portFilter === "other") list = list.filter((p) => !DEV_PORTS[p.port]);
    // Then by search
    const q = search.toLowerCase().trim();
    if (!q) return list;
    return list.filter((p) => {
      const dev = DEV_PORTS[p.port];
      return (
        String(p.port).includes(q) ||
        p.process_name.toLowerCase().includes(q) ||
        (p.project_name && p.project_name.toLowerCase().includes(q)) ||
        (dev && dev.label.toLowerCase().includes(q))
      );
    });
  }, [ports, search, portFilter]);

  const fwSet = new Set(ports.map((p) => DEV_PORTS[p.port]?.label).filter(Boolean));
  const activeConns = Object.values(traffic).reduce((sum, samples) => {
    const last = samples[samples.length - 1];
    return sum + (last?.connections ?? 0);
  }, 0);

  return (
    <div className="app">
      {/* ── Title bar ── */}
      <div className="titlebar" data-tauri-drag-region>
        <div className="titlebar-left">
          <span className="titlebar-title">Portarium</span>
        </div>
        <div className="titlebar-right">
          <button className="tb-btn" title="Minimize" onClick={() => getCurrentWindow().minimize()}>—</button>
          <button className="tb-btn" title="Maximize" onClick={() => getCurrentWindow().toggleMaximize()}>□</button>
          <button className="tb-btn tb-close" title="Close" onClick={() => getCurrentWindow().close()}>✕</button>
        </div>
      </div>

      <div className="main-layout">
        {/* ── Sidebar ── */}
        <nav className="sidebar">
          <SidebarBtn icon={<DashboardIcon />} label="Dashboard" active={page === "dashboard"} onClick={() => setPage("dashboard")} />
          <SidebarBtn icon={<PortsIcon />} label="Ports" sub="(active)" active={page === "ports"} onClick={() => setPage("ports")} />
          <SidebarBtn icon={<TrafficIcon />} label="Traffic" active={page === "traffic"} onClick={() => setPage("traffic")} />
          <SidebarBtn icon={<ServicesIcon />} label="Services" active={page === "services"} onClick={() => setPage("services")} />
          <SidebarBtn icon={<MapIcon />} label="Port Map" active={page === "map"} onClick={() => setPage("map")} />
          <SidebarBtn icon={<SettingsIcon />} label="Settings" active={page === "settings"} onClick={() => setPage("settings")} />
          <div className="sidebar-spacer" />
          <SidebarBtn icon={<LogsIcon />} label="Logs" active={page === "logs"} onClick={() => setPage("logs")} />
        </nav>

        {/* ── Content ── */}
        <div className="content">
          {/* ════════ DASHBOARD ════════ */}
          {page === "dashboard" && (
            <DashboardPage
              ports={ports}
              events={events}
              traffic={traffic}
              fwSet={fwSet}
              activeConns={activeConns}
              onNavigate={setPage}
            />
          )}

          {/* ════════ PORTS ════════ */}
          {page === "ports" && (
            <>
              <div className="search-bar">
                <svg className="search-icon" width="14" height="14" viewBox="0 0 14 14" fill="none">
                  <circle cx="6" cy="6" r="4.5" stroke="currentColor" strokeWidth="1.5"/>
                  <path d="M9.5 9.5L13 13" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
                </svg>
                <input
                  type="text"
                  placeholder="Search ports or services..."
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  className="search-input"
                />
                <button className="map-toggle-btn" onClick={() => setPage("map")} title="Port Map">
                  <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                    <circle cx="3" cy="7" r="2" stroke="currentColor" strokeWidth="1.3"/>
                    <circle cx="11" cy="3" r="2" stroke="currentColor" strokeWidth="1.3"/>
                    <circle cx="11" cy="11" r="2" stroke="currentColor" strokeWidth="1.3"/>
                    <line x1="4.8" y1="6" x2="9.2" y2="3.8" stroke="currentColor" strokeWidth="1.3"/>
                    <line x1="4.8" y1="8" x2="9.2" y2="10.2" stroke="currentColor" strokeWidth="1.3"/>
                  </svg>
                </button>
              </div>

              {/* Filter tabs */}
              <div className="port-tabs">
                <div className="tab-group">
                  <button className={`tab-btn ${portFilter === "all" ? "active" : ""}`} onClick={() => setPortFilter("all")}>
                    All <span className="tab-count">{ports.length}</span>
                  </button>
                  <button className={`tab-btn ${portFilter === "dev" ? "active" : ""}`} onClick={() => setPortFilter("dev")}>
                    Dev <span className="tab-count">{ports.filter(p => DEV_PORTS[p.port]).length}</span>
                  </button>
                  <button className={`tab-btn ${portFilter === "other" ? "active" : ""}`} onClick={() => setPortFilter("other")}>
                    Other <span className="tab-count">{ports.filter(p => !DEV_PORTS[p.port]).length}</span>
                  </button>
                </div>
                <button
                  className="kill-all-btn"
                  onClick={() => filteredPorts.forEach((p) => handleKill(p.pid))}
                  disabled={filteredPorts.length === 0}
                >
                  Kill All ({filteredPorts.length})
                </button>
              </div>

              <div className="summary-line">
                {filteredPorts.length} active connection{filteredPorts.length !== 1 ? "s" : ""}, {fwSet.size} framework{fwSet.size !== 1 ? "s" : ""} detected
              </div>

              {loading ? (
                <div className="loading-state">
                  <div className="loader" />
                  <span>Scanning ports…</span>
                </div>
              ) : filteredPorts.length === 0 && killedPorts.size === 0 ? (
                <div className="empty-state">
                  <div className="empty-ring" />
                  <span>No ports in use</span>
                  <span className="empty-sub">Start a server and it'll appear here</span>
                </div>
              ) : (
                <div className="table-wrap">
                  <table className="port-table">
                    <thead>
                      <tr>
                        <th>PORT</th>
                        <th>SERVICE</th>
                        <th>PROCESS</th>
                        <th>STATUS</th>
                        <th>FRAMEWORK</th>
                        <th>TRAFFIC</th>
                        <th>LAST ACTIVE</th>
                        <th className="th-right">ACTIONS</th>
                      </tr>
                    </thead>
                    <tbody>
                      {filteredPorts.map((p) => {
                        const dev = DEV_PORTS[p.port];
                        const status = getStatus(p);
                        const isKilling = killing.has(p.pid);
                        const isRestarting = restarting.has(p.pid);
                        const canRestart = !!p.start_cmd && !!p.project_path;
                        const samples = traffic[p.port] || [];
                        const sparkData = samples.map((s) => s.connections);
                        const lastConn = samples[samples.length - 1]?.connections ?? 0;
                        const seen = firstSeen[p.port];

                        return (
                          <tr key={`${p.pid}-${p.port}`} className={isKilling || isRestarting ? "row-disabled" : ""}>
                            <td className="td-port">
                              <span className="port-dot" style={{ background: dev ? "#22c55e" : "#6e7681" }} />
                              <span className="port-num">{p.port}</span>
                            </td>
                            <td className="td-service">{getServiceName(p)}</td>
                            <td className="td-process">
                              <code>{p.start_cmd ? p.start_cmd.split(" ").slice(0, 2).join(" ") : p.process_name}</code>
                            </td>
                            <td>
                              <span className={`status-badge ${status.cls}`}>{status.label}</span>
                            </td>
                            <td>
                              {dev ? (
                                <span className="fw-badge" style={{ "--fw-color": dev.color } as React.CSSProperties}>
                                  <span className="fw-icon">{dev.icon}</span>
                                  {dev.label}
                                </span>
                              ) : (
                                <span className="fw-badge fw-generic">{p.protocol}</span>
                              )}
                            </td>
                            <td className="td-traffic">
                              <div className="traffic-cell">
                                <span className="traffic-rate">{lastConn > 0 ? `${lastConn} conn` : "0"}</span>
                                <Sparkline data={sparkData} color={dev?.color ?? "#7c6fff"} />
                              </div>
                            </td>
                            <td className="td-time">
                              {seen ? timeAgo(seen) : "—"}
                            </td>
                            <td className="td-actions">
                              {canRestart && (
                                <button className="action-btn restart always-visible" onClick={() => handleRestart(p.pid, p.start_cmd!, p.project_path!)} disabled={isKilling || isRestarting} title={`Restart: ${p.start_cmd}`}>
                                  {isRestarting ? <span className="mini-spinner" /> : "↻"}
                                </button>
                              )}
                              <button className="action-btn kill always-visible" onClick={() => handleKill(p.pid)} disabled={isKilling || isRestarting} title={`Kill PID ${p.pid}`}>
                                {isKilling ? <span className="mini-spinner" /> : "✕"}
                              </button>
                            </td>
                          </tr>
                        );
                      })}
                      {[...killedPorts.values()].map((p) => {
                        const dev = DEV_PORTS[p.port];
                        return (
                          <tr key={`dead-${p.port}`} className="row-dead">
                            <td className="td-port"><span className="port-dot" style={{ background: "#ef4444" }} /><span className="port-num">{p.port}</span></td>
                            <td className="td-service">{getServiceName(p)}</td>
                            <td className="td-process"><code>{p.process_name}</code></td>
                            <td><span className="status-badge status-stopped">STOPPED</span></td>
                            <td>{dev ? <span className="fw-badge" style={{ "--fw-color": dev.color, opacity: 0.5 } as React.CSSProperties}><span className="fw-icon">{dev.icon}</span>{dev.label}</span> : <span className="fw-badge fw-generic">{p.protocol}</span>}</td>
                            <td className="td-traffic">—</td>
                            <td className="td-time">—</td>
                            <td className="td-actions">
                              {p.start_cmd && p.project_path && (
                                <button className="action-btn restart-visible" onClick={() => handleRestart(p.pid, p.start_cmd!, p.project_path!)} disabled={restarting.has(p.pid)}>
                                  {restarting.has(p.pid) ? <span className="mini-spinner" /> : "↻"}
                                </button>
                              )}
                            </td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
              )}
            </>
          )}

          {/* ════════ TRAFFIC ════════ */}
          {page === "traffic" && <TrafficPage ports={ports} traffic={traffic} />}

          {/* ════════ PORT MAP ════════ */}
          {page === "map" && <PortMap onClose={() => setPage("ports")} />}

          {/* ════════ SERVICES ════════ */}
          {page === "services" && <ServicesPage ports={ports} traffic={traffic} />}

          {/* ════════ LOGS ════════ */}
          {page === "logs" && <LogsPage events={events} onRefresh={fetchEvents} />}

          {/* ════════ SETTINGS ════════ */}
          {page === "settings" && (
            <div className="settings-page">
              <h2>Settings</h2>
              <p className="settings-sub">Coming in v0.2</p>
            </div>
          )}
        </div>
      </div>

      {toast && <div className="toast">{toast}</div>}
    </div>
  );
}

/* ══════════════════════════════════════════════
   DASHBOARD PAGE
   ══════════════════════════════════════════════ */
function DashboardPage({ ports, events, traffic, fwSet, activeConns, onNavigate }: {
  ports: PortInfo[];
  events: PortEvent[];
  traffic: Record<number, TrafficSample[]>;
  fwSet: Set<string>;
  activeConns: number;
  onNavigate: (page: NavPage) => void;
}) {
  const recentEvents = events.slice(0, 5);

  return (
    <div className="dashboard">
      <h2 className="dash-title">Dashboard</h2>
      <p className="dash-sub">Overview of your port activity</p>

      {/* Stat cards */}
      <div className="stat-grid">
        <StatCard label="Active Ports" value={ports.length} icon="⚡" color="#22c55e" onClick={() => onNavigate("ports")} />
        <StatCard label="Frameworks" value={fwSet.size} icon="🧩" color="#7c6fff" />
        <StatCard label="Connections" value={activeConns} icon="🔗" color="#3b82f6" onClick={() => onNavigate("map")} />
        <StatCard label="Events Today" value={events.filter(e => Date.now() - e.timestamp < 86400000).length} icon="📋" color="#eab308" onClick={() => onNavigate("logs")} />
      </div>

      {/* Active services */}
      <div className="dash-section">
        <div className="dash-section-header">
          <h3>Active Services</h3>
          <button className="dash-link" onClick={() => onNavigate("ports")}>View all →</button>
        </div>
        <div className="dash-services">
          {ports.slice(0, 6).map((p) => {
            const dev = DEV_PORTS[p.port];
            const samples = traffic[p.port] || [];
            const sparkData = samples.map(s => s.connections);
            return (
              <div key={p.port} className="dash-svc-card">
                <div className="dash-svc-top">
                  <span className="dash-svc-port" style={{ color: dev?.color ?? "#7c6fff" }}>:{p.port}</span>
                  <span className="status-badge status-active" style={{ fontSize: 8, padding: "2px 5px" }}>ACTIVE</span>
                </div>
                <div className="dash-svc-name">{getServiceName(p)}</div>
                <Sparkline data={sparkData} color={dev?.color ?? "#7c6fff"} width={100} height={24} />
              </div>
            );
          })}
        </div>
      </div>

      {/* Recent events */}
      <div className="dash-section">
        <div className="dash-section-header">
          <h3>Recent Events</h3>
          <button className="dash-link" onClick={() => onNavigate("logs")}>View all →</button>
        </div>
        {recentEvents.length === 0 ? (
          <p className="dash-empty">No events yet — start a server to see activity</p>
        ) : (
          <div className="dash-events">
            {recentEvents.map((ev, i) => (
              <div key={`${ev.timestamp}-${i}`} className="dash-event-row">
                <span className={`ev-dot ${ev.event_type === "started" ? "ev-green" : "ev-red"}`} />
                <span className="ev-port">:{ev.port}</span>
                <span className="ev-name">{ev.process_name}</span>
                <span className={`ev-type ${ev.event_type}`}>{ev.event_type}</span>
                <span className="ev-time">{timeAgo(ev.timestamp)}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function StatCard({ label, value, icon, color, onClick }: {
  label: string; value: number; icon: string; color: string; onClick?: () => void;
}) {
  return (
    <div className="stat-card" onClick={onClick} style={{ cursor: onClick ? "pointer" : "default" }}>
      <div className="stat-icon" style={{ background: `${color}18`, color }}>{icon}</div>
      <div className="stat-info">
        <div className="stat-value">{value}</div>
        <div className="stat-label">{label}</div>
      </div>
    </div>
  );
}

/* ══════════════════════════════════════════════
   LOGS PAGE
   ══════════════════════════════════════════════ */
function LogsPage({ events, onRefresh }: { events: PortEvent[]; onRefresh: () => void }) {
  return (
    <div className="logs-page">
      <div className="logs-header">
        <div>
          <h2>Event Logs</h2>
          <p className="logs-sub">{events.length} event{events.length !== 1 ? "s" : ""} recorded</p>
        </div>
        <button className="logs-refresh" onClick={onRefresh}>↻ Refresh</button>
      </div>

      {events.length === 0 ? (
        <div className="empty-state">
          <div className="empty-ring" />
          <span>No events yet</span>
          <span className="empty-sub">Port start and stop events will appear here</span>
        </div>
      ) : (
        <div className="logs-list">
          {events.map((ev, i) => (
            <div key={`${ev.timestamp}-${i}`} className="log-row">
              <div className="log-left">
                <span className={`log-dot ${ev.event_type === "started" ? "ev-green" : "ev-red"}`} />
                <div className="log-info">
                  <div className="log-main">
                    <span className="log-port">:{ev.port}</span>
                    <span className="log-process">{ev.process_name}</span>
                    {ev.framework && (
                      <span className="log-fw" style={{ color: DEV_PORTS[ev.port]?.color ?? "#7c6fff" }}>
                        {ev.framework}
                      </span>
                    )}
                  </div>
                  <div className="log-detail">
                    PID {ev.pid} · {ev.event_type === "started" ? "Service started" : "Service stopped"}
                  </div>
                </div>
              </div>
              <div className="log-right">
                <span className={`log-type-badge ${ev.event_type}`}>{ev.event_type}</span>
                <span className="log-time">{new Date(ev.timestamp).toLocaleTimeString()}</span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

/* ══════════════════════════════════════════════
   TRAFFIC PAGE
   ══════════════════════════════════════════════ */
function TrafficPage({ ports, traffic }: { ports: PortInfo[]; traffic: Record<number, TrafficSample[]> }) {
  const totalConns = Object.values(traffic).reduce((sum, samples) => {
    const last = samples[samples.length - 1];
    return sum + (last?.connections ?? 0);
  }, 0);

  const peakConns = Object.values(traffic).reduce((sum, samples) => {
    return sum + Math.max(0, ...samples.map(s => s.connections));
  }, 0);

  return (
    <div className="traffic-page">
      <div className="traffic-header">
        <div>
          <h2>Traffic Monitor</h2>
          <p className="traffic-page-sub">Real-time connection activity across all ports</p>
        </div>
      </div>

      {/* Overview stats */}
      <div className="traffic-stats">
        <div className="traf-stat">
          <div className="traf-stat-value">{ports.length}</div>
          <div className="traf-stat-label">Active Ports</div>
        </div>
        <div className="traf-stat">
          <div className="traf-stat-value">{totalConns}</div>
          <div className="traf-stat-label">Current Connections</div>
        </div>
        <div className="traf-stat">
          <div className="traf-stat-value">{peakConns}</div>
          <div className="traf-stat-label">Peak (Session)</div>
        </div>
      </div>

      {/* Per-port traffic cards */}
      {ports.length === 0 ? (
        <div className="empty-state">
          <div className="empty-ring" />
          <span>No active ports</span>
          <span className="empty-sub">Start a server to see traffic</span>
        </div>
      ) : (
        <div className="traffic-list">
          {ports.map((p) => {
            const dev = DEV_PORTS[p.port];
            const samples = traffic[p.port] || [];
            const sparkData = samples.map(s => s.connections);
            const current = sparkData[sparkData.length - 1] ?? 0;
            const peak = Math.max(0, ...sparkData);
            const color = dev?.color ?? "#7c6fff";

            return (
              <div key={p.port} className="traf-card">
                <div className="traf-card-left">
                  <div className="traf-card-header">
                    <span className="traf-port-dot" style={{ background: color, boxShadow: `0 0 8px ${color}` }} />
                    <span className="traf-port-num" style={{ color }}>:{p.port}</span>
                    <span className="traf-svc-name">{getServiceName(p)}</span>
                    {dev && (
                      <span className="fw-badge" style={{ "--fw-color": dev.color, fontSize: 9, padding: "2px 6px" } as React.CSSProperties}>
                        <span className="fw-icon">{dev.icon}</span>
                        {dev.label}
                      </span>
                    )}
                  </div>
                  <div className="traf-card-stats">
                    <span className="traf-metric">
                      <span className="traf-metric-val">{current}</span>
                      <span className="traf-metric-label">current</span>
                    </span>
                    <span className="traf-metric-sep">·</span>
                    <span className="traf-metric">
                      <span className="traf-metric-val">{peak}</span>
                      <span className="traf-metric-label">peak</span>
                    </span>
                    <span className="traf-metric-sep">·</span>
                    <span className="traf-metric">
                      <span className="traf-metric-val">{samples.length}</span>
                      <span className="traf-metric-label">samples</span>
                    </span>
                  </div>
                </div>
                <div className="traf-card-chart">
                  <Sparkline data={sparkData} color={color} width={240} height={40} />
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

/* ══════════════════════════════════════════════
   SERVICES PAGE
   ══════════════════════════════════════════════ */
interface ServiceGroup {
  name: string;
  color: string;
  icon: string;
  ports: PortInfo[];
}

function ServicesPage({ ports, traffic }: { ports: PortInfo[]; traffic: Record<number, TrafficSample[]> }) {
  // Group ports by project_name or framework
  const groups = useMemo(() => {
    const map = new Map<string, ServiceGroup>();

    for (const p of ports) {
      const dev = DEV_PORTS[p.port];
      const key = p.project_name ?? dev?.label ?? p.process_name;
      const existing = map.get(key);

      if (existing) {
        existing.ports.push(p);
      } else {
        map.set(key, {
          name: key,
          color: dev?.color ?? "#7c6fff",
          icon: dev?.icon ?? "📦",
          ports: [p],
        });
      }
    }

    // Sort: most ports first
    return [...map.values()].sort((a, b) => b.ports.length - a.ports.length);
  }, [ports]);

  return (
    <div className="services-page">
      <div className="services-header">
        <div>
          <h2>Services</h2>
          <p className="services-sub">{groups.length} service{groups.length !== 1 ? "s" : ""} running across {ports.length} port{ports.length !== 1 ? "s" : ""}</p>
        </div>
      </div>

      {groups.length === 0 ? (
        <div className="empty-state">
          <div className="empty-ring" />
          <span>No services running</span>
          <span className="empty-sub">Start a server to see it here</span>
        </div>
      ) : (
        <div className="services-grid">
          {groups.map((group) => {
            const totalConns = group.ports.reduce((sum, p) => {
              const samples = traffic[p.port] || [];
              return sum + (samples[samples.length - 1]?.connections ?? 0);
            }, 0);

            // Merge sparkline data from all ports in this group
            const mergedSpark: number[] = [];
            for (const p of group.ports) {
              const samples = traffic[p.port] || [];
              samples.forEach((s, i) => {
                mergedSpark[i] = (mergedSpark[i] || 0) + s.connections;
              });
            }

            return (
              <div key={group.name} className="svc-card" style={{ "--svc-color": group.color } as React.CSSProperties}>
                <div className="svc-card-header">
                  <div className="svc-card-icon">{group.icon}</div>
                  <div className="svc-card-info">
                    <div className="svc-card-name">{group.name}</div>
                    <div className="svc-card-meta">
                      {group.ports.length} port{group.ports.length !== 1 ? "s" : ""} · {totalConns} conn{totalConns !== 1 ? "s" : ""}
                    </div>
                  </div>
                  <span className="status-badge status-active" style={{ fontSize: 8, padding: "2px 6px" }}>RUNNING</span>
                </div>

                <div className="svc-sparkline-wrap">
                  <Sparkline data={mergedSpark} color={group.color} width={200} height={32} />
                </div>

                <div className="svc-ports-list">
                  {group.ports.map((p) => (
                    <div key={p.port} className="svc-port-item">
                      <span className="svc-port-dot" style={{ background: group.color }} />
                      <span className="svc-port-num">:{p.port}</span>
                      <span className="svc-port-process">{p.process_name}</span>
                      <span className="svc-port-pid">PID {p.pid}</span>
                    </div>
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

/* ── Sidebar Buttons ── */
function SidebarBtn({ icon, label, sub, active, onClick }: {
  icon: React.ReactNode; label: string; sub?: string; active: boolean; onClick: () => void;
}) {
  return (
    <button className={`sidebar-btn ${active ? "active" : ""}`} onClick={onClick}>
      <span className="sb-icon">{icon}</span>
      <span className="sb-label">{label}</span>
      {sub && <span className="sb-sub">{sub}</span>}
    </button>
  );
}

/* ── SVG Icons ── */
function DashboardIcon() {
  return (<svg width="18" height="18" viewBox="0 0 18 18" fill="none"><rect x="1" y="1" width="7" height="7" rx="2" stroke="currentColor" strokeWidth="1.5"/><rect x="10" y="1" width="7" height="7" rx="2" stroke="currentColor" strokeWidth="1.5"/><rect x="1" y="10" width="7" height="7" rx="2" stroke="currentColor" strokeWidth="1.5"/><rect x="10" y="10" width="7" height="7" rx="2" stroke="currentColor" strokeWidth="1.5"/></svg>);
}
function PortsIcon() {
  return (<svg width="18" height="18" viewBox="0 0 18 18" fill="none"><rect x="2" y="3" width="14" height="12" rx="3" stroke="currentColor" strokeWidth="1.5"/><circle cx="6" cy="9" r="1" fill="currentColor"/><circle cx="9" cy="9" r="1" fill="currentColor"/><circle cx="12" cy="9" r="1" fill="currentColor"/></svg>);
}
function TrafficIcon() {
  return (<svg width="18" height="18" viewBox="0 0 18 18" fill="none"><path d="M1 14l3-4 3 2 4-7 3 4 3-2" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/><path d="M1 16h16" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" opacity="0.4"/></svg>);
}
function MapIcon() {
  return (<svg width="18" height="18" viewBox="0 0 18 18" fill="none"><circle cx="4" cy="9" r="2.5" stroke="currentColor" strokeWidth="1.5"/><circle cx="14" cy="4" r="2.5" stroke="currentColor" strokeWidth="1.5"/><circle cx="14" cy="14" r="2.5" stroke="currentColor" strokeWidth="1.5"/><line x1="6.2" y1="8" x2="11.8" y2="5" stroke="currentColor" strokeWidth="1.3"/><line x1="6.2" y1="10" x2="11.8" y2="13" stroke="currentColor" strokeWidth="1.3"/></svg>);
}
function ServicesIcon() {
  return (<svg width="18" height="18" viewBox="0 0 18 18" fill="none"><rect x="2" y="2" width="6" height="6" rx="1.5" stroke="currentColor" strokeWidth="1.5"/><rect x="10" y="2" width="6" height="6" rx="1.5" stroke="currentColor" strokeWidth="1.5"/><rect x="2" y="10" width="6" height="6" rx="1.5" stroke="currentColor" strokeWidth="1.5"/><path d="M13 11v5M10.5 13.5h5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/></svg>);
}
function SettingsIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 20 20" fill="none">
      <path d="M10 13a3 3 0 100-6 3 3 0 000 6z" stroke="currentColor" strokeWidth="1.5"/>
      <path d="M17.4 12.1a1.3 1.3 0 00.26 1.43l.05.05a1.57 1.57 0 11-2.22 2.22l-.05-.05a1.3 1.3 0 00-1.43-.26 1.3 1.3 0 00-.79 1.19v.14a1.57 1.57 0 11-3.14 0v-.07a1.3 1.3 0 00-.85-1.19 1.3 1.3 0 00-1.43.26l-.05.05a1.57 1.57 0 11-2.22-2.22l.05-.05a1.3 1.3 0 00.26-1.43 1.3 1.3 0 00-1.19-.79h-.14a1.57 1.57 0 110-3.14h.07a1.3 1.3 0 001.19-.85 1.3 1.3 0 00-.26-1.43l-.05-.05A1.57 1.57 0 117.7 3.62l.05.05a1.3 1.3 0 001.43.26h.06a1.3 1.3 0 00.79-1.19v-.14a1.57 1.57 0 113.14 0v.07a1.3 1.3 0 00.79 1.19 1.3 1.3 0 001.43-.26l.05-.05a1.57 1.57 0 112.22 2.22l-.05.05a1.3 1.3 0 00-.26 1.43v.06a1.3 1.3 0 001.19.79h.14a1.57 1.57 0 010 3.14h-.07a1.3 1.3 0 00-1.19.79z" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round"/>
    </svg>
  );
}
function LogsIcon() {
  return (<svg width="18" height="18" viewBox="0 0 18 18" fill="none"><rect x="3" y="2" width="12" height="14" rx="2" stroke="currentColor" strokeWidth="1.5"/><path d="M6 6h6M6 9h4M6 12h5" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round"/></svg>);
}
