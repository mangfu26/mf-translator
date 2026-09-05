//! 全局共享状态（Tauri managed state）。

use std::sync::{Arc, Mutex};

use crate::config::{AppConfig, ConfigStore};
use crate::error::AppError;
use crate::history::HistoryDb;
use crate::prompt_store::PromptStore;
use crate::translation::TaskRegistry;

pub struct AppState {
    pub client: reqwest::Client,
    pub config: Arc<Mutex<AppConfig>>,
    pub config_store: Arc<ConfigStore>,
    pub tasks: Arc<TaskRegistry>,
    pub history: Arc<Mutex<HistoryDb>>,
    pub prompt_store: Arc<Mutex<PromptStore>>,
}

pub fn lock_poisoned() -> AppError {
    AppError::Config("内部状态异常".to_string())
}
