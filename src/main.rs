#![feature(proc_macro_hygiene, decl_macro)]
extern crate multipart;

use id::PasteId;
use tracing::{Level, error, trace};
use tracing_subscriber::FmtSubscriber;

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

use crate::util::find_syntax_by_name;
use rocket::response::Body;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxReference, SyntaxSet};
use tera::Tera;
use util::HostHeader;

#[get("/gui")]
fn gui(config: State<ConfigState>) -> Result<Response<'static>, Status> {
    let context = tera::Context::new();
    let rendered_template = config.tera.render("gui", &context).unwrap();
    Ok(util::create_response_from_string(rendered_template, None))
}

#[get("/")]
fn index<'a>(host: HostHeader, config: State<ConfigState>) -> Result<Response<'a>, Status> {
    let mut context = tera::Context::new();
    context.insert("url", host.0);
    let rendered_template = config.tera.render("index", &context).unwrap();
    Ok(util::create_response_from_string(rendered_template, None))
}

#[get("/static/<path..>")]
fn static_file(path: PathBuf) -> Option<Response<'static>> {
    let mut res = Response::new();
    res.set_status(Status::Ok);

    match path.to_str() {
        Some("styles/main.css") => {
            Some(
                util::create_response_from_string(MAIN_CSS.into(),
                ContentType::CSS.into()),
            )
        }
        _ => None,
    }
}

#[get("/<paste_id>?<lang>")]
fn retrieve(
    paste_id: PasteId,
    lang: Option<String>,
    config: State<ConfigState>,
) -> Result<Response, Status> {
    let paste = file::get_db(&paste_id.id, &config.db)?;
    let now = Utc::now().timestamp();

    if paste.expires < now {
        file::delete(file::build_path(&paste_id.id, &config))?;
        return Err(Status::Gone);
    }

    let mut file = file::get(file::build_path(&paste_id.id, &config))?;

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

            // 1. Try to find syntax by exact match
            let syntax = find_syntax_by_name(&config.syntax_set, |it: &&SyntaxReference| {
                it.name.to_lowercase() == l.to_lowercase()
            })
            // 2. Try to find syntax by "contains" match
            .unwrap_or(
                find_syntax_by_name(&config.syntax_set, |it: &&SyntaxReference| {
                    it.name.to_lowercase().contains(&l.to_lowercase())
                })
                // 3. Try to auto-detect syntax
                .unwrap_or(
                    config
                        .syntax_set
                        .find_syntax_by_first_line(&buffer)
                        // 4. Use plaintext syntax
                        .unwrap_or(config.syntax_set.find_syntax_plain_text()),
                ),
            );

            println!("Using syntax: {}", syntax.name);

            let html = syntect::html::highlighted_html_for_string(
                &buffer,
                &config.syntax_set,
                syntax,
                &config.theme_set.themes["base16-eighties.dark"],
            );

            let mut context = tera::Context::new();
            context.insert("id", &paste_id.id);
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
            if paste.mime.contains("text/") {
                res.set_header(ContentType::parse_flexible("text/plain").unwrap());
            }

            res.set_streamed_body(file);
            Ok(res)
        }
    }
}

#[get("/delete/<id>?<token>")]
fn delete_get<'a>(id: PasteId, token: PasteId, host: HostHeader, config: State<ConfigState>) -> Result<Response<'a>, Status> {
    match delete(&id.id, token, &config) {
        Ok(_) => {
            let mut context = tera::Context::new();
            context.insert("id", &id);
            context.insert("host", &host.0);
            let rendered_template = config.tera.render("delete_result", &context).unwrap();
            Ok(util::create_response_from_string(rendered_template, None))
        },
        Err(e) => Err(e),
    }
}

#[delete("/<id>?<token>")]
fn delete_delete(id: PasteId, token: PasteId, config: State<ConfigState>) -> Result<Status, Status> {
    delete(&id.id, token, &config)
}

fn delete(id: &str, token: PasteId, config: &State<ConfigState>) -> Result<Status, Status> {
    let paste = file::get_db(id, &config.db)?;

    if paste.token != token {
        return Err(Status::Forbidden);
    }

    file::delete(file::build_path(id, &config))?;
    config.db.remove(id).unwrap();
    return Ok(Status::Ok);
}

#[derive(Responder)]
pub enum CreateReturnType<'a> {
    Raw(String),
    Response(Response<'a>),
}

