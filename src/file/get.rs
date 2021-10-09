use rocket::http::Status;
use tokio::fs::File;

pub async fn get(filename: std::path::PathBuf) -> Result<File, Status> {
    let file = File::open(&filename).await;
    match file {
        Err(_) => return Err(Status::NotFound),
        _ => (),
    };

    let file = file.unwrap();

    Ok(file)
}

pub fn get_db(id: &str, db: &sled::Db) -> Result<crate::Paste, Status> {
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
