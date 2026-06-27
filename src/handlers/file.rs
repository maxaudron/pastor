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

macro_rules! e {
    ($exp:expr, $p:expr, $id:expr) => {
        match $exp {
            Ok(ok) => ok,
            Err(err) => {
                $p.push(Err(($id, err)));
                break;
            }
        }
    };
}

#[instrument(level = "trace")]
async fn upload(
    State(state): State<FileState>,
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    TypedHeader(host): TypedHeader<Host>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, PasteError> {
    let mut inner = async || -> Result<Vec<Result<Paste, (PasteId, PasteError)>>, PasteError> {
        let mut pastes = Vec::new();

        'multi: while let Some(mut field) = multipart
            .next_field()
            .await
            .map_err(PasteError::MultipartError)?
        {
            let id = PasteId::new();
            debug!("new file with id: {id}");
            let mut handle = e!(
                Paste::get_handle_create(&state.storage.join(&id)).await,
                pastes,
                id
            );

            while let Some(chunk) = match field.chunk().await {
                Ok(ok) => ok,
                Err(err) => {
                    pastes.push(Err((id, PasteError::from(err))));
                    break 'multi;
                }
            } {
                match handle.write_all(&chunk).await {
                    Ok(_) => (),
                    Err(err) => {
                        pastes.push(Err((id, PasteError::from(err))));
                        break 'multi;
                    }
                };
            }

            handle.flush().await?;

            let paste = e!(
                Paste::from_handle(id.clone(), handle, bearer.token()).await,
                pastes,
                id
            );
            e!(paste.write(&state.storage).await, pastes, id);

            pastes.push(Ok(paste));
        }

        Ok(pastes)
    };

    let pastes = inner().await.unwrap();

    let mut result = String::new();

    for p in pastes {
        match p {
            Ok(paste) => {
                result.push_str(&format!("https://{host}/{}", paste.id.to_string()));
                result.push('\n');
            }
            Err((id, err)) => {
                error!("failed to create paste: {err}");
                match tokio::fs::remove_file(id.path(&state.storage)).await {
                    Ok(_) => (),
                    Err(err) => error!("error while trying to delete errored upload: {err}"),
                };
                result.push_str(&err.to_string());
                result.push('\n');
            }
        }
    }

    Ok(result)
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
