use rocket::form::FromForm;
use rocket::fs::TempFile;

#[derive(FromForm)]
pub struct AlbumCreateForm<'r> {
}

#[derive(FromForm)]
pub struct AlbumUpdateForm<'r> {
}

#[derive(FromForm)]
pub struct AlbumDeleteForm<'r> {
}