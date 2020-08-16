#![feature(proc_macro_hygiene, decl_macro, const_fn)]
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

use std::collections::HashMap;
use std::vec::Vec;

use chrono::Utc;

mod dict;
mod file;
mod id;
mod util;

use util::HostHeader;

#[get("/")]
fn index(host: HostHeader) -> Template {
    let mut context = HashMap::<&str, &str>::new();
    context.insert("url", host.0);

    Template::render("index", context)
}

#[get("/<id>")]
fn get(id: String, config: State<ConfigState>) -> Result<Response, Status> {
    let paste = file::get_db(&id, &config.db)?;
    let now = Utc::now().timestamp();

    if paste.expires < now {
        file::delete(file::build_path(id, config))?;
        return Err(Status::Gone);
    }

    let (file, mime) = file::get(file::build_path(id, config))?;

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

    Ok(res)
}

#[delete("/<id>?<token>")]
fn delete(id: String, token: String, config: State<ConfigState>) -> Result<Status, Status> {
    let paste = file::get_db(&id, &config.db)?;

    if paste.token != token {
        return Err(Status::Forbidden);
    }

    file::delete(file::build_path(id, config))?;
    return Err(Status::Gone);
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
            "https://{host}/{id} {token}",
            host = host.0,
            id = id,
            token = token
        ))
    }
    Ok(urls.join("\n"))
}

// #[put("/<id>?<token>", data = "<data>")]
// pub fn update(
//     cont_type: &ContentType,
//     data: Data,
//     token: String,
//     config: State<crate::ConfigState>,
//     host: HostHeader,
// ) -> Result<(), Status> {
//     if !cont_type.is_form_data() {
//         return Err(Status::MethodNotAllowed);
//     }
//
//     println!("token: {:}", token);
//
//     file::update(cont_type, data, config)
// }

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

static INDEX_TEMPLATE: &str = include_str!("../templates/index.html.tera");

fn main() {
    rocket::ignite()
        .mount("/", routes![index, get, create, delete])
        .attach(Template::custom(|engine| {
            match std::env::var("ROCKET_INDEX_TEMPLATE") {
                Ok(template) => engine
                    .tera
                    .add_template_file(template, Some("index"))
                    .unwrap(),
                Err(_) => engine
                    .tera
                    .add_raw_template("index", INDEX_TEMPLATE)
                    .unwrap(),
            };
        }))
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
