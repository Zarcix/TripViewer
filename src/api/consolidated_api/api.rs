use std::path::{
    Path,
    PathBuf
};

use rocket::Data;
use rocket::form::Form;
use rocket::http::{
    Status,
};

use crate::api::consolidated_api::models::FileServerResponse;
use crate::constants::server_constants::SERVE_PATH;

use super::fs_helpers;
use super::forms::PhotoSetUpdateForm;

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
pub async fn list_photoset(path: PathBuf) -> Result<FileServerResponse, Status> {
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
pub async fn create_photoset(path: PathBuf) -> Result<Status, Status> {
    info!("Creating PhotoSet at {}", path.display());

    let photoset_path = resolve_photoset_path(&path)?;

    fs_helpers::create_dir(&photoset_path).await?;

    Ok(Status::Created)
}

#[patch("/<path..>", data = "<form>")]
pub async fn update_photoset(path: PathBuf, form: Form<PhotoSetUpdateForm>) -> Result<Status, Status> {
    info!("Updating PhotoSet at {}", path.display());

    let photoset_path = resolve_photoset_path(&path)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    let new_path = photoset_path.parent()
        .ok_or(Status::InternalServerError)?
        .join(&form.new_name);

    fs_helpers::rename_entry(&photoset_path, &new_path).await?;

    Ok(Status::Accepted)
}

#[put("/<path..>", data = "<data>")]
pub async fn put_photoset(path: PathBuf, data: Data<'_>) -> Result<(), Status> {
    let target_path = resolve_photoset_path(&path)?;

    if target_path.exists() {
        error!("Target file already exists. target_path={}", target_path.display());
        return Err(Status::Conflict);
    }

    target_path
        .parent()
        .ok_or(Status::BadRequest)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    fs_helpers::save_data(data, &target_path).await?;

    Ok(())
}