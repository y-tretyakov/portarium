export interface PortInfo {
  port: number;
  pid: number;
  process_name: string;
  project_name: string | null;
  project_path: string | null;
  protocol: string;
  start_cmd: string | null;
}

export interface PortEvent {
  port: number;
  pid: number;
  process_name: string;
  framework: string | null;
  event_type: "started" | "stopped";
  timestamp: number;
}

export type NavPage = "ports" | "dashboard" | "services" | "logs";
export type PortFilter = "all" | "dev" | "other";

export interface FrameworkMeta {
  label: string;
  color: string;
  icon: string;
}
