//! 翻译历史：SQLite 持久化（应用数据目录 history.db），自动裁剪至最近 1000 条。

use std::path::Path;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::provider::Usage;

pub struct HistoryInsert<'a> {
    pub source_text: &'a str,
    pub translated_text: &'a str,
    pub source_language: Option<&'a str>,
    pub target_language: &'a str,
    pub provider: &'a str,
    pub model: &'a str,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub id: i64,
    pub created_at: i64,
    pub source_text: String,
    pub translated_text: String,
    pub source_language: Option<String>,
    pub target_language: String,
    pub provider: String,
    pub model: String,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
}

const KEEP_ROWS: i64 = 1000;

fn db_err(e: rusqlite::Error) -> AppError {
    AppError::Config(format!("历史记录读写失败：{e}"))
}

pub struct HistoryDb {
    conn: Connection,
}

impl HistoryDb {
    pub fn open(path: &Path) -> AppResult<Self> {
        let conn = Connection::open(path)
            .map_err(|e| AppError::Config(format!("无法打开历史数据库：{e}")))?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS translations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at INTEGER NOT NULL DEFAULT (unixepoch('now')),
                source_text TEXT NOT NULL,
                translated_text TEXT NOT NULL,
                source_language TEXT,
                target_language TEXT NOT NULL,
                provider TEXT NOT NULL DEFAULT '',
                model TEXT NOT NULL DEFAULT '',
                prompt_tokens INTEGER,
                completion_tokens INTEGER
             );
             CREATE INDEX IF NOT EXISTS idx_translations_created
                ON translations (created_at DESC);",
        )
        .map_err(db_err)?;
        Ok(Self { conn })
    }

    pub fn insert(&self, row: &HistoryInsert<'_>) -> AppResult<()> {
        self.conn
            .execute(
                "INSERT INTO translations
                    (source_text, translated_text, source_language, target_language,
                     provider, model, prompt_tokens, completion_tokens)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    row.source_text,
                    row.translated_text,
                    row.source_language,
                    row.target_language,
                    row.provider,
                    row.model,
                    row.usage.map(|u| u.prompt_tokens as i64),
                    row.usage.map(|u| u.completion_tokens as i64),
                ],
            )
            .map_err(db_err)?;
        self.conn
            .execute(
                "DELETE FROM translations WHERE id NOT IN
                    (SELECT id FROM translations ORDER BY id DESC LIMIT ?1)",
                params![KEEP_ROWS],
            )
            .map_err(db_err)?;
        // 强制 checkpoint 落盘，防止强杀/崩溃丢失 WAL 中未合并的翻译记录
        self.checkpoint()
    }

    /// 强制 WAL checkpoint 落盘。
    fn checkpoint(&self) -> AppResult<()> {
        self.conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(db_err)
    }

    pub fn list(&self, query: Option<&str>, limit: i64) -> AppResult<Vec<HistoryItem>> {
        let keyword = query.map(str::trim).filter(|q| !q.is_empty()).map(|q| {
            let escaped: String = q.chars().filter(|c| *c != '%' && *c != '_').collect();
            format!("%{escaped}%")
        });

        let sql = "SELECT id, created_at, source_text, translated_text, source_language,
                          target_language, provider, model, prompt_tokens, completion_tokens
                   FROM translations";
        let mut stmt = if keyword.is_some() {
            self.conn
                .prepare(&format!(
                    "{sql} WHERE source_text LIKE ?1 OR translated_text LIKE ?1 \
                     ORDER BY id DESC LIMIT ?2"
                ))
                .map_err(db_err)?
        } else {
            self.conn
                .prepare(&format!("{sql} ORDER BY id DESC LIMIT ?1"))
                .map_err(db_err)?
        };

        let map_row = |row: &rusqlite::Row| -> rusqlite::Result<HistoryItem> {
            Ok(HistoryItem {
                id: row.get(0)?,
                created_at: row.get(1)?,
                source_text: row.get(2)?,
                translated_text: row.get(3)?,
                source_language: row.get(4)?,
                target_language: row.get(5)?,
                provider: row.get(6)?,
                model: row.get(7)?,
                prompt_tokens: row.get::<_, Option<i64>>(8)?.map(|v| v as u32),
                completion_tokens: row.get::<_, Option<i64>>(9)?.map(|v| v as u32),
            })
        };

        let items = if let Some(pattern) = keyword {
            stmt.query_map(params![pattern, limit], map_row)
                .map_err(db_err)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(db_err)?
        } else {
            stmt.query_map(params![limit], map_row)
                .map_err(db_err)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(db_err)?
        };
        Ok(items)
    }

    pub fn delete(&self, id: i64) -> AppResult<()> {
        self.conn
            .execute("DELETE FROM translations WHERE id = ?1", params![id])
            .map_err(db_err)?;
        self.checkpoint()
    }

    pub fn clear(&self) -> AppResult<()> {
        self.conn
            .execute("DELETE FROM translations", [])
            .map_err(db_err)?;
        self.checkpoint()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> (HistoryDb, std::path::PathBuf) {
        let path = std::env::temp_dir().join(format!(
            "mf-history-test-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        (HistoryDb::open(&path).unwrap(), path)
    }

    fn sample<'a>(source: &'a str, translated: &'a str) -> HistoryInsert<'a> {
        HistoryInsert {
            source_text: source,
            translated_text: translated,
            source_language: Some("英语"),
            target_language: "简体中文",
            provider: "Test",
            model: "m1",
            usage: Some(Usage {
                prompt_tokens: 3,
                completion_tokens: 2,
            }),
        }
    }

    #[test]
    fn insert_and_list_roundtrip() {
        let (db, path) = temp_db();
        db.insert(&sample("hello", "你好")).unwrap();
        db.insert(&sample("good morning", "早上好")).unwrap();
        let items = db.list(None, 10).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].translated_text, "早上好"); // 最新在前
        assert_eq!(items[0].prompt_tokens, Some(3));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn search_filters_by_keyword() {
        let (db, path) = temp_db();
        db.insert(&sample("hello world", "你好世界")).unwrap();
        db.insert(&sample("good morning", "早上好")).unwrap();
        assert_eq!(db.list(Some("world"), 10).unwrap().len(), 1);
        assert_eq!(db.list(Some("早上"), 10).unwrap().len(), 1);
        assert_eq!(db.list(Some("不存在的句子"), 10).unwrap().len(), 0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn delete_and_clear_work() {
        let (db, path) = temp_db();
        db.insert(&sample("a", "甲")).unwrap();
        let id = db.list(None, 10).unwrap()[0].id;
        db.delete(id).unwrap();
        assert!(db.list(None, 10).unwrap().is_empty());
        db.insert(&sample("b", "乙")).unwrap();
        db.clear().unwrap();
        assert!(db.list(None, 10).unwrap().is_empty());
        let _ = std::fs::remove_file(path);
    }
}
