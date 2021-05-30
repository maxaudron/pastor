use std::fmt;

use rocket::{http::RawStr, request::FromParam};

use crate::dict::*;

pub fn create_id() -> String {
    use rand::seq::SliceRandom;

    let mut rng = rand::thread_rng();
    return DICT_ADJ.choose(&mut rng).unwrap().to_string()
        + &DICT_NOUN.choose(&mut rng).unwrap().to_string();
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PasteId {
    pub id: String,
    pub ext: Option<String>,
}

impl PasteId {
    pub fn new(input: &str) -> Self {
        let (id, ext) = if input.contains('.') {
            let parts = input.split('.').collect::<Vec<&str>>();
            let id_without = parts[0].to_string();
            let ext = parts[1].to_string();
            (id_without, Some(ext))
        } else {
            (input.to_string(), None)
        };

        Self {
            id,
            ext,
        }
    }

    pub fn ext<'a>(&'a self) -> &'a str {
        self.ext.as_ref().map_or("", |s| s.as_str())
    }
}

impl fmt::Display for PasteId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.id)?;
        match &self.ext {
            Some(e) => {
                f.write_str(".")?;
                f.write_str(&e)?;
            },
            None => (),
        };

        Ok(())
    }
}

fn id_is_valid(id: &str) -> bool {
    let func = |acc, it: &&'static str| if it.len() > acc { it.len() } else { acc };
    let adj_max_length =
        DICT_ADJ
            .iter()
            .fold(0, func);
    let noun_max_length =
        DICT_NOUN
            .iter()
            .fold(0, func);
    let max_length = adj_max_length + noun_max_length;
    id.chars().all(|c| c >= 'a' && c <= 'z') && id.len() <= max_length
}

impl<'a> FromParam<'a> for PasteId {
    type Error = &'a RawStr;

    fn from_param(param: &'a RawStr) -> Result<PasteId, &'a RawStr> {
        let paste_id = PasteId::new(param);
        match id_is_valid(&paste_id.id) {
            true => Ok(paste_id),
            false => Err(param),
        }
    }
}
