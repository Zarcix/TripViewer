use std::path::{
    Path,
    PathBuf
};

use rocket::http::Status;

use crate::{api::consolidated_api::models::FileServerResponse, constants::server_constants::SERVE_PATH};

use super::fs_helpers;

#[get("/<path..>")]
pub async fn list_photoset<'a>(path: PathBuf) -> Result<FileServerResponse, Status> {
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
        return fs_helpers::parse_directory(&photoset_path, &path)
    }

    if photoset_path.is_file() {
        return fs_helpers::parse_file(photoset_path).await;
    }

    Err(Status::NotFound)
}