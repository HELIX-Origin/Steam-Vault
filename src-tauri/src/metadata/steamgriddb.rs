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
/// query. Grids support portrait (`600x900,342x482`) and landscape (`460x215,920x430`).
fn route(kind: &str) -> AppResult<(&'static str, Option<&'static str>)> {
    match kind {
        "grids" | "grids_portrait" | "portrait" => {
            Ok(("grids", Some("dimensions=600x900,342x482")))
        }
        "grids_landscape" | "landscape" | "landscapes" => {
            Ok(("grids", Some("dimensions=460x215,920x430")))
        }
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
    for_steam_app_with_query(client, key, kind, app_id, "nsfw=false").await
}

pub async fn for_steam_app_with_query(
    client: &reqwest::Client,
    key: &str,
    kind: &str,
    app_id: u32,
    extra_query: &str,
) -> AppResult<Vec<SgdbAsset>> {
    let (segment, query) = route(kind)?;
    let query_str = match query {
        Some(query) => format!("{query}&{extra_query}"),
        None => extra_query.to_string(),
    };
    let path = format!("{segment}/steam/{app_id}?{query_str}");
    list_assets(client, key, &path).await
}

/// List an asset kind for a SteamGridDB game id via `/{kind}/game/{game_id}`.
pub async fn for_game_id(
    client: &reqwest::Client,
    key: &str,
    kind: &str,
    game_id: u32,
) -> AppResult<Vec<SgdbAsset>> {
    for_game_id_with_query(client, key, kind, game_id, "nsfw=false").await
}

pub async fn for_game_id_with_query(
    client: &reqwest::Client,
    key: &str,
    kind: &str,
    game_id: u32,
    extra_query: &str,
) -> AppResult<Vec<SgdbAsset>> {
    let (segment, query) = route(kind)?;
    let query_str = match query {
        Some(query) => format!("{query}&{extra_query}"),
        None => extra_query.to_string(),
    };
    let path = format!("{segment}/game/{game_id}?{query_str}");
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
    for_game_with_nsfw(client, key, app_id, name, kind, false).await
}

pub async fn for_game_with_nsfw(
    client: &reqwest::Client,
    key: &str,
    app_id: u32,
    name: &str,
    kind: &str,
    allow_nsfw: bool,
) -> AppResult<Vec<SgdbAsset>> {
    let nsfw_query = if allow_nsfw { "nsfw=any&humor=any" } else { "nsfw=false" };
    let mut results: Vec<SgdbAsset> = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    // 1. If keyword search query is provided, query SteamGridDB autocomplete for that term
    let clean_name = name.trim();
    if !clean_name.is_empty() {
        if let Ok(matches) = search_games(client, key, clean_name).await {
            // Check top 3 matches for keywords
            for game in matches.iter().take(3) {
                if let Ok(assets) = for_game_id_with_query(client, key, kind, game.id, nsfw_query).await {
                    for a in assets {
                        if seen_ids.insert(a.id) {
                            results.push(a);
                        }
                    }
                }
            }
        }
    }

    // 2. Also check direct Steam app ID if it's a real steam game (< SHORTCUT_APP_ID_BASE)
    if app_id != 0 && app_id < SHORTCUT_APP_ID_BASE {
        if let Ok(assets) = for_steam_app_with_query(client, key, kind, app_id, nsfw_query).await {
            for a in assets {
                if seen_ids.insert(a.id) {
                    results.push(a);
                }
            }
        }
    }

    Ok(results)
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
        assert_eq!(route("grids").unwrap().1, Some("dimensions=600x900,342x482"));
        assert_eq!(route("grids_landscape").unwrap().1, Some("dimensions=460x215,920x430"));
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
