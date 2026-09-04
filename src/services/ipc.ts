import { invoke as tauriInvoke } from "@tauri-apps/api/core";

/// Rust 侧 AppError 序列化后的统一形状
export interface AppErrorPayload {
  code: string;
  message: string;
}

export class IpcError extends Error {
  readonly code: string;

  constructor(payload: AppErrorPayload) {
    super(payload.message);
    this.name = "IpcError";
    this.code = payload.code;
  }
}

function normalize(raw: unknown): IpcError {
  if (typeof raw === "object" && raw !== null && "code" in raw && "message" in raw) {
    return new IpcError(raw as AppErrorPayload);
  }
  return new IpcError({ code: "UNKNOWN", message: String(raw) });
}

/// 所有前端 → Rust 调用的唯一入口：统一错误归一化，业务代码禁止绕过。
export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await tauriInvoke<T>(cmd, args);
  } catch (err) {
    throw normalize(err);
  }
}

/// 把任意抛出的错误转成用户可读文案。
export function errorMessage(err: unknown): string {
  if (err instanceof IpcError) return err.message;
  if (err instanceof Error) return err.message;
  return String(err);
}
