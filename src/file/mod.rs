mod delete;
mod get;
mod store;

pub use delete::*;
pub use get::*;
pub use store::*;

use rocket::State;

use crate::ConfigState;
use crate::dict::DICT_MIME_EXT;
use mime_guess::Mime;
use std::str::FromStr;

pub fn get_without_extension(name: &str) -> String {
    if name.contains('.') {
        name.split('.').collect::<Vec<&str>>()[0].to_string()
    } else {
        name.to_string()
    }
}

pub fn build_path(id: &str, config: &State<ConfigState>) -> std::path::PathBuf {
    let id = get_without_extension(id);
    std::path::Path::new(&config.storage_dir).join(&id)
}

pub fn get_ext_from_id(id: &str, config: &State<ConfigState> ) -> Option<String> {
    let mime = tree_magic::from_filepath(&build_path(id, config));
    println!("mime {:?}", mime);

    // 1. Check if our well-known mime types have an entry
    match DICT_MIME_EXT.get(mime) {
        Some(ext) => Some(String::from(".") + ext),
        None => {
            // 2. Check if mime_guess returns exactly one result
            match mime_guess::get_mime_extensions(&Mime::from_str(mime).unwrap()) {
                Some(guesses) if guesses.len() == 1 => {
                    Some(String::from(".") + guesses[0])
                },
                _ => {
                    // 3. If all fails, use no extension at all
                    None
                }
            }
        },
    }
}
