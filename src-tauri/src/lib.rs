pub mod book;
pub use book::*;

pub mod notify;
pub use notify::*;

pub mod config;
pub use config::*;

pub mod driver;
pub use driver::*;

#[tauri::command]
async fn activate(email: String) -> anyhow::Result<String,String> {
    match book_abholung_aufenthaltserlaubnis(&email).await {
        Ok(_) =>{
            return Ok(format!("{}, Successful",email.to_string()));
        },
        Err(e) =>{
            return Err(format!("{}, Fail: {}", email, e.to_string()))
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
        .invoke_handler(tauri::generate_handler![activate])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

