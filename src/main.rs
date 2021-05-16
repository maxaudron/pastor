#![feature(proc_macro_hygiene, decl_macro)]
extern crate multipart;

#[macro_use]
extern crate rocket;
use rocket::fairing::AdHoc;
use rocket::http::hyper::header::{ContentDisposition, DispositionType};
use rocket::http::{ContentType, Status};
use rocket::{Data, Response, State};

extern crate tree_magic;

use std::vec::Vec;

use chrono::Utc;

mod dict;
mod file;
mod id;
mod util;

use rocket::response::Body;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use tera::Tera;
use util::HostHeader;

#[get("/")]
fn index<'a>(host: HostHeader, config: State<ConfigState>) -> Result<Response<'a>, Status> {
    let mut context = tera::Context::new();
    context.insert("url", host.0);
    let rendered_template = config.tera.render("index", &context).unwrap();

    let mut res = Response::new();
    res.set_status(Status::Ok);
    res.set_header(ContentType::HTML);
    let size = rendered_template.len() as u64;
    let body = Body::Sized(Cursor::new(rendered_template), size);
    res.set_raw_body(body);
    Ok(res)
}

#[get("/static/<path..>")]
fn static_file<'a>(path: PathBuf) -> Option<Response<'a>> {
    let mut res = Response::new();
    res.set_status(Status::Ok);

    match path.to_str() {
        Some("styles/main.css") => {
            res.set_header(ContentType::CSS);
            let size = MAIN_CSS.len() as u64;
            let body = Body::Sized(Cursor::new(MAIN_CSS), size);
            res.set_raw_body(body);
            Some(res)
        }
        _ => None,
    }
}

#[get("/<id>?<lang>")]
fn retrieve(
    id: String,
    lang: Option<String>,
    config: State<ConfigState>,
) -> Result<Response, Status> {
    let paste = file::get_db(&id, &config.db)?;
    let now = Utc::now().timestamp();

    if paste.expires < now {
        file::delete(file::build_path(&id, &config))?;
        return Err(Status::Gone);
    }

    let (mut file, mime) = file::get(file::build_path(&id, &config))?;

    let mut res = Response::new();
    res.set_status(Status::Ok);
    res.set_header(ContentDisposition {
        disposition: DispositionType::Inline,
        parameters: vec![],
    });

    match lang {
        Some(l) if !l.is_empty() => {
            let mut buffer = String::new();
            // Could a better error be returned?
            file.read_to_string(&mut buffer)
                .map_err(|_| Status::InternalServerError)?;

            let language = l[..1].to_uppercase() + &l.to_lowercase()[1..];
            let syntax = config
                .syntax_set
                .find_syntax_by_name(&language)
                .unwrap_or_else(|| {
                    config
                        .syntax_set
                        .find_syntax_by_first_line(&buffer)
                        .unwrap_or_else(|| config.syntax_set.find_syntax_plain_text())
                });
            let html = syntect::html::highlighted_html_for_string(
                &buffer,
                &config.syntax_set,
                syntax,
                &config.theme_set.themes["base16-eighties.dark"],
            );

            let mut context = tera::Context::new();
            context.insert("id", &id);
            context.insert("lang", &l);
            context.insert("content", &html);
            let rendered_template = config.tera.render("retrieve", &context).unwrap();

            res.set_header(ContentType::HTML);
            let size = rendered_template.len() as u64;
            let body = Body::Sized(Cursor::new(rendered_template), size);
            res.set_raw_body(body);
            Ok(res)
        }
        None | _ => {
            match mime {
                Some(m) => res.set_header(ContentType::parse_flexible(&m).unwrap()),
                None => false,
            };

            res.set_streamed_body(file);
            Ok(res)
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
    tera: Tera,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
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

const BASE_TEMPLATE: &str = include_str!("../templates/base.html.tera");
const INDEX_TEMPLATE: &str = include_str!("../templates/index.html.tera");
const RETRIEVE_TEMPLATE: &str = include_str!("../templates/retrieve.html.tera");

const MAIN_CSS: &str = include_str!("../static/styles/main.css");

fn main() {
    rocket::ignite()
        .mount("/", routes![index, retrieve, create, delete, static_file])
        .attach(AdHoc::on_attach("Set Config", |rocket| {
            println!("{:?}", rocket.config().limits);
            println!("Adding config to managed state...");

            let storage_dir = rocket
                .config()
                .get_string("storage_dir")
                .unwrap_or("storage".to_string());
            let database_dir = rocket
                .config()
                .get_string("database_dir")
                .unwrap_or("storage/db".to_string());

            let db = sled::open(database_dir).unwrap();

            let template_dir = rocket
                .config()
                .get_string("template_dir")
                .unwrap_or("/templates/*".to_string());

            let mut tera = Tera::parse(&template_dir).unwrap();

            match std::env::var("ROCKET_INDEX_TEMPLATE") {
                Ok(template) => {
                    println!("Using external template");
                    // TODO: Letting user specify both the template dir and specific
                    //  template files seems unnecessary?
                    tera.add_template_file(template, Some("index")).unwrap();
                }
                Err(_) => {
                    println!("Using embedded template");
                    tera.add_raw_templates(vec![
                        ("base", BASE_TEMPLATE),
                        ("index", INDEX_TEMPLATE),
                        ("retrieve", RETRIEVE_TEMPLATE),
                    ])
                    .unwrap();
                }
            };

            tera.build_inheritance_chains().unwrap();

            Ok(rocket.manage(ConfigState {
                storage_dir,
                db,
                tera,
                syntax_set: SyntaxSet::load_defaults_newlines(),
                theme_set: ThemeSet::load_defaults(),
            }))
        }))
        .launch();
}
