mod delete;
mod get;
mod store;

pub use delete::*;
pub use get::*;
pub use store::*;

use rocket::State;

use crate::ConfigState;

pub fn build_path(id: &String, config: &State<ConfigState>) -> std::path::PathBuf {
    let id = id.split(".").collect::<Vec<&str>>()[0];
    std::path::Path::new(&config.storage_dir).join(&id)
}
