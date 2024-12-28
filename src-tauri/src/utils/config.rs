use once_cell::sync::Lazy;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub static CONFIG: Lazy<Value> = Lazy::new(|| load());

pub fn load() -> Value {
    let path = Path::new("static").join("config.json");
    let json_content = fs::read_to_string(path).unwrap();
    serde_json::from_str(&json_content).unwrap()
}

#[test]
pub fn test() {
    use std::env;
    let current_dir = env::current_dir().unwrap();
    log::trace!("{:?}", current_dir);
    let path = Path::new("..");
    let path = path.join("config.json");
    let json_content = fs::read_to_string(path).unwrap();

    let json_data: Value = serde_json::from_str(&json_content).unwrap();

    if let Some(item) = json_data.get("host_email_account") {
        log::trace!("{}", item);
    }
}
