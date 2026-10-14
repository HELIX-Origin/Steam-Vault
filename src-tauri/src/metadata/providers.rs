//! Provider registry: which metadata sources exist and whether each needs a key.

use serde::Serialize;

/// Every metadata provider Steam Vault knows about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Provider {
    /// Steam store `appdetails` endpoint (keyless).
    SteamStore,
    /// Official Steam Web API (`IPlayerService`, `ISteamUser`).
    SteamWebApi,
    /// SteamGridDB artwork (requires a key).
    SteamGridDb,
    /// IGDB (Twitch) game metadata (requires a client id + secret).
    Igdb,
    /// PCGamingWiki (keyless).
    PcGamingWiki,
    /// Wikipedia / Wikidata summaries (keyless).
    Wikipedia,
}

impl Provider {
    pub fn id(self) -> &'static str {
        match self {
            Provider::SteamStore => "steam-store",
            Provider::SteamWebApi => "steam-web-api",
            Provider::SteamGridDb => "steamgriddb",
            Provider::Igdb => "igdb",
            Provider::PcGamingWiki => "pcgamingwiki",
            Provider::Wikipedia => "wikipedia",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Provider::SteamStore => "Steam Store",
            Provider::SteamWebApi => "Steam Web API",
            Provider::SteamGridDb => "SteamGridDB",
            Provider::Igdb => "IGDB",
            Provider::PcGamingWiki => "PCGamingWiki",
            Provider::Wikipedia => "Wikipedia",
        }
    }

    /// Whether this provider needs an API key/secret from the user.
    pub fn requires_key(self) -> bool {
        matches!(
            self,
            Provider::SteamWebApi | Provider::SteamGridDb | Provider::Igdb
        )
    }

    pub fn category(self) -> &'static str {
        match self {
            Provider::SteamStore | Provider::SteamWebApi => "metadata",
            Provider::SteamGridDb => "artwork",
            Provider::Igdb => "metadata",
            Provider::PcGamingWiki | Provider::Wikipedia => "metadata",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Provider::SteamStore => {
                "Genres, developers, publishers, release dates, and feature flags for real Steam apps."
            }
            Provider::SteamWebApi => {
                "Your owned games and play time via IPlayerService. Requires a Steam Web API key."
            }
            Provider::SteamGridDb => {
                "Grids, heroes, logos, and icons for Steam games and non-Steam shortcuts. Requires a SteamGridDB key."
            }
            Provider::Igdb => {
                "Rich game metadata (summary, genres, companies, cover). Requires a Twitch client id and secret."
            }
            Provider::PcGamingWiki => {
                "Keyless cross-platform data: engine, developers, publishers, and release dates."
            }
            Provider::Wikipedia => "Keyless article summaries used as a fallback description.",
        }
    }
}

/// Serialized provider descriptor for the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub label: String,
    pub requires_key: bool,
    pub category: String,
    pub description: String,
}

/// The full ordered provider list.
pub const ALL: [Provider; 6] = [
    Provider::SteamStore,
    Provider::SteamWebApi,
    Provider::SteamGridDb,
    Provider::Igdb,
    Provider::PcGamingWiki,
    Provider::Wikipedia,
];

/// Describe every provider.
pub fn all() -> Vec<ProviderInfo> {
    ALL.iter()
        .map(|provider| ProviderInfo {
            id: provider.id().to_string(),
            label: provider.label().to_string(),
            requires_key: provider.requires_key(),
            category: provider.category().to_string(),
            description: provider.description().to_string(),
        })
        .collect()
}
