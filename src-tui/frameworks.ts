import type { FrameworkMeta, PortInfo } from "./types";

export const DEV_PORTS: Record<number, FrameworkMeta> = {
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
  22: { label: "SSH", color: "#6e7681", icon: "🔒" },
  443: { label: "HTTPS", color: "#22c55e", icon: "🔐" },
  80: { label: "HTTP", color: "#f0a500", icon: "🌐" },
  4173: { label: "Vite", color: "#646cff", icon: "⚡" },
  2000: { label: "Node", color: "#68a063", icon: "⬢" },
  8443: { label: "HTTPS", color: "#22c55e", icon: "🔐" },
};

export function isDevPort(port: number): boolean {
  return port in DEV_PORTS;
}

export function getServiceName(port: PortInfo): string {
  const dev = DEV_PORTS[port.port];
  if (port.project_name) return port.project_name;
  if (dev) return `${dev.label} Server`;
  return port.process_name;
}

export function getFramework(port: number): string | null {
  return DEV_PORTS[port]?.label ?? null;
}

export function timeAgo(ts: number): string {
  const diff = Math.floor((Date.now() - ts) / 1000);
  if (diff < 5) return "just now";
  if (diff < 60) return `${diff}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
}
