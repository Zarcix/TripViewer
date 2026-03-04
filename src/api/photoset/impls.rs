use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use super::models::{
    StreamedFile,
    RangeHeader,
    FileServerResponse
};
use rocket::http::{ContentType, Status};
use rocket::response::{
    self,
    Responder
};
use rocket::{
    Request,
    Response
};

use crate::constants::server_constants::MAX_CHUNK_SIZE_MB;

pub fn parse_range_header(raw_header: &str) -> Result<RangeHeader, Status> {
    let range_str = raw_header.strip_prefix("bytes=").ok_or(Status::BadRequest)?;

    let mut parts_range = range_str.split('-');

    let start: u64 = parts_range
        .next()
        .ok_or(Status::BadRequest)?
        .parse()
        .map_err(|_| Status::BadRequest)?;

    let end: Option<u64> = parts_range
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().map_err(|_| Status::BadRequest))
        .transpose()?;

    let range_result = RangeHeader {
        start: start,
        end: end
    };

    return Ok(range_result);
}

#[rocket::async_trait]
impl <'a> Responder<'a, 'static> for StreamedFile {
    fn respond_to(self, req: &'a Request<'_>) -> response::Result<'static> {
        let file_path = self.file.as_path();
        let mut file = File::open(file_path).map_err(|_| Status::NotFound)?;

        let metadata = file.metadata().map_err(|_| Status::InternalServerError)?;
        let file_size = metadata.len();
        let extension = file_path.extension().unwrap_or_default().to_string_lossy();

        let range_header = req.headers().get_one("Range");
        
        let (mut start, mut end) = (0, file_size - 1);

        if let Some(range_str) = range_header {
            let ranges = parse_range_header(range_str)?;
            start = ranges.start;

            let requested_end = ranges.end.unwrap_or(file_size - 1);

            // Clamp to file bounds first
            let requested_end = requested_end.min(file_size - 1);

            // Now clamp to MAX_CHUNK_SIZE
            let max_end = start.saturating_add(MAX_CHUNK_SIZE_MB.get().ok_or(Status::InternalServerError)? - 1);
            end = requested_end.min(max_end);
        }

        // Ensure start is valid
        if start >= file_size {
            return Err(Status::RangeNotSatisfiable);
        }

        let chunk_size = (end - start + 1) as usize;

        file.seek(SeekFrom::Start(start)).map_err(|_| Status::InternalServerError)?;

        let mut buffer = vec![0; chunk_size];
        file.read_exact(&mut buffer).map_err(|_| Status::InternalServerError)?;

        Response::build()
            .status(Status::PartialContent)
            .header(ContentType::parse_flexible(&extension).unwrap_or(ContentType::MP4))
            .raw_header("Accept-Ranges", "bytes")
            .raw_header("Content-Range", format!("bytes {}-{}/{}", start, end, file_size))
            .sized_body(chunk_size, std::io::Cursor::new(buffer))
            .ok()
    }
}

impl<'r> Responder<'r, 'static> for FileServerResponse {
    fn respond_to(self, req: &'r Request<'_>) -> Result<rocket::Response<'static>, Status> {
        match self {
            FileServerResponse::FullContent(fc_file) => fc_file.respond_to(req),
            FileServerResponse::RangedContent(rc_file) => rc_file.respond_to(req),
            FileServerResponse::DirectoryListing(dir_listing) => dir_listing.respond_to(req),
        }
    }
}