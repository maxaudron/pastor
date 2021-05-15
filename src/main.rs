#![feature(proc_macro_hygiene, decl_macro)]
extern crate multipart;

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
use rocket::fairing::AdHoc;
use rocket::http::hyper::header::{ContentDisposition, DispositionType};
use rocket::http::{ContentType, Status};
use rocket::{Data, Response, State};
use rocket_contrib::templates::Template;

extern crate tree_magic;

use std::vec::Vec;

use chrono::Utc;

mod dict;
mod file;
mod id;
mod util;

use rocket_contrib::serve::StaticFiles;
use std::collections::HashMap;
use std::io::Read;
use util::HostHeader;

#[get("/")]
fn index(host: HostHeader) -> Template {
    let mut context: HashMap<&str, &str> = HashMap::new();
    context.insert("url", &host.0);
    Template::render("index", &context)
}

#[derive(Responder)]
enum GetReturnType<'a> {
    Response(Response<'a>),
    Template(Template),
}

#[get("/<id>?<lang>")]
fn retrieve(
    id: String,
    lang: Option<String>,
    config: State<ConfigState>,
) -> Result<GetReturnType, Status> {
    let paste = file::get_db(&id, &config.db)?;
    let now = Utc::now().timestamp();

    if paste.expires < now {
        file::delete(file::build_path(&id, &config))?;
        return Err(Status::Gone);
    }

    let (mut file, mime) = file::get(file::build_path(&id, &config))?;

    match lang {
        Some(l) if !l.is_empty() => {
            let mut buffer = String::new();
            // Could a better error be returned?
            file.read_to_string(&mut buffer)
                .map_err(|_| Status::ImATeapot)?;

            let mut context: HashMap<&str, String> = HashMap::new();
            context.insert("id", id.to_string());
            context.insert("lang", l);
            context.insert("content", buffer);

            let t = Template::render("retrieve", &context);
            Ok(GetReturnType::Template(t))
        }
        None | _ => {
            let mut res = Response::new();
            res.set_status(Status::Ok);
            res.set_header(ContentDisposition {
                disposition: DispositionType::Inline,
                parameters: vec![],
            });

            match mime {
                Some(m) => res.set_header(ContentType::parse_flexible(&m).unwrap()),
                None => false,
            };

            res.set_streamed_body(file);
            Ok(GetReturnType::Response(res))
        }
    }
}

#[delete("/<id>?<token>")]
fn delete(id: String, token: String, config: State<ConfigState>) -> Result<Status, Status> {
    let paste = file::get_db(&id, &config.db)?;

    if paste.token != token {
        return Err(Status::Forbidden);
    }

    file::delete(file::build_path(&id, &config))?;
    config.db.remove(&id).unwrap();
    return Ok(Status::Ok);
}

#[post("/?<token>", data = "<data>")]
pub fn create(
    cont_type: &ContentType,
    data: Data,
    token: Option<String>,
    config: State<crate::ConfigState>,
    host: HostHeader,
) -> Result<String, Status> {
    if !cont_type.is_form_data() {
        return Err(Status::MethodNotAllowed);
    }

    let ids = file::store(cont_type, data, config)?;

    let mut urls = Vec::new();
    for (id, token) in ids {
        urls.push(format!(
            "https://{host}/{id} {token}\n",
            host = host.0,
            id = id,
            token = token
        ))
    }
    Ok(urls.join("\n"))
}

pub struct ConfigState {
    storage_dir: String,
    db: sled::Db,
}

#[macro_use]
extern crate serde_derive;
extern crate bincode;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Paste {
    created: i64,
    expires: i64,
    token: String,
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, retrieve, create, delete])
        .mount(
            "/",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .attach(Template::fairing())
        .attach(AdHoc::on_attach("Set Config", |rocket| {
            println!("{:?}", rocket.config().limits);
            println!("Adding config to managed state...");

            let storage_dir = rocket
                .config()
                .get_string("storage_dir")
                .unwrap_or("/storage".to_string());
            let database_dir = rocket
                .config()
                .get_string("database_dir")
                .unwrap_or("/storage/db".to_string());

            let db = sled::open(database_dir).unwrap();

            Ok(rocket.manage(ConfigState { storage_dir, db }))
        }))
        .launch();
}
