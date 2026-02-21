use std::sync::OnceLock;


pub static SERVER_PATH: OnceLock<String> = OnceLock::new();
pub static API_KEY: OnceLock<String> = OnceLock::new();
