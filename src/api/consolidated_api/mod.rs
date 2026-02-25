use rocket::Route;

mod api;
mod impls;
mod models;
mod fs_helpers;

pub fn route_list() -> Vec<Route> {
    routes![
        api::list_photoset
    ]
}