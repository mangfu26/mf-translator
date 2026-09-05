//! 翻译领域：翻译模式枚举 + 提示词模板生成。
//! 提示词从 [`PromptStore`] 读取模板（用户改过则用用户版，未改过用默认），
//! 替换占位符后动态生成最终发送给模型的提示词。

use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::prompt_store::{PromptKey, PromptStore};
use crate::provider::{PromptPair, TranslateRequest};

/// 翻译模式：每种模式独立一条系统提示词模板，可由用户微调。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TranslationMode {
    /// 通用：忠于原文，自然通顺（默认）。
    #[default]
    General,
    /// 学术：严谨用词，保留术语与逻辑层次。
    Academic,
    /// 口语：更贴近日常对话，轻松自然。
    Colloquial,
    /// 润色：优化表达但不改变原意。
    Polish,
}

impl TranslationMode {
    pub fn label(&self) -> &'static str {
        match self {
            TranslationMode::General => "通用",
            TranslationMode::Academic => "学术",
            TranslationMode::Colloquial => "口语",
            TranslationMode::Polish => "润色",
        }
    }
}

/// 把占位符替换为实际值的模板渲染。
fn render(template: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{key}}}"), value);
    }
    result
}

/// 从模板生成 system/user 提示词对。
pub fn build_prompt_pair(store: &PromptStore, request: &TranslateRequest) -> AppResult<PromptPair> {
    let system_key = match request.mode {
        TranslationMode::General => PromptKey::SystemGeneral,
        TranslationMode::Academic => PromptKey::SystemAcademic,
        TranslationMode::Colloquial => PromptKey::SystemColloquial,
        TranslationMode::Polish => PromptKey::SystemPolish,
    };
    let system_template = store.get(system_key)?;
    let system = render(
        &system_template,
        &[("target_language", &request.target_language)],
    );

    let user_template = store.get(PromptKey::User)?;
    let user = render(
        &user_template,
        &[
            (
                "source_language",
                request.source_language.as_deref().unwrap_or("自动检测"),
            ),
            ("source_text", &request.source_text),
        ],
    );

    Ok(PromptPair { system, user })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_store() -> PromptStore {
        let path = std::env::temp_dir().join(format!(
            "mf-prompt-build-test-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        PromptStore::open(&path).unwrap()
    }

    #[test]
    fn builds_pair_from_defaults() {
        let store = open_store();
        let req = TranslateRequest {
            model: "m".into(),
            source_text: "hi".into(),
            target_language: "日语".into(),
            source_language: Some("英语".into()),
            mode: TranslationMode::default(),
        };
        let pair = build_prompt_pair(&store, &req).unwrap();
        assert!(pair.system.contains("日语")); // target_language 已替换
        assert!(pair.user.contains("hi")); // source_text 已替换
        assert!(pair.user.contains("英语")); // source_language 已替换
    }

    #[test]
    fn uses_modified_template() {
        let store = open_store();
        store
            .set_modified(
                PromptKey::SystemColloquial,
                "口语模板：翻成 {target_language}",
            )
            .unwrap();
        let req = TranslateRequest {
            model: "m".into(),
            source_text: "x".into(),
            target_language: "中文".into(),
            source_language: None,
            mode: TranslationMode::Colloquial,
        };
        let pair = build_prompt_pair(&store, &req).unwrap();
        assert!(pair.system.contains("口语模板"));
        assert!(pair.system.contains("中文"));
    }

    #[test]
    fn mode_serializes_to_camel() {
        assert_eq!(
            serde_json::to_string(&TranslationMode::Colloquial).unwrap(),
            "\"colloquial\""
        );
    }
}
