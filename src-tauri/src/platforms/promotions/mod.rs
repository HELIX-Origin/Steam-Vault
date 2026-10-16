//! Multi-platform Free Game Promotions, Store Deals, and Patch Notes.

pub mod epic;
pub mod gamerpower;
pub mod gog;
pub mod steam;

use std::collections::HashSet;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub const CHROME_USER_AGENT: &str = if cfg!(target_os = "windows") {
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
} else if cfg!(target_os = "macos") {
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
} else {
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
};

/// 100% Free Game Promotion or Giveaway across storefronts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeGamePromotion {
    pub id: String,
    pub title: String,
    pub platform: String,
    pub description: Option<String>,
    pub url: String,
    pub image_url: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_active: bool,
    pub worth: Option<String>,
}

/// Highlighted Store Deal or Major Discount.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreDeal {
    pub id: String,
    pub title: String,
    pub platform: String,
    pub url: String,
    pub image_url: Option<String>,
    pub discount_percent: u32,
    pub original_price: Option<String>,
    pub final_price: String,
    pub end_date: Option<String>,
}

/// Game update / patch note for a game in user's library.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchNote {
    pub id: String,
    pub app_id: String,
    pub game_name: String,
    pub platform: String,
    pub title: String,
    pub url: String,
    pub date: String,
    pub author: Option<String>,
    pub summary: Option<String>,
}

/// Fetch all free game promotions across enabled platforms.
pub async fn fetch_all_free_games(
    client: &Client,
    enabled_platforms: &HashSet<String>,
) -> Vec<FreeGamePromotion> {
    let mut all_promos = Vec::new();
    let mut seen_titles = HashSet::new();

    // 1. Epic Games (authoritative official store endpoint)
    if enabled_platforms.is_empty() || enabled_platforms.contains("epic") {
        let (epic_free, _) = epic::fetch_epic_promotions(client).await;
        for p in epic_free {
            let key = p.title.to_lowercase();
            if seen_titles.insert(key) {
                all_promos.push(p);
            }
        }
    }

    // 2. Steam (authoritative official store specials)
    if enabled_platforms.is_empty() || enabled_platforms.contains("steam") {
        let (steam_free, _) = steam::fetch_steam_promotions(client).await;
        for p in steam_free {
            let key = p.title.to_lowercase();
            if seen_titles.insert(key) {
                all_promos.push(p);
            }
        }
    }

    // 3. GOG (authoritative official catalog)
    if enabled_platforms.is_empty() || enabled_platforms.contains("gog") {
        let (gog_free, _) = gog::fetch_gog_promotions(client).await;
        for p in gog_free {
            let key = p.title.to_lowercase();
            if seen_titles.insert(key) {
                all_promos.push(p);
            }
        }
    }

    // 4. GamerPower multi-platform giveaway aggregator (deduped)
    let gp_promos = gamerpower::fetch_gamerpower_giveaways(client, enabled_platforms).await;
    for p in gp_promos {
        let norm_title = p.title
            .to_lowercase()
            .replace("(steam)", "")
            .replace("(epic games)", "")
            .replace("(gog)", "")
            .replace("(itch.io)", "")
            .replace("giveaway", "")
            .replace("key", "")
            .trim()
            .to_string();

        if !seen_titles.contains(&norm_title) && seen_titles.insert(p.title.to_lowercase()) {
            all_promos.push(p);
        }
    }

    // Sort: active promotions first, then upcoming by start date
    all_promos.sort_by(|a, b| b.is_active.cmp(&a.is_active));

    all_promos
}

/// Fetch store deals and promotions across enabled platforms.
pub async fn fetch_all_deals(
    client: &Client,
    enabled_platforms: &HashSet<String>,
) -> Vec<StoreDeal> {
    let mut all_deals = Vec::new();
    let mut seen = HashSet::new();

    if enabled_platforms.is_empty() || enabled_platforms.contains("steam") {
        let (_, steam_deals) = steam::fetch_steam_promotions(client).await;
        for d in steam_deals {
            if seen.insert(d.title.to_lowercase()) {
                all_deals.push(d);
            }
        }
    }

    if enabled_platforms.is_empty() || enabled_platforms.contains("epic") {
        let (_, epic_deals) = epic::fetch_epic_promotions(client).await;
        for d in epic_deals {
            if seen.insert(d.title.to_lowercase()) {
                all_deals.push(d);
            }
        }
    }

    if enabled_platforms.is_empty() || enabled_platforms.contains("gog") {
        let (_, gog_deals) = gog::fetch_gog_promotions(client).await;
        for d in gog_deals {
            if seen.insert(d.title.to_lowercase()) {
                all_deals.push(d);
            }
        }
    }

    // Sort by highest discount percentage descending
    all_deals.sort_by(|a, b| b.discount_percent.cmp(&a.discount_percent));

    all_deals
}

/// Fetch patch notes for a list of library games.
pub async fn fetch_library_patch_notes(
    client: &Client,
    games: &[(String, String)], // (app_id, game_name)
) -> Vec<PatchNote> {
    let mut all_notes = Vec::new();

    // Query Steam patch notes for candidate games (limit to 12 games to keep response fast)
    for (app_id, name) in games.iter().take(12) {
        // App IDs that are valid numeric Steam IDs
        if app_id.chars().all(|c| c.is_ascii_digit()) && !app_id.is_empty() {
            let notes = steam::fetch_steam_patch_notes(client, app_id, name).await;
            all_notes.extend(notes);
        }
    }

    // Sort by latest date descending
    all_notes.sort_by(|a, b| b.date.cmp(&a.date));

    all_notes
}
