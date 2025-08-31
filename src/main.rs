use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use axum::{body::Body, extract::{Path, Query}, http::{self, header, Request, StatusCode}, response::{IntoResponse, Response}, routing::get, Router};
use tower::ServiceBuilder;
use tower_http::{trace::TraceLayer, cors::{CorsLayer, Any}};
use tracing::info;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MailParams {
    email: String,
    subject: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(true)
        .compact()
        .init();

    let app = Router::new()
        .route("/", get(root))
        .route("/mail/{filename}", get(image_handler))
        .layer(
            ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(
                CorsLayer::new()
                    .allow_methods([http::Method::GET])
                    .allow_origin(Any)
            )
        );

    let ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    let port = 3000;
    let socket_addr = SocketAddr::new(ip, port);
    let listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();

    info!("🚀 Server listening on http://{}", socket_addr);

    axum::serve(listener, app).await.unwrap();

}

async fn root() -> impl IntoResponse {
    "Hello, world!"
}

const PIXEL_BYTES: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae,
    0x42, 0x60, 0x82,
];

async fn image_handler(Path(filename): Path<String>, Query(params): Query<MailParams>, request: Request<Body>) -> Response {
    info!("   From IP: {:?}", request.headers().get("x-forwarded-for").or_else(|| request.headers().get("host")).unwrap());

    // Get Params
    info!("   Email: {}, Subject: {}", params.email, params.subject);
    let emails = params.email.split(',').collect::<Vec<&str>>();
    info!("   Emails: {:?}", emails);

    match filename.split_once('.') {
        Some((_, ext)) => {
            match ext {
                "png" => return (StatusCode::OK, [(header::CONTENT_TYPE, "image/png")], axum::response::Html(PIXEL_BYTES)).into_response(),
                "webp" => return (StatusCode::OK, [(header::CONTENT_TYPE, "image/webp")], axum::response::Html(PIXEL_BYTES)).into_response(),
                "jpg" | "jpeg" => return (StatusCode::OK, [(header::CONTENT_TYPE, "image/jpeg")], axum::response::Html(PIXEL_BYTES)).into_response(),
                _ => return (StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported Media Type".to_string()).into_response(),
            }
        }
        None => return (StatusCode::NOT_FOUND, "Not Found".to_string()).into_response(),
    };
}