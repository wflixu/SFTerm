use sfterm_core::{ConnectInfo, ConnectParams, Manager, RemoteEntry};
use std::sync::Arc;
use tauri::State;

/// 建立 SSH 连接并打开 SFTP 会话。
#[tauri::command]
pub async fn ssh_connect(
    state: State<'_, Arc<Manager>>,
    params: ConnectParams,
) -> Result<ConnectInfo, String> {
    let manager = Arc::clone(&state);
    tauri::async_runtime::spawn_blocking(move || manager.connect(params))
        .await
        .map_err(|e| format!("连接任务失败：{e}"))?
}

/// 列出远端目录。
#[tauri::command]
pub async fn sftp_list_dir(
    state: State<'_, Arc<Manager>>,
    session_id: String,
    path: String,
) -> Result<Vec<RemoteEntry>, String> {
    let manager = Arc::clone(&state);
    tauri::async_runtime::spawn_blocking(move || manager.list_dir(&session_id, &path))
        .await
        .map_err(|e| format!("列目录任务失败：{e}"))?
}

/// 断开并释放会话。
#[tauri::command]
pub fn ssh_disconnect(state: State<'_, Arc<Manager>>, session_id: String) -> bool {
    state.disconnect(&session_id)
}

/// 当前活跃会话（调试用）。
#[tauri::command]
pub fn ssh_sessions(state: State<'_, Arc<Manager>>) -> Vec<String> {
    state.session_ids()
}
