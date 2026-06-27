use std::path::PathBuf;

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use tracing::debug;

use crate::tokens::Tokens;

#[derive(Debug, Clone)]
pub struct Auth {
    pub tokens: Tokens,
}

impl Auth {
    pub async fn new(tokens: PathBuf) -> Self {
        Self {
            tokens: Tokens::new(tokens.clone()).await,
        }
    }
}

pub async fn auth(
    State(state): State<Auth>,
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    request: Request,
    next: Next,
) -> Response {
    if state.tokens.contains(bearer.token()).await {
        let response = next.run(request).await;
        response
    } else {
        debug!(
            "token: {:?} was not found in allowed tokens: {:?}",
            bearer, state.tokens
        );
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("UNAUTHORIZED"))
            .unwrap()
    }
}
