mod ops_service;
use crate::ops_service::health_check;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use ops_service::ready_check;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, sync::Arc};

#[derive(Clone)]
struct AppState {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    http_client: Client,
}

#[derive(Deserialize)]
struct ExchangeRequest {
    code: String,
}

#[derive(Deserialize, Serialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    scope: String,
    token_type: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let state = AppState {
        client_id: env::var("PATREON_CLIENT_ID").expect("PATREON_CLIENT_ID not set"),
        client_secret: env::var("PATREON_CLIENT_SECRET").expect("PATREON_CLIENT_SECRET not set"),
        redirect_uri: env::var("PATREON_REDIRECT_URI").expect("PATREON_REDIRECT_URI not set"),
        http_client: Client::new(),
    };

    let app = Router::new()
        .route("/patreon/exchange", post(exchange_token))
        .route("/healthz", get(health_check))
        .route("/readyz", get(ready_check))
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn exchange_token(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ExchangeRequest>,
) -> impl IntoResponse {
    let mut form = HashMap::new();
    form.insert("code", payload.code);
    form.insert("grant_type", "authorization_code".to_string());
    form.insert("client_id", state.client_id.clone());
    form.insert("client_secret", state.client_secret.clone());
    form.insert("redirect_uri", state.redirect_uri.clone());

    let token_resp = state
        .http_client
        .post("https://www.patreon.com/api/oauth2/token")
        .form(&form)
        .send()
        .await;

    match token_resp {
        Ok(resp) => {
            if resp.status().is_success() {
                let token = resp.json().await.unwrap();
                (StatusCode::OK, Json(token))
            } else {
                let text = resp.text().await.unwrap_or_default();
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": text })),
                )
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
