use rocket::form::FromForm;

#[derive(FromForm)]
pub struct PhotoSetUpdateForm<'r> {
    pub new_name: Option<&'r str>,
}

#[derive(FromForm)]
pub struct PhotoSetPutForm<'r> {
    pub additions: Vec<&'r str>,
    pub removals: Vec<&'r str>,
}
