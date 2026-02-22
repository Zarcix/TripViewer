use rocket::form::FromForm;
use rocket::fs::TempFile;

#[derive(FromForm)]
pub struct FileUploadForm<'r> {
    pub file: TempFile<'r>,
    pub filename: &'r str,
}

#[derive(FromForm)]
pub struct FileDeleteForm<'r> {
    pub filename: &'r str,
}
