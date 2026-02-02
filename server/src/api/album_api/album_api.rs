use std::fs::read_dir;
use std::path::{
    Path,
    PathBuf
};

use rocket::http::Status;
use rocket::serde::json::Json;

use crate::api::request_guards::UserAuth;

use crate::constants::{
    server_constants::SERVER_PATH,
    filehandle_constants::ALBUM_DIR
};

#[post("/")]
pub fn create_photoset(_user_auth: UserAuth<'_>) {

}

#[patch("/")]
pub fn update_photoset(_user_auth: UserAuth<'_>) {
}

#[delete("/")]
pub fn delete_photoset(_user_auth: UserAuth<'_>) {

}