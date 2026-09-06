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

| 类型     | 命名          | 用途             | 规则                                       |
| -------- | ------------- | ---------------- | ------------------------------------------ |
| 主分支   | `main`        | 稳定分支         | **不接受直接修改**，只能由开发分支合并更新 |
| 开发分支 | `develop/xxx` | 承载所有开发工作 | `xxx` 自定义命名，但必须与开发工作相关     |

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

- 禁止直接在 `main` 上提交代码（所有变更必须经由 `develop/xxx` 分支合并进入）。
  **唯一例外**：发布定稿提交（`chore(release): vX.Y.Z`，见 1.7）允许直接在 `main` 上进行
- 禁止主动创建或推送 git 标签——**标签仅在维护者明确要求时才打**
  （发布打 tag 属于 1.7 发布流程的正常环节，同样需维护者指示后执行）

### 1.5 合并控制权（强制）

**开发分支完成开发后，AI Agent 不得自行将 `develop/xxx` 合并到 `main`。**
合并是维护者决定"哪些功能进入稳定分支"的关键决策点，必须由维护者本人掌控。

- 开发完成并验证通过后，AI Agent 应**汇报完成情况**，并**等待维护者明确指示是否合并**
- 维护者确认合并后，AI Agent 还需**再次询问采用哪种合并方式**（如普通 `merge` / `--no-ff` / `rebase` / `squash`），并**确认是否删除该开发分支**
- 接到明确的合并指示与方式后，才可执行合并；未获指示前不要合并、不要推送开发分支到远程
- **既定实践**：合并完成后删除该开发分支（本地与远程）；开发分支本身可按维护者指示推送到远程（如供审阅）

### 1.6 未约定事项（遇到时询问维护者，勿自行决定）

- 未指明合并方式时的默认值不存在——每次都必须询问，不得沿用上次的猜测

### 1.7 版本发布流程（不设长期 release 分支）

发布与功能开发职责分离：**功能归 `develop/xxx`，发布动作归 `main` + tag**。每次发布三步：

1. **合并功能**：待发布的 `develop/xxx` 按 1.5 规则合并进 `main`
2. **发布定稿**：在 `main` 上做一个独立提交 `chore(release): vX.Y.Z`，内容必须包含：
   - 版本号**三处同步**：`package.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml`
   - `update.json`：`version` 与 `notes` 同步更新（**漏改则用户端"检查更新"检测不到新版**）
3. **打 tag 发布**：推送 tag `vX.Y.Z`（**tag 必须与 `tauri.conf.json` 的 version 一致**，
   Release 的 tag 名由该配置生成），随后按第 3 章 CI 自动构建，维护者在 Releases 页
   检查草稿后手动 **Publish** 正式发布

**不使用长期 release 分支**；若发布定稿期间需要冻结修改，可建 `release/vX.Y.Z` 短期分支，
定稿合并回 `main` 后即删除。历史发布点以 tag 为凭证（如需基于旧版本修 bug，从对应 tag 拉分支）。

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

| 类型              | 含义                                                 | SemVer 影响 |
| ----------------- | ---------------------------------------------------- | ----------- |
| `feat`            | 新增功能（**必须**使用）                             | MINOR       |
| `fix`             | 修复 bug（**必须**使用）                             | PATCH       |
| `BREAKING CHANGE` | 破坏性变更（见 2.3）                                 | MAJOR       |
| `docs`            | 文档变更                                             | 无          |
| `refactor`        | 重构（既非新增功能也非修复 bug）                     | 无          |
| `perf`            | 性能优化                                             | 无          |
| `test`            | 测试的增删改                                         | 无          |
| `build`           | 构建系统或外部依赖变更                               | 无          |
| `ci`              | CI 配置变更                                          | 无          |
| `chore`           | 其他不修改 src 或测试的杂项                          | 无          |
| `style`           | 代码格式（不影响代码含义的空格、格式化等）           | 无          |
| `revert`          | 还原提交（脚注引用被还原的提交，如 `Refs: 676104e`） | 无          |

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

---

## 3. CI/CD 与应用更新机制

> 仓库托管于 **GitHub（mangfu26/mf-translator）**。

