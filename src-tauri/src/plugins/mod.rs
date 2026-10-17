//! Dedicated Native Plugin Engine.
//!
//! Provides backend support for native TypeScript plugins operating via Tauri IPC:
//! - Tracks enabled plugin states
//! - Executes generic platform directory scanning for configured platform roots

pub mod engine;
pub mod manifest;

pub use engine::PluginEngine;
#[allow(unused_imports)]
pub use manifest::{PluginInfo, PluginType, StoreArtwork};
