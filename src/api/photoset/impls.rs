use std::path::PathBuf;

use crate::api::photoset::fs_helpers;

use super::models;
use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::{Request, Response};

const MEDIA_EXTS: &[&str] = &["mp4", "m4v", "mov", "mpeg", "mpg", "webm", "avi"];

#[rocket::async_trait]
impl<'a> Responder<'a, 'static> for models::StreamedFile {
    fn respond_to(self, _req: &'a Request<'_>) -> response::Result<'static> {
        let file = self.file;
        let file_size = self.size;
        let start = self.range.0;
        let end = self.range.1;
        let content_type = self.content_type;

        Response::build()
            .status(Status::PartialContent)
            .header(content_type)
            .raw_header("Accept-Ranges", "bytes")
            .raw_header(
                "Content-Range",
                format!("bytes {}-{}/{}", start, end, file_size),
            )
            .sized_body(file_size as usize, file)
            .ok()
    }
}

impl<'r> Responder<'r, 'static> for models::FileServerResponse {
    fn respond_to(self, req: &'r Request<'_>) -> Result<rocket::Response<'static>, Status> {
        match self {
            models::FileServerResponse::FullContent(fc_file) => fc_file.respond_to(req),
            models::FileServerResponse::RangedContent(rc_file) => rc_file.respond_to(req),
            models::FileServerResponse::DirectoryListing(dir_listing) => {
                dir_listing.respond_to(req)
            }
        }
    }
}

impl models::FileEntry {
    pub fn resolve_path_type(path: PathBuf) -> Self {
        let is_media = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| MEDIA_EXTS.contains(&ext));

        let path_type = if path.is_dir() {
            models::FileType::Directory
        } else if path.is_file() && is_media {
            models::FileType::Media
        } else {
            models::FileType::File
        };

        Self {
            path,
            kind: path_type,
            range: None,
        }
    }

    pub async fn parse_entry(&self) -> Result<models::FileServerResponse, Status> {
        match self.kind {
            models::FileType::Directory => return fs_helpers::parse_directory(&self.path).await,
            models::FileType::Media => {
                return fs_helpers::parse_streamed_file(&self.path, &self.range).await
            }
            models::FileType::File => return fs_helpers::parse_file(&self.path).await,
        }
    }
}
