use std::{path::Path, thread, time::Duration};

use chrono::{TimeZone, Utc};
use rocket::http::Status;
use tracing::debug;

pub fn delete(filename: std::path::PathBuf) -> Result<Status, Status> {
    match std::fs::remove_file(filename) {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[tracing::instrument]
pub fn cleanup_routine(storage_dir: &str, db: &sled::Db, interval_ms: u64) {
    debug!("Using deletion routine interval: {} ms", interval_ms);

    loop {
        thread::sleep(Duration::from_millis(interval_ms));

        cleanup(storage_dir, db);
    }
}

fn cleanup(storage_dir: &str, db: &sled::Db) {
    let now = Utc::now().timestamp();

    db.iter().filter_map(|s| s.ok()).for_each(|(k, v)| {
        let name: &str = std::str::from_utf8(&k).unwrap();
        let paste: crate::Paste = bincode::deserialize(&v).unwrap();

        if paste.expires < now {
            debug!(
                "Deleting: {}. (Expiration date: {}, Now: {})",
                name,
                Utc.timestamp(paste.expires, 0).to_string(),
                Utc.timestamp(now, 0).to_string(),
            );
            delete(Path::new(storage_dir).join(name)).unwrap();
            // This will actually remove it from the original database as well:
            db.remove(name).unwrap();
        }
    })
}

#[cfg(test)]
mod tests {
    use std::{
        fs::OpenOptions,
        io::{Seek, Write},
        path::Path,
        process::Termination,
    };

    use crate::{id::PasteId, Paste};

    use super::*;
    use anyhow::{anyhow, Result};

    use test::Bencher;

    fn create_paste(db: &sled::Db) -> Result<()> {
        let id = PasteId::new();
        let filename = Path::new("/tmp/pastor/storage").join(&id.id);

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&filename)
            .map_err(|_| anyhow!("failed to open file"))?;

        file.write_all("test content".as_bytes())?;

        file.seek(std::io::SeekFrom::Start(0))?;

        let mut paste =
            Paste::from_file(id, &mut file).map_err(|_| anyhow!("failed to create paste"))?;

        // paste.expires = Utc::now().timestamp() - 60;
        crate::file::store_db(db, &paste);

        Ok(())
    }

    fn populate_pastes(db: &sled::Db) -> Result<()> {
        for _ in 0..1000 {
            create_paste(db)?;
        }

        Ok(())
    }

    #[bench]
    fn bench_delete(b: &mut Bencher) -> impl Termination {
        let db = sled::open("/tmp/pastor/storage/db").unwrap();
        populate_pastes(&db).unwrap();

        b.iter(|| cleanup("/tmp/pastor/storage", &db));

        std::fs::remove_dir_all("/tmp/pastor/storage").unwrap();
    }
}
