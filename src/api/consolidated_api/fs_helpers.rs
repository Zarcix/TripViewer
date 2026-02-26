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

pub async fn parse_directory(
    full_path: &Path,
    request_path: &Path,
) -> Result<FileServerResponse, Status> {
    let internal_error = |_| Status::InternalServerError;

    let mut entries = Vec::new();

    let mut dir = tokio::fs::read_dir(full_path)
        .await
        .map_err(internal_error)?;

    while let Ok(Some(entry)) = dir.next_entry().await.map_err(internal_error) {
        let metadata = entry
            .metadata()
            .await
            .map_err(internal_error)?;

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
    })
}

pub async fn rename_entry(old_path: &PathBuf, new_path: &PathBuf) -> Result<(), Status> {
    tokio::fs::rename(old_path, new_path).await.map_err(|e| {
        error!("Failed to rename {} to {}, {}",
            old_path.display(),
            new_path.display(),
            e
        );
        Status::InternalServerError
    })
}