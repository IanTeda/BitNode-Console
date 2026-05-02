use std::error::Error;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;

use axum::body::Body;
use axum::extract::Path;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{HeaderValue, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/assets"]
struct FrontendAssets;

#[derive(Debug)]
pub enum ServerError {
    MissingEmbeddedIndex,
    Io(std::io::Error),
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEmbeddedIndex => {
                write!(f, "frontend index asset was not embedded; run frontend build first")
            }
            Self::Io(error) => write!(f, "server IO error: {error}"),
        }
    }
}

impl Error for ServerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingEmbeddedIndex => None,
            Self::Io(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for ServerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn app_router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/", get(serve_index))
        .route("/{*path}", get(serve_path))
}

/// Runs the HTTP server and serves embedded frontend assets.
///
/// # Errors
///
/// Returns an error when the embedded `index.html` asset is unavailable,
/// when binding the TCP listener fails, or if the Axum server exits with an
/// I/O error.
pub async fn run_server(bind_addr: SocketAddr) -> Result<(), ServerError> {
    if FrontendAssets::get("index.html").is_none() {
        return Err(ServerError::MissingEmbeddedIndex);
    }

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app_router()).await?;
    Ok(())
}

async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn serve_index() -> Response<Body> {
    serve_embedded_or_404("index.html")
}

async fn serve_path(Path(path): Path<String>) -> Response<Body> {
    let trimmed_path = path.trim_start_matches('/');

    if FrontendAssets::get(trimmed_path).is_some() {
        return serve_embedded_or_404(trimmed_path);
    }

    if is_probably_asset_request(trimmed_path) {
        return not_found_response();
    }

    serve_embedded_or_404("index.html")
}

fn serve_embedded_or_404(path: &str) -> Response<Body> {
    let Some(asset) = FrontendAssets::get(path) else {
        return not_found_response();
    };

    let cache_value = if path == "index.html" {
        HeaderValue::from_static("no-store")
    } else {
        HeaderValue::from_static("public, max-age=31536000, immutable")
    };

    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let mut response = Response::new(Body::from(asset.data.into_owned()));
    *response.status_mut() = StatusCode::OK;

    let headers = response.headers_mut();
    headers.insert(CACHE_CONTROL, cache_value);

    if let Ok(content_type) = HeaderValue::from_str(mime.essence_str()) {
        headers.insert(CONTENT_TYPE, content_type);
    }

    response
}

fn not_found_response() -> Response<Body> {
    let mut response = Response::new(Body::from("not found"));
    *response.status_mut() = StatusCode::NOT_FOUND;
    response
}

fn is_probably_asset_request(path: &str) -> bool {
    path.rsplit('/').next().is_some_and(|segment| segment.contains('.'))
}

#[cfg(test)]
mod tests {
    use super::is_probably_asset_request;

    #[test]
    fn detects_asset_like_paths() {
        assert!(is_probably_asset_request("assets/main-12345.js"));
        assert!(is_probably_asset_request("favicon.ico"));
        assert!(!is_probably_asset_request("dashboard"));
        assert!(!is_probably_asset_request("settings/profile"));
    }
}
