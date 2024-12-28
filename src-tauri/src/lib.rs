pub mod book;
pub use book::*;

pub mod notify;
pub use notify::*;

pub mod utils;
use tauri::Manager;
pub use utils::*;

pub mod app;
pub use app::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            /* 日志系统 */
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Webview,
                ))
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{}][{}:{}] {:?}  {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.file().unwrap_or("unknown"),
                        record.line().unwrap_or(0),
                        record.level(),
                        message
                    ))
                })
                .build(),
        )
        .setup(|app| {
            dbg!("setup");
            init_app_ins(app.app_handle().clone())?;
            Ok(())
        })
        .on_window_event(move |windeow, event| match event {
            tauri::WindowEvent::Destroyed => {
                dbg!("stop");
                let _ = get_app_ins().unwrap().stop();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![book::book])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
