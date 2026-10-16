//! Dedicated Community & First-Party Plugin Engine (GOG Galaxy-style plugin architecture).
//!
//! Provides a completely self-contained, drop-in plugin system for:
//! - Game Platform Providers (`platform`)
//! - Metadata Providers (`metadata`)
//! - Media Enhancements like Video Trailers & Ambient Audio (`media`)
//! - Utilities and Tools (`utility`)
//!
//! All plugins live in their own self-contained folders with a `plugin.json` manifest
//! inside `<data_dir>/universal-game-launcher/plugins/<plugin_id>/`. The engine imports
//! and runs them dynamically without hardcoding plugin code into the core app.

pub mod defaults;
pub mod engine;
pub mod manifest;

pub use engine::{plugins_dir, PluginEngine};
#[allow(unused_imports)]
pub use manifest::{PluginInfo, PluginType, StoreArtwork};
