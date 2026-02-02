use rocket::Route;

use super::file_api::{
    upload_photo,
    delete_photo
};

pub fn route_list() -> Vec<Route> {
    routes![
        upload_photo,
        delete_photo
    ]
}