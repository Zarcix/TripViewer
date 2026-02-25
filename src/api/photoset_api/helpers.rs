use log::warn;
use rocket::http::Status;
use sanitize_filename::sanitize;
use std::path::{Path, PathBuf};

use crate::constants::{filehandle_constants::PHOTOSET_DIR, server_constants::SERVE_PATH};

pub fn resolve_photoset_path(photoset: &Path) -> Result<PathBuf, Status> {
    photoset.components().next().ok_or(Status::BadRequest)?;
    let cleaned_photoset: PathBuf = photoset
        .iter()
        .map(|seg| sanitize(seg.to_string_lossy().as_ref()))
        .collect();

    let storage_root = PathBuf::from(SERVE_PATH.get().ok_or(Status::InternalServerError)?).join(PHOTOSET_DIR);
    let full_path = storage_root.join(&cleaned_photoset);

    // 2. Root Guard: Ensure we aren't targeting the root storage folder
    root_guard_check(&storage_root, &full_path)?;

    Ok(full_path)
}

fn root_guard_check<P: AsRef<Path>>(storage_root: P, full_path: P) -> Result<(), Status> {
    let root = storage_root.as_ref();
    let path = full_path.as_ref();

    if path == root || !path.starts_with(root) {
        warn!("Check Failed");
        return Err(Status::Forbidden);
    }

    Ok(())
}
