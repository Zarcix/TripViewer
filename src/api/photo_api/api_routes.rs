use rocket::Route;

use super::api::{delete_photo, upload_photo};

pub fn route_list() -> Vec<Route> {
    routes![upload_photo, delete_photo]
}
