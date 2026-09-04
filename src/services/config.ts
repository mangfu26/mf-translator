import { invoke } from "./ipc";

export type ProtocolId = "chatCompletions" | "responses";

export interface ProviderPresetDto {
  id: string;
  name: string;
  baseUrl: string;
  defaultModels: string[];
  defaultProtocol: ProtocolId;
}

export interface ProviderConfigView {
  id: string;
  name: string;
  baseUrl: string;
  model: string;
  protocol: ProtocolId;
  hasApiKey: boolean;
}

export interface ProviderConfigInput {
  id?: string | null;
  name: string;
  baseUrl: string;
  model: string;
  protocol: ProtocolId;
}

export interface TestConnectionInput {
  providerId?: string | null;
  baseUrl: string;
  model: string;
  protocol: ProtocolId;
  apiKey?: string | null;
}

export const listPresets = (): Promise<ProviderPresetDto[]> => invoke("list_presets");

export const getProviderConfig = (): Promise<ProviderConfigView | null> =>
  invoke("get_provider_config");

export const saveProviderConfig = (
  config: ProviderConfigInput,
  apiKey?: string | null,
): Promise<ProviderConfigView> =>
  // 参数名必须与 Rust command `save_provider_config(input, api_key)` 一致
  invoke("save_provider_config", { input: config, apiKey: apiKey ?? null });

export const testConnection = (input: TestConnectionInput): Promise<string> =>
  invoke("test_connection", { input });
