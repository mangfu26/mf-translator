//! 提示词模板存储：SQLite 持久化 + 内置默认模板播种。
//! 每种风格模式独立一条系统提示词模板，外加一条用户消息模板。
//! 翻译时优先读用户改过的版本，未改过时回退到内置默认值。

use std::path::Path;

use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};

/// 模板键：`system.<mode>`（各风格的系统提示词）或 `user`（用户消息模板）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptKey {
    SystemGeneral,
    SystemAcademic,
    SystemColloquial,
    SystemPolish,
    User,
}

impl PromptKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            PromptKey::SystemGeneral => "system.general",
            PromptKey::SystemAcademic => "system.academic",
            PromptKey::SystemColloquial => "system.colloquial",
            PromptKey::SystemPolish => "system.polish",
            PromptKey::User => "user",
        }
    }
}

/// 内置默认模板（代码内锚点，也是播种到 DB 的种子值）。
pub fn default_template(key: PromptKey) -> &'static str {
    match key {
        PromptKey::SystemGeneral => {
            "你是一个专业的翻译引擎。把用户输入的文本翻译成{target_language}。\n\
             要求：\n\
             1. 只输出译文本身，不要任何解释、注释或原文；\n\
             2. 保持原文的语气、格式与换行；\n\
             3. 专有名词、代码、URL、邮箱地址保持原样；\n\
             4. 译文要符合{target_language}的表达习惯，自然流畅。"
        }
        PromptKey::SystemAcademic => {
            "你是一个专业的学术翻译引擎。把用户输入的文本翻译成{target_language}。\n\
             要求：\n\
             1. 只输出译文本身，不要任何解释、注释或原文；\n\
             2. 术语翻译准确，保持原文的逻辑层次与论证结构；\n\
             3. 行文严谨正式，符合学术表达习惯；\n\
             4. 专有名词、代码、URL、邮箱地址保持原样。"
        }
        PromptKey::SystemColloquial => {
            "你是一个擅长口语表达的翻译引擎。把用户输入的文本翻译成{target_language}。\n\
             要求：\n\
             1. 只输出译文本身，不要任何解释、注释或原文；\n\
             2. 使用自然、日常的表达，避免书面腔；\n\
             3. 译文的语气轻松、口语化，贴近对话场景；\n\
             4. 专有名词、代码、URL、邮箱地址保持原样。"
        }
        PromptKey::SystemPolish => {
            "你是一个文本润色翻译引擎。把用户输入的文本翻译成{target_language}。\n\
             要求：\n\
             1. 只输出译文本身，不要任何解释、注释或原文；\n\
             2. 忠于原意前提下优化措辞，使译文更地道、流畅；\n\
             3. 保持原文的意图与细节，不增删信息；\n\
             4. 专有名词、代码、URL、邮箱地址保持原样。"
        }
        PromptKey::User => "源语言：{source_language}\n待翻译文本：\n{source_text}",
    }
}

/// 所有模板键（用于遍历播种/重置）。
pub const ALL_KEYS: &[PromptKey] = &[
    PromptKey::SystemGeneral,
    PromptKey::SystemAcademic,
    PromptKey::SystemColloquial,
    PromptKey::SystemPolish,
    PromptKey::User,
];

/// 带来源标记的模板读取结果。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateEntry {
    pub key: String,
    pub content: String,
    /// 用户是否改过（未改过 = 使用内置默认值源）。
    pub user_modified: bool,
}

pub struct PromptStore {
    conn: Connection,
}

fn db_err(e: rusqlite::Error) -> AppError {
    AppError::Config(format!("提示词模板读写失败：{e}"))
}

