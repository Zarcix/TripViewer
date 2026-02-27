use std::path::{
    Path,
    PathBuf
};

use rocket::Data;
use rocket::form::Form;
use rocket::http::{
    Status,
};

use crate::api::request_guards::UserAuth;
use crate::constants::server_constants::SERVE_PATH;

use super::fs_helpers;
use super::forms::PhotoSetUpdateForm;
use super::models::FileServerResponse;

fn resolve_photoset_path(short_path: &PathBuf) -> Result<PathBuf, Status> {
    let root = Path::new(
        SERVE_PATH
            .get()
            .ok_or(Status::InternalServerError)?
    );

    let photoset_path = root.join(short_path);

    // Prevent path traversal
    if !photoset_path.starts_with(root) {
        error!(
            "Illegal Paths. photoset_path={}, root={}",
            photoset_path.display(),
            root.display()
        );
        return Err(Status::Forbidden);
    }

    return Ok(photoset_path);
}

#[get("/<path..>")]
pub async fn list_photoset(path: PathBuf, _auth: UserAuth<'_>) -> Result<FileServerResponse, Status> {
    info!("Listing PhotoSets at {}", path.display());
    let photoset_path = resolve_photoset_path(&path)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    if photoset_path.is_dir() {
        return fs_helpers::parse_directory(&photoset_path, &path).await;
    }

    if photoset_path.is_file() {
        return fs_helpers::parse_file(photoset_path).await;
    }

    Err(Status::NotFound)
}

#[post("/<path..>")]
pub async fn create_photoset(path: PathBuf, _auth: UserAuth<'_>) -> Result<Status, Status> {
    info!("Creating PhotoSet at {}", path.display());

    let photoset_path = resolve_photoset_path(&path)?;

    fs_helpers::create_dir(&photoset_path).await?;

    Ok(Status::Created)
}

#[patch("/<path..>", data = "<form>")]
pub async fn update_photoset(path: PathBuf, form: Form<PhotoSetUpdateForm>, _auth: UserAuth<'_>) -> Result<Status, Status> {
    info!("Updating PhotoSet at {}", path.display());

    // Incoming Path Validation //
    let root = Path::new(
        SERVE_PATH
            .get()
            .ok_or(Status::InternalServerError)?
    );

    let photoset_path = resolve_photoset_path(&path)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    if !photoset_path.starts_with(root) {
        error!("Invalid PhotoSet Path: photoset_path={}", photoset_path.display());
        return Err(Status::Forbidden);
    }

    // Form Validation //
    let new_name = form.new_name.trim();

    if new_name.is_empty() {
        return Err(Status::BadRequest);
    }

    let parent = photoset_path
        .parent()
        .ok_or(Status::InternalServerError)?;
    let new_path = parent.join(new_name);

    // New Path Validation //
    if !new_path.starts_with(&root) {
        return Err(Status::Forbidden);
    }

    fs_helpers::rename_entry(&photoset_path, &new_path).await?;

    Ok(Status::Accepted)
}

#[put("/<path..>", data = "<data>")]
pub async fn put_photoset(path: PathBuf, data: Data<'_>, _auth: UserAuth<'_>) -> Result<Status, Status> {
    let target_path = resolve_photoset_path(&path)?;

    target_path
        .parent()
        .ok_or(Status::BadRequest)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    fs_helpers::save_data(data, &target_path).await?;

    Ok(Status::Created)
}

#[delete("/<path..>?<force_removal>")]
pub async fn delete_photoset(path: PathBuf, force_removal: bool, _auth: UserAuth<'_>) -> Result<Status, Status> {
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