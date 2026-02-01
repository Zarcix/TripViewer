use std::fs;
use std::path::{Path, PathBuf};

use rocket::serde::json::Json;
use rocket::{fs::NamedFile, http::Status};
use rocket::request::Request;
use rocket::Route;
use rocket::response::{Responder, Result as RocketResult};

use crate::constants::server_constants::SERVER_PATH;
use super::request_guards::UserAuth;

pub enum BrowseResponse {
    File(NamedFile),
    Listing(Json<Vec<String>>),
}

impl<'r> Responder<'r, 'static> for BrowseResponse {
    fn respond_to(self, req: &'r Request<'_>) -> RocketResult<'static> {
        match self {
            BrowseResponse::File(f) => f.respond_to(req),
            BrowseResponse::Listing(d) => d.respond_to(req),
        }
    }
}

pub fn api_routes() -> Vec<Route> {
    routes![
        serve_files
    ]
}

#[get("/<path..>")]
async fn serve_files(path: PathBuf, _user_auth: UserAuth<'_>) -> Result<BrowseResponse, Status> {
    let base = Path::new(SERVER_PATH);
    let full = base.join(&path);

    // Prevent path traversal
    let full = full.canonicalize().map_err(|_| Status::NotFound)?;
    if !full.starts_with(base) {
        return Err(Status::Forbidden);
    }

    if full.is_dir() {
        let entries = fs::read_dir(&full).map_err(|_| Status::NotFound)?;
        let photo_entries: Vec<String> = entries
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                path.strip_prefix(SERVER_PATH)
                    .ok()
                    .map(|p| p.to_string_lossy().into_owned())
            })
            .collect();
        return Ok(BrowseResponse::Listing(Json(photo_entries)))
    }


    let file = NamedFile::open(full)
        .await
        .map_err(|_| Status::NotFound)?;

    Ok(BrowseResponse::File(file))
}
