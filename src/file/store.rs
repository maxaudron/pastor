use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;

use rocket::http::{ContentType, Status};
use rocket::Data;
use rocket::State;

use chrono::Utc;

use multipart::server::Multipart;

use crate::id;
use crate::Paste;

pub fn store_multipart(
    boundary: &str,
    paste: Data,
    config: State<crate::ConfigState>,
) -> Result<Vec<(String, String)>, Status> {
    let mut ids = Vec::new();

    let mut multipart = Multipart::with_body(paste.open(), boundary);

    loop {
        let data = multipart.read_entry().unwrap();
        match data {
            Some(mut entry) => {
                let (mut file, id) = create_file(&config).unwrap();

                entry
                    .data
                    .save()
                    .size_limit(None)
                    .memory_threshold(0)
                    .write_to(&mut file);

                let token = id::create_id();
                store_db(&config.db, &id, token.clone(), file);

                ids.push((id, token));
            }
            None => break,
        }
    }

    Ok(ids)
}

pub fn update_multipart(
    boundary: &str,
    paste: Data,
    config: State<crate::ConfigState>,
) -> Result<(), Status> {
    let mut multipart = Multipart::with_body(paste.open(), boundary);

    loop {
        let data = multipart.read_entry().unwrap();
        match data {
            Some(mut entry) => {
                let id = &entry.headers.name;

                if id.contains("/") || id.contains("\\") {
                    return Err(Status::NotAcceptable);
                }

                let mut file = update_file(&config, id).unwrap();

                entry
                    .data
                    .save()
                    .size_limit(None)
                    .memory_threshold(0)
                    .write_to(&mut file);
            }
            None => break,
        }
    }

    Ok(())
}

pub fn update(
    cont_type: &ContentType,
    paste: Data,
    config: State<crate::ConfigState>,
) -> Result<(), Status> {
    let (_, boundary) = cont_type
        .params()
        .find(|&(k, _)| k == "boundary")
        .ok_or_else(|| Err::<String, Status>(Status::BadRequest))
        .unwrap();

    update_multipart(boundary, paste, config)
}

pub fn store(
    cont_type: &ContentType,
    paste: Data,
    config: State<crate::ConfigState>,
) -> Result<Vec<(String, String)>, Status> {
    let (_, boundary) = cont_type
        .params()
        .find(|&(k, _)| k == "boundary")
        .ok_or_else(|| Err::<String, Status>(Status::BadRequest))
        .unwrap();

    store_multipart(boundary, paste, config)
}

fn update_file(config: &State<crate::ConfigState>, id: &str) -> Result<File, Status> {
    let filename = Path::new(&config.storage_dir).join(id);

    if !filename.exists() {
        return Err(Status::NotFound);
    }

    let file = OpenOptions::new().write(true).open(&filename).unwrap();

    Ok(file)
}

fn create_file(config: &State<crate::ConfigState>) -> Result<(File, String), Status> {
    let id = id::create_id();
    let filename = Path::new(&config.storage_dir).join(&id);

    if filename.exists() {
        return Err(Status::Conflict);
    };

    let file = File::create(&filename).unwrap();

    Ok((file, id))
}

fn store_db(db: &sled::Db, id: &str, token: String, file: File) {
    let size = file.metadata().unwrap().len();
    let now = Utc::now().timestamp();
    let expiry = now + crate::util::expires(size);

    db.insert(
        id,
        bincode::serialize(&Paste {
            created: now,
            expires: expiry,
            token,
        })
        .unwrap(),
    )
    .unwrap();
    db.flush().unwrap();
}
