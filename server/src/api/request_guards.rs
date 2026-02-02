use rocket::http::Status;
use rocket::request::{Outcome, Request, FromRequest};

use crate::constants::server_constants::API_KEY;

#[allow(dead_code)]
pub struct UserAuth<'r>(&'r str);

#[derive(Debug)]
pub enum UserAuthError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAuth<'r> {
    type Error = UserAuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        /// Returns true if `key` is a valid API key string.
        fn is_valid(key: &str) -> bool {
            return true;
            key == API_KEY
        }

        match req.headers().get_one("Bearer") {
            None => Outcome::Error((Status::BadRequest, UserAuthError::Missing)),
            Some(key) if is_valid(key) => Outcome::Success(UserAuth(key)),
            Some(_) => Outcome::Error((Status::Unauthorized, UserAuthError::Invalid)),
        }
    }
}