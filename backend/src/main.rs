use axum::extract::FromRef;
use axum::Router;
use axum_extra::extract::cookie::Key;
use sqlx::PgPool;
use tower_http::services::{ServeDir, ServeFile};

mod auth;
mod customers;
mod dashboard;
mod deals;
mod mail;
mod order;
mod payments;
mod router;
mod user;

use router::create_api_router;

#[derive(Clone)]
pub struct AppState {
    pub postgres: PgPool,
    pub supabase_postgres: PgPool,
    pub supabase_storage_url: String,
    pub supabase_api_key: String,
    pub stripe_key: String,
    pub stripe_sub_price: String,
    pub mailgun_key: String,
    pub mailgun_url: String,
    pub domain: String,
    pub key: Key,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] postgres: PgPool,
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&postgres)
        .await
        .expect("Failed to run migrations");
    // Initialize Supabase PostgreSQL Pool
    let (
        stripe_key,
        stripe_sub_price,
        mailgun_key,
        mailgun_url,
        domain,
        supabase_url,
        supabase_storage_url,
        supabase_api_key,
    ) = grab_secrets(secrets);

    let supabase_postgres = PgPool::connect(&supabase_url)
        .await
        .expect("Failed to connect to Supabase PostgreSQL");

    let state = AppState {
        postgres,
        supabase_postgres,
        stripe_key,
        stripe_sub_price,
        mailgun_key,
        mailgun_url,
        domain,
        key: Key::generate(),
        supabase_api_key,
        supabase_storage_url,
    };

    let api_router = create_api_router(state);

    let router = Router::new().nest("/api", api_router).nest_service(
        "/",
        ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html")),
    );

    Ok(router.into())
}

fn grab_secrets(
    secrets: shuttle_runtime::SecretStore,
) -> (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
) {
    let stripe_key = secrets
        .get("STRIPE_KEY")
        .unwrap_or_else(|| "None".to_string());

    let stripe_sub_price = secrets
        .get("STRIPE_SUB_PRICE")
        .unwrap_or_else(|| "None".to_string());

    let mailgun_key = secrets
        .get("MAILGUN_KEY")
        .unwrap_or_else(|| "None".to_string());

    let mailgun_url = secrets
        .get("MAILGUN_URL")
        .unwrap_or_else(|| "None".to_string());

    let domain = secrets
        .get("DOMAIN_URL")
        .unwrap_or_else(|| "http://127.0.0.1:8000".to_string());

    // Add Supabase PostgreSQL URL from secrets
    let supabase_url = secrets
        .get("SUPABASE_DB_URL")
        .expect("Supabase DB URL must be set");

    let supabase_storage_url = secrets
        .get("SUPABASE_STORAGE_URL")
        .unwrap_or_else(|| "".to_string());

    let supabase_api_key = secrets
        .get("SUPABASE_API_KEY")
        .unwrap_or_else(|| "".to_string());

    (
        stripe_key,
        stripe_sub_price,
        mailgun_key,
        mailgun_url,
        domain,
        supabase_url,
        supabase_storage_url,
        supabase_api_key,
    )
}
