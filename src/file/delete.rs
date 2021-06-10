use std::{fs, thread, time::Duration};

use chrono::{TimeZone, Utc};
use rocket::http::Status;
use tracing::debug;

use crate::file::get_db;

pub fn delete(filename: std::path::PathBuf) -> Result<Status, Status> {
    match std::fs::remove_file(filename) {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[tracing::instrument]
pub fn deletion_routine(storage_dir: &str, db: &sled::Db, interval_ms: u64) {
    debug!("Using deletion routine interval: {} ms", interval_ms);

    loop {
        thread::sleep(Duration::from_millis(interval_ms));

        let paths = fs::read_dir(storage_dir).unwrap();

        for path in paths {
            let file_path = path.as_ref().unwrap().path();
            let file_name = path.as_ref().unwrap().file_name();
            let file_name = file_name.to_str().unwrap();

            if file_name == "db" {
                continue;
            }

            let paste = get_db(&file_name, db).unwrap();

            let now = Utc::now().timestamp();
            if paste.expires < now {
                debug!(
                    "Deleting: {}. (Expiration date: {}, Now: {})",
                    file_name,
                    Utc.timestamp(paste.expires, 0).to_string(),
                    Utc.timestamp(now, 0).to_string(),
                );
                delete(file_path).unwrap();
                // This will actually remove it from the original database as well:
                db.remove(&file_name).unwrap();
            }
        }
    }
}

