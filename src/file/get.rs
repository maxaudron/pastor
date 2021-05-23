use crate::file;
use rocket::http::Status;
use std::fs::File;

pub fn get<'a>(filename: std::path::PathBuf) -> Result<File, Status> {
    let file = File::open(&filename);
    match file {
        Err(_) => return Err(Status::NotFound),
        _ => (),
    };

    let file = file.unwrap();

    Ok(file)
}

pub fn get_db(id: &String, db: &sled::Db) -> Result<crate::Paste, Status> {
    let id = file::get_without_extension(id);
    match db.get(id) {
        Ok(item) => match item {
            Some(item) => {
                let paste: crate::Paste = bincode::deserialize(&item).unwrap();
                Ok(paste)
            }
            None => Err(Status::NotFound),
        },
        Err(_) => Err(Status::InternalServerError),
    }
}
