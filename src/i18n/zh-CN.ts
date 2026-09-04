/// 中文文案唯一来源。引入多语言时在此结构上抽 vue-i18n。
export const zhCN = {
  app: {
    name: "MF 翻译",
    tagline: "自带大模型 Key 的桌面翻译工作台",
  },
  nav: {
    workbench: "工作台",
    history: "历史",
    settings: "设置",
  },
  engine: {
    ready: "引擎就绪",
    offline: "引擎离线",
  },
  theme: {
    light: "浅色",
    dark: "深色",
    system: "跟随系统",
  },
  status: {
    version: "版本",
    platform: "Windows 优先 · Tauri 2",
  },
  workbench: {
    sourcePlaceholder: "输入要翻译的文本…（Ctrl+Enter 翻译）",
    outputPlaceholder: "译文将出现在这里",
    autoDetect: "自动检测",
    swap: "交换语言",
    translate: "翻译",
    translating: "翻译中…",
    stop: "停止",
    copy: "复制译文",
    copied: "已复制",
    clear: "清空",
    notConfigured: "尚未配置供应商，请先在「设置」中完成配置",
    streamError: "翻译失败",
  },
  languages: {
    auto: "自动检测",
  },
  settings: {
    title: "供应商配置",
    hint: "选择预设模板可自动填充地址与推荐模型；API Key 只保存在本机系统钥匙串，不会进入配置文件。",
    preset: "预设模板",
    custom: "自定义",
    name: "名称",
    baseUrl: "API 地址",
    model: "模型",
    protocol: "对话协议",
    apiKey: "API Key",
    apiKeyPlaceholder: "sk-…",
    apiKeySavedHint: "已保存在系统钥匙串；留空表示不修改，输入新值即覆盖，输入空格后保存即清除",
    test: "测试连接",
    testing: "测试中…",
    save: "保存配置",
    saving: "保存中…",
    saved: "已保存",
  },
  history: {
    title: "翻译历史",
    searchPlaceholder: "搜索历史（Enter 搜索）…",
    empty: "还没有翻译记录",
    clear: "清空全部",
    confirmClear: "再次点击确认清空",
    copy: "复制",
    copied: "已复制",
    delete: "删除",
  },
} as const;

export type Messages = typeof zhCN;
