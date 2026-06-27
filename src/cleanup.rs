use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use tracing::{debug, error};

use crate::file::{Paste, PasteError};

pub async fn cleanup_routine(storage: PathBuf) {
    let mut ticker = tokio::time::interval(Duration::from_secs(60));
    loop {
        ticker.tick().await;
        if let Err(err) = cleanup(&storage).await {
            error!("failure during cleanup: {err}");
        }
    }
}

pub async fn cleanup(storage: &Path) -> Result<(), PasteError> {
    for entry in std::fs::read_dir(storage)? {
        let path = entry?.path();
        let (paste, _) = Paste::load_from_path(&path, None).await?;
        if paste.expired()? {
            debug!(
                "Deleting: {paste:?}. (Now: {})",
                chrono::Utc::now().timestamp()
            );
            tokio::fs::remove_file(path).await?
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tempfile::tempdir;
    use tracing_test::traced_test;

    use crate::{file::Paste, id::PasteId};

    async fn write_paste(storage: &Path, expires: i64) -> Paste {
        let now = chrono::Utc::now().timestamp();
        let paste = Paste {
            id: PasteId::new(),
            created: now,
            expires: now + expires,
            token: "secrettoken".to_string(),
            mime: "text/none".to_string(),
        };

        paste.write(storage).await.unwrap();

        paste
    }

    #[tokio::test]
    #[traced_test]
    async fn cleanup() {
        let dir = tempdir().unwrap();
        let storage = dir.path();
        let paste1 = write_paste(storage, 0).await;
        let paste2 = write_paste(storage, 60).await;

        crate::cleanup::cleanup(storage).await.unwrap();
        assert!(!paste1.path(storage).exists());
        assert!(paste2.path(storage).exists());
    }
}
