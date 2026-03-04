#[macro_use]
extern crate rocket;

use std::path::Path;

use log::LevelFilter;
use simple_logger::SimpleLogger;
use clap::Parser;

mod api;
mod constants;

use crate::constants::server_constants::{
    STAGING_PATH,
    SERVE_PATH,
    API_KEY,
};

// Route List
use api::photoset;

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

    /// Path where uploaded files will be initially uploaded to. 
    #[arg(long)]
    staging_path: String,
}

fn setup() -> Result<(), String> {
    let args = Args::parse();

    // API Key
    API_KEY.set(args.api_key)?; 

    // Data Serve Path
    let serve_path = Path::new(&args.archive_path)
        .canonicalize()
        .map_err(|_| String::from("Invalid Serve Path"))?;
    SERVE_PATH.set(String::from(serve_path.to_string_lossy()))?;

    // Staging Path
    let staging_path = Path::new(&args.staging_path)
        .canonicalize()
        .map_err(|_| String::from("Invalid Staging Path"))?;
    STAGING_PATH.set(String::from(staging_path.to_string_lossy()))?;

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
        response.set_header(Header::new("Access-Control-Allow-Methods","*"));
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
        .mount("/", routes![all_options])
        .mount("/", rocket::fs::FileServer::from(rocket::fs::relative!("frontend")))
        .mount("/api/photoset", photoset::route_list())
}
