/// 支持的目标语言（值会直接进入翻译 Prompt，保持中文表述）。
export const LANGUAGES = [
  "简体中文",
  "英语",
  "日语",
  "韩语",
  "法语",
  "德语",
  "西班牙语",
  "俄语",
] as const;

export type Language = (typeof LANGUAGES)[number];
