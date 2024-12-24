pub mod book;
pub use book::*;

pub mod notify;
pub use notify::*;

pub mod config;
pub use config::*;

pub mod driver;
pub use driver::*;

pub mod logger;
pub use logger::*;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
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
                .build()
        )
        .setup(|app| {
            std::thread::spawn(|| {
                start_chromedriver();
            });
            Ok(())
        })
        .on_window_event(move |windeow,event| match event {
            tauri::WindowEvent::Destroyed => {
                stop_chromedriver();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![book::activate])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

