use std::error::Error;

use axum::extract::Query;
use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use base64::decode;
use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Deserialize, sqlx::FromRow, Serialize)]
pub struct Customer {
    pub id: i32,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub phone: String,
    pub priority: i16,
}

#[derive(Deserialize, sqlx::FromRow, Serialize)]
pub struct Groups {
    pub id: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub number: String,
    pub name: String,
    pub company_id: i64,
}

#[derive(Deserialize, sqlx::FromRow, Serialize)]
pub struct UserResponse {
    id: Option<i64>,
    created_at: Option<DateTime<Utc>>, // Parsing TIMESTAMPTZ
    first_name: Option<String>,
    last_name: Option<String>,
    username: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    title: Option<String>,
    bio: Option<String>,
    photo: Option<String>,
    qr_code: Option<String>,
    theme: Option<String>,
    media: Option<serde_json::Value>,  // A list of Media objects
    social: Option<serde_json::Value>, // A list of Social objects
    linkable_id: Option<i64>,          // Nullable fields
    linkable_type: Option<String>,
    campaign_id: Option<i64>,
    address: Option<String>,
    suburb: Option<String>,
    post_code: Option<String>,
    country: Option<String>,
    state: Option<String>,
    r#type: Option<String>, // Nullable and renamed to avoid conflict with Rust's `type` keyword
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize)]
pub struct UserRequest {
    first_name: String,
    last_name: String,
    email: String,
    phone: String,
    title: String,
    bio: String,
    photo: String,
    qr_code: String,
    theme: String,
    media: Vec<Media>, // A list of Media objects
    social: Vec<Social>, // A list of Social objects
                       // linkable_id: Option<i64>, // Nullable fields
                       // linkable_type: Option<String>,
                       // campaign_id: Option<i64>,
                       // address: Option<String>,
                       // suburb: Option<String>,
                       // post_code: Option<String>,
                       // country: Option<String>,
                       // state: Option<String>,
                       // r#type: Option<String>, // Nullable and renamed to avoid conflict with Rust's `type` keyword
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeUserRequest {
    theme: String,
    #[serde(rename = "profileImage")]
    profile_image: String,
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
    phone: String,
    old_profile_media: Option<String>,
    old_cover_media: Option<String>,
    email: String,
    title: String,
    password: Option<String>,
    #[serde(rename = "confirmPassword")]
    confirm_password: Option<String>,
    bio: String,
    #[serde(rename = "coverImage")]
    cover_image: String,
    #[serde(rename = "coverType")]
    cover_type: String,
    #[serde(rename = "qrCode")]
    qr_code: String,
    social: Vec<Social>, // List of Social media objects
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize)]
struct Media {
    info: String,
    r#type: String,
    media: String,
}

// Struct for social platform information
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize)]
struct Social {
    link: String,
    icons: String,
    platforms: String,
}

