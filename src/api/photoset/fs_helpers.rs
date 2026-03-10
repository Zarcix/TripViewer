use nanoid::nanoid;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};

use rocket::data::ToByteUnit;
use rocket::http::ContentType;
use rocket::tokio::io::{AsyncSeekExt, AsyncWriteExt};
use rocket::{fs::NamedFile, http::Status, serde::json::Json};
use rocket::{tokio, Data};

use crate::api::request_guards::RangeHeader;
use crate::constants::server_constants::{STAGING_PATH, UPLOAD_LIMIT_MB};

use super::models::StreamedFile;
use super::models::{DirectoryEntry, DirectoryListing, FileServerResponse};

pub async fn parse_directory(full_path: &Path) -> Result<FileServerResponse, Status> {
    let internal_error = |_| Status::InternalServerError;

    let mut entries = Vec::new();

    let mut dir = tokio::fs::read_dir(full_path)
        .await
        .map_err(internal_error)?;

    while let Ok(Some(entry)) = dir.next_entry().await.map_err(internal_error) {
        let metadata = entry.metadata().await.map_err(internal_error)?;

        entries.push(DirectoryEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            is_dir: metadata.is_dir(),
        });
    }

    entries.sort_by_key(|e| (!e.is_dir, e.name.clone()));

    let listing = DirectoryListing { entries };

    Ok(FileServerResponse::DirectoryListing(Json(listing)))
}

pub async fn parse_streamed_file(
    full_path: &PathBuf,
    range: &Option<RangeHeader>,
) -> Result<FileServerResponse, Status> {
    let mut file = rocket::tokio::fs::File::open(&full_path)
        .await
        .map_err(|_| Status::NotFound)?;

    let size = file
        .metadata()
        .await
        .map_err(|_| Status::InternalServerError)?
        .len();

    let file_ext = &full_path.extension().unwrap_or_default().to_string_lossy();

    let content_type = ContentType::parse_flexible(file_ext).unwrap_or(ContentType::MP4);

    let (start, end) = match range {
        Some(r) => match r.resolve(size) {
            Some(resolved) => resolved,
            None => return Err(Status::RangeNotSatisfiable),
        },
        None => (0, size - 1),
    };

    file.seek(SeekFrom::Start(start)).await.unwrap();

    let streamed_file = StreamedFile {
        file,
        size,
        content_type,
        range: (start, end),
    };

    Ok(FileServerResponse::RangedContent(streamed_file))
}

pub async fn parse_file(full_path: &Path) -> Result<FileServerResponse, Status> {
    let file = NamedFile::open(&full_path)
        .await
        .map_err(|_| Status::NotFound)?;
    Ok(FileServerResponse::FullContent(file))
}

pub async fn create_dir(photoset_path: &PathBuf) -> Result<(), Status> {
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
        error!(
            "New path exists. old_path={}, new_path={}",
            old_path.display(),
            new_path.display()
        );
        return Err(Status::Conflict);
    }

    tokio::fs::rename(old_path, new_path).await.map_err(|e| {
        error!(
            "Failed to rename {} to {}, {}",
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
        error!(
            "Target file already exists. target_path={}",
            target_path.display()
        );
        return Err(Status::Conflict);
    }

    // Create temp pathing
    let mut staging_path = PathBuf::new();
    staging_path.push(STAGING_PATH.get().ok_or_else(|| {
        error!(
            "Could not get staging path value. staging_path={}",
            staging_path.display()
        );
        Status::InternalServerError
    })?);
    // Parent folder must exist
    staging_path.canonicalize().map_err(|e| {
        error!(
            "Could not canonicalize staging path. staging_path={}, error={}",
            staging_path.display(),
            e
        );
        Status::InternalServerError
    })?;

    let random_id = format!(
        "{}-{}.part",
        target_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy(),
        nanoid!()
    );
    info!(
        "Saving {} temporarily to {}",
        target_path.display(),
        random_id
    );
    staging_path.push(random_id);

    // Upload File to Temp Folder and check for errors
    let mut upload_file = tokio::fs::File::create(&staging_path).await.map_err(|e| {
        error!("Could not create staging file. error={}", e);
        Status::InternalServerError
    })?;

    // Check Upload Size
    let stream = data.open(UPLOAD_LIMIT_MB.mebibytes());
    let stream_res = stream
        .stream_to(&mut upload_file)
        .await
        .map_err(|_| Status::InternalServerError)?;

    if !stream_res.complete {
        let _ = tokio::fs::remove_file(&staging_path).await;
        error!("Data stream too large to read.");
        return Err(Status::PayloadTooLarge);
    }

    if upload_file.flush().await.is_err() {
        let _ = tokio::fs::remove_file(&staging_path).await;
        return Err(Status::InternalServerError);
    }

    drop(upload_file);

    // After check is done, move file to target path
    if let Err(e) = tokio::fs::copy(&staging_path, &target_path).await {
        let _ = tokio::fs::remove_file(&staging_path).await;
        let _ = tokio::fs::remove_file(&target_path).await;
        error!(
            "Could not copy file {} to {}, error={}",
            staging_path.display(),
            target_path.display(),
            e
        );
        return Err(Status::InternalServerError);
    }

    if let Err(e) = tokio::fs::remove_file(&staging_path).await {
        // No error return here since the copy was successful. This might have just left some artifacts
        error!(
            "Could not remove file {}, error={}",
            staging_path.display(),
            e
        );
    }

    Ok(())
}

pub async fn remove_photoset_dir(target_dir: &PathBuf, forced: bool) -> Result<(), Status> {
    let error_handler = |e: tokio::io::Error| {
        error!(
            "Failed to remove photoset directory. target_dir={}, forced={}, error={}",
            target_dir.display(),
            forced,
            e
        );
        match e.kind() {
            std::io::ErrorKind::DirectoryNotEmpty => Status::Conflict,
            std::io::ErrorKind::NotFound => Status::NotFound,
            std::io::ErrorKind::PermissionDenied => Status::Forbidden,
            _ => Status::InternalServerError,
        }
    };

    if forced {
        tokio::fs::remove_dir_all(&target_dir)
            .await
            .map_err(error_handler)
    } else {
        tokio::fs::remove_dir(&target_dir)
            .await
            .map_err(error_handler)
    }
}

pub async fn remove_photoset_file(target_file: &PathBuf) -> Result<(), Status> {
    tokio::fs::remove_file(target_file).await.map_err(|e| {
        error!(
            "Failed to remove photoset file. target_dir={}, error={}",
            target_file.display(),
            e
        );
        match e.kind() {
            std::io::ErrorKind::NotFound => Status::NotFound,
            std::io::ErrorKind::PermissionDenied => Status::Forbidden,
            std::io::ErrorKind::IsADirectory => Status::BadRequest,
            std::io::ErrorKind::InvalidInput => Status::BadRequest,
            _ => Status::InternalServerError,
        }
    })
}
