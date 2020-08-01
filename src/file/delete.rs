use rocket::http::Status;

pub fn delete(filename: std::path::PathBuf) -> Result<Status, Status> {
    match std::fs::remove_file(filename) {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}
