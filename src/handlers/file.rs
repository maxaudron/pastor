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
use tracing::{debug, error, instrument};

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
) -> Result<Response, PasteError> {
    let mut pastes: Vec<Result<Paste, (PasteId, PasteError)>> = Vec::new();

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(err) => return Err(PasteError::from(err)),
        };

        let id = PasteId::new();
        debug!("new file with id: {id}");

        let result: Result<Paste, PasteError> = (|| async {
            let mut handle =
                Paste::get_handle_create(&state.storage.join(&id)).await?;

            let mut field = field;
            while let Some(chunk) = field.chunk().await.map_err(PasteError::from)? {
                handle.write_all(&chunk).await.map_err(PasteError::from)?;
            }

            handle.flush().await.map_err(PasteError::from)?;

            let paste = Paste::from_handle(id.clone(), handle, bearer.token()).await?;
            paste.write(&state.storage).await?;

            Ok(paste)
        })()
        .await;

        match result {
            Ok(paste) => pastes.push(Ok(paste)),
            Err(err) => {
                error!("failed to create paste: {err}");
                if let Err(cleanup_err) = tokio::fs::remove_file(id.path(&state.storage)).await {
                    error!("error while trying to delete errored upload: {cleanup_err}");
                }
                pastes.push(Err((id, err)));
            }
        }
    }

    if pastes.is_empty() {
        return Err(PasteError::NoContent);
    }

    let mut result = String::new();
    let mut first_err_status: Option<StatusCode> = None;

    for p in pastes {
        match p {
            Ok(paste) => {
                result.push_str(&format!("https://{host}/{}", paste.id));
                result.push('\n');
            }
            Err((_id, err)) => {
                error!("failed to create paste: {err}");
                let err_str = err.to_string();
                if first_err_status.is_none() {
                    first_err_status = Some(err.into_response().status().clone());
                }
                result.push_str(&err_str);
                result.push('\n');
            }
        }
    }

    match first_err_status {
        None => Ok(result.into_response()),
        Some(status) => Ok(Response::builder()
            .status(status)
            .body(Body::from(result))
            .unwrap()),
    }
}

#[instrument(level = "trace")]
async fn retrieve(
    State(state): State<FileState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, PasteError> {
    let id = PasteId::from(id.as_str());
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
    Path(id): Path<String>,
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, PasteError> {
    let id = PasteId::from(id.as_str());
    let (paste, _) = Paste::load(&state.storage, id).await?;
    paste.delete(&state.storage, Some(bearer.token())).await?;

    Ok(StatusCode::OK)
}
