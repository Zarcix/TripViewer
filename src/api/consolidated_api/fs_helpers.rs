use std::path::{Path, PathBuf};
use rocket::tokio;

use rocket::{fs::NamedFile, http::Status, serde::json::Json};

use crate::api::consolidated_api::models::StreamedFile;

use super::models::{FileServerResponse, DirectoryEntry, DirectoryListing};

 const MEDIA_EXTS: &[&str] = &[
    "mp4", "m4v", "mov",
    "mpeg", "mpg",
    "webm", "avi",
];

pub fn parse_directory(
    full_path: &Path,
    request_path: &Path,
) -> Result<FileServerResponse, Status> {

    let mut entries = Vec::new();

    for entry in std::fs::read_dir(full_path)
        .map_err(|_| Status::InternalServerError)?
    {
        let entry = entry.map_err(|_| Status::InternalServerError)?;
        let metadata = entry.metadata()
            .map_err(|_| Status::InternalServerError)?;

        entries.push(DirectoryEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            is_dir: metadata.is_dir(),
        });
    }

    entries.sort_by_key(|e| (!e.is_dir, e.name.clone()));

    let listing = DirectoryListing {
        path: request_path.to_string_lossy().into_owned(),
        entries,
    };

    Ok(FileServerResponse::DirectoryListing(Json(listing)))
}

pub async fn parse_file(
    full_path: PathBuf,
) -> Result<FileServerResponse, Status> {

    let file_ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if MEDIA_EXTS.contains(&file_ext) {
        let stream_file = StreamedFile {
            file: full_path
        };
        return Ok(FileServerResponse::RangedContent(stream_file));
    }

    let file = NamedFile::open(full_path).await.map_err(|_| Status::NotFound)?;

    Ok(FileServerResponse::FullContent(file))
}

pub async fn create_dir(photoset_path: &PathBuf) -> Result<(), Status>{
    tokio::fs::create_dir_all(photoset_path).await.map_err(|e| {
        error!(
            "Could not create photoset tree {}: {}",
            photoset_path.display(),
            e
        );
        match e.kind() {
            std::io::ErrorKind::PermissionDenied => Status::Forbidden,
            _ => Status::InternalServerError,
        }
    })?;

    Ok(())
}