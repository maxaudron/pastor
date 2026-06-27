use std::{
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncSeekExt},
};
use tracing::{debug, instrument};

use crate::id::PasteId;

mod error;
mod handle;

pub use error::*;
pub use handle::*;

const XATTR_CREATED: &str = "user.pastor.created";
const XATTR_EXPIRES: &str = "user.pastor.expires";
const XATTR_TOKEN: &str = "user.pastor.token";
const XATTR_MIME: &str = "user.pastor.mime";

#[derive(PartialEq, Debug)]
pub struct Paste {
    pub id: PasteId,
    pub created: i64,
    pub expires: i64,
    pub token: String,
    pub mime: String,
}

impl Paste {
    pub fn path(&self, root: &Path) -> PathBuf {
        self.id.path(root)
    }

    pub fn expired(&self) -> Result<bool, PasteError> {
        Ok(chrono::Utc::now() > chrono::DateTime::from_timestamp(self.expires, 0).ok_or(PasteError::Time)?)
    }

    pub async fn get_handle_create(path: &Path) -> Result<PasteHandle, PasteError> {
        Ok(OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .await?
            .into())
    }

    pub async fn get_handle(path: &Path) -> Result<PasteHandle, PasteError> {
        Ok(OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(path)
            .await?
            .into())
    }

    #[instrument(level = "debug", ret, err, skip(handle))]
    pub async fn from_handle(
        mut id: PasteId,
        mut handle: PasteHandle,
        token: &str,
    ) -> Result<Paste, PasteError> {
        let size = handle.metadata().await.unwrap().size();
        if size == 0 {
            return Err(PasteError::NoContent);
        }

        let created = chrono::Utc::now().timestamp();
        let expires = created + crate::util::expires(size);
        debug!("got file size: {size} will expire: {expires}");

        let mut mime_bytes: Vec<u8> = Vec::with_capacity(2048);
        handle.seek(std::io::SeekFrom::Start(0)).await?;
        handle
            .to_file()
            .take(2048)
            .read_to_end(&mut mime_bytes)
            .await
            .unwrap();

        let mime = crate::MAGIC.with(|magic| magic.buffer(&mime_bytes))?;
        let ext = crate::EXT.with(|magic| magic.buffer(&mime_bytes))?;
        debug!("got mime type: {:?} {:?}", mime, ext);
        if ext != "???" {
            id.ext = Some(ext.split_once("/").map(|(s, _)| s).unwrap_or(&ext).to_string())
        }

        Ok(Paste {
            id,
            created,
            expires,
            token: token.to_string(),
            mime,
        })
    }

    pub async fn write(&self, root: &Path) -> Result<(), PasteError> {
        let file = Paste::get_handle_create(&root.join(&self.id)).await?;
        file.set_xattr_i64(XATTR_CREATED, self.created).unwrap();
        file.set_xattr_i64(XATTR_EXPIRES, self.expires).unwrap();
        file.set_xattr_str(XATTR_TOKEN, &self.token).unwrap();
        file.set_xattr_str(XATTR_MIME, &self.mime).unwrap();

        Ok(())
    }

    pub async fn load(root: &Path, id: PasteId) -> Result<(Paste, PasteHandle), PasteError> {
        Paste::load_from_path(&root.join(&id), Some(id)).await
    }

    pub async fn load_from_path(
        path: &Path,
        id: Option<PasteId>,
    ) -> Result<(Paste, PasteHandle), PasteError> {
        debug!("loading paste from path: {path:?}");
        let file = Paste::get_handle(path).await?;
        let id = if let Some(id) = id {
            id
        } else {
            PasteId::try_from(path)?
        };
        let paste = Paste {
            id,
            created: file.get_xattr_i64(XATTR_CREATED)?,
            expires: file.get_xattr_i64(XATTR_EXPIRES)?,
            token: file.get_xattr_str(XATTR_TOKEN)?,
            mime: file.get_xattr_str(XATTR_MIME)?,
        };

        Ok((paste, file))
    }

    /// Delete the paste from storage if a matching auth token is supplied
    pub async fn delete(&self, root: &Path, token: Option<&str>) -> Result<(), PasteError> {
        if token == Some(&self.token) || !token.is_some() {
            Ok(tokio::fs::remove_file(self.path(root)).await?)
        } else {
            Err(PasteError::Unauthorized)
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use tracing_test::traced_test;

    use super::Paste;
    use crate::id::PasteId;

    fn paste() -> Paste {
        Paste {
            id: PasteId::new(),
            created: 17000000,
            expires: 18000000,
            token: "secrettoken".to_string(),
            mime: "text/plain".to_string(),
        }
    }

    #[tokio::test]
    #[traced_test]
    async fn write_load() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let paste = paste();

        paste.write(root).await.unwrap();
        assert!(root.join(&paste.id).exists());

        let (loaded, _handle) = Paste::load(root, paste.id.clone()).await.unwrap();
        assert_eq!(paste, loaded);
    }
}
