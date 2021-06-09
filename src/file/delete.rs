use std::{fs, thread, time::Duration};

use chrono::Utc;
use rocket::http::Status;

use crate::file::get_db;

pub fn delete(filename: std::path::PathBuf) -> Result<Status, Status> {
    match std::fs::remove_file(filename) {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}

pub fn deletion_routine(storage_dir: &str, db: &sled::Db) {
    loop {
        thread::sleep(Duration::from_millis(10_000));
        println!("Running deletion check");

        let paths = fs::read_dir(storage_dir).unwrap();

        for (i, path) in paths.enumerate() {
            let file_path = path.as_ref().unwrap().path();
            let file_name = path.as_ref().unwrap().file_name();
            let file_name = file_name.to_str().unwrap();

            if file_name == "db" {
                continue;
            }

            let paste = get_db(&file_name, db).unwrap();

            let now = Utc::now().timestamp();
            if paste.expires < now {
            // if i % 2 == 0 {
            // if false {
                println!("Deleting: {}", file_name);
                delete(file_path).unwrap();
                // This will actually remove it from the original database as well:
                db.remove(&file_name).unwrap();
            }
        }
    }
}

