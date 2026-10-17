//! GamerPower verified multi-platform giveaway aggregator.
//!
//! Provides universal giveaway coverage across Steam, GOG, Epic, itch.io, Ubisoft, EA, etc.

use reqwest::Client;
use serde_json::Value;

use super::{FeaturedStoreItem, FreeGamePromotion, CHROME_USER_AGENT};

const GAMERPOWER_GIVEAWAYS_URL: &str = "https://www.gamerpower.com/api/giveaways?type=game";
const GAMERPOWER_FEATURED_URL: &str = "https://www.gamerpower.com/api/giveaways?type=game&sort-by=popularity";

/// Fetch verified active game giveaways from GamerPower for enabled platforms.
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
            "itch" // indie / DRM-free default
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

/// Fetch top high-value game giveaways as universal featured items from GamerPower.
pub async fn fetch_gamerpower_featured(
    client: &Client,
    enabled_platforms: &std::collections::HashSet<String>,
) -> Vec<FeaturedStoreItem> {
    let mut items = Vec::new();

    let resp = match client
        .get(GAMERPOWER_FEATURED_URL)
        .header("User-Agent", CHROME_USER_AGENT)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return items,
    };

    let list: Vec<Value> = match resp.json().await {
        Ok(arr) => arr,
        Err(_) => return items,
    };

    for item in list.iter().take(8) {
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
            "itch"
        };

        if !enabled_platforms.is_empty() && !enabled_platforms.contains(target_platform) {
            continue;
        }

        let id = item.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
        let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("").trim().to_string();
        let description = item.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();
        let url = item.get("open_giveaway_url")
            .or_else(|| item.get("gamerpower_url"))
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();

        let banner_img = item.get("image").and_then(|i| i.as_str()).map(str::to_string);
        let thumb_img = item.get("thumbnail").and_then(|t| t.as_str()).map(str::to_string);
        let worth = item.get("worth").and_then(|w| w.as_str()).filter(|w| *w != "N/A").map(str::to_string);

        let main_image = banner_img.clone().or_else(|| thumb_img.clone()).unwrap_or_default();
        if id <= 0 || title.is_empty() || url.is_empty() || main_image.is_empty() {
            continue;
        }

        let mut thumbnails = Vec::new();
        if let Some(ref b) = banner_img {
            thumbnails.push(b.clone());
        }
        if let Some(ref t) = thumb_img {
            if !thumbnails.contains(t) {
                thumbnails.push(t.clone());
            }
        }
        if thumbnails.is_empty() {
            thumbnails.push(main_image.clone());
        }

        let platform_label = match target_platform {
            "steam" => "Steam",
            "epic" => "Epic Games",
            "gog" => "GOG Galaxy",
            "itch" => "itch.io",
            "ubisoft" => "Ubisoft",
            "ea" => "EA App",
            _ => "DRM-Free",
        };

        let tag_label = Some(format!("100% Free {platform_label} Giveaway"));

        items.push(FeaturedStoreItem {
            id: format!("gp-feat-{id}"),
            title,
            platform: target_platform.to_string(),
            store_url: url,
            main_image,
            thumbnails,
            description,
            tags: vec!["Giveaway".to_string(), "100% Free".to_string(), platform_label.to_string()],
            original_price: worth.or_else(|| Some("$19.99".to_string())),
            final_price: "FREE".to_string(),
            discount_percent: Some(100),
            is_free: true,
            tag_label,
        });
    }

    items
}
