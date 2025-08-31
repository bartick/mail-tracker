use axum::{body::Body, extract::{Path, Query}, http::{header, Request, StatusCode}, response::{IntoResponse, Response}, routing::get, Router};
use tracing::info;
use uuid::Uuid;
use serde::Deserialize;

const PIXEL_BYTES: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae,
    0x42, 0x60, 0x82,
];

pub fn router() -> Router {
    // By having each module responsible for setting up its own routing,
    // it makes the root module a lot cleaner.
    Router::new()
        .route("/mail/{filename}", get(image_handler))
}

#[derive(Debug, Deserialize)]
struct MailParams {
    email: String,
    subject: String,
    uuid: String,
}

fn parse_byte_string_uuid(id_str: &str) -> Result<Uuid, String> {
    let bytes_result: Result<Vec<u8>, _> = id_str
        .trim()
        .split(',')
        .map(|s| s.trim().parse::<u8>()) // .trim() is good practice
        .collect();

    let bytes_vec = match bytes_result {
        Ok(vec) => vec,
        Err(e) => return Err(format!("Failed to parse a byte value: {}. Part was '{}'.", e, id_str)),
    };

    if bytes_vec.len() != 16 {
        return Err(format!(
            "Expected 16 bytes, but found {}. Values: {:?}",
            bytes_vec.len(),
            bytes_vec
        ));
    }

    let bytes_array: [u8; 16] = bytes_vec.try_into().unwrap();
    Ok(Uuid::from_bytes(bytes_array))
}

async fn image_handler(Path(filename): Path<String>, Query(params): Query<MailParams>, request: Request<Body>) -> Response {
    info!("   From IP: {:?}", request.headers().get("x-forwarded-for").or_else(|| request.headers().get("host")).unwrap());

    info!("   Email: {}, Subject: {}", params.email, params.subject);
    let emails = params.email.split(',').collect::<Vec<&str>>();
    info!("   Emails: {:?}", emails);
    let (uuid, err) = match parse_byte_string_uuid(&params.uuid) {
        Ok(uuid) => {
            (uuid, None)
        }
        Err(e) => {
            (Uuid::nil(), Some(e))
        }
    };
    if err.is_some() {
        info!("   Invalid UUID format: {}. Raw string: {}", err.unwrap(), params.uuid);
    } else {
        info!("   UUID: {}", uuid);
    }

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