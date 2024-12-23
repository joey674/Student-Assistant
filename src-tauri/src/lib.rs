pub mod book;
pub use book::*;

pub mod notify;
pub use notify::*;

pub mod config;
pub use config::*;

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

fn start_chromedriver() {
    use std::process::Command;
    use std::path::Path;
    let path = Path::new("..");
    let path = path.join("static");
    let path = path.join("chromedriver.exe");
    let mut child = Command::new(path)
        .arg("--port=4444")
        .spawn()
        .expect("Failed to start chromedriver");

    child.wait().expect("Failed to wait on chromedriver");
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let _ = std::thread::spawn(move || {
                start_chromedriver();
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![activate])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
