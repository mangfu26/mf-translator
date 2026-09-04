use tauri::State;

use crate::error::AppResult;
use crate::history::HistoryItem;
use crate::state::{lock_poisoned, AppState};

#[tauri::command]
pub fn list_history(
    query: Option<String>,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> AppResult<Vec<HistoryItem>> {
    let db = state.history.lock().map_err(|_| lock_poisoned())?;
    db.list(query.as_deref(), limit.unwrap_or(200))
}

#[tauri::command]
pub fn delete_history_item(id: i64, state: State<'_, AppState>) -> AppResult<()> {
    let db = state.history.lock().map_err(|_| lock_poisoned())?;
    db.delete(id)
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> AppResult<()> {
    let db = state.history.lock().map_err(|_| lock_poisoned())?;
    db.clear()
}
