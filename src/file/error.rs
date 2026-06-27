use std::{error::Error, path::PathBuf, str::Utf8Error};

use axum::{
    body::Body,
    extract::multipart::MultipartError,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum PasteError {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("xattr field not found: {0}")]
    XattrNotFound(&'static str),
    #[error("failed to parse: {0:?}")]
    ParseError(Vec<u8>),
    #[error("failed to parse: {0}")]
    StrParseError(#[from] Utf8Error),
    #[error("failed to guess mime type: {0}")]
    MagicError(#[from] magic::cookie::Error),
    #[error("failed to parse PasteId from path: {0}")]
    PasteIdFromPath(PathBuf),
    #[error("failed to read multipart data: {0}")]
    MultipartError(MultipartError),

    #[error("Unauthorized to operate on this paste")]
    Unauthorized,
    #[error("paste did not contain any content")]
    NoContent,
    #[error("NOT FOUND")]
    NotFound,
    #[error("time did not work")]
    Time,
}

impl IntoResponse for PasteError {
    fn into_response(self) -> axum::response::Response {
        match &self {
            PasteError::MultipartError(err) => {
                error!(
                    "multipart error: source: {:?} status: {}",
                    err.source(),
                    err.status()
                );

                return Response::builder()
                    .status(err.status())
                    .body(Body::from(err.status().to_string()))
                    .unwrap();
            }
            PasteError::NotFound => {
                return Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from(self.to_string()))
                    .unwrap();
            }
            PasteError::Unauthorized => {
                return Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::from(self.to_string()))
                    .unwrap();
            }
            PasteError::NoContent => {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from(self.to_string()))
                    .unwrap();
            }
            PasteError::IOError(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    return Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::from("NOT FOUND\n"))
                        .unwrap();
                }
                _ => (),
            },
            _ => (),
        }

        error!("internal error while processing request: {}", self);
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap()
    }
}
