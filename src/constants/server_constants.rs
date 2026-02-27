use std::sync::OnceLock;

pub static STAGING_PATH: OnceLock<String> = OnceLock::new();
pub static SERVE_PATH: OnceLock<String> = OnceLock::new();
pub static API_KEY: OnceLock<String> = OnceLock::new();

pub const UPLOAD_LIMIT_MB: i16 = 5000;
pub const STAGING_NAME: &str = "file.part";