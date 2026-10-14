//! IGDB (Twitch) client — requires a Twitch client id + secret.
//!
//! Auth: `POST https://id.twitch.tv/oauth2/token?client_id=…&client_secret=…&grant_type=client_credentials`.
//! Queries: `POST https://api.igdb.com/v4/<endpoint>` with `Client-ID` and
//! `Authorization: Bearer <token>` headers and an Apache-Query body.

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";
const API_URL: &str = "https://api.igdb.com/v4";

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

/// Request an app access token via the client-credentials flow.
pub async fn request_token(
    client: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
) -> AppResult<String> {
    let response = client
        .post(TOKEN_URL)
        .query(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", "client_credentials"),
        ])
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(AppError::msg(format!(
            "IGDB token request failed with status {}",
            response.status()
        )));
    }
    let parsed: TokenResponse = response.json().await?;
    Ok(parsed.access_token)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IgdbNamed {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IgdbCompany {
    pub company: Option<IgdbNamed>,
    #[serde(default)]
    pub developer: Option<bool>,
    #[serde(default)]
    pub publisher: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IgdbCover {
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgdbGame {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub first_release_date: Option<i64>,
    #[serde(default)]
    pub genres: Vec<IgdbNamed>,
    #[serde(default)]
    pub involved_companies: Vec<IgdbCompany>,
    #[serde(default)]
    pub cover: Option<IgdbCover>,
}

impl IgdbGame {
    /// A cover URL with the IGDB `//` scheme normalized to `https://`.
    pub fn cover_url(&self) -> Option<String> {
        self.cover
            .as_ref()
            .and_then(|cover| cover.url.as_deref())
            .map(|url| {
                let url = url.replace("t_thumb", "t_cover_big");
                if let Some(rest) = url.strip_prefix("//") {
                    format!("https://{rest}")
                } else {
                    url
                }
            })
    }
}

/// Search IGDB games by name.
pub async fn search_games(
    client: &reqwest::Client,
    client_id: &str,
    token: &str,
    query: &str,
    limit: u32,
) -> AppResult<Vec<IgdbGame>> {
    let escaped = query.replace('"', "");
    let body = format!(
        "search \"{escaped}\"; fields name,summary,genres.name,first_release_date,cover.url,involved_companies.company.name,involved_companies.developer,involved_companies.publisher; limit {limit};"
    );
    let response = client
        .post(format!("{API_URL}/games"))
        .header("Client-ID", client_id)
        .header("Authorization", format!("Bearer {token}"))
        .body(body)
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(AppError::msg(format!(
            "IGDB query failed with status {}",
            response.status()
        )));
    }
    let games: Vec<IgdbGame> = response.json().await?;
    Ok(games)
}
