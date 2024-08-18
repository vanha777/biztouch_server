use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar, SameSite};
use serde::Deserialize;
use sqlx::Row;
use time::Duration;

use crate::AppState;

#[derive(Deserialize)]
pub struct RegisterDetails {
    email: String,
    password: String,
    role: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginDetails {
    email: String,
    password: String,
}

pub async fn create(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let query = sqlx::query("INSERT INTO my_table (name, data) VALUES ($1, $2)")
        .bind("test 0".to_string())
        .bind(request)
        .execute(&state.postgres);
    match query.await {
        Ok(_) => (StatusCode::CREATED, "Order created!".to_string()).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            format!("Something went wrong: {e}"),
        )
            .into_response(),
    }
}
