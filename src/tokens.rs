use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Tokens {
    path: PathBuf,
    tokens: Arc<tokio::sync::RwLock<Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TomlTokens {
    tokens: Vec<String>,
}

impl Tokens {
    pub async fn new(path: PathBuf) -> Self {
        let mut tokens = Tokens {
            path,
            tokens: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        };
        tokens.read().await;
        tokens
    }

    pub async fn contains(&self, token: &str) -> bool {
        self.tokens.read().await.contains(&token.to_string())
    }

    pub async fn read(&mut self) {
        let file = tokio::fs::read_to_string(&self.path).await.unwrap();
        let value: TomlTokens = toml::from_str(&file).unwrap();
        let mut s = self.tokens.write().await;
        s.clear();
        s.extend_from_slice(&value.tokens);
    }

    pub async fn refresh(mut self) {
        use inotify::{Inotify, StreamExt, WatchMask};

        let inotify = Inotify::init().expect("Error while initializing inotify instance");

        // Watch for modify and close events.
        inotify
            .watches()
            .add(&self.path, WatchMask::MODIFY | WatchMask::CLOSE_WRITE)
            .expect("Failed to add file watch");

        // Read events that were added with `Watches::add` above.
        let buffer = [0; 1024];
        let mut events = inotify
            .into_event_stream(buffer)
            .expect("Error while reading events");

        while let Some(event_or_error) = events.next().await {
            tracing::debug!("event: {:?}", event_or_error.unwrap());
            self.read().await
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Write, time::Duration};

    use anyhow::Error;
    use tempfile::NamedTempFile;
    use tokio::time::sleep;
    use tracing_test::traced_test;

    use super::{Tokens, TomlTokens};

    fn sample_tokens() -> TomlTokens {
        TomlTokens {
            tokens: ["testtoken", "supertoken", "anothertoken"]
                .iter()
                .map(|x| x.to_string())
                .collect(),
        }
    }

    #[tokio::test]
    #[traced_test]
    async fn initial_load() -> Result<(), Error> {
        let mut file = NamedTempFile::new()?;
        file.write_all(toml::to_string(&sample_tokens())?.as_bytes())?;

        let tokens = Tokens::new(file.path().into()).await;

        assert!(tokens.contains("testtoken").await);
        assert!(tokens.contains("supertoken").await);
        assert!(tokens.contains("anothertoken").await);

        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    async fn add_token() -> Result<(), Error> {
        let mut file = NamedTempFile::new()?;
        let mut toml_tokens = sample_tokens();
        file.write_all(toml::to_string(&toml_tokens)?.as_bytes())?;
        let tokens = Tokens::new(file.path().into()).await;

        let handle = tokio::spawn(tokens.clone().refresh());

        assert!(!tokens.contains("aftertoken").await);

        toml_tokens.tokens.push("aftertoken".to_string());
        tokio::fs::write(file.path(), toml::to_string(&toml_tokens)?.as_bytes()).await?;

        sleep(Duration::from_millis(10)).await;
        assert!(tokens.contains("aftertoken").await);

        handle.abort();
        Ok(())
    }
}
