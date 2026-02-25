use rocket::form::Form;
use rocket::http::Status;
use sanitize_filename::sanitize;
use std::fs::{create_dir_all, remove_file};
use std::path::{Path, PathBuf};

use crate::api::request_guards::UserAuth;
use crate::constants::{filehandle_constants::PHOTO_DIR, server_constants::SERVE_PATH};

use super::forms::{FileDeleteForm, FileUploadForm};

#[post("/", data = "<form>")]
pub async fn upload_photo(
    mut form: Form<FileUploadForm<'_>>,
    _user_auth: UserAuth<'_>,
) -> Result<Status, Status> {
    // Grab stuff from form first
    let filename = sanitize(form.filename);
    let file = &mut form.file;

    // Build Directory Variables
    let photo_dir: PathBuf = Path::new(SERVE_PATH.get().ok_or(Status::InternalServerError)?).join(PHOTO_DIR);
    create_dir_all(&photo_dir).map_err(|_| Status::InternalServerError)?;

    let photo_path: PathBuf = photo_dir.join(filename);
    if photo_path.exists() {
        Err(Status::Conflict)
    } else {
        Ok(())
    }?;

    // Move upload file into path
    file.move_copy_to(&photo_path)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Status::Ok)
}

#[delete("/", data = "<form>")]
pub async fn delete_photo(
    form: Form<FileDeleteForm<'_>>,
    _user_auth: UserAuth<'_>,
) -> Result<Status, Status> {
    // Get path and make sure it actually exists
    let filename = sanitize(form.filename);
    let photo_path: PathBuf = Path::new(SERVE_PATH.get().ok_or(Status::InternalServerError)?).join(PHOTO_DIR).join(filename);

    if !photo_path.exists() {
        info!("Photo Path: {}", photo_path.display());
        Err(Status::NotFound)
    } else {
        Ok(())
    }?;

    // Attempt to delete file
    info!("Deleting File {}", photo_path.display());
    remove_file(photo_path).map_err(|_| Status::InternalServerError)?;

    Ok(Status::NoContent)
}
