use std::path::PathBuf;

use axum::{
    Router,
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{header, status::StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, Host, authorization::Bearer},
};
use tokio::io::AsyncWriteExt;
use tracing::{debug, instrument};

use crate::{
    file::{Paste, PasteError},
    handlers::auth,
    id::PasteId,
    tokens::Tokens,
};

#[derive(Debug, Clone)]
pub struct FileState {
    pub tokens: Tokens,
    pub storage: PathBuf,
}

impl FileState {
    pub async fn new(storage: PathBuf, tokens: PathBuf) -> FileState {
        FileState {
            tokens: Tokens::new(tokens.clone()).await,
            storage,
        }
    }
}

pub fn router(state: FileState, auth_state: auth::Auth, file_size_limit: usize) -> Router {
    let auth = middleware::from_fn_with_state(auth_state, auth::auth);

    Router::new()
        .route("/", routing::post(upload))
        .route_layer(DefaultBodyLimit::max(file_size_limit * 1024 * 1024))
        .route("/{id}", routing::delete(delete))
        .route_layer(auth)
        .route("/{id}", routing::get(retrieve))
        .with_state(state)
}

#[instrument(level = "trace")]
async fn upload(
    State(state): State<FileState>,
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    TypedHeader(host): TypedHeader<Host>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, PasteError> {
    let mut pastes = Vec::new();
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(PasteError::MultipartError)?
    {
        let id = PasteId::new();
        debug!("new file with id: {id}");
        let mut handle = Paste::get_handle_create(&state.storage.join(&id)).await?;
        while let Some(chunk) = field.chunk().await.map_err(PasteError::MultipartError)? {
            handle.write_all(&chunk).await?;
        }

        handle.flush().await?;

        let paste = Paste::from_handle(id, handle, bearer.token()).await?;
        paste.write(&state.storage).await?;
        pastes.push(paste);
    }

    Ok(pastes
        .iter()
        .map(|p| p.id.to_string())
        .fold(String::new(), |mut s, p| {
            s.push_str(&format!("https://{host}/{p}"));
            s.push('\n');
            s
        }))
}

#[instrument(level = "trace")]
async fn retrieve(
    State(state): State<FileState>,
    Path(id): Path<PasteId>,
) -> Result<impl IntoResponse, PasteError> {
    let (paste, file) = Paste::load(&state.storage, id).await?;
    if paste.expired()? {
        paste.delete(&state.storage, None).await?;
        return Err(PasteError::NotFound);
    }

    let stream = tokio_util::io::ReaderStream::new(file.to_file());

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, paste.mime)
        .header(header::CONTENT_DISPOSITION, "inline")
        .body(Body::from_stream(stream))
        .unwrap())
}

#[instrument(level = "trace")]
async fn delete(
    State(state): State<FileState>,
    Path(id): Path<PasteId>,
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, PasteError> {
    let (paste, _) = Paste::load(&state.storage, id).await?;
    paste.delete(&state.storage, Some(bearer.token())).await?;

    Ok(StatusCode::OK)
}
