#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate rocket_contrib;

use rocket::fairing::AdHoc;
use rocket::State;
use rocket::Data;

use std::collections::HashMap;
use rocket_contrib::templates::{Template};

use std::io;
use std::path::{Path};

mod id;

#[get("/")]
fn index() -> Template {
    let mut context = HashMap::<String, String>::new();
    context.insert("url".to_string(), "http://localhost:8000".to_string());
    Template::render("index", context)
}

#[post("/", data = "<paste>")]
fn upload_post_route(paste: Data, config: State<ConfigState>) -> io::Result<String> {
    upload(paste, config)
}

#[put("/<file>", data = "<paste>")]
fn upload_put_route(paste: Data, file: String, config: State<ConfigState>) -> io::Result<String> {
    upload(paste, config)
}

fn upload(paste: Data, config: State<ConfigState>) -> io::Result<String> {
    let id = id::create_id();
    let filename = Path::new(&config.storage_path).join(&id);
    let url = format!("{host}/{id}\n", host = "http://localhost:8000", id = id);

    // Write the paste out to the file and return the URL.
    paste.stream_to_file(Path::new(&filename))?;
    Ok(url)
}

struct ConfigState {
    storage_path: String,
}

// #[derive(Serialize)]
// struct TemplateContext {
//     title: &'static str,
//     // This key tells handlebars which template is the parent.
//     parent: &'static str,
// }

fn main() {
    rocket::ignite()
        .mount("/", routes![index,upload_post_route,upload_put_route,rand])
        .attach(Template::fairing())
        .attach(AdHoc::on_attach("Set Config", |rocket| {
            println!("Adding config to managed state...");
            let storage_path = rocket.config().get_string("storage_path").unwrap();
            Ok(rocket.manage(ConfigState {storage_path: storage_path}))
        }))
        .launch();
}

