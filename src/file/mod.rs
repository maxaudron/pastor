mod delete;
mod get;
mod store;

pub use delete::*;
pub use get::*;
pub use store::*;

use rocket::State;

use crate::ConfigState;

pub fn get_without_extension(name: &str) -> String {
    if name.contains('.') {
        name.split('.').collect::<Vec<&str>>()[0].to_string()
    } else {
        name.to_string()
    }
}

pub fn build_path(id: &String, config: &State<ConfigState>) -> std::path::PathBuf {
    let id = get_without_extension(id);
    std::path::Path::new(&config.storage_dir).join(&id)
}
