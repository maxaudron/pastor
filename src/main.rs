#![feature(proc_macro_hygiene, decl_macro, const_fn)]

extern crate multipart;

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
use rocket::fairing::AdHoc;
use rocket::http::hyper::header::{ContentDisposition, DispositionType};
use rocket::http::{ContentType, Status};
use rocket::Response;
use rocket::State;
use rocket_contrib::templates::Template;

extern crate tree_magic;

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::vec::Vec;

mod dict;
mod id;
mod upload;
mod util;

use util::HostHeader;

#[get("/")]
fn index(host: HostHeader) -> Template {
    let mut context = HashMap::<&str, &str>::new();
    context.insert("url", host.0);
    Template::render("index", context)
}

#[get("/<id>")]
fn get_file(id: String, config: State<ConfigState>) -> Result<Response, Status> {
    let id = id.split(".").collect::<Vec<&str>>()[0];
    let filename = Path::new(&config.storage_dir).join(&id);
    let file = File::open(&filename);
    match file {
        Err(_) => return Err(Status::NotFound),
        _ => (),
    };

    let file = file.unwrap();

    let mime_source = tree_magic::from_filepath(&filename);
    let mime;

    match mime_source {
        x if x.contains("text/") => mime = Some("text/plain; charset=utf-8"),
        _ => mime = None,
    }

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

pub struct ConfigState {
    storage_dir: String,
}

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                index,
                get_file,
                upload::upload_post_route,
                upload::upload_put_route
            ],
        )
        .attach(Template::fairing())
        .attach(AdHoc::on_attach("Set Config", |rocket| {
            println!("{:?}", rocket.config().limits);
            println!("Adding config to managed state...");
            let storage_dir = rocket.config().get_string("storage_dir").unwrap();
            Ok(rocket.manage(ConfigState { storage_dir }))
        }))
        .launch();
}
