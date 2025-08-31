use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use axum::{http::{self, StatusCode}, response::IntoResponse, routing::get, Router};
use tower::ServiceBuilder;
use tower_http::{trace::TraceLayer, cors::{CorsLayer, Any}};
use tracing::info;

// use crate::config::Config;

mod mail;

pub async fn serve() {
    tracing_subscriber::fmt()
        .with_target(true)
        .compact()
        .init();

    let app = api_router()
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

fn api_router() -> Router {
    Router::new()
        .route("/", get(root))
        .merge(mail::router())
}

async fn root() -> impl IntoResponse {
    (StatusCode::OK, "Mail Tracker API")
}