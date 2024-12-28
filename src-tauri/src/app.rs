use super::*;
use std::collections::HashMap;
use std::fs;
use std::os::windows::process::CommandExt;
use std::{
    process::Child,
    sync::{Arc, Mutex},
};
use tokio::sync::OnceCell;
use winapi::um::winbase::CREATE_NO_WINDOW;

#[derive(Debug, Clone)]
pub enum CommandStatus {
    Book {
        user_info: UserInfo,
        appointment_type: AppointmentType,
    },
    UndefinedCommand {
        user_info: UserInfo,
    },
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub vorname: String,
    pub nachname: String,
    pub email: String,
    pub telefonnummer: String,
    pub geburtsdatum: [u64; 3], /* day, month,year */
}

#[derive(Debug)]
pub struct App {
    app_handle: tauri::AppHandle,
    driver_handle: Arc<Mutex<Child>>,
    command_list: Arc<Mutex<HashMap<uuid::Uuid, CommandStatus>>>,
    config: serde_json::Value,
}

impl App {
    /// 初始化服务
    ///
    ///
    pub fn init(app_handle: tauri::AppHandle) -> Self {
        /* 初始化chromedriver */
        let path = std::path::Path::new("static").join("chromedriver.exe");
        let child = std::process::Command::new(path)
            .arg("--port=4444")
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .expect("Failed to start chromedriver");

        /* 初始化配置文件 */
        let path = std::path::Path::new("static").join("config.json");
        let json_content = fs::read_to_string(path).unwrap();

        App {
            app_handle: app_handle,
            driver_handle: Arc::new(Mutex::new(child)),
            command_list: Arc::new(Mutex::new(HashMap::new())),
            config: serde_json::from_str(&json_content).unwrap(),
        }
    }

    /// 关闭服务
    ///
    ///
    pub fn stop(&self) -> anyhow::Result<()> {
        /* 关闭chromedriver */
        let mut child = self.driver_handle.lock().unwrap();
        (*child).kill()?;

        Ok(())
    }

    /// 注册服务指令;
    /// 每当前端调用一次tauri::command,都需要记录指令状态,这样全局只需要传递uid就可以实现数据传递
    ///
    pub fn add_command(&self, command_status: CommandStatus) -> uuid::Uuid {
        let command_id = uuid::Uuid::new_v4(); /* 为一次预定生成一个特定的预定id */
        let mut list = self.command_list.lock().unwrap();
        (*list).insert(command_id, command_status);
        command_id
    }

    /// 获取服务指令状态
    ///
    ///
    pub fn get_command_status(&self, command_id: uuid::Uuid) -> anyhow::Result<CommandStatus> {
        let list = self.command_list.lock().unwrap();
        list.get(&command_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("command not found"))
    }

    /// 更新服务指令状态
    ///
    ///
    pub fn update_command_status(&self, command_id: uuid::Uuid, command_status: CommandStatus) {
        let mut list = self.command_list.lock().unwrap();
        list.entry(command_id).and_modify(|s| *s = command_status);
    }

    /// 获取配置文件中配置参数
    /// 输入key值, 返回json文件中对应的数据结构
    /// 注: 现在的配置文件暂时只支持 (k,v) = (&str,&str)
    pub fn get_config_value(&self, key: &str) -> &str {
        dbg!(key);
        self.config.get(key).unwrap().as_str().unwrap()
    }
}

static APP_INS: OnceCell<App> = OnceCell::const_new();

/// 初始化app
/// 只在tauri的setup时使用;
///
pub fn init_app_ins(app_handle: tauri::AppHandle) -> anyhow::Result<()> {
    APP_INS.set(App::init(app_handle))?;
    Ok(())
}

/// 获取app
/// 全局可用
///
pub fn get_app_ins() -> anyhow::Result<&'static App> {
    APP_INS
        .get()
        .ok_or_else(|| anyhow::anyhow!("Error: app not initialized yet"))
}
