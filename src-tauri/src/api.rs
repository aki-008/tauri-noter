use serde::{Deserialize, Serialize};

use crate::state::UserInfo;
use crate::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct SyncChange {
    pub id: String,
    pub title: String,
    pub content: String,
    pub updated_at: String,
    pub deleted: bool,
}

#[derive(Debug, Serialize)]
pub struct SyncRequestBody {
    pub changes: Vec<SyncChange>,
}

#[derive(Debug, Deserialize)]
pub struct NoteResponse {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SyncResponseBody {
    pub notes: Vec<NoteResponse>,
}

pub async fn wait_for_backend(base_url: &str) -> Result<(), AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| AppError::Http(e.to_string()))?;

    for i in 0..15 {
        let resp = client.get(format!("{}/api/health", base_url)).send().await;
        if resp.is_ok() && resp.unwrap().status().is_success() {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(500 * (i + 1))).await;
    }
    Err(AppError::Http("Backend not reachable. Ensure FastAPI is running on localhost:8000 and JWT_SECRET is set.".to_string()))
}

fn client_with_timeout() -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Http(e.to_string()))
}

pub async fn register(
    base_url: &str,
    username: &str,
    password: &str,
) -> Result<AuthResponse, AppError> {
    wait_for_backend(base_url).await?;
    let client = client_with_timeout()?;
    let body = serde_json::json!({ "username": username, "password": password });
    let resp = client
        .post(format!("{}/api/auth/register", base_url))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Http(e.to_string()))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::Api(text));
    }
    resp.json()
        .await
        .map_err(|e| AppError::Http(e.to_string()))
}

pub async fn login(
    base_url: &str,
    username: &str,
    password: &str,
) -> Result<AuthResponse, AppError> {
    wait_for_backend(base_url).await?;
    let client = client_with_timeout()?;
    let body = serde_json::json!({ "username": username, "password": password });
    let resp = client
        .post(format!("{}/api/auth/login", base_url))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Http(e.to_string()))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::Api(text));
    }
    resp.json()
        .await
        .map_err(|e| AppError::Http(e.to_string()))
}

pub async fn sync(
    base_url: &str,
    token: &str,
    changes: Vec<SyncChange>,
) -> Result<Vec<NoteResponse>, AppError> {
    let client = client_with_timeout()?;
    let body = SyncRequestBody { changes };
    let resp = client
        .post(format!("{}/api/notes/sync", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Http(e.to_string()))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::Api(text));
    }
    let sync_resp: SyncResponseBody = resp
        .json()
        .await
        .map_err(|e| AppError::Http(e.to_string()))?;
    Ok(sync_resp.notes)
}
