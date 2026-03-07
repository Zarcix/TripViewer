use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use crate::constants::server_constants::API_KEY;


#[allow(dead_code)]
pub struct UserAuth<'r>(pub &'r str);

#[allow(dead_code)]
pub struct RangeHeader {
    pub start: Option<u64>,
    pub end: Option<u64>,
}

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
        match req.cookies().get("auth") {
            Some(key) if is_valid(key.value()) => {
                Outcome::Success(UserAuth(key.value()))
            }
            Some(_) => {
                error!("Invalid User Credentials");
                Outcome::Error((Status::Unauthorized, UserAuthError::Invalid))
            }
            None => {
                error!("No User Auth Provided");
                Outcome::Error((Status::Unauthorized, UserAuthError::Invalid))
            }
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RangeHeader {
    type Error = Status;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let value = match req.headers().get_one("Range") {
            Some(v) => v,
            None => return Outcome::Forward(Status::BadRequest),
        };

        let range_str = match value.strip_prefix("bytes=") {
            Some(v) => v,
            None => return Outcome::Error((Status::BadRequest, Status::BadRequest)),
        };

        if range_str.contains(',') {
            return Outcome::Error((Status::BadRequest, Status::RangeNotSatisfiable));
        }

        let mut parts = range_str.splitn(2, '-');

        let start_str = parts.next().unwrap();
        let end_str = parts.next().unwrap_or("");

        let start = if start_str.is_empty() {
            None
        } else {
            match start_str.parse::<u64>() {
                Ok(v) => Some(v),
                Err(_) => return Outcome::Error((Status::BadRequest, Status::BadRequest)),
            }
        };

        let end = if end_str.is_empty() {
            None
        } else {
            match end_str.parse::<u64>() {
                Ok(v) => Some(v),
                Err(_) => return Outcome::Error((Status::BadRequest, Status::BadRequest)),
            }
        };

        if start.is_none() && end.is_none() {
            return Outcome::Error((Status::BadRequest, Status::BadRequest));
        }

        Outcome::Success(RangeHeader { start, end })
    }
}

impl RangeHeader {
    pub fn resolve(&self, len: u64) -> Option<(u64, u64)> {
        match (self.start, self.end) {
            (Some(s), Some(e)) if s <= e && e < len => Some((s, e)),
            (Some(s), None) if s < len => Some((s, len - 1)),
            (None, Some(e)) if e != 0 => Some((len.saturating_sub(e), len - 1)),
            _ => None,
        }
    }
}