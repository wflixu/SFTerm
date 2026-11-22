mod cmds;
mod db;

use sfterm_core::Manager;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Arc::new(Manager::new()))
        .manage(Arc::new(cmds::Shells::default()))
        .setup(|app| {
            use tauri::Manager;

            let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("创建应用数据目录失败：{e}"))?;
            app.manage(db::Db::open(&dir.join("sfterm.db"))?);

            if cfg!(debug_assertions) {
                // 日志插件只是开发便利，初始化失败（例如受限环境没有写权限）
                // 不应该阻止应用启动。
                let result = app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                );
                if let Err(error) = result {
                    eprintln!("⚠️ 日志插件初始化失败（不影响启动）：{error}");
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmds::ssh_connect,
            cmds::sftp_list_dir,
            cmds::sftp_home,
            cmds::sftp_upload,
            cmds::sftp_download,
            cmds::sftp_mkdir,
            cmds::sftp_rename,
            cmds::sftp_remove,
            cmds::fs_list_local,
            cmds::fs_home,
            cmds::ssh_disconnect,
            cmds::shell_open,
            cmds::shell_write,
            cmds::shell_resize,
            cmds::shell_close,
            cmds::ssh_sessions,
            cmds::secret_set,
            cmds::secret_get,
            cmds::secret_delete,
            cmds::db_get_state,
            cmds::db_set_state,
            cmds::history_add,
            cmds::history_list,
            cmds::history_clear,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
