use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar, SameSite};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, Row};
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

#[derive(Deserialize, sqlx::FromRow, Serialize)]
#[allow(non_snake_case)]
pub struct Order {
    pub id: i32,
    pub name: String,
    pub data: serde_json::Value,
}

pub async fn get_all(State(state): State<AppState>) -> Result<Json<Vec<Order>>, impl IntoResponse> {
    match sqlx::query_as::<_, Order>("SELECT * FROM my_table")
        .fetch_all(&state.postgres)
        .await
    {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
}
