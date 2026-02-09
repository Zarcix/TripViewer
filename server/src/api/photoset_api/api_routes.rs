use rocket::Route;

use super::api::{assign_photoset, create_photoset, delete_photoset, update_photoset};

pub fn route_list() -> Vec<Route> {
    routes![
        create_photoset,
        update_photoset,
        delete_photoset,
        assign_photoset
    ]
}
