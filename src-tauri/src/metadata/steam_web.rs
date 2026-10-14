//! Official Steam Web API client (`IPlayerService`, `ISteamUser`).
//!
//! Requires an API key from <https://steamcommunity.com/dev/apikey>.

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const BASE: &str = "https://api.steampowered.com";

/// One entry from `IPlayerService/GetOwnedGames`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OwnedGame {
    pub appid: u32,
    pub name: Option<String>,
    pub playtime_forever: Option<u64>,
    pub playtime_2weeks: Option<u64>,
    pub img_icon_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OwnedGamesResponse {
    response: OwnedGamesInner,
}

#[derive(Debug, Deserialize, Default)]
struct OwnedGamesInner {
    #[serde(default)]
    game_count: u32,
    #[serde(default)]
    games: Vec<OwnedGame>,
}

/// Fetch the games owned by a SteamID64. The profile must be public.
pub async fn get_owned_games(
    client: &reqwest::Client,
    api_key: &str,
    steamid64: &str,
    include_app_info: bool,
) -> AppResult<Vec<OwnedGame>> {
    let app_info = if include_app_info { "1" } else { "0" };
    let url = format!(
        "{BASE}/IPlayerService/GetOwnedGames/v1/?key={api_key}&steamid={steamid64}&include_appinfo={app_info}&include_played_free_games=1&format=json"
    );
    let response = client.get(&url).send().await?;
    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(AppError::msg(
            "Steam rejected the request: check the API key and that the profile's game details are public.",
        ));
    }
    let parsed: OwnedGamesResponse = response.json().await?;
    Ok(parsed.response.games)
}

/// A `ISteamUser/GetPlayerSummaries` record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSummary {
    pub steamid: String,
    #[serde(default)]
    pub personaname: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub profileurl: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PlayerSummariesResponse {
    response: PlayerSummariesInner,
}

#[derive(Debug, Deserialize, Default)]
struct PlayerSummariesInner {
    #[serde(default)]
    players: Vec<PlayerSummary>,
}

/// Fetch player summaries for one or more SteamID64 values.
pub async fn get_player_summaries(
    client: &reqwest::Client,
    api_key: &str,
    steamids: &[String],
) -> AppResult<Vec<PlayerSummary>> {
    if steamids.is_empty() {
        return Ok(Vec::new());
    }
    let ids = steamids.join(",");
    let url = format!("{BASE}/ISteamUser/GetPlayerSummaries/v2/?key={api_key}&steamids={ids}");
    let response = client.get(&url).send().await?;
    let parsed: PlayerSummariesResponse = response.json().await?;
    Ok(parsed.response.players)
}
