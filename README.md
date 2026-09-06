# MF Translator

自带大模型 Key（BYOK）的桌面翻译工作台。用户填入模型服务商的 API 地址与 API Key，
即可获得流式、高质量的大模型翻译体验。

- 技术底座：**Tauri 2（Rust）+ Vue 3 + TypeScript + Tailwind CSS 4**
- 目标平台：Windows 优先，架构保持跨平台能力（macOS / Linux 后续扩展）
- 界面语言：中文优先，文案集中于 `src/i18n/`

## 当前状态：M2 MVP ✅

| 里程碑  | 内容                                                                                  | 状态      |
| ------- | ------------------------------------------------------------------------------------- | --------- |
| M1 骨架 | 工程搭建、双协议抽象层接口、CI、界面空壳                                              | ✅        |
| M2 MVP  | 流式文本翻译（双协议 + 取消）、供应商配置（预设 + 钥匙串 + 连接测试）、翻译历史、托盘 | ✅        |
| M3 体验 | 应用内检查更新、全局快捷键快捷小窗、多翻译模式、窗口置顶                              | 🚧 进行中 |

> 应用内“检查更新”已落地（设置页「关于与更新」），带签名的静默自动更新在公开发布前接入。

## 快速开始

前置要求：Node.js ≥ 20、Rust stable（MSVC 工具链）、VS Build Tools（C++ 工作负载）、WebView2 Runtime。

```bash
npm install
npm run tauri dev     # 开发模式（热更新）
npm run tauri build   # 产出安装包
```

## 常用命令

| 命令                                        | 说明                                                      |
| ------------------------------------------- | --------------------------------------------------------- |
| `npm run tauri dev`                         | 启动桌面应用开发模式                                      |
| `npm run dev`                               | 仅启动前端（浏览器调试）                                  |
| `npm run build`                             | 类型检查 + 前端构建                                       |
| `npm run lint` / `npm run format`           | ESLint 检查 / Prettier 格式化                             |
| `npm run test` / `npm run test:watch`       | Vitest 单元测试                                           |
| `npm run typecheck`                         | vue-tsc 类型检查                                          |
| `cargo fmt` / `cargo clippy` / `cargo test` | Rust 侧格式化 / 静态检查 / 测试（在 `src-tauri/` 下执行） |

## 目录结构

```
mf-translator/
├─ src/                        # 前端（Vue 3 + TS）
│  ├─ components/              # UI 组件
│  ├─ stores/                  # Pinia 状态
│  ├─ services/                # Tauri IPC 封装（前端唯一出口，统一错误归一化）
│  ├─ i18n/                    # 界面文案（中文优先，预留多语言）
│  ├─ lib/                     # 通用工具（cn 等）
│  ├─ styles/                  # Tailwind v4 设计令牌（亮/暗主题变量）
│  └─ theme.ts                 # 主题偏好与应用逻辑
├─ src-tauri/
│  └─ src/
│     ├─ commands/             # Tauri command（IPC 入口：翻译/配置/历史/自检）
│     ├─ provider/             # 供应商协议抽象层
│     │  ├─ mod.rs             #   Protocol trait + TranslationEvent 统一事件流
│     │  ├─ chat_completions.rs#   OpenAI 兼容协议（POST /chat/completions）
│     │  ├─ responses.rs       #   OpenAI Responses API（POST /responses）
│     │  ├─ sse.rs             #   SSE 行解码（处理半包/CRLF）
│     │  └─ presets.rs         #   内置供应商预设表（OpenAI/DeepSeek/智谱/通义/Kimi/OpenRouter/Ollama）
│     ├─ translation/          # 翻译领域：Prompt 模板、任务编排（流式转发/取消/落库）
│     ├─ config.rs             # 应用配置（JSON 持久化 + keyring 存 Key）
│     ├─ history.rs            # 翻译历史（SQLite，自动裁剪至 1000 条）
│     ├─ error.rs              # AppError 统一错误（序列化为 {code, message}）
│     ├─ state.rs              # 全局 managed state
│     └─ lib.rs                # 应用装配（托盘、关窗隐藏、命令注册）
│  └─ tests/mock_stream.rs     # 双协议端到端测试（本地 TcpListener 模拟 SSE 上游）
```

## 架构要点

1. **双协议抽象**：所有供应商请求由 Rust 侧发起。`Protocol` trait 屏蔽
   Chat Completions 与 Responses API 的端点/鉴权/SSE 差异，对上统一输出
   `TranslationEvent`（Delta / Finished / Failed）。新增协议只需新增实现。
   协议层由本地 TcpListener mock 的端到端集成测试覆盖（`src-tauri/tests/mock_stream.rs`）。
2. **密钥安全**：API Key 存系统钥匙串（Windows 凭据管理器），配置文件只存引用；
   所有 LLM 请求在 Rust 侧完成，WebView 无 CORS 限制也无密钥泄露面。
3. **流式优先**：翻译结果以增量事件推送到前端（Tauri Channel），支持中途取消
   （Esc / 重新翻译都会中断上游请求，不浪费 token）。
4. **统一错误**：Rust `AppError` → `{code, message}`，前端 `IpcError` 归一化，
   用户看到的永远是可读信息与下一步引导（如 404 时提示切换协议）。

## 推荐的 IDE 插件

VS Code + [Vue - Official (Volar)](https://marketplace.visualstudio.com/items?itemName=Vue.volar)

- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
  （`.vscode/extensions.json` 已配置推荐列表）
