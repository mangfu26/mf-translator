import { invoke } from "./ipc";
import type { ProtocolId } from "./config";

export interface ProtocolEntry {
  id: ProtocolId;
  label: string;
}

export interface HealthReport {
  appVersion: string;
  engineReady: boolean;
  protocols: ProtocolEntry[];
}

export function fetchHealth(): Promise<HealthReport> {
  return invoke<HealthReport>("health_check");
}
