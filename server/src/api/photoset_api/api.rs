use std::fs::remove_dir;

use log::{error, info, warn};

use std::path::PathBuf;

use rocket::http::Status;
use sanitize_filename::sanitize;

use crate::api::request_guards::UserAuth;

use super::helpers::root_guard_check;
use crate::constants::{filehandle_constants::PHOTOSET_DIR, server_constants::SERVER_PATH};

#[post("/<photoset..>")]
pub fn create_photoset(photoset: PathBuf, _userauth: UserAuth) -> Result<Status, Status> {
    // 1. Path must not be empty
    photoset.components().next().ok_or(Status::BadRequest)?;

    let cleaned_photoset: PathBuf = photoset
        .iter()
        .map(|seg| sanitize(seg.to_string_lossy().as_ref()))
        .collect();

    let storage_root = PathBuf::from(SERVER_PATH).join(PHOTOSET_DIR);
    let full_path = storage_root.join(&cleaned_photoset);

    // 2. Root Guard: Ensure we aren't targeting the root storage folder
    root_guard_check(&storage_root, &full_path)?;

    // 3. Manual Conflict Check: Do we want to allow overwriting/re-creating?
    if full_path.exists() {
        warn!("Creation failed: path already exists at {:?}", full_path);
        return Err(Status::Conflict);
    }

    info!("Creating Photoset: {}", full_path.display());

    // 4. Create the tree
    std::fs::create_dir_all(&full_path).map_err(|e| {
        error!(
            "Could not create photoset tree {}: {}",
            full_path.display(),
            e
        );
        match e.kind() {
            std::io::ErrorKind::PermissionDenied => Status::Forbidden,
            _ => Status::InternalServerError,
        }
    })?;

    Ok(Status::Created)
}

#[patch("/")]
pub fn update_photoset(_user_auth: UserAuth) {}

#[put("/<photoset..>")]
pub fn assign_photoset(photoset: PathBuf, _userauth: UserAuth) {}

#[delete("/<photoset..>")]
pub fn delete_photoset(photoset: PathBuf, _userauth: UserAuth) -> Result<Status, Status> {
    // 1. Path must not be empty
    photoset.components().next().ok_or(Status::BadRequest)?;

    let cleaned_photoset: PathBuf = photoset
        .iter()
        .map(|seg| sanitize(seg.to_string_lossy().as_ref()))
        .collect();
    info!("Deleting Photoset: {}", &cleaned_photoset.display());

    let storage_root = PathBuf::from(SERVER_PATH).join(PHOTOSET_DIR);
    let full_path = storage_root.join(&cleaned_photoset);

    // 2. Root Guard Check
    root_guard_check(&storage_root, &full_path)?;

    // 3. Remove Dir. If it's not empty it will fail.
    remove_dir(&full_path).map_err(|e| {
        error!(
            "Failed to delete {}. System error: {}",
            cleaned_photoset.display(),
            e
        );
        match e.kind() {
            std::io::ErrorKind::DirectoryNotEmpty => Status::Conflict,
            std::io::ErrorKind::NotFound => Status::NotFound,
            std::io::ErrorKind::PermissionDenied => Status::Forbidden,
            _ => Status::InternalServerError,
        }
    })?;

    Ok(Status::NoContent)
}
