#[macro_use] extern crate rocket;

mod api;
mod tasks;
mod constants;

// Route List
use api::photo_api;
use api::album_api;
use api::file_server;


#[launch]
fn run_server() -> _ {
    rocket::build()
        .mount("/api/album", album_api::api_routes::route_list())
        .mount("/api/photo", photo_api::api_routes::route_list())
        .mount("/imageserver", file_server::api_routes())
}