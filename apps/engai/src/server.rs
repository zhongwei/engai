use axum::{response::IntoResponse, routing::get, Router};
use rust_embed::Embed;
use tower_http::cors::{Any, CorsLayer};

use crate::handlers::{chat, notes, phrases, readings, reviews, stats, sync, words};
use crate::state::AppState;

#[derive(Embed)]
#[folder = "static"]
struct Assets;

pub async fn run_server(state: AppState, port: u16) -> anyhow::Result<()> {
    let addr = format!("{}:{}", state.config.server.host, port);
    let app = create_app(state);
    tracing::info!("Engai server running on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let api = Router::new()
        .nest("/words", words::router())
        .nest("/phrases", phrases::router())
        .nest("/review", reviews::router())
        .nest("/readings", readings::router())
        .nest("/notes", notes::router())
        .nest("/chat", chat::router())
        .nest("/sync", sync::router())
        .nest("/stats", stats::router());
    let app = Router::new()
        .nest("/api", api)
        .fallback_service(get(serve_static));
    app.with_state(state).layer(cors)
}

async fn serve_static(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl axum::response::IntoResponse {
    let path = if path.is_empty() {
        "index.html".to_string()
    } else {
        path
    };
    if let Some(content) = Assets::get(&path) {
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        axum::response::Response::builder()
            .header("Content-Type", mime.as_ref())
            .body(axum::body::Body::from(content.data.to_vec()))
            .unwrap()
            .into_response()
    } else if let Some(content) = Assets::get("index.html") {
        axum::response::Response::builder()
            .header("Content-Type", "text/html")
            .body(axum::body::Body::from(content.data.to_vec()))
            .unwrap()
            .into_response()
    } else {
        axum::response::Response::builder()
            .status(404)
            .body(axum::body::Body::from("Not found"))
            .unwrap()
            .into_response()
    }
}
