use std::path::{
    Component, Path, PathBuf
};

use rocket::fs::NamedFile;
use rocket::serde::json::Json;
use rocket::{Data};
use rocket::form::Form;
use rocket::http::{
    ContentType, Status
};

use crate::api::request_guards::{RangeHeader, UserAuth};
use crate::constants::server_constants::SERVE_PATH;

use super::fs_helpers;
use super::forms::PhotoSetUpdateForm;
use super::models::FileServerResponse;

fn resolve_photoset_path(short_path: &PathBuf) -> Result<PathBuf, Status> {
    let root = Path::new(
        SERVE_PATH
            .get()
            .ok_or(Status::InternalServerError)?
    );

    // Reject absolute paths immediately
    if short_path.is_absolute() {
        return Err(Status::Forbidden);
    }

    // Reject traversal components
    for component in short_path.components() {
        match component {
            Component::ParentDir => return Err(Status::Forbidden),
            Component::RootDir => return Err(Status::Forbidden),
            _ => {}
        }
    }

    Ok(root.join(short_path))
}


#[derive(Responder)]
pub enum MyResponder {
    #[response(status = 200)]
    FullHeaderContent(NamedFile, ContentType),

    #[response(status = 206)]
    RangedHeaderContent(NamedFile, ContentType),

    #[response(status = 200, content_type = "json")]
    DirectoryHeaderListing(Json<super::models::DirectoryListing>)
}

#[head("/<path..>")]
pub async fn head_photoset(path: PathBuf, _auth: UserAuth<'_>) -> Result<MyResponder, Status> {
    info!("Listing PhotoSets at {}", path.display());
    let photoset_path = resolve_photoset_path(&path)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    let res: FileServerResponse = if photoset_path.is_dir() {
        fs_helpers::parse_directory(&photoset_path, &path).await?
    } else if photoset_path.is_file() {
        fs_helpers::parse_file(photoset_path.clone(), None).await?
    } else {
        return Err(Status::NotFound);
    };

    println!("{:?}", res);

    let resp = match res {
    FileServerResponse::FullContent(named_file) => {
        // Get content type from extension
        let content_type = ContentType::from_extension(
            photoset_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("jpeg"),
        )
        .unwrap_or(ContentType::Binary);

        MyResponder::FullHeaderContent(named_file, content_type)
    }
    FileServerResponse::RangedContent(_) => {
        // Video/media content type
        let ext = photoset_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let content_type = ContentType::from_extension(&ext).unwrap_or(ContentType::Binary);
        let named = NamedFile::open(photoset_path).await.unwrap();
        MyResponder::RangedHeaderContent(named, content_type)
    }
    FileServerResponse::DirectoryListing(json_listing) => {
        MyResponder::DirectoryHeaderListing(json_listing)
    }
    };

    Ok(resp)
}

#[get("/<path..>")]
pub async fn list_photoset(path: PathBuf, range_header: Option<RangeHeader> ,_auth: UserAuth<'_>) -> Result<FileServerResponse, Status> {
    info!("Listing PhotoSets at {}", path.display());
    let photoset_path = resolve_photoset_path(&path)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    if photoset_path.is_dir() {
        return fs_helpers::parse_directory(&photoset_path, &path).await;
    }

    if photoset_path.is_file() {
        return fs_helpers::parse_file(photoset_path, range_header).await;
    }

    Err(Status::NotFound)
}

#[post("/<path..>")]
pub async fn create_photoset(path: PathBuf, _auth: UserAuth<'_>) -> Result<Status, Status> {
    info!("Creating PhotoSet at {}", path.display());

    let photoset_path = resolve_photoset_path(&path)?;

    fs_helpers::create_dir(&photoset_path).await?;

    Ok(Status::Created)
}

#[patch("/<path..>", data = "<form>")]
pub async fn update_photoset(path: PathBuf, form: Form<PhotoSetUpdateForm>, _auth: UserAuth<'_>) -> Result<Status, Status> {
    info!("Updating PhotoSet at {}", path.display());

    // Incoming Path Validation //
    let root = Path::new(
        SERVE_PATH
            .get()
            .ok_or(Status::InternalServerError)?
    );

    let photoset_path = resolve_photoset_path(&path)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    if !photoset_path.starts_with(&root) {
        error!("Invalid PhotoSet Path: photoset_path={}", photoset_path.display());
        return Err(Status::Forbidden);
    }

    // Form Validation //
    let new_name = form.new_name.trim();
    if new_name.is_empty() {
        return Err(Status::BadRequest);
    }

    let new_path = Path::new(&photoset_path.parent().ok_or(Status::InternalServerError)?).to_path_buf().join(new_name);
    let new_parent = new_path
        .parent()
        .ok_or(Status::InternalServerError)?
        .canonicalize()
        .map_err(|e| {
            error!("Could not canonicalize path. path={}, error={}", new_path.display(), e);
            Status::Forbidden
        })?;

    if !new_parent.starts_with(&root) {
        error!("Invalid Path. new_parent={}, root={}", new_parent.display(), root.display());
        return Err(Status::Forbidden);
    }

    fs_helpers::rename_entry(&photoset_path, &new_path).await?;

    Ok(Status::Accepted)
}

#[put("/<path..>", data = "<data>")]
pub async fn put_photoset(path: PathBuf, data: Data<'_>, _auth: UserAuth<'_>) -> Result<Status, Status> {
    let target_path = resolve_photoset_path(&path)?;

    let parent_path = target_path
        .parent()
        .ok_or(Status::BadRequest)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    if !parent_path.is_dir() {
        error!("Path to upload is not a directory. parent_path={}", parent_path.display());
        return Err(Status::BadRequest);
    }

    fs_helpers::save_data(data, &target_path).await?;

    Ok(Status::Created)
}

#[delete("/<path..>?<force_removal>")]
pub async fn delete_photoset(path: PathBuf, force_removal: bool, _auth: UserAuth<'_>) -> Result<Status, Status> {
    let target_path = resolve_photoset_path(&path)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    if target_path.is_dir() {
        fs_helpers::remove_photoset_dir(&target_path, force_removal).await?;
        return Ok(Status::NoContent);
    }

    if target_path.is_file() {
        fs_helpers::remove_photoset_file(&target_path).await?;
        return Ok(Status::NoContent);
    }

    Err(Status::BadRequest)
}
