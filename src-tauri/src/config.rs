//! 应用配置：JSON 持久化 + 系统钥匙串存 API Key。明文 Key 永不写入配置文件。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::provider::ProtocolKind;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemePref {
    Light,
    Dark,
    #[default]
    System,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub protocol: ProtocolKind,
    /// 指向系统钥匙串中的条目；明文 Key 永不进入配置文件。
    pub api_key_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub schema_version: u32,
    pub theme: ThemePref,
    pub active_provider: Option<ProviderConfig>,
}

/// 配置文件存取（应用数据目录下 config.json）。
#[derive(Debug, Clone)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> AppConfig {
        std::fs::read_to_string(&self.path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, config: &AppConfig) -> AppResult<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::Config(format!("无法创建配置目录：{e}")))?;
        }
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| AppError::Config(format!("配置序列化失败：{e}")))?;
        std::fs::write(&self.path, json).map_err(|e| AppError::Config(format!("配置写入失败：{e}")))
    }
}

const KEYRING_SERVICE: &str = "com.mftranslator.app";

fn keyring_entry(provider_id: &str) -> AppResult<keyring::Entry> {
    keyring::Entry::new(KEYRING_SERVICE, &format!("provider/{provider_id}"))
        .map_err(|e| AppError::Config(format!("无法访问系统钥匙串：{e}")))
}

pub fn store_api_key(provider_id: &str, api_key: &str) -> AppResult<()> {
    keyring_entry(provider_id)?
        .set_password(api_key)
        .map_err(|e| AppError::Config(format!("保存 API Key 失败：{e}")))
}

pub fn delete_api_key(provider_id: &str) -> AppResult<()> {
    match keyring_entry(provider_id)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Config(format!("清除 API Key 失败：{e}"))),
    }
}

/// 读取供应商 API Key。未配置 Key 引用时返回空串（视为免鉴权，如本地 Ollama）。
pub fn load_api_key(provider: &ProviderConfig) -> AppResult<String> {
    if provider.api_key_ref.is_none() {
        return Ok(String::new());
    }
    match keyring_entry(&provider.id)?.get_password() {
        Ok(key) => Ok(key),
        Err(keyring::Error::NoEntry) => Err(AppError::Config(
            "API Key 缺失，请在设置中重新填写".to_string(),
        )),
        Err(e) => Err(AppError::Config(format!("读取 API Key 失败：{e}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_roundtrips_through_json() {
        let config = AppConfig {
            theme: ThemePref::Dark,
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.theme, ThemePref::Dark);
        assert!(parsed.active_provider.is_none());
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let parsed: AppConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(parsed.schema_version, AppConfig::default().schema_version);
        assert_eq!(parsed.theme, ThemePref::System);
    }

    /// 钥匙串真实读写回环（Windows 凭据管理器）。
    /// keyring 3.x 未启用平台 feature 时此处会失败——正是“测试连接成功、保存失败”的根因防线。
    #[cfg(windows)]
    #[test]
    fn keyring_roundtrip() {
        let id = format!("mf-keyring-test-{}", std::process::id());
        store_api_key(&id, "sk-test-value").unwrap();

        let provider = ProviderConfig {
            id: id.clone(),
            api_key_ref: Some(id.clone()),
            ..Default::default()
        };
        assert_eq!(load_api_key(&provider).unwrap(), "sk-test-value");

        delete_api_key(&id).unwrap();
        assert!(load_api_key(&provider).is_err(), "删除后读取应报 Key 缺失");
    }

    #[test]
    fn config_file_save_then_load_roundtrips() {
        let path = std::env::temp_dir().join(format!("mf-config-test-{}.json", std::process::id()));
        let store = ConfigStore::new(path.clone());
        let config = AppConfig {
            schema_version: 1,
            theme: ThemePref::Light,
            active_provider: Some(ProviderConfig {
                id: "p1".into(),
                name: "Test".into(),
                base_url: "https://api.example.com/v1".into(),
                model: "m1".into(),
                protocol: ProtocolKind::Responses,
                api_key_ref: Some("p1".into()),
            }),
        };
        store.save(&config).unwrap();
        let loaded = store.load();
        assert_eq!(loaded, config);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn provider_config_json_never_contains_raw_key_field() {
        let json = serde_json::to_string(&ProviderConfig::default()).unwrap();
        assert!(
            !json.contains("apiKey") || json.contains("apiKeyRef"),
            "配置只允许出现脱敏的 apiKeyRef 字段"
        );
    }
}
