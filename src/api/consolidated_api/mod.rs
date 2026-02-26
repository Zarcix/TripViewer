use rocket::Route;

mod api;

mod impls;
mod models;
mod fs_helpers;
mod forms;

pub fn route_list() -> Vec<Route> {
    routes![
        api::list_photoset,
        api::create_photoset,
        api::update_photoset,
    ]
}