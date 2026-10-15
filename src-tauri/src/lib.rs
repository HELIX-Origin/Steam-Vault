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
mod platforms;
mod settings;
mod steam;

use commands::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .setup(|app| {
            // Pin the taskbar/window icon to the Steam Vault brand mark so the
            // frameless window never falls back to a generic default icon.
            if let (Some(window), Some(icon)) =
                (app.get_webview_window("main"), app.default_window_icon())
            {
                let _ = window.set_icon(icon.clone());
            }
            Ok(())
        })
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
            commands::sync_library_metadata,
            commands::fetch_owned_games,
            commands::steamgriddb_search,
            commands::steamgriddb_assets,
            commands::igdb_search,
            commands::pcgamingwiki_lookup,
            commands::wikipedia_summary,
            commands::wikipedia_game_lookup,
            commands::vndb_lookup,
            commands::steam_store_search,
            commands::compute_autocats,
            commands::detect_platforms,
            commands::find_platform_folders,
            commands::import_platform_games,
            commands::set_game_platform,
            commands::get_owned_library,
            commands::refresh_owned_games,
            commands::compute_categorization,
            commands::apply_categorization,
            commands::get_storage_report,
            commands::preview_move_library,
            commands::move_library,
            commands::preview_move_game,
            commands::move_game,
            commands::add_library_folder,
            commands::remove_library_folder,
            commands::get_artwork,
            commands::read_artwork,
            commands::apply_artwork_from_url,
            commands::apply_artwork_from_file,
            commands::remove_artwork,
            commands::steamgriddb_for_game,
            commands::auto_fetch_artwork_for_games,
            commands::get_filter_presets,
            commands::save_filter_preset,
            commands::delete_filter_preset,
            commands::evaluate_filter,
            commands::save_smart_collection,
            commands::refresh_smart_collection,
            commands::get_catalog_stats,
            commands::export_catalog_archive,
            commands::import_catalog_archive,
            commands::sync_online_catalog,
            commands::fetch_online_game_entry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
