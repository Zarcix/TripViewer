#[macro_use] extern crate rocket;

use rocket::fs::{FileServer, Options};

mod api;
mod tasks;
mod constants;

use constants::server_constants::SERVER_PATH;

// Route List
use api::photo_api;
use api::album_api;


#[launch]
fn run_server() -> _ {
    rocket::build()
        .mount("/api/album", album_api::api_routes::route_list())
        .mount("/api/photo", photo_api::api_routes::route_list())
        .mount("/image/serve", FileServer::new(SERVER_PATH, Options::Index | Options::NormalizeDirs).rank(-1))
}