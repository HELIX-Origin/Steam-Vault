//! Metadata providers.
//!
//! Only three providers require an API key: **Steam Web API**, **SteamGridDB**,
//! and **IGDB** (Twitch). Every other provider here is keyless. `RAWG` is
//! intentionally **not** supported.

pub mod local;
pub mod providers;
pub mod steam_search;
pub mod steam_store;
pub mod steam_web;

use std::time::Duration;

/// Build the shared HTTP client used by every provider.
pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("UniversalGameLauncher/1.0 (HELIX Origin; +https://github.com/HELIX-Origin/Universal-Game-Launcher)")
        .timeout(Duration::from_secs(30))
        .build()
        .expect("failed to build HTTP client")
}
