use axum::{
    Router,
    extract::{Multipart, Path, State},
    http::{HeaderMap, header, status::StatusCode},
    response::IntoResponse,
    routing,
};

use crate::tokens::Tokens;

#[derive(Debug, Clone)]
pub struct FileState {
    pub tokens: Tokens,
}

impl FileState {
    pub fn new() -> FileState {
        FileState {
            tokens: Tokens::new(),
        }
    }
}

pub fn router(state: FileState) -> Router {
    Router::new()
        .route("/", routing::post(upload))
        .route("/{id}", routing::get(retrieve))
        .route("/{id}", routing::delete(delete))
        .with_state(state)
}

async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!("Length of `{}` is {} bytes", name, data.len());
    }

    "fuck you"
}

async fn retrieve(Path(id): Path<String>) -> impl IntoResponse {
    "fuck you"
}

async fn delete(State(state): State<FileState>, Path(id): Path<String>, headers: HeaderMap) -> impl IntoResponse {
    if let Some(token) = headers.get(header::AUTHORIZATION)
        && let Ok(token) = token.to_str()
        && let Some(token) = token.strip_prefix("Bearer ")
        && state.tokens.contains(token).await
    {
        return StatusCode::OK;
    }

    StatusCode::UNAUTHORIZED
}
