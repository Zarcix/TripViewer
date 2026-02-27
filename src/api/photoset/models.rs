use std::path::PathBuf;

use rocket::serde::{Serialize, Deserialize};
use rocket::fs::NamedFile;
use rocket::serde::json::Json;

pub struct RangeHeader {
    pub start: u64,
    pub end: Option<u64>,
}

#[derive(Debug)]
pub struct StreamedFile {
    pub file: PathBuf
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DirectoryEntry {
    pub name: String,
    pub is_dir: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DirectoryListing {
    pub path: String,
    pub entries: Vec<DirectoryEntry>
}

pub enum FileServerResponse {
    FullContent(NamedFile),
    RangedContent(StreamedFile),
    DirectoryListing(Json<DirectoryListing>),
}
