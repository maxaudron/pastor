use axum::{
    Router,
    body::Body,
    extract::State,
    http::{StatusCode, header},
    response::Response,
    routing::get,
};
use tera::{Context, Tera};

const MAIN_CSS: &str = include_str!("../../static/styles/main.css");
const FAVICON: &[u8] = include_bytes!("../../static/favicon.ico");

#[derive(Debug, Clone)]
pub struct UIState {
    tera: Tera,
    context: Context,
}

pub fn router() -> Router {
    let mut tera = Tera::default();
    tera.add_raw_template("base", include_str!("../../templates/base.html.tera"))
        .unwrap();
    tera.add_raw_template("index", include_str!("../../templates/index.html.tera"))
        .unwrap();
    let mut context = Context::new();
    context.insert("url", "https://test");
    let state = UIState { tera, context };

    Router::new()
        .route("/", get(ui))
        .route("/favicon.ico", get(favicon))
        .route("/main.css", get(style))
        .with_state(state)
}

async fn favicon() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/x-icon")
        .body(Body::from(FAVICON))
        .unwrap()
}

async fn style() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/css")
        .body(Body::from(MAIN_CSS))
        .unwrap()
}

async fn ui(State(state): State<UIState>) -> Response {
    let rendered = state.tera.render("index", &state.context).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .body(Body::from(rendered))
        .unwrap()
}
