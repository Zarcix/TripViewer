use std::{fs, path::{
    Path,
    PathBuf
}};

use rocket::http::{
    Status,
};

use crate::{api::consolidated_api::models::FileServerResponse, constants::server_constants::SERVE_PATH};

use super::fs_helpers;

#[get("/<path..>")]
pub async fn list_photoset(path: PathBuf) -> Result<FileServerResponse, Status> {
    rocket::info!("Listing PhotoSets at {}", path.display());
    let root = Path::new(
        SERVE_PATH
            .get()
            .ok_or(Status::InternalServerError)?
    );
    let photoset_path = root
        .join(&path)
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    // Prevent path traversal
    if !photoset_path.starts_with(root) {
        error!(
            "Illegal Paths. photoset_path={}, root={}",
            photoset_path.display(),
            root.display()
        );
        return Err(Status::Forbidden);
    }

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
    let root = Path::new(
        SERVE_PATH
            .get()
            .ok_or(Status::InternalServerError)?
    );
    let photoset_path = root
        .join(&path);

    if photoset_path.exists() {
        warn!(
            "Creation failed: path already exists at {:?}",
            photoset_path
        );
        return Err(Status::Conflict);
    }

    fs_helpers::create_dir(&photoset_path).await?;

    Ok(Status::Created)
}