use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrawlApiError {
    #[error("HTTP request failed: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
}

pub type Result<T> = std::result::Result<T, BrawlApiError>;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Player {
    pub tag: String,
    pub name: String,
    pub trophies: i32,
    #[serde(rename = "highestTrophies")]
    pub highest_trophies: Option<i32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Clan {
    pub tag: String,
    pub name: String,
    pub trophies: i32,
    #[serde(rename = "memberCount")]
    pub member_count: i32,
}

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    base_url: String,
}

impl Client {
    pub fn new(token: &str) -> Self {
        let mut headers = HeaderMap::new();
        let auth_value = format!("Bearer {}", token);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value).expect("Invalid token"),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http,
            base_url: "https://api.brawlstars.com/v1".to_string(),
        }
    }

    pub async fn get_player(&self, tag: &str) -> Result<Player> {
        let normalized_tag = if tag.starts_with('#') {
            tag.to_string()
        } else {
            format!("#{}", tag)
        };

        let encoded_tag = urlencoding::encode(&normalized_tag);
        let url = format!("{}/players/{}", self.base_url, encoded_tag);

        let response = self.http.get(&url).send().await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(BrawlApiError::Api(format!("{}: {}", status, text)));
        }

        if text.is_empty() {
            return Err(BrawlApiError::Api("Empty response body".to_string()));
        }

        let player = serde_json::from_str::<Player>(&text).map_err(|e| {
            BrawlApiError::Api(format!("Failed to parse JSON: {} - body: {}", e, &text[..text.len().min(200)]))
        })?;
        Ok(player)
    }

    pub async fn get_clan(&self, tag: &str) -> Result<Clan> {
        let normalized_tag = if tag.starts_with('#') {
            tag.to_string()
        } else {
            format!("#{}", tag)
        };

        let encoded_tag = urlencoding::encode(&normalized_tag);
        let url = format!("{}/clubs/{}", self.base_url, encoded_tag);

        let response = self.http.get(&url).send().await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(BrawlApiError::Api(format!("{}: {}", status, text)));
        }

        if text.is_empty() {
            return Err(BrawlApiError::Api("Empty response body".to_string()));
        }

        let clan = serde_json::from_str::<Clan>(&text).map_err(|e| {
            BrawlApiError::Api(format!("Failed to parse JSON: {} - body: {}", e, &text[..text.len().min(200)]))
        })?;
        Ok(clan)
    }
}
