use rocket::Route;

use super::photoset_api::{create_photoset, delete_photoset, update_photoset};

pub fn route_list() -> Vec<Route> {
    routes![create_photoset, update_photoset, delete_photoset]
}
