//! SteamGridDB v2 artwork client (requires an API key).
//!
//! Base: `https://www.steamgriddb.com/api/v2`, authenticated with a bearer token.
//! Used for grid/hero/logo/icon artwork, especially for non-Steam shortcuts.

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const BASE: &str = "https://www.steamgriddb.com/api/v2";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SgdbGame {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub release_date: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SgdbAsset {
    pub id: u32,
    pub url: String,
    #[serde(default)]
    pub thumb: Option<String>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default)]
    pub mime: Option<String>,
}

fn auth(key: &str) -> String {
    format!("Bearer {key}")
}

fn check(response: &reqwest::Response) -> AppResult<()> {
    if !response.status().is_success() {
        return Err(AppError::msg(format!(
            "SteamGridDB request failed with status {}",
            response.status()
        )));
    }
    Ok(())
}

/// Extract the `data` array from a SteamGridDB envelope.
///
/// Parsing via `serde_json::Value` avoids adding a `T: Default` bound that a
/// generic `#[serde(default)]` field would otherwise require.
async fn parse_list<T: serde::de::DeserializeOwned>(response: reqwest::Response) -> AppResult<Vec<T>> {
    let value: serde_json::Value = response.json().await?;
    match value.get("data") {
        Some(data) => serde_json::from_value(data.clone())
            .map_err(|error| AppError::msg(format!("failed to parse SteamGridDB response: {error}"))),
        None => Ok(Vec::new()),
    }
}

/// Autocomplete a game name, returning SteamGridDB game ids.
pub async fn search_games(
    client: &reqwest::Client,
    key: &str,
    term: &str,
) -> AppResult<Vec<SgdbGame>> {
    let url = format!("{BASE}/search/autocomplete/{term}");
    let response = client.get(&url).header("Authorization", auth(key)).send().await?;
    check(&response)?;
    parse_list(response).await
}

/// Fetch grids (covers) for a game, optionally filtered by dimensions.
pub async fn grids(
    client: &reqwest::Client,
    key: &str,
    game_id: u32,
    dimensions: Option<&str>,
) -> AppResult<Vec<SgdbAsset>> {
    let path = match dimensions {
        Some(dimensions) => format!("grids/game/{game_id}?dimensions={dimensions}"),
        None => format!("grids/game/{game_id}"),
    };
    list_assets(client, key, &path).await
}

pub async fn heroes(client: &reqwest::Client, key: &str, game_id: u32) -> AppResult<Vec<SgdbAsset>> {
    list_assets(client, key, &format!("heroes/game/{game_id}")).await
}

pub async fn logos(client: &reqwest::Client, key: &str, game_id: u32) -> AppResult<Vec<SgdbAsset>> {
    list_assets(client, key, &format!("logos/game/{game_id}")).await
}

pub async fn icons(client: &reqwest::Client, key: &str, game_id: u32) -> AppResult<Vec<SgdbAsset>> {
    list_assets(client, key, &format!("icons/game/{game_id}")).await
}

async fn list_assets(
    client: &reqwest::Client,
    key: &str,
    path: &str,
) -> AppResult<Vec<SgdbAsset>> {
    let url = format!("{BASE}/{path}");
    let response = client.get(&url).header("Authorization", auth(key)).send().await?;
    check(&response)?;
    parse_list(response).await
}
