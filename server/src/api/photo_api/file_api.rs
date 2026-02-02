use std::fs::{create_dir_all, remove_file};
use std::path::{
    Path, PathBuf
};
use rocket::form::Form;
use rocket::http::Status;
use sanitize_filename::sanitize;

use crate::constants::{
    server_constants::SERVER_PATH,
    filehandle_constants::UPLOAD_DIR
};
use crate::api::request_guards::UserAuth;

use super::forms::{
    FileUploadForm,
    FileDeleteForm
};

#[post("/", data = "<form>")]
pub async fn upload_photo(mut form: Form<FileUploadForm<'_>>, _user_auth: UserAuth<'_>) -> Result<Status, Status>{
    // Grab stuff from form first
    let filename = sanitize(form.filename);
    let file = &mut form.file;

    // Build Directory Variables
    let photo_dir: PathBuf = Path::new(SERVER_PATH).join(UPLOAD_DIR);
    create_dir_all(&photo_dir).map_err(|_| Status::InternalServerError)?;

    let photo_path: PathBuf = photo_dir.join(filename);
    photo_path.exists().then(|| Err(Status::Conflict)).unwrap_or(Ok(()))?;

    // Move upload file into path
    file.move_copy_to(&photo_path).await.map_err(|_| Status::InternalServerError)?;

    Ok(Status::Ok)
}

#[delete("/", data = "<form>")]
pub async fn delete_photo(form: Form<FileDeleteForm<'_>>, _user_auth: UserAuth<'_>) -> Result<Status, Status> {
    // Get path and make sure it actually exists
    let filename = sanitize(form.filename);
    let photo_path: PathBuf = Path::new(SERVER_PATH).join(UPLOAD_DIR).join(filename);
    (!photo_path.exists()).then(|| Err(Status::NotFound)).unwrap_or(Ok(()))?;

    // Attempt to delete file
    println!("Deleting File {:?}", &photo_path);
    remove_file(photo_path).map_err(|_| Status::InternalServerError)?;

    Ok(Status::NoContent)
}