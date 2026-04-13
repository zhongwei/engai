#[cfg(embed_static)]
use axum::http::Uri;
use axum::{body::Body, http::header, response::Response, Router};
use tower_http::cors::{Any, CorsLayer};

#[cfg(embed_static)]
use rust_embed::Embed;

use crate::handlers::{chat, notes, phrases, readings, reviews, stats, sync, words};
use crate::state::AppState;

#[cfg(embed_static)]
#[derive(Embed)]
#[folder = "static"]
struct StaticAssets;

#[cfg(embed_static)]
fn serve_static(path: &str) -> Response {
    let br_path = format!("{}.br", path);

    if let Some(content) = StaticAssets::get(&br_path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return Response::builder()
            .header(header::CONTENT_TYPE, mime.as_ref())
            .header(header::CONTENT_ENCODING, "br")
            .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
            .body(Body::from(content.data))
            .unwrap();
    }

    match StaticAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data))
                .unwrap()
        }
        None => Response::builder()
            .status(404)
            .body(Body::from("Not Found"))
            .unwrap(),
    }
}

#[cfg(embed_static)]
async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || !path.contains('.') {
        return serve_static("index.html");
    }

    let response = serve_static(path);
    if response.status() == axum::http::StatusCode::NOT_FOUND {
        serve_static("index.html")
    } else {
        response
    }
}

#[cfg(not(embed_static))]
async fn dev_mode_fallback() -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"message":"Running in dev mode. Start frontend with: cd web && bun run dev"}"#,
        ))
        .unwrap()
}

pub async fn run_server(state: AppState, port: u16) -> anyhow::Result<()> {
    let addr = format!("{}:{}", state.config.server.host, port);
    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::AddrInUse {
            anyhow::anyhow!(
                "Port {} is already in use. Try a different port with --port <PORT>",
                port
            )
        } else {
            anyhow::anyhow!("Failed to bind to {}: {}", addr, e)
        }
    })?;

    tracing::info!("Engai server running on http://{}", addr);
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
    let mut app = Router::new()
        .nest("/api", api)
        .with_state(state)
        .layer(cors);

    #[cfg(embed_static)]
    {
        app = app.fallback(static_handler);
    }

    #[cfg(not(embed_static))]
    {
        app = app.fallback(dev_mode_fallback);
    }

    app
}
