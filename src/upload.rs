use rocket::Data;
use rocket::State;

use std::io;
use std::path::Path;

use crate::id;
use crate::HostHeader;

#[post("/", data = "<paste>")]
pub fn upload_post_route(
    paste: Data,
    config: State<crate::ConfigState>,
    host: HostHeader,
) -> io::Result<String> {
    upload(paste, config, host)
}

#[put("/<file>", data = "<paste>")]
#[allow(unused_variables)]
pub fn upload_put_route(
    paste: Data,
    file: String,
    config: State<crate::ConfigState>,
    host: HostHeader,
) -> io::Result<String> {
    upload(paste, config, host)
}

pub fn upload(
    paste: Data,
    config: State<crate::ConfigState>,
    host: HostHeader,
) -> io::Result<String> {
    let id = id::create_id();
    let filename = Path::new(&config.storage_dir).join(&id);
    let url = format!("http://{host}/{id}\n", host = host.0, id = id);

    // Write the paste out to the file and return the URL.
    paste.stream_to_file(Path::new(&filename))?;
    Ok(url)
}
