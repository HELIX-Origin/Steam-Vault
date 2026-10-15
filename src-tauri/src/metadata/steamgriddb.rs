//! SteamGridDB v2 artwork client (requires an API key).
//!
//! Base: `https://www.steamgriddb.com/api/v2`, authenticated with a bearer token.
//! Used for grid/hero/logo/icon artwork, especially for non-Steam shortcuts.

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const BASE: &str = "https://www.steamgriddb.com/api/v2";

/// Steam's synthetic non-Steam-shortcut app ids begin at `0x80000000`.
const SHORTCUT_APP_ID_BASE: u32 = 0x8000_0000;

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
async fn parse_list<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> AppResult<Vec<T>> {
    let value: serde_json::Value = response.json().await?;
    match value.get("data") {
        Some(data) => serde_json::from_value(data.clone()).map_err(|error| {
            AppError::msg(format!("failed to parse SteamGridDB response: {error}"))
        }),
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
    let response = client
        .get(&url)
        .header("Authorization", auth(key))
        .send()
        .await?;
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

pub async fn heroes(
    client: &reqwest::Client,
    key: &str,
    game_id: u32,
) -> AppResult<Vec<SgdbAsset>> {
    list_assets(client, key, &format!("heroes/game/{game_id}")).await
}

pub async fn logos(client: &reqwest::Client, key: &str, game_id: u32) -> AppResult<Vec<SgdbAsset>> {
    list_assets(client, key, &format!("logos/game/{game_id}")).await
}

pub async fn icons(client: &reqwest::Client, key: &str, game_id: u32) -> AppResult<Vec<SgdbAsset>> {
    list_assets(client, key, &format!("icons/game/{game_id}")).await
}

async fn list_assets(client: &reqwest::Client, key: &str, path: &str) -> AppResult<Vec<SgdbAsset>> {
    let url = format!("{BASE}/{path}");
    let response = client
        .get(&url)
        .header("Authorization", auth(key))
        .send()
        .await?;
    check(&response)?;
    parse_list(response).await
}

/// Map a friendly artwork kind to its SteamGridDB route segment and default
/// query. Grids default to the portrait `600x900` capsule.
fn route(kind: &str) -> AppResult<(&'static str, Option<&'static str>)> {
    match kind {
        "grids" => Ok(("grids", Some("dimensions=600x900"))),
        "heroes" => Ok(("heroes", None)),
        "logos" => Ok(("logos", None)),
        "icons" => Ok(("icons", None)),
        other => Err(AppError::Invalid(format!("unknown artwork kind '{other}'"))),
    }
}

/// List an asset kind for a real Steam app id via `/{kind}/steam/{app_id}`.
pub async fn for_steam_app(
    client: &reqwest::Client,
    key: &str,
    kind: &str,
    app_id: u32,
) -> AppResult<Vec<SgdbAsset>> {
    let (segment, query) = route(kind)?;
    let path = match query {
        Some(query) => format!("{segment}/steam/{app_id}?{query}"),
        None => format!("{segment}/steam/{app_id}"),
    };
    list_assets(client, key, &path).await
}

/// List an asset kind for a SteamGridDB game id via `/{kind}/game/{game_id}`.
pub async fn for_game_id(
    client: &reqwest::Client,
    key: &str,
    kind: &str,
    game_id: u32,
) -> AppResult<Vec<SgdbAsset>> {
    let (segment, query) = route(kind)?;
    let path = match query {
        Some(query) => format!("{segment}/game/{game_id}?{query}"),
        None => format!("{segment}/game/{game_id}"),
    };
    list_assets(client, key, &path).await
}

/// Resolve artwork for a library entry.
///
/// Real Steam games (an app id below the shortcut range) are looked up by their
/// exact app id. Non-Steam shortcuts carry a synthetic app id with no store
/// page, so they are resolved by name — preferring an exact case-insensitive
/// match and otherwise the first autocomplete hit.
pub async fn for_game(
    client: &reqwest::Client,
    key: &str,
    app_id: u32,
    name: &str,
    kind: &str,
) -> AppResult<Vec<SgdbAsset>> {
    if app_id != 0 && app_id < SHORTCUT_APP_ID_BASE {
        return for_steam_app(client, key, kind, app_id).await;
    }
    let matches = search_games(client, key, name).await?;
    let Some(game) = pick_match(&matches, name) else {
        return Ok(Vec::new());
    };
    for_game_id(client, key, kind, game.id).await
}

/// Prefer an exact (case-insensitive) name match, else the first result.
fn pick_match<'a>(matches: &'a [SgdbGame], name: &str) -> Option<&'a SgdbGame> {
    matches
        .iter()
        .find(|game| game.name.eq_ignore_ascii_case(name))
        .or_else(|| matches.first())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_rejects_unknown_kinds() {
        assert!(route("grids").is_ok());
        assert_eq!(route("grids").unwrap().1, Some("dimensions=600x900"));
        assert_eq!(route("logos").unwrap().1, None);
        assert!(route("banners").is_err());
    }

    #[test]
    fn pick_match_prefers_an_exact_name() {
        let games = vec![
            SgdbGame {
                id: 1,
                name: "Portal 2".into(),
                release_date: None,
            },
            SgdbGame {
                id: 2,
                name: "Portal".into(),
                release_date: None,
            },
        ];
        assert_eq!(pick_match(&games, "portal").unwrap().id, 2);
        assert_eq!(pick_match(&games, "Portal 2").unwrap().id, 1);
        assert_eq!(pick_match(&games, "Portal 3").unwrap().id, 1);
        assert!(pick_match(&[], "Portal").is_none());
    }
}
