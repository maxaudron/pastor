use std::path::PathBuf;
use std::vec::Vec;

use magic::{
    Cookie,
    cookie::{self, Flags},
};

use axum::Router;
use clap::Parser;

mod cleanup;
mod dict;
mod file;
mod handlers;
mod id;
mod tokens;
mod util;

use crate::{cleanup::cleanup_routine, handlers::auth};

fn load_magic(flags: Flags) -> Cookie<cookie::Load> {
    let magic = Cookie::open(cookie::Flags::default() | flags).unwrap();

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
            .collect::<Vec<String>>()
            .try_into()
            .unwrap(),
        )
        .expect("failed to load magic database");

    magic
}

thread_local! {
    static MAGIC: Cookie<cookie::Load> = load_magic(cookie::Flags::MIME_TYPE);
    static EXT: Cookie<cookie::Load> = load_magic(cookie::Flags::EXTENSION);
}

#[cfg(feature = "magic_static")]
const MIME_DB: &[u8] = include_bytes!(env!("PASTOR_MIME_DB"));

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

    let auth_state = auth::Auth::new(args.tokens.clone()).await;
    let file_state = handlers::file::FileState::new(args.storage.clone(), args.tokens.clone()).await;

    let tokens = file_state.tokens.clone();
    let token_handle = tokio::spawn(tokens.refresh());
    let cleanup_handle = tokio::spawn(cleanup_routine(args.storage));

    // build our application with a single route
    let app = Router::new()
        .merge(handlers::ui::router())
        .merge(handlers::file::router(file_state, auth_state));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", args.address, args.port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    let (token, cleanup) = tokio::join!(token_handle, cleanup_handle);
    token.unwrap();
    cleanup.unwrap();
}
