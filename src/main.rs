#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate nanoid;

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
use rocket::fairing::AdHoc;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::FromRequest;
use rocket::response::{self, Content};
use rocket::Outcome;
use rocket::Request;
use rocket::State;
use rocket_contrib::templates::Template;
use std::path::PathBuf;

extern crate tree_magic;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;
use std::vec::Vec;

mod id;
mod upload;

pub struct HostHeader<'a>(pub &'a str);
impl<'a, 'r> FromRequest<'a, 'r> for HostHeader<'a> {
    type Error = ();

    fn from_request(request: &'a Request) -> rocket::request::Outcome<Self, Self::Error> {
        match request.headers().get_one("Host") {
            Some(h) => Outcome::Success(HostHeader(h)),
            None => Outcome::Forward(()),
        }
    }
}

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

    if mime.contains("text/") {
        mime = "text/plain".to_string()
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
            Ok(rocket.manage(ConfigState {
                storage_dir: storage_dir,
            }))
        }))
        .launch();
}
