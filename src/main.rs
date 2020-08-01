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

#[delete("/<id>")]
fn delete(id: String, config: State<ConfigState>) -> Result<Status, Status> {
    file::delete(file::build_path(id, config))
}

#[post("/", data = "<paste>")]
pub fn create(
    cont_type: &ContentType,
    paste: Data,
    config: State<crate::ConfigState>,
    host: HostHeader,
) -> Result<String, Status> {
    file::store(Some(cont_type), paste, config, host)
}

#[put("/<file>", data = "<paste>")]
#[allow(unused_variables)]
pub fn update(
    paste: Data,
    file: String,
    config: State<crate::ConfigState>,
    host: HostHeader,
) -> Result<String, Status> {
    file::store(None, paste, config, host)
}

pub struct ConfigState {
    storage_dir: String,
}

static INDEX_TEMPLATE: &str = include_str!("../templates/index.html.tera");

fn main() {
    rocket::ignite()
        .mount("/", routes![index, get, create, update,])
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

            let storage_dir = rocket.config().get_string("storage_dir").unwrap();

            Ok(rocket.manage(ConfigState { storage_dir }))
        }))
        .launch();
}
