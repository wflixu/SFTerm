use sfterm_core::{ConnectInfo, ConnectParams, Manager, RemoteEntry, TransferProgress};
use std::sync::mpsc;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

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

/// 远端会话的初始目录（登录用户主目录）。
#[tauri::command]
pub async fn sftp_home(
    state: State<'_, Arc<Manager>>,
    session_id: String,
) -> Result<String, String> {
    let manager = Arc::clone(&state);
    tauri::async_runtime::spawn_blocking(move || manager.remote_home(&session_id))
        .await
        .map_err(|e| format!("获取主目录任务失败：{e}"))?
}

/// 列出本地目录。
#[tauri::command]
pub async fn fs_list_local(path: String) -> Result<Vec<RemoteEntry>, String> {
    tauri::async_runtime::spawn_blocking(move || sfterm_core::list_local(&path))
        .await
        .map_err(|e| format!("列目录任务失败：{e}"))?
}

/// 当前用户主目录。
#[tauri::command]
pub fn fs_home() -> String {
    sfterm_core::home_dir()
}

#[derive(Clone, serde::Serialize)]
struct TransferEvent {
    id: String,
    done: u64,
    total: u64,
}

/// 上传单个文件，进度通过 `transfer-progress` 事件回传。
#[tauri::command]
pub async fn sftp_upload(
    app: AppHandle,
    state: State<'_, Arc<Manager>>,
    id: String,
    session_id: String,
    local_path: String,
    remote_path: String,
    overwrite: bool,
) -> Result<(), String> {
    let manager = Arc::clone(&state);

    tauri::async_runtime::spawn_blocking(move || {
        let (progress_tx, progress_rx) = mpsc::channel::<TransferProgress>();
        let worker = std::thread::spawn(move || {
            manager.upload(&session_id, local_path, remote_path, overwrite, progress_tx)
        });

        for progress in progress_rx {
            let _ = app.emit(
                "transfer-progress",
                TransferEvent {
                    id: id.clone(),
                    done: progress.done,
                    total: progress.total,
                },
            );
        }

        worker
            .join()
            .map_err(|_| "传输线程崩溃".to_string())?
    })
    .await
    .map_err(|e| format!("上传任务失败：{e}"))?
}

/// 下载单个文件，进度通过 `transfer-progress` 事件回传。
#[tauri::command]
pub async fn sftp_download(
    app: AppHandle,
    state: State<'_, Arc<Manager>>,
    id: String,
    session_id: String,
    remote_path: String,
    local_path: String,
    overwrite: bool,
) -> Result<(), String> {
    let manager = Arc::clone(&state);

    tauri::async_runtime::spawn_blocking(move || {
        let (progress_tx, progress_rx) = mpsc::channel::<TransferProgress>();
        let worker = std::thread::spawn(move || {
            manager.download(&session_id, remote_path, local_path, overwrite, progress_tx)
        });

        for progress in progress_rx {
            let _ = app.emit(
                "transfer-progress",
                TransferEvent {
                    id: id.clone(),
                    done: progress.done,
                    total: progress.total,
                },
            );
        }

        worker
            .join()
            .map_err(|_| "传输线程崩溃".to_string())?
    })
    .await
    .map_err(|e| format!("下载任务失败：{e}"))?
}

/// 新建远端目录。
#[tauri::command]
pub async fn sftp_mkdir(
    state: State<'_, Arc<Manager>>,
    session_id: String,
    path: String,
) -> Result<(), String> {
    let manager = Arc::clone(&state);
    tauri::async_runtime::spawn_blocking(move || manager.mkdir(&session_id, path))
        .await
        .map_err(|e| format!("新建文件夹任务失败：{e}"))?
}

/// 重命名远端文件。
#[tauri::command]
pub async fn sftp_rename(
    state: State<'_, Arc<Manager>>,
    session_id: String,
    from: String,
    to: String,
) -> Result<(), String> {
    let manager = Arc::clone(&state);
    tauri::async_runtime::spawn_blocking(move || manager.rename(&session_id, from, to))
        .await
        .map_err(|e| format!("重命名任务失败：{e}"))?
}

/// 删除远端文件或目录。
#[tauri::command]
pub async fn sftp_remove(
    state: State<'_, Arc<Manager>>,
    session_id: String,
    path: String,
    is_dir: bool,
) -> Result<(), String> {
    let manager = Arc::clone(&state);
    tauri::async_runtime::spawn_blocking(move || manager.remove(&session_id, path, is_dir))
        .await
        .map_err(|e| format!("删除任务失败：{e}"))?
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
