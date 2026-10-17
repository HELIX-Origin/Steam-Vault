//! Dedicated Native Plugin Engine.
//!
//! Provides backend support for native TypeScript plugins operating via Tauri IPC:
//! - Tracks enabled plugin states
//! - Executes generic platform directory scanning for configured platform roots

pub mod client_discovery;
pub mod engine;
pub mod manifest;
pub mod title_normalizer;

#[allow(unused_imports)]
pub use client_discovery::{
    discover_ea_client_games, discover_epic_client_games, discover_gog_client_games,
    discover_itch_client_games,
};
pub use engine::PluginEngine;
#[allow(unused_imports)]
pub use manifest::{PluginInfo, PluginType, StoreArtwork};
#[allow(unused_imports)]
pub use title_normalizer::{canonical_title_key, invert_article, normalize_game_title, normalize_rom_title};

