use rocket::form::FromForm;

#[derive(FromForm)]
pub struct PhotoSetUpdateForm<'r> {
    pub photoset_path: &'r str,
}
