use std::{borrow::Cow, fmt};

use rocket::{http::RawStr, request::FromParam};

use crate::dict::*;

pub fn create_id() -> String {
    use rand::seq::SliceRandom;

    let mut rng = rand::thread_rng();
    return DICT_ADJ.choose(&mut rng).unwrap().to_string()
        + &DICT_NOUN.choose(&mut rng).unwrap().to_string();
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PasteId<'a>(pub Cow<'a, str>);

impl PasteId<'_> {
    pub fn new(id: &str) -> Self {
        Self(Cow::Owned(id.to_string()))
    }
}

impl<'a> fmt::Display for PasteId<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn valid_id(id: &str) -> bool {
    let id = crate::file::get_without_extension(id);
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

impl<'a> FromParam<'a> for PasteId<'a> {
    type Error = &'a RawStr;

    fn from_param(param: &'a RawStr) -> Result<PasteId<'a>, &'a RawStr> {
        match valid_id(param) {
            true => Ok(PasteId(Cow::Borrowed(param))),
            false => Err(param),
        }
    }
}
