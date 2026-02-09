use log::warn;
use rocket::http::Status;
use std::path::Path;

pub fn root_guard_check<P: AsRef<Path>>(storage_root: P, full_path: P) -> Result<(), Status> {
    let root = storage_root.as_ref();
    let path = full_path.as_ref();

    if path == root || !path.starts_with(root) {
        warn!("Check Failed");
        return Err(Status::Forbidden);
    }

    Ok(())
}