#[post("/?<token>&<from_gui>", data = "<data>")]
pub fn create<'a>(
    cont_type: &ContentType,
    data: Data,
    token: Option<String>,
    from_gui: bool,
    config: State<crate::ConfigState>,
    host: HostHeader,
) -> Result<CreateReturnType<'a>, Status> {
    if !cont_type.is_form_data() {
        return Err(Status::MethodNotAllowed);
    }

    let pastes = file::store(cont_type, data, &config)?;

    let mut urls = Vec::new();
    for paste in &pastes {
        trace!("paste: {:?}", paste);
        urls.push(format!(
            "https://{host}/{id} {token}\n",
            host = host.0,
            id = paste.id,
            token = paste.token
        ));
        trace!("urls: {:?}", urls);
    }

    if from_gui {
        let mut context = tera::Context::new();
        // The gui is only able to create one upload at a time
        if urls.len() > 1 {
            println!("Warning: GUI somehow created more than one upload.");
        }
        context.insert("id", &pastes[0].id);
        context.insert("token", &pastes[0].token);
        context.insert("host", &host.0);
        let rendered_template = config.tera.render("gui_result", &context)
            .unwrap();

        let res = util::create_response_from_string(rendered_template, None);
        Ok(CreateReturnType::Response(res))
    } else {
        Ok(CreateReturnType::Raw(urls.join("\n")))
    }
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
    id: PasteId,
    created: i64,
    expires: i64,
    token: PasteId,
    mime: String,
}

impl Paste {
    #[tracing::instrument]
    pub fn from_file(mut id: PasteId, file: &mut std::fs::File) -> Result<Paste, rocket::http::Status> {
        let size = file.metadata().unwrap().len();
        let now = Utc::now().timestamp();
        let expiry = now + crate::util::expires(size);

        let token = PasteId::new();

        let mut mime_bytes: Vec<u8> = Vec::with_capacity(2048);
        file.take(2048).read_to_end(&mut mime_bytes)
            .map_err(|e| {
                error!("failed to read file: {:?}", e);
                Status::InternalServerError
            })?;

        trace!("read bytes for mime parsing: {:x?}", mime_bytes);

        let mime = tree_magic::from_u8(&mime_bytes).to_string();
        let ext = util::ext_from_mime(&mime);

        trace!("got file ext: {:?}", ext);

        id.ext = ext;

        Ok(Paste {
            id,
            created: now,
            expires: expiry,
            token,
            mime,
        })
    }
}

const BASE_TEMPLATE: &str = include_str!("../templates/base.html.tera");
const INDEX_TEMPLATE: &str = include_str!("../templates/index.html.tera");
const RETRIEVE_TEMPLATE: &str = include_str!("../templates/retrieve.html.tera");
const GUI_TEMPLATE: &str = include_str!("../templates/gui.html.tera");
const GUI_RESULT_TEMPLATE: &str = include_str!("../templates/gui_result.html.tera");
const DELETE_RESULT_TEMPLATE: &str = include_str!("../templates/delete_result.html.tera");

const MAIN_CSS: &str = include_str!("../static/styles/main.css");

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");


    rocket::ignite()
        .mount("/", routes![index, gui, retrieve, create, delete_get, delete_delete, static_file])
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

            let mut tera = match rocket.config().get_string("template_dir") {
                Ok(s) => {
                    let mut tera = Tera::parse(&format!("{}/*", s)).unwrap();
                    println!("Using external templates at {}", s);
                    tera.add_template_files(vec![
                        (format!("{}/base.html.tera", s), Some("base")),
                        (format!("{}/index.html.tera", s), Some("index")),
                        (format!("{}/retrieve.html.tera", s), Some("retrieve")),
                        (format!("{}/gui.html.tera", s), Some("gui")),
                        (format!("{}/gui_result.html.tera", s), Some("gui_result")),
                        (format!("{}/delete_result_result.html.tera", s), Some("delete_result")),
                    ])
                    .unwrap();
                    tera
                }
                _ => {
                    let mut tera = Tera::parse("/templates/*").unwrap();
                    println!("Using embedded templates");
                    tera.add_raw_templates(vec![
                        ("base", BASE_TEMPLATE),
                        ("index", INDEX_TEMPLATE),
                        ("retrieve", RETRIEVE_TEMPLATE),
                        ("gui", GUI_TEMPLATE),
                        ("gui_result", GUI_RESULT_TEMPLATE),
                        ("delete_result", DELETE_RESULT_TEMPLATE),
                    ])
                    .unwrap();
                    tera
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
