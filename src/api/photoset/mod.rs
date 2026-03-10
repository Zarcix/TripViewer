use rocket::Route;

mod api;

mod forms;
mod fs_helpers;
mod impls;
mod models;

pub fn route_list() -> Vec<Route> {
    routes![
        api::list_photoset,
        api::create_photoset,
        api::update_photoset,
        api::put_photoset,
        api::delete_photoset,
    ]
}
