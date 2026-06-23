use std::{path::{Path, PathBuf}, sync::Arc};


#[derive(Debug, Clone)]
pub struct Tokens(Arc<tokio::sync::RwLock<Vec<String>>>);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TomlTokens {
    tokens: Vec<String>,
}

impl Tokens {
    pub fn new() -> Self {
        Tokens(Arc::new(tokio::sync::RwLock::new(Vec::new())))
    }

    pub async fn contains(&self, token: &str) -> bool {
        self.0.read().await.contains(&token.to_string())
    }

    pub async fn read(&mut self, path: &Path) {
        let file = tokio::fs::read_to_string(path).await.unwrap();
        let value: TomlTokens = toml::from_str(&file).unwrap();
        let mut s = self.0.write().await;
        s.clear();
        s.extend_from_slice(&value.tokens);
    }

    pub async fn refresh(mut self, path: PathBuf) {
        use inotify::{Inotify, WatchMask, StreamExt};

        let inotify = Inotify::init().expect("Error while initializing inotify instance");

        // Watch for modify and close events.
        inotify
            .watches()
            .add(&path, WatchMask::MODIFY | WatchMask::CLOSE)
            .expect("Failed to add file watch");

        // Read events that were added with `Watches::add` above.
        let buffer = [0; 1024];
        let mut events = inotify
            .into_event_stream(buffer)
            .expect("Error while reading events");

        while let Some(event_or_error) = events.next().await {
            tracing::debug!("event: {:?}", event_or_error.unwrap());
            self.read(&path).await
        }
    }
}
