use std::path::Path;

use rocket::data::ToByteUnit;
use tokio::fs::OpenOptions;
// use tokio::{io::Seek, path::Path};

use tokio::fs::File;

use rocket::form::Form;
use rocket::http::Status;
use rocket::State;

use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tracing::{error, trace};

use crate::id::PasteId;
use crate::Paste;

// pub fn update_multipart(
//     boundary: &str,
//     paste: Data,
//     config: State<crate::ConfigState>,
// ) -> Result<(), Status> {
//     let mut multipart = Multipart::with_body(paste.open(), boundary);

//     loop {
//         let data = multipart.read_entry().unwrap();
//         match data {
//             Some(mut entry) => {
//                 let id = &entry.headers.name;

//                 if id.contains("/") || id.contains("\\") {
//                     return Err(Status::NotAcceptable);
//                 }

//                 let mut file = update_file(&config, id).unwrap();

//                 entry
//                     .data
//                     .save()
//                     .size_limit(None)
//                     .memory_threshold(0)
//                     .write_to(&mut file);
//             }
//             None => break,
//         }
//     }

//     Ok(())
// }

// pub fn update(
//     cont_type: &ContentType,
//     paste: Data,
//     config: State<crate::ConfigState>,
// ) -> Result<(), Status> {
//     let (_, boundary) = cont_type
//         .params()
//         .find(|&(k, _)| k == "boundary")
//         .ok_or_else(|| Err::<String, Status>(Status::BadRequest))
//         .unwrap();

//     update_multipart(boundary, paste, config)
// }
//
// async fn update_file(config: &State<crate::ConfigState>, id: &str) -> Result<File, Status> {
//     let filename = Path::new(&config.app_config.storage_dir).join(id);

//     if !filename.exists() {
//         return Err(Status::NotFound);
//     }

//     let file = OpenOptions::new()
//         .write(true)
//         .open(&filename)
//         .await
//         .unwrap();

//     Ok(file)
// }

#[tracing::instrument(skip(data, config))]
pub async fn store<'a>(
    data: Form<Vec<crate::Bytes<'a>>>,
    config: &State<crate::ConfigState>,
) -> Result<Vec<Paste>, Status> {
    let mut pastes = Vec::new();

    for paste in data.into_inner() {
        trace!("storing file");
        let (mut file, id) = create_file(&config).await.unwrap();

        match paste {
            crate::Bytes::Value(v) => {
                file.write_all(v.as_bytes()).await.map_err(|_| {
                    error!("failed to save file to disk");
                    Status::InternalServerError
                })?;
            }
            crate::Bytes::Data(v) => {
                // TODO check if write complete and do something with that?
                let mut stream = v.open(10.gigabytes());

                tokio::io::copy(&mut stream, &mut file).await.map_err(|err| {
                    error!("failed to stream file to disk: {:?}", err);
                    Status::InternalServerError
                })?;

                // .stream_to(&mut file)
                // .await
                // .map_err(|_| {
                //     error!("failed to stream file to disk");
                //     Status::InternalServerError
                // })?;
            }
        };

        // Go back to beginning of file for us to be able to read it again
        file.seek(std::io::SeekFrom::Start(0)).await.map_err(|e| {
            error!("failed to seek file: {:?}", e);
            Status::InternalServerError
        })?;

        let paste = Paste::from_file(id, &mut file).await?;
        trace!("paste: {:?}", paste);
        store_db(&config.db, &paste);

        pastes.push(paste);
    }

    Ok(pastes)
}

pub(crate) async fn create_file(
    config: &State<crate::ConfigState>,
) -> Result<(File, PasteId), Status> {
    let id = PasteId::new();
    let filename = Path::new(&config.app_config.storage_dir).join(&id.id);

    if filename.exists() {
        return Err(Status::Conflict);
    };

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&filename)
        .await
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
