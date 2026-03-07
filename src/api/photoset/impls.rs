use super::models::{
    StreamedFile,
    FileServerResponse
};
use rocket::http::Status;
use rocket::response::{
    self,
    Responder
};
use rocket::{
    Request,
    Response
};


#[rocket::async_trait]
impl <'a> Responder<'a, 'static> for StreamedFile {
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
            .raw_header("Content-Range", format!("bytes {}-{}/{}", start, end, file_size))
            .sized_body(file_size as usize, file)
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