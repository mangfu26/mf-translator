import { getCurrentWindow } from "@tauri-apps/api/window";

/// 当前窗口 label（"main" 主窗口 / "quick" 快捷小窗）。
export async function currentWindowLabel(): Promise<string> {
  try {
    return await getCurrentWindow().label;
  } catch {
    return "main";
  }
}