pub async fn get(
    State(state): State<AppState>,
    // Query(id): Query<Option<String>>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, UserResponse>("SELECT * FROM users")
        .persistent(false)
        .fetch_all(&state.supabase_postgres)
        .await
    {
        Ok(users) => Json(users).into_response(),
        Err(e) => {
            eprintln!("Error fetching groups: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn create(
    State(state): State<AppState>,
    Json(mut new_user): Json<UserResponse>,
) -> impl IntoResponse {
    if new_user.username.is_none() {
        // Generate a unique random username
        let random_username = loop {
            let random_string: String = std::iter::repeat_with(fastrand::alphanumeric)
                .take(5)
                .collect();
            let username = format!(
                "{}.{}.{}",
                new_user.first_name.as_deref().unwrap_or(""),
                new_user.last_name.as_deref().unwrap_or(""),
                random_string
            );

            // Check if the username already exists in the database
            let exists: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
                    .persistent(false)
                    .bind(&username)
                    .fetch_one(&state.supabase_postgres)
                    .await
                    .unwrap_or(true); // Assume it exists if there's an error, to be safe

            if !exists {
                break username;
            }
        };
        new_user.username = Some(random_username);
    }

    let query = "INSERT INTO users (first_name, last_name, username, email, phone, title, bio, photo, qr_code, theme, media, social, linkable_id, linkable_type, campaign_id, address, suburb, post_code, country, state, type) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11::jsonb, $12::jsonb, $13, $14, $15, $16, $17, $18, $19, $20, $21) RETURNING *";
    match sqlx::query(query)
        .persistent(false)
        .bind(new_user.first_name)
        .bind(new_user.last_name)
        .bind(new_user.username)
        .bind(new_user.email)
        .bind(new_user.phone)
        .bind(new_user.title)
        .bind(new_user.bio)
        .bind(new_user.photo)
        .bind(new_user.qr_code)
        .bind(new_user.theme)
        .bind(new_user.media)
        .bind(new_user.social)
        .bind(new_user.linkable_id)
        .bind(new_user.linkable_type)
        .bind(new_user.campaign_id)
        .bind(new_user.address)
        .bind(new_user.suburb)
        .bind(new_user.post_code)
        .bind(new_user.country)
        .bind(new_user.state)
        .bind(new_user.r#type)
        .execute(&state.supabase_postgres)
        .await
    {
        Ok(created_user) => {
            if created_user.rows_affected() == 0 {
                (StatusCode::BAD_REQUEST, "Failed to created user").into_response()
            } else {
                (StatusCode::OK, "User created successfully").into_response()
            }
        }
        Err(e) => {
            eprintln!("Error creating user: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> impl IntoResponse {
    let query = "DELETE FROM users WHERE username = $1";
    match sqlx::query(query)
        .persistent(false)
        .bind(username)
        .execute(&state.supabase_postgres)
        .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                (StatusCode::NOT_FOUND, "User not found").into_response()
            } else {
                (StatusCode::OK, "User deleted successfully").into_response()
            }
        }
        Err(e) => {
            eprintln!("Error deleting user: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(updated_user): Json<FeUserRequest>,
) -> impl IntoResponse {
    println!("debug 0");
    // mapping updated_user to struct UserRequest
    // remove profile photo and upload ......
    let mut profile_media = String::new();
    let mut cover_media: Vec<Media> = Vec::new();
    match updated_user.old_profile_media {
        Some(x) if !x.is_empty() => {
            println!("debug 1");
            // overwrite existing one with new one
            match overwrite_in_supabase(&state.supabase_api_key, &x, &updated_user.profile_image)
                .await
            {
                Ok((file_link, _mime_type)) => profile_media = file_link,
                Err(e) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
                }
            }
            println!("debug 2");
        }
        _ => {
            println!("debug 3");
            // Check if the cover_image is a base64 string or a link
            if updated_user.profile_image.starts_with("http://")
                || updated_user.profile_image.starts_with("https://")
            {
                println!("debug 4");
                // It's a link, no need to upload
                profile_media = updated_user.profile_image;
            } else {
                println!("debug 5");
                // upload new first profile photo
                match upload_to_supabase(
                    &state.supabase_api_key,
                    &state.supabase_storage_url,
                    "profile_media",
                    &updated_user.profile_image,
                )
                .await
                {
                    Ok((file_link, _mime_type)) => profile_media = file_link,
                    Err(e) => {
                        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
                    }
                }
            }
        }
    }
    println!("debug 6");
    match updated_user.old_cover_media {
        Some(x) if !x.is_empty() && (x.starts_with("http://") || x.starts_with("https://")) => {
            println!("debug 7");
            // overwrite existing one with new one
            match overwrite_in_supabase(&state.supabase_api_key, &x, &updated_user.cover_image)
                .await
            {
                Ok((file_link, mime_type)) => {
                    println!("debug 8");
                    cover_media = vec![Media {
                        info: "This Is My Cover Media".to_string(),
                        r#type: match mime_type {
                            mime if mime.starts_with("image/") => "image".to_string(),
                            mime if mime.starts_with("video/") => "video".to_string(),
                            _ => "image".to_string(),
                        },
                        media: file_link,
                    }]
                }
                Err(e) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
                }
            }
        }
        _ => {
            println!("debug 9");
            // Check if the cover_image is a base64 string or a link
            if updated_user.cover_image.starts_with("http://")
                || updated_user.cover_image.starts_with("https://")
            {
                // It's a link, no need to upload
                cover_media = vec![Media {
                    info: "This Is My Cover Media".to_string(),
                    r#type: "image".to_string(), // Assuming it's an image link
                    media: updated_user.cover_image.clone(),
                }];
            } else {
                println!("debug 10");
                // If it's not a link, assume it's base64 and continue with the upload process
                // upload new first profile photo
                match upload_to_supabase(
                    &state.supabase_api_key,
                    &state.supabase_storage_url,
                    "cover_media",
                    &updated_user.cover_image,
                )
                .await
                {
                    Ok((file_link, mime_type)) => {
                        println!("debug 11");
                        cover_media = vec![Media {
                            info: "This Is My Cover Media".to_string(),
                            r#type: match mime_type {
                                mime if mime.starts_with("image/") => "image".to_string(),
                                mime if mime.starts_with("video/") => "video".to_string(),
                                _ => "image".to_string(),
                            },
                            media: file_link,
                        }]
                    }
                    Err(e) => {
                        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
                    }
                }
            }
        }
    }
    println!("debug 12");
    println!("this is upload profile response {:?}", profile_media);
    println!("this is upload cover response {:?}", cover_media);
    let request = UserRequest {
        first_name: updated_user.first_name,
        last_name: updated_user.last_name,
        email: updated_user.email,
        phone: updated_user.phone,
        title: updated_user.title,
        bio: updated_user.bio,
        photo: profile_media,
        qr_code: updated_user.qr_code,
        theme: updated_user.theme,
        media: cover_media,
        social: updated_user.social,
    };
    let query = "UPDATE users SET first_name = $1, last_name = $2, email = $3, phone = $4, title = $5, bio = $6, photo = $7, qr_code = $8, theme = $9, media = $10::jsonb, social = $11::jsonb WHERE username = $12 RETURNING *";
    println!("debug 13");
    match sqlx::query(query)
        .persistent(false)
        .bind(request.first_name)
        .bind(request.last_name)
        .bind(request.email)
        .bind(request.phone)
        .bind(request.title)
        .bind(request.bio)
        .bind(request.photo)
        .bind(request.qr_code)
        .bind(request.theme)
        .bind(match serde_json::to_value(&request.media) {
            Ok(serialized_social) => serialized_social,
            Err(e) => {
                eprintln!("Error serializing social: {:?}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error serializing social",
                )
                    .into_response();
            }
        })
        .bind(
            serde_json::to_value(request.social)
                .map_err(|e| {
                    eprintln!("Error serializing media: {:?}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Error serializing media")
                        .into_response();
                })
                .unwrap_or_default(),
        )
        // .bind(updated_user.linkable_id)
        // .bind(updated_user.linkable_type)
        // .bind(updated_user.campaign_id)
        // .bind(updated_user.address)
        // .bind(updated_user.suburb)
        // .bind(updated_user.post_code)
        // .bind(updated_user.country)
        // .bind(updated_user.state)
        // .bind(updated_user.r#type)
        .bind(username)
        .execute(&state.supabase_postgres)
        .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                (StatusCode::BAD_REQUEST, "User not updated").into_response()
            } else {
                (StatusCode::OK, "User updated successfully").into_response()
            }
        }
        Err(e) => {
            eprintln!("Error deleting user: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

// Function to overwrite profile photo to Supabase Storage
pub async fn overwrite_in_supabase(
    supabase_api_key: &str,
    file_link: &str,
    base64_data: &str,
) -> Result<(String, String), Box<dyn Error>> {
    let client = Client::new();

    // Decode the base64 string
    let base64_data = base64_data
        .trim_start_matches("data:image/jpeg;base64,")
        .trim_start_matches("data:image/png;base64,")
        .trim_start_matches("data:image/gif;base64,")
        .trim_start_matches("data:image/webp;base64,")
        .trim_start_matches("data:video/mp4;base64,")
        .trim_start_matches("data:video/quicktime;base64,")
        .trim_start_matches("data:video/webm;base64,");

    let decoded_bytes = decode(base64_data)?;

    // Detect MIME type from the decoded data
    let mime_type = detect_mime_type(&decoded_bytes)?;

    // Use PUT request to overwrite the file in Supabase Storage
    let read_url = file_link;
    let upload_url = file_link.replace("public/", "");
    println!("Uploading to: {}", upload_url);

    let response = client
        .put(upload_url)
        .bearer_auth(supabase_api_key) // Supabase API key
        .header("Content-Type", mime_type.as_str()) // Set the appropriate content type
        .body(decoded_bytes) // Send the decoded bytes
        .send()
        .await?;

    if response.status().is_success() {
        println!("File overwritten successfully.");
        return Ok((read_url.to_string(), mime_type));
    } else {
        return Err(format!("Failed to overwrite file: {}", response.text().await?).into());
    }
}

// Function to upload file to Supabase Storage
pub async fn upload_to_supabase(
    supabase_api_key: &str,
    supabase_url: &str,
    upload_bucket: &str,
    base64_data: &str,
) -> Result<(String, String), Box<dyn Error>> {
    let client = Client::new();
    // Decode the base64 string
    let data = base64_data
        .trim_start_matches("data:image/jpeg;base64,")
        .trim_start_matches("data:image/png;base64,")
        .trim_start_matches("data:image/gif;base64,")
        .trim_start_matches("data:image/webp;base64,")
        .trim_start_matches("data:video/mp4;base64,")
        .trim_start_matches("data:video/quicktime;base64,")
        .trim_start_matches("data:video/webm;base64,");
    let decoded_bytes = decode(data)?;

    // Determine if the file is an image or a video based on the first few bytes
    let mime_type = detect_mime_type(&decoded_bytes)?;

    // Generate a random hash for the file name
    let file_name = uuid::Uuid::new_v4().to_string();
    let read_url = format!(
        "{}/storage/v1/object/public/biz_touch/{}/{}",
        supabase_url, upload_bucket, file_name
    );
    let upload_url = format!(
        "{}/storage/v1/object/biz_touch/{}/{}",
        supabase_url, upload_bucket, file_name
    );

    // Upload to Supabase Storage
    match client
        .post(&upload_url)
        .bearer_auth(supabase_api_key) // Supabase API key
        .header("Content-Type", mime_type.as_str()) // Set the appropriate content type
        .body(decoded_bytes) // Send the decoded bytes
        .send()
        .await
    {
        Ok(x) => {
            if x.status().is_success() {
                Ok((read_url, mime_type))
            } else {
                Err(format!("Failed to upload file: {}", x.text().await?).into())
            }
        }
        Err(e) => Err(format!("Failed to upload file: {}", e.to_string()).into()),
    }
}

// Function to detect MIME type (image/video) based on the first few bytes of the file
fn detect_mime_type(bytes: &[u8]) -> Result<String, Box<dyn Error>> {
    // Use infer to detect the MIME type based on file content
    if let Some(kind) = infer::get(bytes) {
        Ok(kind.mime_type().to_string())
    } else {
        Ok("application/octet-stream".to_string()) // Default to octet-stream if not recognized
    }
}

fn get_file_extension(mime_type: &str) -> &str {
    match mime_type {
        "image/jpeg" => "jpeg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "video/mp4" => "mp4",
        "video/quicktime" => "mov",
        "video/webm" => "webm",
        _ => "unknown", // Handle unknown types gracefully
    }
}
