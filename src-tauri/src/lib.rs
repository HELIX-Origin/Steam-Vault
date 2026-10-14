//! Steam Vault — Tauri v2 backend entry point.

// Parts of the Steam data layer and metadata providers are intentionally exposed
// ahead of the milestones that call them (the UI wires them up incrementally).
#![allow(dead_code)]

mod autocat;
mod backup;
mod cache;
mod commands;
mod error;
mod filter;
mod metadata;
mod settings;
mod steam;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_providers,
            commands::detect_steam,
            commands::get_settings,
            commands::save_settings,
            commands::is_steam_running,
            commands::load_library,
            commands::load_collections,
            commands::update_collection,
            commands::create_collection,
            commands::delete_collection,
            commands::fetch_steam_store_metadata,
            commands::fetch_owned_games,
            commands::steamgriddb_search,
            commands::steamgriddb_assets,
            commands::igdb_search,
            commands::pcgamingwiki_lookup,
            commands::wikipedia_summary,
            commands::compute_autocats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
