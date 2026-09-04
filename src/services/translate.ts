import { Channel } from "@tauri-apps/api/core";
import { invoke } from "./ipc";
import type { Usage } from "./model";

export type TranslationEvent =
  | { type: "delta"; text: string }
  | { type: "finished"; usage: Usage | null }
  | { type: "failed"; error: { code: string; message: string } };

export interface TranslateRequestDto {
  model: string;
  sourceText: string;
  targetLanguage: string;
  sourceLanguage?: string | null;
}

/// 发起流式翻译，返回任务 id（用于取消）。事件经 Channel 推送。
export async function startTranslation(
  request: TranslateRequestDto,
  onEvent: (event: TranslationEvent) => void,
): Promise<string> {
  const channel = new Channel<TranslationEvent>();
  channel.onmessage = onEvent;
  return invoke<string>("translate_text", { request, channel });
}

export function cancelTranslation(taskId: string): Promise<boolean> {
  return invoke<boolean>("cancel_translation", { taskId });
}
