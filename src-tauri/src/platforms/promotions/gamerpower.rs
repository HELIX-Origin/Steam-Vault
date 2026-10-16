//! GamerPower verified multi-platform giveaway aggregator.
//!
//! Provides fallback and key giveaway coverage across Steam, GOG, Epic, itch.io, Ubisoft, etc.

use reqwest::Client;
use serde_json::Value;

use super::{FreeGamePromotion, CHROME_USER_AGENT};

const GAMERPOWER_GIVEAWAYS_URL: &str = "https://www.gamerpower.com/api/giveaways";

/// Fetch verified active giveaways from GamerPower for enabled platforms.
pub async fn fetch_gamerpower_giveaways(
    client: &Client,
    enabled_platforms: &std::collections::HashSet<String>,
) -> Vec<FreeGamePromotion> {
    let mut promotions = Vec::new();

    let resp = match client
        .get(GAMERPOWER_GIVEAWAYS_URL)
        .header("User-Agent", CHROME_USER_AGENT)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return promotions,
    };

    let items: Vec<Value> = match resp.json().await {
        Ok(arr) => arr,
        Err(_) => return promotions,
    };

    for item in items {
        let status = item.get("status").and_then(|s| s.as_str()).unwrap_or("");
        if status != "Active" {
            continue;
        }

        let platforms_str = item.get("platforms").and_then(|p| p.as_str()).unwrap_or("").to_lowercase();
        let target_platform = if platforms_str.contains("steam") {
            "steam"
        } else if platforms_str.contains("epic") {
            "epic"
        } else if platforms_str.contains("gog") {
            "gog"
        } else if platforms_str.contains("itch") {
            "itch"
        } else if platforms_str.contains("ubisoft") {
            "ubisoft"
        } else if platforms_str.contains("ea") || platforms_str.contains("origin") {
            "ea"
        } else {
            continue;
        };

        if !enabled_platforms.is_empty() && !enabled_platforms.contains(target_platform) {
            continue;
        }

        let id = item.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
        let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("").trim().to_string();
        let description = item.get("description").and_then(|d| d.as_str()).map(str::to_string);
        let url = item.get("open_giveaway_url")
            .or_else(|| item.get("gamerpower_url"))
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();
        let img = item.get("image")
            .or_else(|| item.get("thumbnail"))
            .and_then(|i| i.as_str())
            .map(str::to_string);
        let worth = item.get("worth").and_then(|w| w.as_str()).map(str::to_string);
        let end_date = item.get("end_date").and_then(|e| e.as_str())
            .filter(|&e| e != "N/A" && !e.is_empty())
            .map(str::to_string);

        if id > 0 && !title.is_empty() && !url.is_empty() {
            promotions.push(FreeGamePromotion {
                id: format!("gp-{id}"),
                title,
                platform: target_platform.to_string(),
                description,
                url,
                image_url: img,
                start_date: None,
                end_date,
                is_active: true,
                worth,
            });
        }
    }

    promotions
}
