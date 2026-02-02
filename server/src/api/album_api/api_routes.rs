use rocket::Route;

use super::album_api::{
    create_photoset,
    update_photoset,
    delete_photoset
};

pub fn route_list() -> Vec<Route> {
    routes![
        create_photoset,
        update_photoset,
        delete_photoset
    ]
}