### 3.1 工作流触发规则（改 workflow 前必读）

| Workflow                        | 触发条件                        | 作用                                                                                                               |
| ------------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `.github/workflows/ci.yml`      | push 到 `main`、面向 main 的 PR | **质量门禁**：前端 lint / format:check / typecheck / vitest / build（Ubuntu）+ Rust fmt / clippy / test（Windows） |
| `.github/workflows/release.yml` | **仅推送 `v*` tag**             | Windows runner 构建 NSIS + MSI 安装包，自动创建 **GitHub Release 草稿**并挂载附件                                  |

- 两条流水线互不影响：推 main 只跑门禁，打 tag 只跑构建
- **推 main 前必须本地全绿**（含 `npm run format:check`）——曾有功能改动漏跑格式化
  导致 CI 失败的先例；Prettier 格式问题以 CI（Linux/LF）判定为准，本地 CRLF 行尾噪音不算
- Release 产物当前**仅 Windows 平台**；扩展 macOS/Linux 需修改 release.yml 的构建矩阵
- Release 为**草稿模式**（`releaseDraft: true`）：构建成功后须维护者在 Releases 页
  检查草稿并手动 **Publish** 才正式可见（防止坏包直接公开）

### 3.2 应用内更新机制（改发布相关代码前必读）

- 应用「检查更新」读取 **main 分支 raw 的 `update.json`**，URL 硬编码于
  `src-tauri/src/commands/update.rs` 的 `UPDATE_MANIFEST_URL`
  （`https://raw.githubusercontent.com/mangfu26/mf-translator/main/update.json`）
- **`update.json` 的变更必须落在 `main` 分支**——改在其他分支用户端永远读不到；
  这就是 1.7 发布流程中它必须随 `chore(release)` 提交进 main 的原因
- 版本比较逻辑：清单 `version`（semver）**严格大于**当前应用版本才提示更新，
  允许 `v` 前缀；`downloadUrl` 跳转 GitHub Releases 页
- GitHub 公开仓库的 Release 附件**无需登录**即可下载
- 已知问题：`raw.githubusercontent.com` 国内访问偶发超时（表现为"检查更新"报
  网络异常）；备选方案为 jsDelivr CDN 多源回退（**尚未实施**，维护者未拍板）

### 3.3 数据存储位置（排查"数据丢失"类问题前必读）

- `%APPDATA%/com.mftranslator.app/`：`config.json`（供应商配置，仅存 `apiKeyRef` 引用）、
  `history.db`（翻译历史）、`prompts.db`（提示词模板）——均为 SQLite WAL 模式
- **API Key 明文只存 Windows 凭据管理器**（keyring crate），配置文件与仓库中均无明文
- 所有用户写入（保存模板、翻译落库）后均执行 `wal_checkpoint(TRUNCATE)` 强制落盘——
  此前发生过强杀进程导致 WAL 未合并数据丢失的事故，**勿移除这些 checkpoint 调用**

---

## 4. 安全红线（违反即严重事故）

### 4.1 敏感信息零入库

- **禁止将任何敏感信息推送到远程仓库**：API Key、密钥、令牌、密码、个人数据等。
  必须推送的内容先脱敏，或完全不推送——此为红线，违反即**严重事故**
- 真实 API Key 只允许存在于系统钥匙串（用户在 APP 内填写）；仓库与构建产物中
  不应出现任何真实 Key
- **提交与推送前的自检动作**：`git grep -iE "sk-[a-zA-Z0-9]{16,}"` 扫描真实密钥形态、
  `git ls-files` 确认无 `.env`/数据库/凭据类文件被跟踪；代码中的测试占位值
  （如 `sk-test`）必须是明显的假值
- 一旦发现敏感信息已被推送：**立即告知维护者**并协助轮换密钥、清理历史，不得隐瞒

### 4.2 禁止自动推送

- **禁止自动推送本地分支到远程仓库**——只有维护者**明确指示**推送时才执行 `git push`
- "明确指示"指维护者明确提出推送要求（如"推送到远程""推送该分支"）；
  提交、合并、打 tag 等本地操作**均不构成推送授权**
