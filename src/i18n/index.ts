import { zhCN, type Messages } from "./zh-CN";

export type { Messages };

/// 当前语言固定中文；i18n 框架化时替换此函数实现。
export function t(): Messages {
  return zhCN;
}
