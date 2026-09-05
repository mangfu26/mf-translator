/// 翻译模式（与 Rust 侧 `TranslationMode` 枚举的 camelCase 值一一对应）。
export const TRANSLATION_MODES = [
  { id: "general", label: "通用" },
  { id: "academic", label: "学术" },
  { id: "colloquial", label: "口语" },
  { id: "polish", label: "润色" },
] as const;

export type TranslationModeId = (typeof TRANSLATION_MODES)[number]["id"];
