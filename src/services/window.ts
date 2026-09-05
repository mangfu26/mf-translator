import { invoke } from "./ipc";

export interface WindowStateDto {
  label: string;
  visible: boolean;
  alwaysOnTop: boolean;
}

/// 查询指定窗口可见性 + 置顶状态。
export const getWindowState = (label: string): Promise<WindowStateDto> =>
  invoke("get_window_state", { label });

/// 设置窗口是否置顶。
export const setAlwaysOnTop = (label: string, value: boolean): Promise<void> =>
  invoke("set_always_on_top", { label, value });

/// 隐藏快捷小窗（Esc 调用）。
export const hideQuickWindow = (): Promise<void> => invoke("hide_quick_window");
