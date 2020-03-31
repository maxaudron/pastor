use std::path::Path;

use rocket::http::{ContentType, Status};
use rocket::Data;
use rocket::State;

use multipart::server::Multipart;

use crate::id;
use crate::HostHeader;

#[post("/", data = "<paste>")]
pub fn upload_post_route(
    cont_type: &ContentType,
    paste: Data,
    config: State<crate::ConfigState>,
    host: HostHeader,
) -> Result<String, Status> {
    upload(Some(cont_type), paste, config, host)
}

#[put("/<file>", data = "<paste>")]
#[allow(unused_variables)]
pub fn upload_put_route_content_type(
    cont_type: &ContentType,
    paste: Data,
    file: String,
    config: State<crate::ConfigState>,
    host: HostHeader,
) -> Result<String, Status> {
    upload(Some(cont_type), paste, config, host)
}

#[put("/<file>", data = "<paste>")]
#[allow(unused_variables)]
pub fn upload_put_route(
    paste: Data,
    file: String,
    config: State<crate::ConfigState>,
    host: HostHeader,
) -> Result<String, Status> {
    upload(None, paste, config, host)
}

pub fn upload(
    cont_type: Option<&ContentType>,
    paste: Data,
    config: State<crate::ConfigState>,
    host: HostHeader,
) -> Result<String, Status> {
    let id = id::create_id();
    let filename = Path::new(&config.storage_dir).join(&id);
    let url = format!("http://{host}/{id}\n", host = host.0, id = id);

    match cont_type {
        Some(cont_type) if cont_type.is_form_data() => {
            let (_, boundary) = cont_type
                .params()
                .find(|&(k, _)| k == "boundary")
                .ok_or_else(|| Err::<String, Status>(Status::BadRequest))
                .unwrap();

            let mut urls = Vec::new();
            Multipart::with_body(paste.open(), boundary)
                .foreach_entry({
                    |mut entry| {
                        let id = id::create_id();
                        let filename = Path::new(&config.storage_dir).join(&id);
                        let url = format!("http://{host}/{id}\n", host = host.0, id = id);

                        entry.data.save().memory_threshold(0).with_path(filename);

                        urls.extend(vec![url]);
                    }
                })
                .unwrap();

            Ok::<String, Status>(urls.join(""))
        }
        _ => {
            println!("AAAAAAA");
            match paste.stream_to_file(&filename) {
                Ok(_) => Ok(url),
                Err(_) => Err(Status::InternalServerError),
            }
        }
    }
}
