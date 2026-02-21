#[macro_use]
extern crate rocket;

use log::LevelFilter;
use simple_logger::SimpleLogger;
use clap::Parser;

mod api;
mod constants;
mod tasks;

use crate::constants::server_constants::{
    SERVER_PATH,
    API_KEY
};

// Route List
use api::file_server;
use api::photo_api;
use api::photoset_api;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// API Key to use for deployment
    #[arg(long)]
    api_key: String,

    /// Server Photo Archive Path
    #[arg(long)]
    archive_path: String,
}

fn setup() -> Result<(), String> {
    let args = Args::parse();

    API_KEY.set(args.api_key)?; 
    SERVER_PATH.set(args.archive_path)?;

    Ok(())
}

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    // Fairing to remove localhost limitations for API testing
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, PUT, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
        response.remove_header("X-Frame-Options");
    }
}

#[options("/<_..>")]
fn all_options() {
    // This just returns a 200 OK with the headers
    // added by the CORS fairing above.
}

#[launch]
fn run_server() -> _ {
    setup().expect("Setup Failed");

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    rocket::build()
        .attach(CORS)
        .mount("/api/photoset", photoset_api::api_routes::route_list())
        .mount("/api/photo", photo_api::api_routes::route_list())
        .mount("/photos", file_server::api_routes())
        .mount("/", routes![all_options])
}
