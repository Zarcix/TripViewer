use std::sync::OnceLock;

pub static STAGING_PATH: OnceLock<String> = OnceLock::new();
pub static SERVE_PATH: OnceLock<String> = OnceLock::new();
pub static API_KEY: OnceLock<String> = OnceLock::new();

#[cfg(not(test))]
pub const UPLOAD_LIMIT_MB: i16 = 5000;

#[cfg(test)]
pub const UPLOAD_LIMIT_MB: i16 = 1;

pub const STAGING_NAME: &str = "file.part";