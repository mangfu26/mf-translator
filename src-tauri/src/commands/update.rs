//! 应用内检查更新：拉取远程版本清单并与当前版本做语义化比较。
//! 阶段一实现“检查 + 提示 + 跳转下载页”；带签名的静默自动更新
//! （tauri-plugin-updater）在公开发布前接入。

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::provider::transport_error;
use crate::state::AppState;

/// 更新清单托管在公开仓库 main 分支（Gitee raw），发布新版时随仓库更新。
const UPDATE_MANIFEST_URL: &str = "https://gitee.com/mangfu-it/mf-translator/raw/main/update.json";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManifest {
    pub version: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub download_url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub notes: String,
    pub download_url: String,
}

/// 语义化版本比较：仅当 latest 严格大于 current 时视为有更新，允许 `v` 前缀。
pub fn is_update_available(current: &str, latest: &str) -> AppResult<bool> {
    let parse = |v: &str| {
        semver::Version::parse(v.trim().trim_start_matches('v'))
            .map_err(|e| AppError::MalformedResponse(format!("无法解析版本号 {v}：{e}")))
    };
    Ok(parse(latest)? > parse(current)?)
}

#[tauri::command]
pub async fn check_update(state: State<'_, AppState>) -> AppResult<UpdateStatus> {
    let response = state
        .client
        .get(UPDATE_MANIFEST_URL)
        .send()
        .await
        .map_err(transport_error)?;
    if !response.status().is_success() {
        return Err(AppError::Network(format!(
            "更新清单服务响应异常（HTTP {}）",
            response.status().as_u16()
        )));
    }
    let manifest: UpdateManifest = response
        .json()
        .await
        .map_err(|e| AppError::MalformedResponse(format!("更新清单格式无法解析：{e}")))?;

    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let update_available = is_update_available(&current_version, &manifest.version)?;

    Ok(UpdateStatus {
        current_version,
        latest_version: manifest.version,
        update_available,
        notes: manifest.notes,
        download_url: manifest.download_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_version_is_detected() {
        assert!(is_update_available("0.2.0", "0.3.0").unwrap());
        assert!(is_update_available("0.2.0", "1.0.0").unwrap());
        assert!(is_update_available("v0.2.0", "0.2.1").unwrap());
    }

    #[test]
    fn same_or_older_version_is_ignored() {
        assert!(!is_update_available("0.2.0", "0.2.0").unwrap());
        assert!(!is_update_available("0.2.1", "0.2.0").unwrap());
        assert!(!is_update_available("0.10.0", "0.9.9").unwrap());
    }

    #[test]
    fn invalid_version_is_a_clean_error() {
        let err = is_update_available("0.2.0", "不是版本号").unwrap_err();
        assert!(err.to_string().contains("无法解析版本号"));
    }
}
