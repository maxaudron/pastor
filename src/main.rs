#![cfg_attr(all(test, feature = "bench"), feature(test))]
#[cfg(all(test, feature = "bench"))]
extern crate test;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::vec::Vec;

use magic::{Cookie, cookie};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, trace};

use chrono::Utc;

use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use tera::Tera;

use tokio::io::AsyncReadExt;

mod config;
mod dict;
mod file;
mod id;
mod util;

// mod multipart;

mod tokens;
use tokens::*;

use id::PasteId;

// #[get("/gui")]
// fn gui(config: &State<ConfigState>) -> Result<content::Html<String>, Status> {
//     let context = tera::Context::new();
//     let rendered_template = config.tera.render("gui", &context).unwrap();
//     Ok(content::Html(rendered_template))
// }
//
// #[get("/")]
// fn index<'a>(
//     host: HostHeader,
//     config: &State<ConfigState>,
// ) -> Result<content::Html<String>, Status> {
//     let mut context = tera::Context::new();
//     context.insert("url", host.0);
//     let rendered_template = config.tera.render("index", &context).unwrap();
//     Ok(content::Html(rendered_template))
// }
//
// #[get("/favicon.ico")]
// fn favicon() -> content::Custom<&'static [u8]> {
//     content::Custom(ContentType::Icon, FAVICON.into())
// }
//
// #[get("/static/<path..>")]
// fn static_file(path: PathBuf) -> Option<content::Custom<String>> {
//     let mut res = Response::new();
//     res.set_status(Status::Ok);
//
//     match path.to_str() {
//         Some("styles/main.css") => Some(content::Custom(ContentType::CSS.into(), MAIN_CSS.into())),
//         _ => None,
//     }
// }

