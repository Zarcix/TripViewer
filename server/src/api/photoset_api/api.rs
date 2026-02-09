use std::fs;

use log::{error, info, warn};
use sanitize_filename::sanitize;

use std::path::PathBuf;

use rocket::{form::Form, http::Status};

use super::helpers::resolve_photoset_path;
use super::forms::{
    PhotoSetUpdateForm,
    PhotoSetPutForm
};

use crate::api::request_guards::UserAuth;


#[post("/<photoset..>")]
pub async fn create_photoset(photoset: PathBuf, _userauth: UserAuth<'_>) -> Result<Status, Status> {
    let full_path = resolve_photoset_path(&photoset)?;
    if full_path.exists() {
        warn!("Creation failed: path already exists at {:?}", full_path);
        return Err(Status::Conflict);
    }

    info!("Creating Photoset: {}", full_path.display());

    fs::create_dir_all(&full_path).map_err(|e| {
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

#[patch("/<photoset..>", data = "<form>")]
pub async fn update_photoset(photoset: PathBuf, form: Form<PhotoSetUpdateForm<'_>>, _user_auth: UserAuth<'_>) -> Result<Status, Status> {
    let full_path = resolve_photoset_path(&photoset)?;
    if !full_path.exists() {
        warn!("Photoset update path not found: {}", &photoset.display());
        return Err(Status::NotFound);
    }

    if let Some(new_name) = form.new_name {
        let sanitized_name = sanitize(new_name);

        let parent = full_path.parent().ok_or(Status::InternalServerError)?;
        let new_full_path = parent.join(sanitized_name);

        fs::rename(&full_path, &new_full_path).map_err(|e| {
            error!("Failed to rename photoset: {}", e);
            Status::InternalServerError
        })?;
    }

    Ok(Status::Accepted)
}

#[put("/<photoset..>", data = "<form>")]
pub async fn assign_photoset(photoset: PathBuf, form: Form<PhotoSetPutForm<'_>>, _userauth: UserAuth<'_>) {}

#[delete("/<photoset..>")]
pub async fn delete_photoset(photoset: PathBuf, _userauth: UserAuth<'_>) -> Result<Status, Status> {
    let full_path = resolve_photoset_path(&photoset)?;
    fs::remove_dir(&full_path).map_err(|e| {
        error!(
            "Failed to delete {}. System error: {}",
            full_path.display(),
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
