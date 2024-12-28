use super::*;
use std::os::windows::process::CommandExt;
use std::process::Child;
use std::sync::{Arc, Mutex};
use winapi::um::winbase::CREATE_NO_WINDOW;

static CHROMEDRIVER_HANDLE: once_cell::sync::Lazy<Arc<Mutex<Option<Child>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

fn is_chromedriver_running() -> bool {
    std::net::TcpStream::connect("127.0.0.1:4444").is_ok()
}

pub fn start_chromedriver() {
    // 其实下面的代码杀不掉driver进程;这里判断如果已经启动了就不再启动了 用之前的僵尸进程
    if is_chromedriver_running() {
        dbg!("will not restart chrome driver");
        return;
    }

    dbg!("start chrome driver");
    let path = std::path::Path::new("static").join("chromedriver.exe");
    let mut child = std::process::Command::new(path)
        .arg("--port=4444")
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .expect("Failed to start chromedriver");

    child.wait().expect("Failed to wait on chromedriver");

    *CHROMEDRIVER_HANDLE.lock().unwrap() = Some(child);
}

pub fn stop_chromedriver() {
    if let Some(mut child) = CHROMEDRIVER_HANDLE.lock().unwrap().take() {
        let r = child.kill();
        log::trace!("{:?}", r);
        let _ = child.wait();
        log::info!("Chromedriver stopped.");
    }
}

#[test]
fn test_start_chromedriver() {
    let path = std::path::Path::new("static").join("chromedriver.exe");
    let mut child = std::process::Command::new(path)
        .arg("--port=4444")
        .arg("--disable-ipv6")
        .spawn()
        .expect("Failed to start chromedriver");

    // child.wait().expect("Failed to wait on chromedriver");

    let r = child.kill();
    log::trace!("{:?}", r);
}

#[test]
fn test_restart_chromedriver() {
    let path = std::path::Path::new("static").join("chromedriver.exe");
    let mut child = std::process::Command::new(path)
        .arg("--port=4444")
        .spawn()
        .expect("Failed to start chromedriver");

    log::trace!("{}", std::net::TcpStream::connect("127.0.0.1:4444").is_ok());
}
