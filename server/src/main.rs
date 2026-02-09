#[macro_use]
extern crate rocket;

use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::env;

mod api;
mod constants;
mod tasks;

use crate::constants::server_constants::API_KEY;

// Route List
use api::file_server;
use api::photo_api;
use api::photoset_api;

fn setup() -> Result<(), String> {
    let api_key = env::var("API_KEY").map_err(|_| String::from("API_KEY Required"))?;
    API_KEY.set(api_key)?;
    Ok(())
}

#[launch]
fn run_server() -> _ {
    setup().expect("Setup Failed");

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    rocket::build()
        .mount("/api/photoset", photoset_api::api_routes::route_list())
        .mount("/api/photo", photo_api::api_routes::route_list())
        .mount("/photos", file_server::api_routes())
}
