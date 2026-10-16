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
    /// Visual Novel Database (keyless).
    Vndb,
    /// Wikipedia / Wikidata summaries (keyless).
    Wikipedia,
    /// GOG Store and Galaxy (keyless).
    Gog,
    /// Epic Games Store Catalog (keyless).
    Epic,
    /// IndieDB Indie games and engines portal (keyless).
    IndieDb,
    /// itch.io Indie platform and games (keyless).
    Itch,
}

impl Provider {
    pub fn id(self) -> &'static str {
        match self {
            Provider::SteamStore => "steam-store",
            Provider::SteamWebApi => "steam-web-api",
            Provider::SteamGridDb => "steamgriddb",
            Provider::Igdb => "igdb",
            Provider::PcGamingWiki => "pcgamingwiki",
            Provider::Vndb => "vndb",
            Provider::Wikipedia => "wikipedia",
            Provider::Gog => "gog",
            Provider::Epic => "epic",
            Provider::IndieDb => "indiedb",
            Provider::Itch => "itch",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Provider::SteamStore => "Steam Store",
            Provider::SteamWebApi => "Steam Web API",
            Provider::SteamGridDb => "SteamGridDB",
            Provider::Igdb => "IGDB",
            Provider::PcGamingWiki => "PCGamingWiki",
            Provider::Vndb => "VNDB",
            Provider::Wikipedia => "Wikipedia",
            Provider::Gog => "GOG",
            Provider::Epic => "Epic Games",
            Provider::IndieDb => "IndieDB",
            Provider::Itch => "itch.io",
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
            Provider::PcGamingWiki | Provider::Vndb | Provider::Wikipedia => "metadata",
            Provider::Gog | Provider::Epic | Provider::IndieDb | Provider::Itch => "metadata",
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
            Provider::Vndb => {
                "Keyless visual novel database: developers, publishers, tags, and covers for visual novels and Ren'Py titles."
            }
            Provider::Wikipedia => "Keyless article summaries used as a fallback description.",
            Provider::Gog => "Official GOG Store covers, heroes, developers, publishers, and tags.",
            Provider::Epic => "Official Epic Games Store covers, banners, logos, and descriptions.",
            Provider::IndieDb => "Indie games, engine classifications, developers, and artwork from IndieDB.",
            Provider::Itch => "Indie games, creators, covers, and descriptions from itch.io.",
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
pub const ALL: [Provider; 11] = [
    Provider::SteamStore,
    Provider::SteamWebApi,
    Provider::SteamGridDb,
    Provider::Igdb,
    Provider::PcGamingWiki,
    Provider::Vndb,
    Provider::Wikipedia,
    Provider::Gog,
    Provider::Epic,
    Provider::IndieDb,
    Provider::Itch,
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
