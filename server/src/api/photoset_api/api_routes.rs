use rocket::Route;

use super::api::{create_photoset, delete_photoset, update_photoset};

pub fn route_list() -> Vec<Route> {
    routes![create_photoset, update_photoset, delete_photoset]
}
