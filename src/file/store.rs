use std::fs::File;
use std::fs::OpenOptions;
use std::{io::Seek, path::Path};

use rocket::http::{ContentType, Status};
use rocket::Data;
use rocket::State;

use multipart::server::Multipart;
use tracing::{error, trace};

use crate::id::PasteId;
use crate::Paste;

pub fn store_multipart(
    boundary: &str,
    paste: Data,
    config: &State<crate::ConfigState>,
) -> Result<Vec<Paste>, Status> {
    let mut pastes = Vec::new();

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

                // Go back to beginning of file for us to be able to read it again
                file.seek(std::io::SeekFrom::Start(0)).map_err(|e| {
                    error!("failed to seek file: {:?}", e);
                    Status::InternalServerError
                })?;

                let paste = Paste::from_file(id, &mut file)?;
                trace!("paste: {:?}", paste);
                store_db(&config.db, &paste);

                pastes.push(paste);
            }
            None => break,
        }
    }

    Ok(pastes)
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
    config: &State<crate::ConfigState>,
) -> Result<Vec<Paste>, Status> {
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

pub(crate) fn create_file(config: &State<crate::ConfigState>) -> Result<(File, PasteId), Status> {
    let id = PasteId::new();
    let filename = Path::new(&config.storage_dir).join(&id.id);

    if filename.exists() {
        return Err(Status::Conflict);
    };

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&filename)
        .map_err(|e| {
            error!("failed to create file: {:?}", e);
            Status::InternalServerError
        })?;

    Ok((file, id))
}

pub(crate) fn store_db(db: &sled::Db, paste: &Paste) {
    db.insert(&paste.id.id, bincode::serialize(&paste).unwrap())
        .unwrap();
    db.flush().unwrap();
}
