use rocket::http::Status;
use std::fs::File;

pub fn get(filename: std::path::PathBuf) -> Result<(File, Option<&'static str>), Status> {
    let file = File::open(&filename);
    match file {
        Err(_) => return Err(Status::NotFound),
        _ => (),
    };

    let file = file.unwrap();

    let mime_source = tree_magic::from_filepath(&filename);
    let mime;

    match mime_source {
        x if x.contains("text/") => mime = Some("text/plain; charset=utf-8"),
        _ => mime = None,
    };

    Ok((file, mime))
}
