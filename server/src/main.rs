#[macro_use]
extern crate rocket;

use log::LevelFilter;
use simple_logger::SimpleLogger;

mod api;
mod constants;
mod tasks;

// Route List
use api::file_server;
use api::photo_api;
use api::photoset_api;

#[launch]
fn run_server() -> _ {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    rocket::build()
        .mount("/api/photoset", photoset_api::api_routes::route_list())
        .mount("/api/photo", photo_api::api_routes::route_list())
        .mount("/photos", file_server::api_routes())
}
