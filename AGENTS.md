# AGENTS.md — AI Agent 维护须知

> 本文件是交给后续参与开发/维护本仓库的 AI Agent 的"记忆锚点"，记录无法从代码本身
> 推导出来的团队约定与决策。**约定优先于聪明**：按本文件执行，不要自行发明流程；
> 遇到"未约定事项"时，向维护者询问，不要猜测。

## 项目速览

- **MF 翻译**：自带大模型 Key（BYOK）的桌面翻译工作台，Tauri 2（Rust）+ Vue 3 + TypeScript
- 详细架构、目录结构、常用命令见 [README.md](./README.md)
- 里程碑进度以 README.md 的里程碑表为准（本文档不重复维护进度信息）

---

## 1. Git 分支规范

仓库分支分为主分支和开发分支两种类型：

### 1.1 分支类型

| 类型 | 命名 | 用途 | 规则 |
| --- | --- | --- | --- |
| 主分支 | `main` | 稳定分支 | **不接受直接修改**，只能由开发分支合并更新 |
| 开发分支 | `develop/xxx` | 承载所有开发工作 | `xxx` 自定义命名，但必须与开发工作相关 |

开发分支命名示例：未来想支持俄语，就创建 `develop/i18n-russian`；其他开发工作以此类推
（名称要能让人一眼看出这轮开发在做什么）。

### 1.2 工作流程

1. 需要添加新功能、修复 bug、优化等，**从 `main` 分支创建一个 `develop/xxx` 分支**
2. 在开发分支上完成开发与测试
3. 将 `develop/xxx` 分支**合并到 `main` 分支**

### 1.3 示例命令

```bash
# ① 从 main 创建开发分支
git checkout main
git checkout -b develop/i18n-russian

# ② 在开发分支上开发、提交、测试（提交信息遵循 Conventional Commits）

# ③ 开发测试完成后，合并回 main
git checkout main
git merge develop/i18n-russian
```

### 1.4 禁止事项

- 禁止直接在 `main` 上提交代码（所有变更必须经由 `develop/xxx` 分支合并进入）
- 禁止主动创建或推送 git 标签——**标签仅在维护者明确要求时才打**

### 1.5 未约定事项（遇到时询问维护者，勿自行决定）

- 合并方式（普通 merge / `--no-ff` / squash）、合并后是否删除开发分支
- 远程仓库是否推送 `develop/xxx` 开发分支，还是仅推送 `main`
  （现有实践：仅推送 `main`，开发分支保留在本地，仅供参考）

---

## 2. Git Commit 规范（Conventional Commits 1.0.0）

> 官方规范原文：<https://www.conventionalcommits.org/zh-hans/v1.0.0/>
> 本节为提炼摘要，与官方规范冲突时以官方规范为准。

### 2.1 提交信息结构

```
<type>[可选 scope]: <描述>

[可选正文]

[可选脚注]
```

- `type` 后接可选的 scope（描述变更范围的名词，圆括号包围）、可选的 `!`，
  然后是**英文半角冒号 + 空格**，再接描述
- 正文与描述之间**空一行**；脚注与正文之间**空一行**
- 脚注格式：`令牌: 内容` 或 `令牌 #内容`，令牌用 `-` 连字符（如 `Reviewed-by`），
  唯一例外是 `BREAKING CHANGE`

### 2.2 类型（type）

| 类型 | 含义 | SemVer 影响 |
| --- | --- | --- |
| `feat` | 新增功能（**必须**使用） | MINOR |
| `fix` | 修复 bug（**必须**使用） | PATCH |
| `BREAKING CHANGE` | 破坏性变更（见 2.3） | MAJOR |
| `docs` | 文档变更 | 无 |
| `refactor` | 重构（既非新增功能也非修复 bug） | 无 |
| `perf` | 性能优化 | 无 |
| `test` | 测试的增删改 | 无 |
| `build` | 构建系统或外部依赖变更 | 无 |
| `ci` | CI 配置变更 | 无 |
| `chore` | 其他不修改 src 或测试的杂项 | 无 |
| `style` | 代码格式（不影响代码含义的空格、格式化等） | 无 |
| `revert` | 还原提交（脚注引用被还原的提交，如 `Refs: 676104e`） | 无 |

- `feat` / `fix` 是规范**强制**要求；其余类型沿用 Angular 约定，可按需扩展
- 除 `BREAKING CHANGE` 外，工具解析不区分类型大小写

### 2.3 破坏性变更（Breaking Change）的两种声明方式

1. 脚注方式（必须大写）：

   ```
   feat: allow provided config object to extend other configs

   BREAKING CHANGE: `extends` key in config file is now used for extending other config files
   ```

2. `!` 前缀方式（可省略脚注，描述中应说明破坏点）：

   ```
   feat(api)!: send an email to the customer when a product is shipped
   ```

两种方式可同时使用。

### 2.4 完整示例（含正文与脚注）

```
fix: prevent racing of requests

Introduce a request id and a reference to latest request. Dismiss
incoming responses other than from latest request.

Remove timeouts which were used to mitigate the racing issue but are
obsolete now.

Reviewed-by: Z
Refs: #123
```

### 2.5 本仓库适用约定

- **语言**：描述与正文使用中文（现有实践，便于团队阅读；规范本身不限制语言）
- **scope 参考词表**（推荐与目录结构对应，非强制）：
  `provider`（协议层）、`translate`（翻译服务）、`settings`（配置）、`history`（历史）、
  `ui`（前端界面）、`ci`（流水线）、`deps`（依赖）
- **回归防线**：改动 IPC 命令签名时，提交信息正文应说明前后端两侧是否同步
  （参见 `src/services/contract.test.ts` 的由来）
- 所有提交（包括 `main` 上经合并产生的提交）都遵循本规范
