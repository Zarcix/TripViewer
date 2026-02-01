use rocket::Route;

use super::file_api::{
    get_photos,
    upload_photo,
    delete_photo
};

pub fn route_list() -> Vec<Route> {
    routes![
        get_photos,
        upload_photo,
        delete_photo
    ]
}