// #[derive(rocket::Responder)]
// enum PasteResponse {
//     File(PasteFileResponse),
//     Html(content::Html<String>),
// }
//
// struct PasteFileResponse {
//     paste: Paste,
//     file: tokio::fs::File,
// }
//
// impl PasteFileResponse {
//     fn new(paste: Paste, file: tokio::fs::File) -> Self {
//         Self { paste, file }
//     }
// }
//
// impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for PasteFileResponse {
//     fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
//         let mut response = Response::build()
//             .status(Status::Ok)
//             .header(Header::new(header::CONTENT_DISPOSITION.as_str(), "inline"))
//             .streamed_body(self.file)
//             .finalize();
//
//         if self.paste.mime.contains("text/")
//             || self.paste.mime.contains("application/xhtml")
//             || self.paste.mime.contains("application/xml")
//         {
//             response.set_header(ContentType::parse_flexible("text/plain; charset=utf-8").unwrap());
//         } else {
//             response.set_header(ContentType::parse_flexible(&self.paste.mime).unwrap());
//         }
//
//         Ok(response)
//     }
// }
//
// #[get("/<paste_id>?<lang>")]
// async fn retrieve(
//     paste_id: PasteId,
//     lang: Option<String>,
//     config: &State<ConfigState>,
// ) -> Result<PasteResponse, Status> {
//     let paste = file::get_db(&paste_id.id, &config.db)?;
//     let now = Utc::now().timestamp();
//
//     if paste.expires < now {
//         file::delete(file::build_path(&paste_id.id, &config))?;
//         return Err(Status::Gone);
//     }
//
//     let mut file = file::get(file::build_path(&paste_id.id, &config)).await?;
//
//     match lang {
//         Some(l) if !l.is_empty() => {
//             let mut buffer = String::new();
//             // Could a better error be returned?
//             file.read_to_string(&mut buffer)
//                 .await
//                 .map_err(|_| Status::InternalServerError)?;
//
//             // 1. Try to find syntax by exact match
//             let syntax = find_syntax_by_name(&config.syntax_set, |it: &&SyntaxReference| {
//                 it.name.to_lowercase() == l.to_lowercase()
//             })
//             // 2. Try to find syntax by "contains" match
//             .unwrap_or(
//                 find_syntax_by_name(&config.syntax_set, |it: &&SyntaxReference| {
//                     it.name.to_lowercase().contains(&l.to_lowercase())
//                 })
//                 // 3. Try to auto-detect syntax
//                 .unwrap_or(
//                     config
//                         .syntax_set
//                         .find_syntax_by_first_line(&buffer)
//                         // 4. Use plaintext syntax
//                         .unwrap_or(config.syntax_set.find_syntax_plain_text()),
//                 ),
//             );
//
//             println!("Using syntax: {}", syntax.name);
//
//             let html = syntect::html::highlighted_html_for_string(
//                 &buffer,
//                 &config.syntax_set,
//                 syntax,
//                 &config.theme_set.themes["base16-eighties.dark"],
//             );
//
//             let mut context = tera::Context::new();
//             context.insert("id", &paste_id.id);
//             context.insert("lang", &l);
//             context.insert("content", &html);
//             let rendered_template = config.tera.render("retrieve", &context).unwrap();
//
//             Ok(PasteResponse::Html(content::Html(rendered_template)))
//         }
//         None | _ => Ok(PasteResponse::File(PasteFileResponse::new(paste, file))),
//     }
// }
//
// #[get("/delete/<id>?<token>")]
// fn delete_get<'a>(
//     id: PasteId,
//     token: PasteId,
//     host: HostHeader,
//     config: &State<ConfigState>,
// ) -> Result<content::Html<String>, Status> {
//     match delete(&id.id, token, &config) {
//         Ok(_) => {
//             let mut context = tera::Context::new();
//             context.insert("id", &format!("{}", &id.id));
//             context.insert("host", &host.0);
//             let rendered_template = config.tera.render("delete_result", &context).unwrap();
//             Ok(content::Html(rendered_template))
//         }
//         Err(e) => Err(e),
//     }
// }
//
// #[delete("/<id>?<token>")]
// fn delete_delete(
//     id: PasteId,
//     token: PasteId,
//     config: &State<ConfigState>,
// ) -> Result<Status, Status> {
//     delete(&id.id, token, config)
// }
//
// fn delete(id: &str, token: PasteId, config: &State<ConfigState>) -> Result<Status, Status> {
//     let paste = file::get_db(id, &config.db)?;
//
//     if paste.token != token {
//         return Err(Status::Forbidden);
//     }
//
//     file::delete(file::build_path(id, &config))?;
//     config.db.remove(id).unwrap();
//     return Ok(Status::Ok);
// }
//
// #[derive(rocket::Responder)]
// pub enum CreateReturnType {
//     Raw(String),
//     Response(content::Html<String>),
// }
//
// pub enum Bytes<'v> {
//     Value(String),
//     Data(Data<'v>),
// }
//
// #[async_trait::async_trait]
// impl<'v> FromFormField<'v> for Bytes<'v> {
//     fn from_value(field: rocket::form::ValueField<'v>) -> rocket::form::Result<'v, Self> {
//         Ok(Bytes::Value(field.value.to_owned()))
//     }
//
//     async fn from_data(field: rocket::form::DataField<'v, '_>) -> rocket::form::Result<'v, Self> {
//         Ok(Bytes::Data(field.data))
//     }
//
//     fn default() -> Option<Self> {
//         None
//     }
// }
//
// #[post("/?<token>&<from_gui>", data = "<data>")]
// #[tracing::instrument(skip_all)]
// pub async fn create<'a>(
//     data: multipart::Form<'a>,
//     token: String,
//     from_gui: bool,
//     config: &State<crate::ConfigState>,
//     host: HostHeader<'_>,
// ) -> Result<CreateReturnType, Status> {
//     if !config.tokens.contains(&token).await {
//         return Err(Status::Forbidden);
//     }
//
//     trace!("creating paste");
//     let pastes = file::store(data, config, Some(token)).await?;
//
//     if from_gui {
//         trace!("created from gui");
//         let mut context = tera::Context::new();
//
//         // The gui is only able to create one upload at a time
//         if pastes.len() > 1 {
//             warn!("Warning: GUI created more than one upload.");
//         } else if pastes.len() < 1 {
//             return Err(Status::InternalServerError);
//         }
//
//         context.insert("id", &format!("{}", &pastes[0].id));
//         context.insert("mime", &format!("{}", &pastes[0].mime));
//         context.insert("host", &host.0);
//         let rendered_template = config.tera.render("gui_result", &context).unwrap();
//
//         Ok(CreateReturnType::Response(content::Html(rendered_template)))
//     } else {
//         let mut urls = Vec::new();
//         for paste in &pastes {
//             trace!("paste: {:?}", paste);
//             urls.push(format!(
//                 "https://{host}/{id}\n",
//                 host = host.0,
//                 id = paste.id,
//             ));
//             trace!("urls: {:?}", urls);
//         }
//
//         Ok(CreateReturnType::Raw(urls.join("\n")))
//     }
// }

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
    pub async fn from_file(
        mut id: PasteId,
        file: &mut tokio::fs::File,
        token: Option<String>,
    ) -> Result<Paste, ()> {
        let size = file.metadata().await.unwrap().len();
        if size == 0 {
            return Err(());
        }
        let now = Utc::now().timestamp();
        let expiry = now + crate::util::expires(size);

        let token = if let Some(token) = token {
            token.as_str().into()
        } else {
            PasteId::new()
        };

        let mut mime_bytes: Vec<u8> = Vec::with_capacity(2048);
        file.take(2048)
            .read_to_end(&mut mime_bytes)
            .await
            .map_err(|e| {
                error!("failed to read file: {:?}", e);
                ()
            })?;

        trace!("read bytes for mime parsing: {:x?}", mime_bytes);

        let mime = MAGIC.with(|magic| {
            magic.buffer(&mime_bytes).map_err(|err| {
                error!("failed to parse mime type: {}", err);
                ()
            })
        })?;

        let ext = util::ext_from_mime(&mime);

        trace!("got file ext: {:?}", ext);

        id.ext = ext;

        Ok(Paste {
            id,
            created: now,
            expires: expiry,
            token,
            mime: mime.to_string(),
        })
    }
}

