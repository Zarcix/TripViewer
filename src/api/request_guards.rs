use rocket::http::{HeaderMap, Status};
use rocket::request::{FromRequest, Outcome, Request};

use crate::constants::server_constants::API_KEY;

pub struct UserAuth;

pub struct RequestHeaders<'r> {
    headermap: &'r HeaderMap<'r>,
}

#[derive(Debug)]
pub struct RangeHeader {
    pub start: Option<u64>,
    pub end: Option<u64>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAuth {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        /// Returns true if `key` is a valid API key string.
        fn is_valid(key: &str) -> bool {
            key == API_KEY.get().unwrap_or(&String::new())
        }
        match req.cookies().get("auth") {
            Some(key) if is_valid(key.value()) => Outcome::Success(UserAuth),
            Some(_) => {
                error!("Invalid User Credentials");
                Outcome::Error((Status::Unauthorized, ()))
            }
            None => {
                error!("No User Auth Provided");
                Outcome::Error((Status::Unauthorized, ()))
            }
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestHeaders<'r> {
    type Error = Status;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(RequestHeaders {
            headermap: req.headers(),
        })
    }
}

impl RequestHeaders<'_> {
    pub fn extract_range_header(&self) -> Option<RangeHeader> {
        let value = self.headermap.get_one("Range")?;

        let range_str = value.strip_prefix("bytes=")?;

        if range_str.contains(',') {
            return None;
        }

        let mut parts = range_str.splitn(2, '-');

        let start_str = parts.next().unwrap();
        let end_str = parts.next().unwrap_or("");

        let start = if start_str.is_empty() {
            None
        } else {
            match start_str.parse::<u64>() {
                Ok(v) => Some(v),
                Err(_) => return None,
            }
        };

        let end = if end_str.is_empty() {
            None
        } else {
            match end_str.parse::<u64>() {
                Ok(v) => Some(v),
                Err(_) => return None,
            }
        };

        if start.is_none() && end.is_none() {
            return None;
        }

        Some(RangeHeader { start, end })
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
