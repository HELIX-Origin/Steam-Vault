//! Default drop-in plugin definitions.
//!
//! These standard plugins are unpacked into the plugins directory as self-contained
//! folders containing their own `plugin.json` manifests on first run.

use std::fs;
use std::path::Path;

use super::manifest::{
    PluginArtworkConfig, PluginManifest, PluginMetadataConfig, PluginPlatformConfig, PluginType,
};

/// Generates all standard first-party drop-in plugin manifests.
pub fn default_plugin_manifests() -> Vec<PluginManifest> {
    vec![
        // -------------------------------------------------------------------
        // Platforms
        // -------------------------------------------------------------------
        PluginManifest {
            id: "platform-steam".to_string(),
            name: "Steam Integration".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Platform,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Discovers and launches games from Steam libraries and manifests.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["discovery".to_string(), "launch".to_string(), "store_cdn".to_string()],
            entrypoint: None,
            platform: Some(PluginPlatformConfig {
                platform_id: "steam".to_string(),
                launch_protocols: vec!["steam://rungameid/{appId}".to_string()],
                known_folders: vec![],
                executable_names: vec![],
            }),
            metadata: Some(PluginMetadataConfig {
                api_url: Some("https://store.steampowered.com/api/appdetails?appids={appId}".to_string()),
                artwork: Some(PluginArtworkConfig {
                    capsule_url: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{appId}/capsule_616x353.jpg".to_string()),
                    hero_url: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{appId}/library_hero.jpg".to_string()),
                    logo_url: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{appId}/logo.png".to_string()),
                }),
            }),
        },
        PluginManifest {
            id: "platform-gog".to_string(),
            name: "GOG Galaxy Integration".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Platform,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Discovers and launches games from GOG Galaxy registry and manifests.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["discovery".to_string(), "launch".to_string(), "direct_run".to_string()],
            entrypoint: None,
            platform: Some(PluginPlatformConfig {
                platform_id: "gog".to_string(),
                launch_protocols: vec!["goggalaxy://openGameView/{appId}".to_string()],
                known_folders: vec![
                    r"C:\Program Files (x86)\GOG Galaxy\Games".to_string(),
                    r"C:\GOG Games".to_string(),
                ],
                executable_names: vec![],
            }),
            metadata: Some(PluginMetadataConfig {
                api_url: Some("https://catalog.gog.com/v1/catalog".to_string()),
                artwork: Some(PluginArtworkConfig {
                    capsule_url: None,
                    hero_url: None,
                    logo_url: None,
                }),
            }),
        },
        PluginManifest {
            id: "platform-epic".to_string(),
            name: "Epic Games Integration".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Platform,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Discovers and launches games via Epic Games manifests and Legendary CLI.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["discovery".to_string(), "launch".to_string(), "legendary_sync".to_string()],
            entrypoint: None,
            platform: Some(PluginPlatformConfig {
                platform_id: "epic".to_string(),
                launch_protocols: vec![
                    "com.epicgames.launcher://apps/{appName}?action=launch&silent=true".to_string(),
                    "legendary://launch/{appName}".to_string(),
                ],
                known_folders: vec![
                    r"C:\Program Files\Epic Games".to_string(),
                ],
                executable_names: vec![],
            }),
            metadata: None,
        },
        PluginManifest {
            id: "platform-itch".to_string(),
            name: "itch.io Integration".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Platform,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Discovers indie titles and downloads from itch.io desktop client.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["discovery".to_string(), "launch".to_string()],
            entrypoint: None,
            platform: Some(PluginPlatformConfig {
                platform_id: "itch".to_string(),
                launch_protocols: vec!["itch://games/{appId}".to_string()],
                known_folders: vec![],
                executable_names: vec![],
            }),
            metadata: None,
        },
        PluginManifest {
            id: "platform-xbox".to_string(),
            name: "Xbox / PC Game Pass Integration".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Platform,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Discovers installed PC Game Pass and Microsoft Store titles.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["discovery".to_string(), "launch".to_string()],
            entrypoint: None,
            platform: Some(PluginPlatformConfig {
                platform_id: "xbox".to_string(),
                launch_protocols: vec!["shell:AppsFolder\\{appId}".to_string()],
                known_folders: vec![r"C:\XboxGames".to_string()],
                executable_names: vec![],
            }),
            metadata: None,
        },
        PluginManifest {
            id: "platform-battlenet".to_string(),
            name: "Battle.net Integration".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Platform,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Discovers installed Blizzard Battle.net games.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["discovery".to_string(), "launch".to_string()],
            entrypoint: None,
            platform: Some(PluginPlatformConfig {
                platform_id: "battlenet".to_string(),
                launch_protocols: vec!["battlenet://{appId}".to_string()],
                known_folders: vec![r"C:\Program Files (x86)\Battle.net".to_string()],
                executable_names: vec![],
            }),
            metadata: None,
        },
        PluginManifest {
            id: "platform-ea".to_string(),
            name: "EA App / Origin Integration".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Platform,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Discovers installed games from EA App and legacy Origin manifests.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["discovery".to_string(), "launch".to_string()],
            entrypoint: None,
            platform: Some(PluginPlatformConfig {
                platform_id: "ea".to_string(),
                launch_protocols: vec!["origin2://game/launch?offerId={appId}".to_string()],
                known_folders: vec![r"C:\Program Files\EA Games".to_string()],
                executable_names: vec![],
            }),
            metadata: None,
        },
        PluginManifest {
            id: "platform-ubisoft".to_string(),
            name: "Ubisoft Connect Integration".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Platform,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Discovers installed games from Ubisoft Connect.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["discovery".to_string(), "launch".to_string()],
            entrypoint: None,
            platform: Some(PluginPlatformConfig {
                platform_id: "ubisoft".to_string(),
                launch_protocols: vec!["uplay://launch/{appId}/0".to_string()],
                known_folders: vec![r"C:\Program Files (x86)\Ubisoft\Ubisoft Game Launcher\games".to_string()],
                executable_names: vec![],
            }),
            metadata: None,
        },
        PluginManifest {
            id: "platform-amazon".to_string(),
            name: "Amazon Games Integration".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Platform,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Discovers installed games from Amazon Games app.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["discovery".to_string(), "launch".to_string()],
            entrypoint: None,
            platform: Some(PluginPlatformConfig {
                platform_id: "amazon".to_string(),
                launch_protocols: vec!["amazon-games://play/{appId}".to_string()],
                known_folders: vec![r"C:\Amazon Games\Library".to_string()],
                executable_names: vec![],
            }),
            metadata: None,
        },
        PluginManifest {
            id: "platform-emulators".to_string(),
            name: "Emulators & RetroArch Integration".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Platform,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Scans custom ROM directories and launches via RetroArch or standalone emulators.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["discovery".to_string(), "rom_scanning".to_string(), "launch".to_string()],
            entrypoint: None,
            platform: Some(PluginPlatformConfig {
                platform_id: "emulators".to_string(),
                launch_protocols: vec![],
                known_folders: vec![],
                executable_names: vec!["retroarch.exe".to_string()],
            }),
            metadata: None,
        },

        // -------------------------------------------------------------------
        // Metadata Providers
        // -------------------------------------------------------------------
        PluginManifest {
            id: "metadata-steam".to_string(),
            name: "Steam Store Metadata".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Metadata,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Fetches official Steam Store CDN artwork, descriptions, tags, genres, and reviews.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["artwork".to_string(), "genres".to_string(), "tags".to_string(), "reviews".to_string()],
            entrypoint: None,
            platform: None,
            metadata: Some(PluginMetadataConfig {
                api_url: Some("https://store.steampowered.com/api/appdetails".to_string()),
                artwork: Some(PluginArtworkConfig {
                    capsule_url: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{appId}/capsule_616x353.jpg".to_string()),
                    hero_url: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{appId}/library_hero.jpg".to_string()),
                    logo_url: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{appId}/logo.png".to_string()),
                }),
            }),
        },
        PluginManifest {
            id: "metadata-gog".to_string(),
            name: "GOG Store Catalog".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Metadata,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Queries official GOG catalog endpoints for store artwork, game summaries, and details.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["artwork".to_string(), "summaries".to_string()],
            entrypoint: None,
            platform: None,
            metadata: Some(PluginMetadataConfig {
                api_url: Some("https://catalog.gog.com/v1/catalog".to_string()),
                artwork: None,
            }),
        },
        PluginManifest {
            id: "metadata-epic".to_string(),
            name: "Epic Games Catalog".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Metadata,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Fetches Epic Games Store catalog manifests, promotional artwork, and descriptions.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["artwork".to_string(), "promotions".to_string()],
            entrypoint: None,
            platform: None,
            metadata: Some(PluginMetadataConfig {
                api_url: Some("https://store-content.ak.epicgames.com/api/en-US/content/products/{appId}".to_string()),
                artwork: None,
            }),
        },
        PluginManifest {
            id: "metadata-itch".to_string(),
            name: "itch.io Catalog".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Metadata,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Resolves indie game information, developer credits, and screenshots from itch.io.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["indie_meta".to_string(), "screenshots".to_string()],
            entrypoint: None,
            platform: None,
            metadata: Some(PluginMetadataConfig {
                api_url: Some("https://itch.io/api/1/{apiKey}/game/{appId}".to_string()),
                artwork: None,
            }),
        },
        PluginManifest {
            id: "metadata-igdb".to_string(),
            name: "IGDB Metadata".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Metadata,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Comprehensive video game database queries via IGDB / Twitch API.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["genres".to_string(), "themes".to_string(), "release_dates".to_string(), "ratings".to_string()],
            entrypoint: None,
            platform: None,
            metadata: Some(PluginMetadataConfig {
                api_url: Some("https://api.igdb.com/v4/games".to_string()),
                artwork: None,
            }),
        },
        PluginManifest {
            id: "metadata-steamgriddb".to_string(),
            name: "SteamGridDB".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Metadata,
            author: Some("HELIX-Origin".to_string()),
            description: Some("High-resolution community cover capsules, heroes, logos, and icons.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["capsules".to_string(), "heroes".to_string(), "logos".to_string(), "icons".to_string()],
            entrypoint: None,
            platform: None,
            metadata: Some(PluginMetadataConfig {
                api_url: Some("https://www.steamgriddb.com/api/v2/grids/game/{appId}".to_string()),
                artwork: None,
            }),
        },
        PluginManifest {
            id: "metadata-pcgamingwiki".to_string(),
            name: "PCGamingWiki".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Metadata,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Controller support flags, video/audio configurations, and community bugfixes.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["compatibility".to_string(), "fixes".to_string(), "controller_support".to_string()],
            entrypoint: None,
            platform: None,
            metadata: Some(PluginMetadataConfig {
                api_url: Some("https://www.pcgamingwiki.com/w/api.php".to_string()),
                artwork: None,
            }),
        },
        PluginManifest {
            id: "metadata-wikipedia".to_string(),
            name: "Wikipedia Summaries".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Metadata,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Detailed encyclopedia summaries, release history, and reception data.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["summaries".to_string(), "history".to_string()],
            entrypoint: None,
            platform: None,
            metadata: Some(PluginMetadataConfig {
                api_url: Some("https://en.wikipedia.org/api/rest_v1/page/summary/{title}".to_string()),
                artwork: None,
            }),
        },
        PluginManifest {
            id: "metadata-vndb".to_string(),
            name: "Visual Novel Database (VNDB)".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Metadata,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Specialized metadata, tags, and character information for visual novels.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["visual_novels".to_string(), "tags".to_string()],
            entrypoint: None,
            platform: None,
            metadata: Some(PluginMetadataConfig {
                api_url: Some("https://api.vndb.org/kana/vn".to_string()),
                artwork: None,
            }),
        },
        PluginManifest {
            id: "metadata-indiedb".to_string(),
            name: "IndieDB".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Metadata,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Community indie database articles, development news, and download metadata.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["indie_articles".to_string(), "news".to_string()],
            entrypoint: None,
            platform: None,
            metadata: Some(PluginMetadataConfig {
                api_url: Some("https://www.indiedb.com/games".to_string()),
                artwork: None,
            }),
        },

        // -------------------------------------------------------------------
        // Media Enhancements
        // -------------------------------------------------------------------
        PluginManifest {
            id: "media-trailers".to_string(),
            name: "Video Trailer Player".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Media,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Streams and previews official video trailers in the game detail hero banner.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["video_playback".to_string(), "trailers".to_string()],
            entrypoint: None,
            platform: None,
            metadata: None,
        },
        PluginManifest {
            id: "media-ambient".to_string(),
            name: "Game Audio & Soundtracks".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Media,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Plays ambient background themes and official game soundtracks on selection.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["audio_playback".to_string(), "ambiance".to_string()],
            entrypoint: None,
            platform: None,
            metadata: None,
        },

        // -------------------------------------------------------------------
        // Utilities
        // -------------------------------------------------------------------
        PluginManifest {
            id: "utility-legendary".to_string(),
            name: "Legendary Epic CLI Manager".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Utility,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Installs, updates, and manages authentication for the Legendary Epic Games client.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["binary_management".to_string(), "auth_sync".to_string()],
            entrypoint: None,
            platform: None,
            metadata: None,
        },
        PluginManifest {
            id: "utility-storage".to_string(),
            name: "Storage & Drive Analyzer".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Utility,
            author: Some("HELIX-Origin".to_string()),
            description: Some("Calculates disk footprint across all platform directories and library drives.".to_string()),
            icon: None,
            homepage: Some("https://github.com/HELIX-Origin/Steam-Vault".to_string()),
            features: vec!["disk_analysis".to_string(), "drive_reporting".to_string()],
            entrypoint: None,
            platform: None,
            metadata: None,
        },
    ]
}

/// Deploys any missing standard drop-in plugin folders into the target directory.
pub fn deploy_default_plugins(target_dir: &Path) {
    for manifest in default_plugin_manifests() {
        let plugin_folder = target_dir.join(&manifest.id);
        if !plugin_folder.exists() {
            let _ = fs::create_dir_all(&plugin_folder);
            let manifest_path = plugin_folder.join("plugin.json");
            if let Ok(json) = serde_json::to_string_pretty(&manifest) {
                let _ = fs::write(manifest_path, json);
            }
        }
    }
}
