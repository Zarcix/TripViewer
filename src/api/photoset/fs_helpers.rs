use std::path::{Path, PathBuf};
use rocket::data::ToByteUnit;
use rocket::tokio::io::AsyncWriteExt;
use rocket::{Data, tokio};

use rocket::{fs::NamedFile, http::Status, serde::json::Json};

use crate::constants::server_constants::{
    STAGING_NAME, STAGING_PATH, UPLOAD_LIMIT_MB
};

use super::models::{FileServerResponse, DirectoryEntry, DirectoryListing};
use super::models::StreamedFile;

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
            std::io::ErrorKind::NotADirectory => Status::BadRequest,
            _ => Status::InternalServerError,
        }
    })
}

pub async fn rename_entry(old_path: &PathBuf, new_path: &PathBuf) -> Result<(), Status> {
    if tokio::fs::metadata(&new_path).await.is_ok() {
        error!("New path exists. old_path={}, new_path={}", old_path.display(), new_path.display());
        return Err(Status::Conflict);
    }

    tokio::fs::rename(old_path, new_path).await.map_err(|e| {
        error!("Failed to rename {} to {}, {}",
            old_path.display(),
            new_path.display(),
            e
        );
        Status::InternalServerError
    })
}

pub async fn save_data(data: Data<'_>, target_path: &PathBuf) -> Result<(), Status> {
    // Validation

    if tokio::fs::metadata(target_path).await.is_ok() {
        error!("Target file already exists. target_path={}", target_path.display());
        return Err(Status::Conflict);
    }

    // Create temp pathing
    let mut staging_path = PathBuf::new();
    staging_path.push(&STAGING_PATH
        .get()
        .ok_or_else(|| {
            error!("Could not get staging path value. staging_path={}", staging_path.display());
            Status::InternalServerError
        })?
    );
    // Parent folder must exist
    staging_path.canonicalize().map_err(|e| {
        error!("Could not canonicalize staging path. staging_path={}, error={}", staging_path.display(), e);
        Status::InternalServerError
    })?;
    staging_path.push(&STAGING_NAME);

    // Upload File to Temp Folder and check for errors
    let mut upload_file = tokio::fs::File::create(&staging_path)
        .await
        .map_err(|e| {
            error!("Could not create staging file. error={}", e);
            Status::InternalServerError
        })?;

    // Check Upload Size
    let stream = data.open(UPLOAD_LIMIT_MB.mebibytes());
    let stream_res = stream.stream_to(&mut upload_file).await.map_err(|_| Status::InternalServerError)?;

    if !stream_res.complete {
        let _ = tokio::fs::remove_file(&staging_path).await;
        error!("Data stream too large to read.");
        return Err(Status::PayloadTooLarge);
    }

    if let Err(_) = upload_file.flush().await {
        let _ = tokio::fs::remove_file(&staging_path).await;
        return Err(Status::InternalServerError);
    }

    drop(upload_file);

    // After check is done, move file to target path
    if let Err(e) = tokio::fs::copy(&staging_path, &target_path).await {
        let _ = tokio::fs::remove_file(&staging_path).await;
        let _ = tokio::fs::remove_file(&target_path).await;
        error!("Could not copy file {} to {}, error={}", staging_path.display(), target_path.display(), e);
        return Err(Status::InternalServerError);
    }

    if let Err(e) = tokio::fs::remove_file(&staging_path).await {
        // No error return here since the copy was successful. This might have just left some artifacts
        error!("Could not remove file {}, error={}", staging_path.display(), e);
    }

    Ok(())
}

pub async fn remove_photoset_dir(target_dir: &PathBuf, forced: bool) -> Result<(), Status> {
    let error_handler = |e: tokio::io::Error| {
        error!("Failed to remove photoset directory. target_dir={}, forced={}, error={}", target_dir.display(), forced, e);
        match e.kind() {
            std::io::ErrorKind::DirectoryNotEmpty => Status::Conflict,
            std::io::ErrorKind::NotFound => Status::NotFound,
            std::io::ErrorKind::PermissionDenied => Status::Forbidden,
            _ => Status::InternalServerError,
        }
    };

    if forced {
        tokio::fs::remove_dir_all(&target_dir).await.map_err(error_handler)
    } else {
        tokio::fs::remove_dir(&target_dir).await.map_err(error_handler)
    }
}

pub async fn remove_photoset_file(target_file: &PathBuf) -> Result<(), Status> {
    tokio::fs::remove_file(target_file).await.map_err(|e| {
        error!("Failed to remove photoset file. target_dir={}, error={}", target_file.display(), e);
        match e.kind() {
            std::io::ErrorKind::NotFound => Status::NotFound,
            std::io::ErrorKind::PermissionDenied => Status::Forbidden,
            std::io::ErrorKind::IsADirectory => Status::BadRequest,
            std::io::ErrorKind::InvalidInput => Status::BadRequest,
            _ => Status::InternalServerError,
        }
    })
}