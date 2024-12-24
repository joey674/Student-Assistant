
// use std::io::Write;
// use tauri::Window;
// use tracing_subscriber::fmt::MakeWriter;
// use tauri::Emitter;

// pub struct TauriLogger {
//     window: Window,
// }

// impl TauriLogger {
//     pub fn new(window: Window) -> Self {
//         TauriLogger { window }
//     }
// }

// impl Write for TauriLogger {
//     fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
//         // 将日志内容转换为字符串
//         if let Ok(log) = std::str::from_utf8(buf) {
//             // 发送日志到前端
//             if let Err(e) = self.window.emit("log", log.to_string()) {
//                 eprintln!("Failed to emit log: {}", e);
//             }
//         }
//         Ok(buf.len())
//     }

//     fn flush(&mut self) -> std::io::Result<()> {
//         Ok(())
//     }
// }


// pub fn init_logger(window: tauri::Window) {
//     let logger = TauriLogger::new(window);

//     tracing_subscriber::fmt()
//     .with_writer(logger)
//     .event_format(
//         tracing_subscriber::fmt::format()
//             .with_file(true)
//             .with_line_number(true)
//     )
//     .init();
// }


