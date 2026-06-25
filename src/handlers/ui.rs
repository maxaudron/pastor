use axum::{
    Router,
    body::Body,
    extract::State,
    http::{StatusCode, header},
    response::Response,
    routing::get,
};
use axum_extra::{TypedHeader, headers::Host};
use tera::{Context, Tera};
use tracing::instrument;

const MAIN_CSS: &str = include_str!("../../static/styles/main.css");
const FAVICON: &[u8] = include_bytes!("../../static/favicon.ico");

const BASE_TEMPLATE: &str = include_str!("../../templates/base.html.tera");
const INDEX_TEMPLATE: &str = include_str!("../../templates/index.html.tera");
const RETRIEVE_TEMPLATE: &str = include_str!("../../templates/retrieve.html.tera");
const GUI_TEMPLATE: &str = include_str!("../../templates/gui.html.tera");
const GUI_RESULT_TEMPLATE: &str = include_str!("../../templates/gui_result.html.tera");
const DELETE_RESULT_TEMPLATE: &str = include_str!("../../templates/delete_result.html.tera");

#[derive(Debug, Clone)]
pub struct UIState {
    tera: Tera,
}

pub fn router() -> Router {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("base", BASE_TEMPLATE),
        ("index", INDEX_TEMPLATE),
        ("retrieve", RETRIEVE_TEMPLATE),
        ("gui", GUI_TEMPLATE),
        ("gui_result", GUI_RESULT_TEMPLATE),
        ("delete_result", DELETE_RESULT_TEMPLATE),
    ])
    .unwrap();
    let state = UIState { tera };

    Router::new()
        .route("/", get(ui))
        .route("/favicon.ico", get(favicon))
        .route("/main.css", get(style))
        .with_state(state)
}

#[instrument(level = "trace")]
async fn favicon() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/x-icon")
        .body(Body::from(FAVICON))
        .unwrap()
}

#[instrument(level = "trace")]
async fn style() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/css")
        .body(Body::from(MAIN_CSS))
        .unwrap()
}

fn context(host: Host) -> Context {
    let mut context = Context::new();
    context.insert("host", &host.to_string());
    context
}

#[instrument(level = "trace")]
async fn ui(State(state): State<UIState>, TypedHeader(host): TypedHeader<Host>) -> Response {
    let rendered = state.tera.render("index", &context(host)).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .body(Body::from(rendered))
        .unwrap()
}
