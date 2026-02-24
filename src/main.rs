#[macro_use] extern crate rocket;

use rocket::http::{Status, ContentType};
use rocket::request::{Outcome, FromRequest};
use rocket::response::{self, Response, Responder};
use rocket::Request;

use std::path::Path;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug)]
struct RangeError;

#[derive(Default, Debug)]
struct Range {
}

impl Range {
    pub fn parse_range_header(raw_header: &str) -> (u64, Option<u64>) {
        let range = raw_header.strip_prefix("bytes=").unwrap();
        let parts: Vec<&str> = range.split('-').collect();

        let start: u64 = parts[0].parse().map_err(|_| Status::BadRequest).unwrap();
        let end: Option<u64> = if parts.len() > 1 && !parts[1].is_empty() {
                Some(parts[1].parse().map_err(|_| Status::BadRequest).unwrap())
        } else {
            None
        };

        return (start, end);
    }
}
#[rocket::async_trait]
impl <'r> Responder<'r, 'static> for Range {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let path = Path::new("video.mp4");

        let mut file = File::open(path).map_err(|_| Status::NotFound).unwrap();
        let metadata = file.metadata().map_err(|_| Status::InternalServerError).unwrap();
        println!("{:?}", metadata);
        let file_size = metadata.len();

        let range_header = req.headers().get_one("Range");

        let (range_start, range_end) = Range::parse_range_header(range_header.unwrap());
        println!("{:?}: {:?}->{:?}", range_header, range_start, range_end);

        let start = range_start;
        let end = range_end.unwrap_or(file_size - 1);
        let chunk_size: usize = (end - start + 1).try_into().unwrap();

        file.seek(SeekFrom::Start(start)).map_err(|_| Status::InternalServerError).unwrap();

        let mut buffer = vec![0; chunk_size as usize];
        file.read_exact(&mut buffer).map_err(|_| Status::InternalServerError).unwrap();

        Response::build().status(Status::PartialContent).header(ContentType::Video)
            .raw_header("Accept-Ranges", "bytes").raw_header("Content-Range", format!("bytes {}-{}/{}", start, end, file_size))
            .sized_body(chunk_size, std::io::Cursor::new(buffer))
            .ok()
    }
}

#[get("/test")]
async fn test_video() -> Range {
    Range {}
}

// #[get("/stream")]
// async fn stream_video() -> Result<Response<'static>, Status> {
//     let path = Path::new("video.mp4");
//     let mut file = File::open(path).await.map_err(|_| Status::NotFound)?;

//     let metadata = file.metadata().await.map_err(|_| Status::InternalServerError)?;
//     let file_size = metadata.len();

//     let range_header = req.headers().get_one("Range");

//     if let Some(range) = range_header {
//         // Example: "bytes=0-99"
//         let range = range.strip_prefix("bytes=").ok_or(Status::BadRequest)?;
//         let parts: Vec<&str> = range.split('-').collect();

//         let start: u64 = parts[0].parse().map_err(|_| Status::BadRequest)?;
//         let end: u64 = if parts.len() > 1 && !parts[1].is_empty() {
//             parts[1].parse().map_err(|_| Status::BadRequest)?
//         } else {
//             file_size - 1
//         };

//         let chunk_size = end - start + 1;

//         file.seek(SeekFrom::Start(start)).await.map_err(|_| Status::InternalServerError)?;

//         let mut buffer = vec![0; chunk_size as usize];
//         file.read_exact(&mut buffer).await.map_err(|_| Status::InternalServerError)?;

//         Ok(Response::build()
//             .status(Status::PartialContent)
//             .header(ContentType::MP4)
//             .raw_header("Accept-Ranges", "bytes")
//             .raw_header("Content-Range", format!("bytes {}-{}/{}", start, end, file_size))
//             .sized_body(chunk_size, std::io::Cursor::new(buffer))
//             .finalize())
//     } else {
//         Err(Status::BadRequest)
//     }
// }

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![test_video])
}