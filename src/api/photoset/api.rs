use std::path::{Component, Path, PathBuf};

use rocket::form::Form;
use rocket::http::Status;
use rocket::Data;

use crate::api::photoset::models::FileEntry;
use crate::api::request_guards::{RequestHeaders, UserAuth};
use crate::constants::server_constants::SERVE_PATH;

use super::forms::PhotoSetUpdateForm;
use super::fs_helpers;
use super::models::FileServerResponse;

fn resolve_photoset_path(short_path: &PathBuf) -> Result<PathBuf, Status> {
    let root = Path::new(SERVE_PATH.get().ok_or(Status::InternalServerError)?);

    // Reject absolute paths immediately
    if short_path.is_absolute() {
        return Err(Status::Forbidden);
    }

    // Reject traversal components
    for component in short_path.components() {
        match component {
            Component::ParentDir => return Err(Status::Forbidden),
            Component::RootDir => return Err(Status::Forbidden),
            _ => {}
        }
    }

    Ok(root.join(short_path))
}

#[get("/<path..>")]
pub async fn list_photoset(
    path: PathBuf,
    headers: RequestHeaders<'_>,
    _auth: UserAuth,
) -> Result<FileServerResponse, Status> {
    info!("Listing PhotoSets at {}", path.display());
    let photoset_path = resolve_photoset_path(&path)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    let mut photoset_entry = FileEntry::resolve_path_type(photoset_path);
    photoset_entry.range = headers.extract_range_header();

    let photoset_res = photoset_entry.parse_entry().await;

    photoset_res
}

#[post("/<path..>")]
pub async fn create_photoset(path: PathBuf, _auth: UserAuth) -> Result<Status, Status> {
    info!("Creating PhotoSet at {}", path.display());

    let photoset_path = resolve_photoset_path(&path)?;

    fs_helpers::create_dir(&photoset_path).await?;

    Ok(Status::Created)
}

#[patch("/<path..>", data = "<form>")]
pub async fn update_photoset(
    path: PathBuf,
    form: Form<PhotoSetUpdateForm>,
    _auth: UserAuth,
) -> Result<Status, Status> {
    info!("Updating PhotoSet at {}", path.display());

    // Incoming Path Validation //
    let root = Path::new(SERVE_PATH.get().ok_or(Status::InternalServerError)?);

    let photoset_path = resolve_photoset_path(&path)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    if !photoset_path.starts_with(root) {
        error!(
            "Invalid PhotoSet Path: photoset_path={}",
            photoset_path.display()
        );
        return Err(Status::Forbidden);
    }

    // Form Validation //
    let new_name = form.new_name.trim();
    if new_name.is_empty() {
        return Err(Status::BadRequest);
    }

    let new_path = Path::new(&photoset_path.parent().ok_or(Status::InternalServerError)?)
        .to_path_buf()
        .join(new_name);
    let new_parent = new_path
        .parent()
        .ok_or(Status::InternalServerError)?
        .canonicalize()
        .map_err(|e| {
            error!(
                "Could not canonicalize path. path={}, error={}",
                new_path.display(),
                e
            );
            Status::Forbidden
        })?;

    if !new_parent.starts_with(root) {
        error!(
            "Invalid Path. new_parent={}, root={}",
            new_parent.display(),
            root.display()
        );
        return Err(Status::Forbidden);
    }

    fs_helpers::rename_entry(&photoset_path, &new_path).await?;

    Ok(Status::Accepted)
}

#[put("/<path..>", data = "<data>")]
pub async fn put_photoset(
    path: PathBuf,
    data: Data<'_>,
    _auth: UserAuth,
) -> Result<Status, Status> {
    let target_path = resolve_photoset_path(&path)?;

    let parent_path = target_path
        .parent()
        .ok_or(Status::BadRequest)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    if !parent_path.is_dir() {
        error!(
            "Path to upload is not a directory. parent_path={}",
            parent_path.display()
        );
        return Err(Status::BadRequest);
    }

    fs_helpers::save_data(data, &target_path).await?;

    Ok(Status::Created)
}

#[delete("/<path..>?<force_removal>")]
pub async fn delete_photoset(
    path: PathBuf,
    force_removal: bool,
    _auth: UserAuth,
) -> Result<Status, Status> {
    let target_path = resolve_photoset_path(&path)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    if target_path.is_dir() {
        fs_helpers::remove_photoset_dir(&target_path, force_removal).await?;
        return Ok(Status::NoContent);
    }

    if target_path.is_file() {
        fs_helpers::remove_photoset_file(&target_path).await?;
        return Ok(Status::NoContent);
    }

    Err(Status::BadRequest)
}
