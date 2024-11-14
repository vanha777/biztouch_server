use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Form, Json,
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar, SameSite};
use serde::{Deserialize, Serialize};
use serde_json::json;
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

#[derive(Serialize, Deserialize)]
pub struct OrderRequest {
    details: serde_json::Value,
    items: serde_json::Value,
    r#type: String,
    status: String,
    customer: serde_json::Value,
    warehouse: serde_json::Value,
    version: i32,
    references: serde_json::Value,
    id: String,
}

pub async fn create(
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Check for Authorization header
    let auth_header = match headers.get("Authorization") {
        Some(header) => header.to_str().unwrap_or(""),
        None => {
            return (StatusCode::UNAUTHORIZED, "No authorization header provided").into_response()
        }
    };

    if !auth_header.starts_with("Bearer ") {
        return (StatusCode::UNAUTHORIZED, "Invalid authorization format").into_response();
    }
    // let request = serde_json::to_value(&request).unwrap();
    let query = sqlx::query("INSERT INTO my_table (data) VALUES ($1)")
        .bind(&request)
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
pub struct ApiKey {
    pub client_id: String,
    pub client_secret: String,
}

pub async fn oauth_token(
    State(state): State<AppState>,
    Form(request): Form<ApiKey>,
) -> impl IntoResponse {
    let res = json!({
        "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJjb21wYW55X2lkIjoiZDYyZjNiMGItMTY2OS00MjFkLWFlOWYtODEwYTQxZjdlYTllIiwibG9jIjoiIiwic2Vzc2lvbiI6Inp1R1BzTDJuSXdBTllKcWJpT3p3b3pKQXpQdjB4N1AyT3U4ckcyaUkiLCJYLVRlbmFudCI6IiIsImV4cCI6MTczMDI2MDA2Nywicm9sZSI6InBoYXJtYWN5IiwicHJvZHVjdCI6WyJwaGFybWFjeSJdLCJ0b2tlbiI6ImF1dGhlbnRpY2F0aW9uIiwic3JfbG9naW4iOmZhbHNlLCJyZXF1aXJlX21mYSI6ZmFsc2V9.mIUe-6qFL8Eu7TkfDPNc_EzgvTLcW_AvTF9NEF4d0i6vjgFmpr_WssNgWupVjkoQr-JU3BWQN0wb7Ats-vqT62DsBvPf3WTX2JXTsztTOveKSVlEzPRNA7cPvHDvcB-hRHk-bMxJvDnVdg6i52eLaOpPqX4BFS_xdWUzv64B32fK26-jE0bZgSg9LN_x4ma5Noi1T9h3lt8qF0GCcAeFYPlXOpLTZqIOEi4ukYHb3iNA7EEcK0oAWQ9UbQRtPG9f8un0CRT6AJAFUifb3NMqs8mMK4-AJG2RIBusMorqu4LEFVYc6tmm69MYldwRhvyHmNfQJkZ1cbqNo8cdLeyu_w"
    });
    (StatusCode::CREATED, Json(res)).into_response()
}

#[derive(Deserialize, sqlx::FromRow, Serialize)]
#[allow(non_snake_case)]
pub struct Order {
    pub id: i32,
    pub name: Option<String>,
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
