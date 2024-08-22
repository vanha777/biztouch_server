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
pub async fn get_all(
    State(state): State<AppState>,
    // Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let query = sqlx::query("SELECT * FROM my_table")
        .fetch_all(&state.postgres)
        .await;
    match query {
        Ok(rows) => {
            let result: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|row| {
                    serde_json::json!({
                        "type": row.get::<String, _>("name"),
                        "order": row.get::<String, _>("data"),
                        "created_at": row.get::<String, _>("created_at"),
                    })
                })
                .collect();

            (StatusCode::OK, Json(result)).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Something went wrong"})),
        )
            .into_response(),
    }
}
