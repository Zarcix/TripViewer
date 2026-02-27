use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use crate::constants::server_constants::API_KEY;


#[allow(dead_code)]
pub struct UserAuth<'r>(&'r str);

#[allow(dead_code)]
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
            key == API_KEY.get().unwrap_or(&String::new())
        }
        match req.cookies().get("Badge") {
            None => Outcome::Error((Status::BadRequest, UserAuthError::Missing)),
            Some(key) if is_valid(key.value()) => Outcome::Success(UserAuth(key.value())),
            Some(_) => Outcome::Error((Status::Unauthorized, UserAuthError::Invalid)),
        }
    }
}
