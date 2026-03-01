use std::path::{
    Component, Path, PathBuf
};

use rocket::Data;
use rocket::form::Form;
use rocket::http::{
    Status,
};

use crate::api::request_guards::UserAuth;
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
#[get("/<path..>")]
pub async fn list_photoset(path: PathBuf, _auth: UserAuth<'_>) -> Result<FileServerResponse, Status> {
    info!("Listing PhotoSets at {}", path.display());
    let photoset_path = resolve_photoset_path(&path)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

    if photoset_path.is_dir() {
        return fs_helpers::parse_directory(&photoset_path, &path).await;
    }

    if photoset_path.is_file() {
        return fs_helpers::parse_file(photoset_path).await;
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

    if !photoset_path.starts_with(root) {
        error!("Invalid PhotoSet Path: photoset_path={}", photoset_path.display());
        return Err(Status::Forbidden);
    }

    // Form Validation //
    let new_name = form.new_name.trim();

    if new_name.is_empty() {
        return Err(Status::BadRequest);
    }

    let parent = photoset_path
        .parent()
        .ok_or(Status::InternalServerError)?;
    let new_path = parent.join(new_name);

    // New Path Validation //
    if !new_path.starts_with(&root) {
        return Err(Status::Forbidden);
    }

    fs_helpers::rename_entry(&photoset_path, &new_path).await?;

    Ok(Status::Accepted)
}

#[put("/<path..>", data = "<data>")]
pub async fn put_photoset(path: PathBuf, data: Data<'_>, _auth: UserAuth<'_>) -> Result<Status, Status> {
    let target_path = resolve_photoset_path(&path)?;

    target_path
        .parent()
        .ok_or(Status::BadRequest)?
        .canonicalize()
        .map_err(|_| Status::NotFound)?;

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

#[cfg(test)]
mod test_endpoints {
    use ctor::ctor;

    use crate::constants::server_constants::STAGING_PATH;

    use super::*;

    const TEST_ROOT: &'static str = "/home/personal/Containers/manjaroarm/home/alarm/TripViewer/src/test/TestPhotoSet";

    // Existing PhotoSets
    const PHOTOSET_1: (&'static str, &'static str) = ("Set1", "Globe Spin Test.mp4");
    const PHOTOSET_2: (&'static str, &'static str) = ("Set2", "42502671.png");

    // Photoset To Add
    const PHOTOSET_3: (&'static str, &'static str) = ("Set3", "filetoadd.png");

    const INVALID_PHOTOSET_RELATIVE_BACK: &'static str = "../Set1";

    #[ctor]
    fn setup() {
        SERVE_PATH.get_or_init(|| String::from(TEST_ROOT));
        STAGING_PATH.get_or_init(|| String::from("/tmp"));
    }

    mod photopaths {
        use super::*;

        #[test]
        fn valid_photopath() {
            let mut test_path = PathBuf::new();
            test_path.push(PHOTOSET_2.0);

            let full_path = resolve_photoset_path(&test_path);

            assert!(full_path.is_ok());
            assert!(full_path.unwrap().to_str().unwrap() == format!("{}/{}", TEST_ROOT, PHOTOSET_2.0));
        }

        #[test]
        fn valid_empty_photopath() {
            let test_path = PathBuf::new();

            let full_path = resolve_photoset_path(&test_path);

            assert!(full_path.is_ok());
            assert!(full_path.unwrap().to_str().unwrap() == format!("{}/", TEST_ROOT));
        }

        #[test]
        fn valid_nested_path() {
            let test_path = PathBuf::from("a/b/c");

            let full_path = resolve_photoset_path(&test_path).unwrap();

            let expected = PathBuf::from(TEST_ROOT).join("a/b/c");

            assert_eq!(full_path, expected);
        }

        #[test]
        fn valid_path_with_curdir() {
            let test_path = PathBuf::from("./child_photoset");

            let full_path = resolve_photoset_path(&test_path).unwrap();

            let expected = PathBuf::from(TEST_ROOT).join("child_photoset");

            assert_eq!(full_path, expected);
        }

        #[test]
        fn invalid_photopath_relative_back() {
            let mut test_path = PathBuf::new();
            test_path.push(INVALID_PHOTOSET_RELATIVE_BACK);

            let full_path = resolve_photoset_path(&test_path);

            assert!(full_path.is_err());
            assert_eq!(full_path.err().unwrap(), Status::Forbidden);
        }

        #[test]
        fn invalid_photopath_absolute() {
            let test_path = PathBuf::from("/etc/passwd");

            let full_path = resolve_photoset_path(&test_path);

            assert!(full_path.is_err());
            assert_eq!(full_path.unwrap_err(), Status::Forbidden);
        }

        #[test]
        fn invalid_photopath_nested_traversal() {
            let test_path = PathBuf::from("a/b/../../c");

            let full_path = resolve_photoset_path(&test_path);

            assert!(full_path.is_err());
            assert_eq!(full_path.unwrap_err(), Status::Forbidden);
        }

        #[test]
        fn invalid_photopath_prefix_traversal() {
            let test_path = PathBuf::from("../");

            let full_path = resolve_photoset_path(&test_path);

            assert!(full_path.is_err());
            assert_eq!(full_path.unwrap_err(), Status::Forbidden);
        }

        #[test]
        fn invalid_photopath_rootdir_component() {
            // On Unix this produces a RootDir component.
            // On Windows this also produces RootDir (without Prefix).
            let test_path = PathBuf::from("/child_photoset");

            let full_path = resolve_photoset_path(&test_path);

            assert!(full_path.is_err());
            assert_eq!(full_path.unwrap_err(), Status::Forbidden);
        }

    }

    mod apis {
        use super::*;
        use rocket::tokio;

        mod list_photoset {

            use std::collections::HashSet;

            use crate::api::photoset::models::DirectoryEntry;

            use super::*;

            #[tokio::test]
            async fn test_list_parent_photoset() {
                let auth = UserAuth("");
                let path = PathBuf::new();

                let mut acting_dir = Path::new(SERVE_PATH.get().unwrap()).to_path_buf();
                acting_dir.push(&path);

                let res = list_photoset(path, auth).await;
                assert!(res.is_ok());

                let fs_response: FileServerResponse = res.unwrap();
                
                let listing = match fs_response {
                    FileServerResponse::DirectoryListing(listing) => listing,
                    other => panic!("Expected DirectoryListing, got {:?}", other),
                };

                // Collect actual filesystem entries
                let mut actual_entries = Vec::new();

                let mut dir = tokio::fs::read_dir(&acting_dir).await.unwrap();
                while let Some(entry) = dir.next_entry().await.unwrap() {
                    let metadata = entry.metadata().await.unwrap();

                    actual_entries.push(DirectoryEntry {
                        name: entry.file_name().to_string_lossy().to_string(),
                        is_dir: metadata.is_dir(),
                    });
                }

                // Convert both sides into sets (order independent comparison)
                let actual: HashSet<_> = actual_entries.into_iter().collect();
                let expected: HashSet<_> = listing.entries.clone().into_iter().collect();

                assert_eq!(actual, expected);
            }

            #[tokio::test]
            async fn test_list_child_photoset() {
                let auth = UserAuth("");
                let mut path = PathBuf::new();
                path.push(PHOTOSET_2.0);

                let mut acting_dir = Path::new(SERVE_PATH.get().unwrap()).to_path_buf();
                acting_dir.push(&path);

                let res = list_photoset(path, auth).await;
                assert!(res.is_ok());

                let fs_response: FileServerResponse = res.unwrap();
                
                let listing = match fs_response {
                    FileServerResponse::DirectoryListing(listing) => listing,
                    other => panic!("Expected DirectoryListing, got {:?}", other),
                };

                // Collect actual filesystem entries
                let mut actual_entries = Vec::new();

                let mut dir = tokio::fs::read_dir(&acting_dir).await.unwrap();
                while let Some(entry) = dir.next_entry().await.unwrap() {
                    let metadata = entry.metadata().await.unwrap();

                    actual_entries.push(DirectoryEntry {
                        name: entry.file_name().to_string_lossy().to_string(),
                        is_dir: metadata.is_dir(),
                    });
                }

                // Convert both sides into sets (order independent comparison)
                let actual: HashSet<_> = actual_entries.into_iter().collect();
                let expected: HashSet<_> = listing.entries.clone().into_iter().collect();

                assert_eq!(actual, expected);
            }

            #[tokio::test]
            async fn test_list_file_in_photoset() {
                let auth = UserAuth("");
                let mut path = PathBuf::new();
                path.push(PHOTOSET_2.0);
                path.push(PHOTOSET_2.1);

                let mut acting_dir = Path::new(SERVE_PATH.get().unwrap()).to_path_buf();
                acting_dir.push(&path);

                let res = list_photoset(path, auth).await;
                assert!(res.is_ok());

                let fs_response: FileServerResponse = res.unwrap();
                
                let listing = match fs_response {
                    FileServerResponse::FullContent(listing) => listing,
                    other => panic!("Expected FullContent, got {:?}", other),
                };

                assert_eq!(acting_dir.canonicalize().unwrap(), listing.path().to_path_buf());
            }

            #[tokio::test]
            async fn test_list_video_in_photoset() {
                let auth = UserAuth("");
                let mut path = PathBuf::new();
                path.push(PHOTOSET_1.0);
                path.push(PHOTOSET_1.1);

                let mut acting_dir = Path::new(SERVE_PATH.get().unwrap()).to_path_buf();
                acting_dir.push(&path);

                let res = list_photoset(path, auth).await;
                assert!(res.is_ok());

                let fs_response: FileServerResponse = res.unwrap();
                
                let listing = match fs_response {
                    FileServerResponse::RangedContent(listing) => listing,
                    other => panic!("Expected RangedContent, got {:?}", other),
                };

                assert_eq!(acting_dir.canonicalize().unwrap(), listing.file.to_path_buf());
            }

            #[tokio::test]
            async fn test_invalid_file_in_photoset() {
                let auth = UserAuth("");
                let mut path = PathBuf::new();
                path.push(PHOTOSET_1.0);
                path.push("Globe Spin Test.png");

                let mut acting_dir = Path::new(SERVE_PATH.get().unwrap()).to_path_buf();
                acting_dir.push(&path);

                let res = list_photoset(path, auth).await;
                assert!(res.is_err());
            }
        }

        mod create_photoset {
            use super::*;
            use rocket::tokio;

            async fn teardown() {
                let photoset_path = Path::new(PHOTOSET_3.0).to_path_buf();
                let full_path = resolve_photoset_path(&photoset_path).unwrap();
                std::fs::remove_dir_all(full_path).unwrap();
            }


            #[tokio::test]
            async fn test_create_photoset() {
                let auth = UserAuth("");
                let mut path = PathBuf::new();
                path.push(PHOTOSET_3.0);

                let res = create_photoset(path.clone(), auth).await;
                assert!(res.is_ok());

                let full_path = resolve_photoset_path(&path).unwrap();
                assert!(full_path.exists());
                teardown().await;
            }

            #[tokio::test]
            async fn test_create_existing_photoset() {
                let auth = UserAuth("");
                let mut path = PathBuf::new();
                path.push(PHOTOSET_2.0);


                let res = create_photoset(path.clone(), auth).await;
                assert!(res.is_ok());

                // Check that photoset and elements in photoset exist
                let mut full_path = resolve_photoset_path(&path).unwrap();
                assert!(full_path.exists());
                full_path.push(PHOTOSET_2.1);
            }
        }

        mod update_photoset {
            use super::*;

            async fn setup() {
                let auth = UserAuth("");
                let path = Path::new(PHOTOSET_3.0).to_path_buf();

                let _ = create_photoset(path.clone(), auth).await.unwrap();
            }

            async fn teardown() {
                let photoset_path = Path::new(PHOTOSET_3.0).to_path_buf();
                let full_path = resolve_photoset_path(&photoset_path).unwrap();
                std::fs::remove_dir_all(full_path).unwrap();
            }

            #[tokio::test]
            async fn test_update_photoset() {
                setup().await;

                let path = Path::new(PHOTOSET_3.0).to_path_buf();
                let update_form = PhotoSetUpdateForm {
                    new_name: "Test".to_string()
                };
                let to_test_res = update_photoset(path.clone(), update_form.into(), UserAuth("")).await;
                assert!(to_test_res.is_ok());

                
                let full_path = resolve_photoset_path(&Path::new("Test").to_path_buf()).unwrap();
                assert!(full_path.exists());


                let path = Path::new("Test").to_path_buf();
                let update_form = PhotoSetUpdateForm {
                    new_name: PHOTOSET_3.0.to_string()
                };
                let to_photoset3_res = update_photoset(path, update_form.into(), UserAuth("")).await;
                assert!(to_photoset3_res.is_ok());

                teardown().await;
            }

            #[tokio::test]
            async fn test_empty_new_photoset() {
                setup().await;

                let path = Path::new(PHOTOSET_3.0).to_path_buf();
                let update_form = PhotoSetUpdateForm {
                    new_name: "".to_string()
                };
                let to_test_res = update_photoset(path.clone(), update_form.into(), UserAuth("")).await;
                assert!(to_test_res.is_err(), "{:?}", to_test_res);
            }

            #[tokio::test]
            async fn test_invalid_new_photoset() {
                setup().await;

                let path = Path::new(PHOTOSET_3.0).to_path_buf();
                let update_form = PhotoSetUpdateForm {
                    new_name: "/etc/resolv.conf".to_string()
                };
                let to_test_res = update_photoset(path.clone(), update_form.into(), UserAuth("")).await;
                assert!(to_test_res.is_err(), "{:?}", to_test_res);
            }

            #[tokio::test]
            async fn test_existing_new_photoset() {
                setup().await;

                let path = Path::new(PHOTOSET_3.0).to_path_buf();
                let update_form = PhotoSetUpdateForm {
                    new_name: PHOTOSET_2.0.to_string()
                };
                let to_test_res = update_photoset(path.clone(), update_form.into(), UserAuth("")).await;
                assert!(to_test_res.is_err(), "{:?}", to_test_res);
            }
        }

        mod put_photoset {

        }

        mod delete_photoset {
            use super::*;
            static TEMP_FILE_NAME: &'static str = "tempfile";

            async fn setup() {
                let path = Path::new(PHOTOSET_3.0).to_path_buf();
                let _ = create_photoset(path.clone(), UserAuth("")).await.unwrap();

                let mut full_path = resolve_photoset_path(&path).unwrap();
                full_path.push(TEMP_FILE_NAME);
                tokio::fs::File::create(full_path).await.unwrap();
            }

            async fn teardown() {
                let photoset_path = Path::new(PHOTOSET_3.0).to_path_buf();
                let full_path = resolve_photoset_path(&photoset_path).unwrap();
                std::fs::remove_dir_all(full_path).unwrap();
            }

            #[tokio::test]
            async fn delete_valid_photoset() {
                setup().await;

                let path = Path::new(PHOTOSET_3.0).to_path_buf();
                let force_removal = true;

                let res = delete_photoset(path, force_removal, UserAuth("")).await;
                assert!(res.is_ok());
            }

            #[tokio::test]
            async fn delete_nonexistent_photoset() {
                setup().await;

                let path = Path::new("FakePhotoSetNotReal").to_path_buf();
                let force_removal = false;

                let res = delete_photoset(path, force_removal, UserAuth("")).await;
                assert!(res.is_err_and(|status| status.code == 404));
            }

            #[tokio::test]
            async fn delete_used_photoset() {
                setup().await;

                let path = Path::new(PHOTOSET_3.0).to_path_buf();
                let force_removal = false;

                let res = delete_photoset(path, force_removal, UserAuth("")).await;
                assert!(res.is_err_and(|status| status.code == 409));

                teardown().await;
            }

            #[tokio::test]
            async fn delete_file_in_photoset() {
                setup().await;

                let mut path = Path::new(PHOTOSET_3.0).to_path_buf();
                path.push(TEMP_FILE_NAME);

                let force_removal = false;

                let res = delete_photoset(path, force_removal, UserAuth("")).await;
                assert!(res.is_ok());

                teardown().await;
            }
        }
    }
}
