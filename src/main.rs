#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate nanoid;

extern crate multipart;

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
use rocket::fairing::AdHoc;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::response::Content;
use rocket::State;
use rocket_contrib::templates::Template;

extern crate tree_magic;

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::vec::Vec;

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
fn get_file(id: String, config: State<ConfigState>) -> Result<Content<File>, Status> {
    let id = id.split(".").collect::<Vec<&str>>()[0];
    let filename = Path::new(&config.storage_dir).join(&id);
    let file = File::open(&filename);
    match file {
        Err(_) => return Err(Status::NotFound),
        _ => (),
    };

    let mut mime = tree_magic::from_filepath(&filename);

    match mime {
        x if x.contains("text/") => mime = "text/plain",
        _ => {}
    }

    Ok(Content(
        ContentType::parse_flexible(&mime).unwrap(),
        file.unwrap(),
    ))
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
            println!("Adding config to managed state...");
            let storage_dir = rocket.config().get_string("storage_dir").unwrap();
            Ok(rocket.manage(ConfigState { storage_dir }))
        }))
        .launch();
}
