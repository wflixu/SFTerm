mod cmds;

use sfterm_core::Manager;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Arc::new(Manager::new()))
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmds::ssh_connect,
            cmds::sftp_list_dir,
            cmds::ssh_disconnect,
            cmds::ssh_sessions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
