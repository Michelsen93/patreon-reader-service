use axum::response::IntoResponse;

pub async fn health_check() -> impl IntoResponse {
    "OK"
}

pub async fn ready_check() -> impl IntoResponse {
    "READY"
}
