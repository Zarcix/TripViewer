use std::path::PathBuf;

use rocket::fs::NamedFile;
use rocket::http::ContentType;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::fs::File;

use crate::api::request_guards::RangeHeader;

#[derive(Debug)]
pub struct StreamedFile {
    pub file: File,
    pub size: u64,
    pub range: (u64, u64),
    pub content_type: ContentType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(crate = "rocket::serde")]
pub struct DirectoryEntry {
    pub name: String,
    pub is_dir: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct DirectoryListing {
    pub entries: Vec<DirectoryEntry>,
}

#[derive(Debug)]
pub enum FileServerResponse {
    FullContent(NamedFile),
    RangedContent(StreamedFile),
    DirectoryListing(Json<DirectoryListing>),
}

#[derive(Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub kind: FileType,
    pub range: Option<RangeHeader>,
}

#[derive(Debug)]
pub enum FileType {
    File,
    Media,
    Directory,
}
