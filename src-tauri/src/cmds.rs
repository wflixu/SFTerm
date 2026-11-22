use crate::db::{Db, HistoryEntry};
use base64::Engine;
use sfterm_core::{
    open_shell, ConnectInfo, ConnectParams, Manager, RemoteEntry, ShellHandle, TransferProgress,
};
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

/// 活跃的终端句柄。
#[derive(Default)]
pub struct Shells(Mutex<HashMap<String, ShellHandle>>);

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

#[derive(Clone, serde::Serialize)]
struct ShellEvent {
    id: String,
    data: String,
}

/// 打开一个交互式终端。
#[tauri::command]
pub async fn shell_open(
    app: AppHandle,
    shells: State<'_, Arc<Shells>>,
    params: ConnectParams,
    shell_id: String,
    cols: u32,
    rows: u32,
) -> Result<(), String> {
    let app_handle = app.clone();
    let event_id = shell_id.clone();

    let handle = tauri::async_runtime::spawn_blocking(move || {
        open_shell(params, cols, rows, move |bytes| {
            let data = base64::engine::general_purpose::STANDARD.encode(bytes);
            let _ = app_handle.emit(
                "shell-output",
                ShellEvent {
                    id: event_id.clone(),
                    data,
                },
            );
        })
    })
    .await
    .map_err(|e| format!("终端启动失败：{e}"))??;

    shells
        .0
        .lock()
        .map_err(|_| "终端表被污染".to_string())?
        .insert(shell_id, handle);
    Ok(())
}

/// 向终端写入按键（base64）。
#[tauri::command]
pub fn shell_write(
    shells: State<'_, Arc<Shells>>,
    shell_id: String,
    data: String,
) -> Result<(), String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| format!("终端输入解码失败：{e}"))?;
    let map = shells.0.lock().map_err(|_| "终端表被污染".to_string())?;
    let handle = map.get(&shell_id).ok_or_else(|| "终端不存在".to_string())?;
    handle.write(bytes)
}

/// 调整终端尺寸。
#[tauri::command]
pub fn shell_resize(
    shells: State<'_, Arc<Shells>>,
    shell_id: String,
    cols: u32,
    rows: u32,
) -> Result<(), String> {
    let map = shells.0.lock().map_err(|_| "终端表被污染".to_string())?;
    let handle = map.get(&shell_id).ok_or_else(|| "终端不存在".to_string())?;
    handle.resize(cols, rows)
}

/// 关闭终端。
#[tauri::command]
pub fn shell_close(shells: State<'_, Arc<Shells>>, shell_id: String) -> bool {
    if let Ok(mut map) = shells.0.lock() {
        if let Some(handle) = map.remove(&shell_id) {
            handle.close();
            return true;
        }
    }
    false
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

// ---------------------------------------------------------------------------
// 系统钥匙串：密码、私钥文本、口令都放这里，不落 localStorage。
// ---------------------------------------------------------------------------

/// macOS 钥匙串里的 service 名；account 由前端按用途拼接（host:<id>:password 等）。
const KEYCHAIN_SERVICE: &str = "cn.wflixu.sfterm";

fn keychain_entry(account: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYCHAIN_SERVICE, account).map_err(|e| format!("打开钥匙串失败：{e}"))
}

/// 写入或覆盖一条凭据。
#[tauri::command]
pub fn secret_set(account: String, value: String) -> Result<(), String> {
    keychain_entry(&account)?
        .set_password(&value)
        .map_err(|e| format!("写入钥匙串失败：{e}"))
}

/// 读取一条凭据；不存在时返回 null。
#[tauri::command]
pub fn secret_get(account: String) -> Result<Option<String>, String> {
    match keychain_entry(&account)?.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("读取钥匙串失败：{e}")),
    }
}

/// 删除一条凭据；不存在时视为成功。
#[tauri::command]
pub fn secret_delete(account: String) -> Result<(), String> {
    match keychain_entry(&account)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("删除钥匙串条目失败：{e}")),
    }
}

// ---------------------------------------------------------------------------
// SQLite：主机/密钥元数据用 JSON 存在 state 表，命令历史单独一张表。
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn db_get_state(db: State<'_, Db>, key: String) -> Result<Option<String>, String> {
    db.get_state(&key)
}

#[tauri::command]
pub fn db_set_state(db: State<'_, Db>, key: String, value: String) -> Result<(), String> {
    db.set_state(&key, &value)
}

#[tauri::command]
pub fn history_add(
    db: State<'_, Db>,
    host_id: String,
    host_label: String,
    command: String,
    executed_at: i64,
) -> Result<i64, String> {
    db.add_history(&host_id, &host_label, &command, executed_at)
}

#[tauri::command]
pub fn history_list(
    db: State<'_, Db>,
    query: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<HistoryEntry>, String> {
    let query = query.map(|q| q.trim().to_string()).filter(|q| !q.is_empty());
    db.list_history(query.as_deref(), limit.unwrap_or(200))
}

#[tauri::command]
pub fn history_clear(db: State<'_, Db>) -> Result<(), String> {
    db.clear_history()
}
