import { invoke } from "./ipc";

export interface HistoryItemDto {
  id: number;
  createdAt: number;
  sourceText: string;
  translatedText: string;
  sourceLanguage: string | null;
  targetLanguage: string;
  provider: string;
  model: string;
  promptTokens: number | null;
  completionTokens: number | null;
}

export const listHistory = (query?: string): Promise<HistoryItemDto[]> =>
  invoke("list_history", { query: query ?? null, limit: 200 });

export const deleteHistoryItem = (id: number): Promise<void> =>
  invoke("delete_history_item", { id });

export const clearHistory = (): Promise<void> => invoke("clear_history");
