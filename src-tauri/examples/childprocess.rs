use std::os::windows::process::CommandExt;
use std::process::Command;
use winapi::um::winbase::CREATE_NO_WINDOW;

fn main() {
    let path = std::path::Path::new("static").join("chromedriver.exe");
    let mut child = std::process::Command::new(path)
        .arg("--port=4444")
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .expect("Failed to start chromedriver");

    let pid = dbg!(child.id());
    std::thread::sleep(std::time::Duration::from_secs(10));

    child.kill().expect("Failed to kill child process");
}

// Get-Process -Id
