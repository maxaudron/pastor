use std::fmt;

use serde::{Deserialize, Serialize};

use crate::dict::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PasteId {
    pub id: String,
    pub ext: Option<String>,
}

impl From<&str> for PasteId {
    fn from(input: &str) -> Self {
        let (id, ext) = if input.contains('.') {
            let parts = input.split('.').collect::<Vec<&str>>();
            let id_without = parts[0].to_string();
            let ext = parts[1].to_string();
            (id_without, Some(ext))
        } else {
            (input.to_string(), None)
        };

        Self { id, ext }
    }
}

impl PasteId {
    pub fn new() -> Self {
        use rand::seq::SliceRandom;

        let mut rng = rand::thread_rng();
        let id = DICT_ADJ.choose(&mut rng).unwrap().to_string()
            + &DICT_NOUN.choose(&mut rng).unwrap().to_string();

        Self { id, ext: None }
    }

    pub fn ext<'a>(&'a self) -> &'a str {
        self.ext.as_ref().map_or("", |s| s.as_str())
    }

    pub fn is_valid(&self) -> bool {
        self.id.chars().all(|c| c >= 'a' && c <= 'z') && self.id.len() <= 128
    }
}

impl fmt::Display for PasteId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.id)?;
        match &self.ext {
            Some(e) => {
                f.write_str(".")?;
                f.write_str(&e)?;
            }
            None => (),
        };

        Ok(())
    }
}

impl AsRef<std::path::Path> for PasteId {
    fn as_ref(&self) -> &std::path::Path {
        std::path::Path::new(&self.id)
    }
}
