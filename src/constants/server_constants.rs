use std::sync::OnceLock;


pub static SERVE_PATH: OnceLock<String> = OnceLock::new();
pub static API_KEY: OnceLock<String> = OnceLock::new();