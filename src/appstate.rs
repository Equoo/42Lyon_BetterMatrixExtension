use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::env;
use tokio::fs;

use crate::data::DataBase;

pub const BASE_URL: &str = "https://api.intra.42.fr";

#[derive(Debug, Clone)]
pub struct AppState {
    pub client: Client,
    pub uid: String,
    pub secret: String,
    pub db: DataBase,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: String,
    error_description: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ApiResponse {
    Token(TokenResponse),
    Error(ErrorResponse),
}

impl AppState {
    pub async fn new() -> Self {
        Self {
            client: Client::new(),
            uid: env::var("UID").expect("UID missing"),
            secret: env::var("SECRET").expect("SECRET missing"),
            db: DataBase::new().await.expect("Database creation failed"),
        }
    }

    pub async fn get_token(&self) -> Result<String> {
        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", &self.uid),
            ("client_secret", &self.secret),
        ];

        let res = self
            .client
            .post(format!("{}/oauth/token", BASE_URL))
            .form(&params)
            .send()
            .await?;

        let bytes = res.bytes().await?;

        match serde_json::from_slice::<ApiResponse>(&bytes)? {
            ApiResponse::Token(t) => Ok(t.access_token),
            ApiResponse::Error(e) => {
                Err(anyhow!("Failed to retrieve token: {}", e.error_description))
            }
        }
    }
}
