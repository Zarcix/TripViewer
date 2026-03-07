use rocket::http::ContentType;
use rocket::serde::{Serialize, Deserialize};
use rocket::fs::NamedFile;
use rocket::serde::json::Json;

#[derive(Debug)]
pub struct StreamedFile {
    pub file: rocket::tokio::fs::File,
    pub size: u64,
    pub range: (u64, u64),
    pub content_type: ContentType
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
    pub path: String,
    pub entries: Vec<DirectoryEntry>
}

#[derive(Debug)]
pub enum FileServerResponse {
    FullContent(NamedFile),
    RangedContent(StreamedFile),
    DirectoryListing(Json<DirectoryListing>),
}
