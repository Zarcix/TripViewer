use std::sync::OnceLock;

pub const SERVER_PATH: &str = "/home/personal/Pictures/Sample Photos";
// pub const API_KEY: Option<&str> = option_env!("API_KEY");
pub static API_KEY: OnceLock<String> = OnceLock::new();
