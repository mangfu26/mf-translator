//! 内置供应商预设：支撑“选模板 → 粘 Key → 开翻”的一分钟配置路径。
//! 预设只是默认值，用户可在设置中修改任意字段或添加自定义供应商。

use super::ProtocolKind;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderPreset {
    pub id: &'static str,
    pub name: &'static str,
    pub base_url: &'static str,
    pub default_models: &'static [&'static str],
    pub default_protocol: ProtocolKind,
}

pub fn builtin() -> Vec<ProviderPreset> {
    vec![
        ProviderPreset {
            id: "openai",
            name: "OpenAI",
            base_url: "https://api.openai.com/v1",
            default_models: &["gpt-5", "gpt-5-mini", "gpt-4.1", "gpt-4o-mini"],
            default_protocol: ProtocolKind::Responses,
        },
        ProviderPreset {
            id: "openrouter",
            name: "OpenRouter",
            base_url: "https://openrouter.ai/api/v1",
            default_models: &[
                "openai/gpt-4o-mini",
                "anthropic/claude-sonnet-4",
                "google/gemini-2.5-flash",
            ],
            default_protocol: ProtocolKind::ChatCompletions,
        },
        ProviderPreset {
            id: "deepseek",
            name: "DeepSeek",
            base_url: "https://api.deepseek.com/v1",
            default_models: &["deepseek-chat", "deepseek-reasoner"],
            default_protocol: ProtocolKind::ChatCompletions,
        },
        ProviderPreset {
            id: "zhipu",
            name: "智谱 GLM",
            base_url: "https://open.bigmodel.cn/api/paas/v4",
            default_models: &["glm-4.5", "glm-4.5-air", "glm-4-flash"],
            default_protocol: ProtocolKind::ChatCompletions,
        },
        ProviderPreset {
            id: "qwen",
            name: "通义千问",
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
            default_models: &["qwen-plus", "qwen-turbo", "qwen-max"],
            default_protocol: ProtocolKind::ChatCompletions,
        },
        ProviderPreset {
            id: "moonshot",
            name: "Kimi (Moonshot)",
            base_url: "https://api.moonshot.cn/v1",
            default_models: &["kimi-k2-0905-preview", "moonshot-v1-8k"],
            default_protocol: ProtocolKind::ChatCompletions,
        },
        ProviderPreset {
            id: "ollama",
            name: "Ollama（本地）",
            base_url: "http://localhost:11434/v1",
            default_models: &["qwen2.5:7b", "llama3.1:8b"],
            default_protocol: ProtocolKind::ChatCompletions,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_have_valid_metadata() {
        for preset in builtin() {
            assert!(
                preset.base_url.starts_with("http"),
                "{} 的 base_url 非法",
                preset.id
            );
            assert!(
                !preset.default_models.is_empty(),
                "{} 缺少默认模型",
                preset.id
            );
            assert!(!preset.name.is_empty());
        }
    }

    #[test]
    fn preset_ids_are_unique() {
        let mut ids: Vec<&str> = builtin().iter().map(|p| p.id).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "预设 id 存在重复");
    }

    #[test]
    fn openai_preset_defaults_to_responses_protocol() {
        let openai = builtin()
            .into_iter()
            .find(|p| p.id == "openai")
            .expect("openai 预设必须存在");
        assert_eq!(openai.default_protocol, ProtocolKind::Responses);
    }
}