thread_local! {
    static MAGIC: Cookie<cookie::Load> = {
        let magic = Cookie::open(cookie::Flags::default() | cookie::Flags::MIME_TYPE).unwrap();

        #[cfg(feature = "magic_static")]
        let magic = magic
            .load_buffers(&[MIME_DB])
            .expect("failed to load magic database");

        #[cfg(not(feature = "magic_static"))]
        let magic = magic
            .load(
                &std::fs::read_dir(
                    std::env::var("PASTOR_MIME_DB").unwrap_or("/usr/share/misc/magic".to_string()),
                )
                .unwrap()
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path().to_str().unwrap().to_string())
                .collect::<Vec<String>>().try_into().unwrap(),
            )
            .expect("failed to load magic database");

        magic
    };
}

pub struct ConfigState {
    db: Arc<sled::Db>,
    tera: Tera,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    app_config: config::AppConfig,

    tokens: Tokens,
}

#[cfg(feature = "magic_static")]
const MIME_DB: &[u8] = include_bytes!(env!("PASTOR_MIME_DB"));

use axum::Router;
use clap::Parser;
mod handlers;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Address to bind to
    #[arg(short, long, default_value = "0.0.0.0", env = "PASTOR_BIND")]
    address: String,

    /// Port to bind to
    #[arg(short, long, default_value_t = 3000, env = "PASTOR_PORT")]
    port: u16,

    /// Path to a file containing authentication tokens
    #[arg(short, long, env = "PASTOR_TOKENS_FILE")]
    tokens: PathBuf,

    /// Path to the folder where pastes are stored in
    #[arg(short, long, env = "PASTOR_STORAGE")]
    storage: PathBuf,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let file_state = handlers::file::FileState::new();

    let tokens = file_state.tokens.clone();
    let handle = tokio::spawn(tokens.refresh(args.tokens));

    // build our application with a single route
    let app = Router::new()
        .merge(handlers::ui::router())
        .merge(handlers::file::router(file_state));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    tokio::join!(handle).0.unwrap();
}
