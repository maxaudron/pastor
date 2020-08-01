use std::path::Path;

use rocket::http::{ContentType, Status};
use rocket::Data;
use rocket::State;

use multipart::server::Multipart;

use crate::id;
use crate::HostHeader;

pub fn store(
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

                        entry
                            .data
                            .save()
                            .size_limit(None)
                            .memory_threshold(0)
                            .with_path(filename);

                        urls.extend(vec![url]);
                    }
                })
                .unwrap();

            Ok::<String, Status>(urls.join(""))
        }
        _ => match paste.stream_to_file(&filename) {
            Ok(_) => Ok(url),
            Err(_) => Err(Status::InternalServerError),
        },
    }
}