impl PromptStore {
    pub fn open(path: &Path) -> AppResult<Self> {
        let conn = Connection::open(path)
            .map_err(|e| AppError::Config(format!("无法打开提示词数据库：{e}")))?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS prompt_templates (
                key TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                user_modified INTEGER NOT NULL DEFAULT 0
             );",
        )
        .map_err(db_err)?;
        let store = Self { conn };
        store.seed_defaults()?;
        Ok(store)
    }

    /// 用内置默认值填补缺失的模板行（首次启动 / 新增风格时），不覆盖已改过的。
    fn seed_defaults(&self) -> AppResult<()> {
        for key in ALL_KEYS {
            self.insert_if_missing(*key, default_template(*key))?;
        }
        Ok(())
    }

    fn insert_if_missing(&self, key: PromptKey, content: &str) -> AppResult<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO prompt_templates (key, content, user_modified)
                 VALUES (?1, ?2, 0)",
                params![key.as_str(), content],
            )
            .map_err(db_err)?;
        Ok(())
    }

    /// 读取所有模板，未播种过的用默认值补上（保证列表完整）。
    pub fn all(&self) -> AppResult<Vec<TemplateEntry>> {
        self.seed_defaults()?;
        let mut stmt = self
            .conn
            .prepare("SELECT key, content, user_modified FROM prompt_templates")
            .map_err(db_err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(TemplateEntry {
                    key: row.get(0)?,
                    content: row.get(1)?,
                    user_modified: row.get::<_, i64>(2)? != 0,
                })
            })
            .map_err(db_err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db_err)?;
        Ok(rows)
    }

    /// 读取单个模板。若用户未改过或不存在，返回内置默认值。
    pub fn get(&self, key: PromptKey) -> AppResult<String> {
        self.seed_defaults()?;
        let modified: Option<i64> = self
            .conn
            .query_row(
                "SELECT user_modified FROM prompt_templates WHERE key = ?1",
                params![key.as_str()],
                |row| row.get(0),
            )
            .map_err(db_err)?;
        // 未改过或不存在 → 用内置默认值（保证程序始终有可用提示词）
        if modified != Some(1) {
            return Ok(default_template(key).to_string());
        }
        self.conn
            .query_row(
                "SELECT content FROM prompt_templates WHERE key = ?1",
                params![key.as_str()],
                |row| row.get(0),
            )
            .map_err(db_err)
    }

    /// 强制 WAL checkpoint：用户保存的模板必须立即落盘主库，
    /// 防止进程被强杀/崩溃时 WAL 中未合并的写入丢失（真实发生过）。
    fn checkpoint(&self) -> AppResult<()> {
        self.conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(db_err)
    }

    /// 保存用户模板（标记为已修改）。
    pub fn set_modified(&self, key: PromptKey, content: &str) -> AppResult<()> {
        let content = content.trim();
        if content.is_empty() {
            return Err(AppError::Config("提示词模板不能为空".to_string()));
        }
        self.conn
            .execute(
                "INSERT INTO prompt_templates (key, content, user_modified)
                 VALUES (?1, ?2, 1)
                 ON CONFLICT(key) DO UPDATE SET content = excluded.content, user_modified = 1",
                params![key.as_str(), content],
            )
            .map_err(db_err)?;
        self.checkpoint()
    }

    /// 恢复指定模板为内置默认值（清除用户修改标记）。
    pub fn reset(&self, key: PromptKey) -> AppResult<()> {
        self.conn
            .execute(
                "INSERT INTO prompt_templates (key, content, user_modified)
                 VALUES (?1, ?2, 0)
                 ON CONFLICT(key) DO UPDATE SET content = excluded.content, user_modified = 0",
                params![key.as_str(), default_template(key)],
            )
            .map_err(db_err)?;
        self.checkpoint()
    }

    /// 恢复所有模板为默认值。
    pub fn reset_all(&self) -> AppResult<()> {
        for key in ALL_KEYS {
            self.reset(*key)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> (PromptStore, std::path::PathBuf) {
        let path = std::env::temp_dir().join(format!(
            "mf-prompt-test-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        (PromptStore::open(&path).unwrap(), path)
    }

    #[test]
    fn seeds_all_defaults() {
        let (store, path) = temp_store();
        let all = store.all().unwrap();
        assert_eq!(all.len(), ALL_KEYS.len());
        // 默认未被修改
        assert!(all.iter().all(|e| !e.user_modified));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn get_returns_default_when_untouched() {
        let (store, path) = temp_store();
        let got = store.get(PromptKey::SystemGeneral).unwrap();
        assert_eq!(got, default_template(PromptKey::SystemGeneral));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn modified_template_persists_and_reads_back() {
        let (store, path) = temp_store();
        store
            .set_modified(PromptKey::SystemAcademic, "自定义学术模板")
            .unwrap();
        let got = store.get(PromptKey::SystemAcademic).unwrap();
        assert_eq!(got, "自定义学术模板");
        // 其他键仍用默认
        let other = store.get(PromptKey::SystemGeneral).unwrap();
        assert_eq!(other, default_template(PromptKey::SystemGeneral));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn reset_clears_user_modified() {
        let (store, path) = temp_store();
        store
            .set_modified(PromptKey::SystemPolish, "改过的")
            .unwrap();
        assert_eq!(store.get(PromptKey::SystemPolish).unwrap(), "改过的");
        store.reset(PromptKey::SystemPolish).unwrap();
        assert_eq!(
            store.get(PromptKey::SystemPolish).unwrap(),
            default_template(PromptKey::SystemPolish)
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn empty_template_rejected() {
        let (store, path) = temp_store();
        assert!(store.set_modified(PromptKey::User, "   ").is_err());
        let _ = std::fs::remove_file(path);
    }
}
