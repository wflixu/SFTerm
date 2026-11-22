mod cmds;

use sfterm_core::Manager;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Arc::new(Manager::new()))
        .setup(|app| {
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
            cmds::ssh_sessions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
