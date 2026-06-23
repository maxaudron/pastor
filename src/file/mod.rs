mod delete;
mod get;
// mod store;

pub use delete::*;
pub use get::*;
// pub use store::*;

use rocket::State;

use crate::ConfigState;

pub fn build_path(id: &str, config: &State<ConfigState>) -> std::path::PathBuf {
    std::path::Path::new(&config.app_config.storage_dir).join(&id)
}
