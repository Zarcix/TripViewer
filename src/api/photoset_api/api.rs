use std::fs;

use log::{error, info, warn};
use sanitize_filename::sanitize;

use std::path::PathBuf;

use rocket::{form::Form, http::Status};

use super::forms::{PhotoSetPutForm, PhotoSetUpdateForm};
use super::helpers::resolve_photoset_path;

use crate::api::request_guards::UserAuth;

use crate::constants::{filehandle_constants::PHOTO_DIR, server_constants::SERVE_PATH};

#[post("/<photoset..>")]
pub async fn create_photoset(photoset: PathBuf, _userauth: UserAuth<'_>) -> Result<Status, Status> {
    let photoset_path = resolve_photoset_path(&photoset)?;
    if photoset_path.exists() {
        warn!(
            "Creation failed: path already exists at {:?}",
            photoset_path
        );
        return Err(Status::Conflict);
    }

    info!("Creating Photoset: {}", photoset_path.display());

    fs::create_dir_all(&photoset_path).map_err(|e| {
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

    Ok(Status::Created)
}

#[patch("/<photoset..>", data = "<form>")]
pub async fn update_photoset(
    photoset: PathBuf,
    form: Form<PhotoSetUpdateForm<'_>>,
    _user_auth: UserAuth<'_>,
) -> Result<Status, Status> {
    let photoset_path = resolve_photoset_path(&photoset)?;
    if !photoset_path.exists() {
        warn!("Photoset update path not found: {}", &photoset.display());
        return Err(Status::NotFound);
    }

    if let Some(new_name) = form.new_name {
        let sanitized_name = sanitize(new_name);

        let parent = photoset_path.parent().ok_or(Status::InternalServerError)?;
        let new_full_path = parent.join(sanitized_name);

        fs::rename(&photoset_path, &new_full_path).map_err(|e| {
            error!("Failed to rename photoset: {}", e);
            Status::InternalServerError
        })?;
    }

    Ok(Status::Accepted)
}

#[put("/<photoset..>", data = "<form>")]
pub async fn assign_photoset(
    photoset: PathBuf,
    form: Form<PhotoSetPutForm<'_>>,
    _userauth: UserAuth<'_>,
) -> Result<Status, Status> {
    let photoset_path = resolve_photoset_path(&photoset)?;
    let photos_path = PathBuf::from(SERVE_PATH.get().ok_or(Status::InternalServerError)?).join(PHOTO_DIR);

    if !photoset_path.exists() {
        warn!("Photoset update path not found: {}", &photoset.display());
        return Err(Status::NotFound);
    }

    /* Validate the Paths */

    let mut move_list = Vec::new();

    // Addition Moves

    for file_name in &form.additions {
        let src = photos_path.join(file_name);
        let dest = photoset_path.join(file_name);

        if !src.exists() {
            error!("Could not find file {}", src.display());
            return Err(Status::UnprocessableEntity);
        }
        move_list.push((src, dest));
    }

    // Removal Moves

    for file_name in &form.removals {
        let src = photoset_path.join(file_name);
        let dest = photos_path.join(file_name);

        if !src.exists() {
            error!("Could not find file {}", src.display());
            return Err(Status::UnprocessableEntity);
        }
        move_list.push((src, dest));
    }

    /* Perform Moves */

    for (old_path, new_path) in move_list {
        fs::rename(&old_path, &new_path).map_err(|e| {
            error!(
                "Could not move {} to {}: {}",
                old_path.display(),
                new_path.display(),
                e
            );
            Status::InternalServerError
        })?;
    }

    Ok(Status::Ok)
}

#[delete("/<photoset..>")]
pub async fn delete_photoset(photoset: PathBuf, _userauth: UserAuth<'_>) -> Result<Status, Status> {
    let photoset_path = resolve_photoset_path(&photoset)?;
    fs::remove_dir(&photoset_path).map_err(|e| {
        error!(
            "Failed to delete {}. System error: {}",
            photoset_path.display(),
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
