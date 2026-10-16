//! Steam Store giveaways, deals, and patch notes fetcher.

use reqwest::Client;
use serde_json::Value;

use super::{FreeGamePromotion, PatchNote, StoreDeal, CHROME_USER_AGENT};

const STEAM_SPECIALS_SEARCH: &str =
    "https://store.steampowered.com/search/results/?query=&start=0&count=50&maxprice=free&specials=1&json=1";

const STEAM_FEATURED_CATEGORIES: &str =
    "https://store.steampowered.com/api/featuredcategories/?cc=us&l=en";

/// Fetch Steam 100% free giveaways and active store specials.
pub async fn fetch_steam_promotions(client: &Client) -> (Vec<FreeGamePromotion>, Vec<StoreDeal>) {
    let mut promotions = Vec::new();
    let mut deals = Vec::new();

    // 1. Check direct 100% discount specials on Steam Store
    if let Ok(resp) = client
        .get(STEAM_SPECIALS_SEARCH)
        .header("User-Agent", CHROME_USER_AGENT)
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<Value>().await {
                if let Some(items) = json.get("items").and_then(|i| i.as_array()) {
                    for item in items {
                        let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("").trim().to_string();
                        let logo = item.get("logo").and_then(|l| l.as_str()).unwrap_or("");
                        if name.is_empty() {
                            continue;
                        }

                        // Extract appid from logo URL if present: /apps/{appid}/
                        let app_id = logo
                            .split("/apps/")
                            .nth(1)
                            .and_then(|s| s.split('/').next())
                            .unwrap_or("");

                        if !app_id.is_empty() {
                            let url = format!("https://store.steampowered.com/app/{app_id}");
                            let img = format!("https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/{app_id}/header.jpg");
                            promotions.push(FreeGamePromotion {
                                id: format!("steam-{app_id}"),
                                title: name,
                                platform: "steam".to_string(),
                                description: Some("100% free to keep on Steam! Limited time promotion.".to_string()),
                                url,
                                image_url: Some(img),
                                start_date: None,
                                end_date: None,
                                is_active: true,
                                worth: None,
                            });
                        }
                    }
                }
            }
        }
    }

    // 2. Check Steam featured specials & deals
    if let Ok(resp) = client
        .get(STEAM_FEATURED_CATEGORIES)
        .header("User-Agent", CHROME_USER_AGENT)
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<Value>().await {
                if let Some(specials) = json.pointer("/specials/items").and_then(|s| s.as_array()) {
                    for sp in specials.iter().take(24) {
                        let id = sp.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                        let name = sp.get("name").and_then(|n| n.as_str()).unwrap_or("").trim().to_string();
                        let discount_percent = sp.get("discount_percent").and_then(|d| d.as_u64()).unwrap_or(0) as u32;
                        let orig_price = sp.get("original_price").and_then(|p| p.as_i64()).unwrap_or(0);
                        let final_price = sp.get("final_price").and_then(|p| p.as_i64()).unwrap_or(0);
                        let img = sp.get("header_image").and_then(|h| h.as_str())
                            .or_else(|| sp.get("large_capsule_image").and_then(|c| c.as_str()))
                            .map(str::to_string);

                        if id > 0 && !name.is_empty() && discount_percent >= 10 {
                            let end_date = sp.get("discount_expiration")
                                .and_then(|e| e.as_i64())
                                .filter(|&ts| ts > 0)
                                .map(|ts| {
                                    chrono::DateTime::from_timestamp(ts, 0)
                                        .map(|dt| dt.to_rfc3339())
                                        .unwrap_or_default()
                                });

                            deals.push(StoreDeal {
                                id: format!("steam-deal-{id}"),
                                title: name,
                                platform: "steam".to_string(),
                                url: format!("https://store.steampowered.com/app/{id}"),
                                image_url: img,
                                discount_percent,
                                original_price: Some(format!("${:.2}", orig_price as f64 / 100.0)),
                                final_price: format!("${:.2}", final_price as f64 / 100.0),
                                end_date,
                            });
                        }
                    }
                }
            }
        }
    }

    (promotions, deals)
}

/// Fetch patch notes / news for a specific Steam game.
pub async fn fetch_steam_patch_notes(client: &Client, app_id: &str, game_name: &str) -> Vec<PatchNote> {
    let mut notes = Vec::new();
    let url = format!(
        "https://api.steampowered.com/ISteamNews/GetNewsForApp/v2/?appid={app_id}&count=2&maxlength=400"
    );

    if let Ok(resp) = client.get(&url).header("User-Agent", CHROME_USER_AGENT).send().await {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<Value>().await {
                if let Some(items) = json.pointer("/appnews/newsitems").and_then(|n| n.as_array()) {
                    for item in items {
                        let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("").trim().to_string();
                        let news_url = item.get("url").and_then(|u| u.as_str()).unwrap_or("").to_string();
                        let gid = item.get("gid").and_then(|g| g.as_str()).unwrap_or("").to_string();
                        let date_ts = item.get("date").and_then(|d| d.as_i64()).unwrap_or(0);
                        let author = item.get("author").and_then(|a| a.as_str()).map(str::to_string);
                        let contents = item.get("contents").and_then(|c| c.as_str()).map(str::to_string);

                        if !title.is_empty() && !news_url.is_empty() {
                            let date_str = if date_ts > 0 {
                                chrono::DateTime::from_timestamp(date_ts, 0)
                                    .map(|dt| dt.to_rfc3339())
                                    .unwrap_or_default()
                            } else {
                                String::new()
                            };

                            // Clean BBCode/HTML fragments from summary snippet
                            let clean_summary = contents.map(|raw| {
                                let mut s = raw.replace("[b]", "").replace("[/b]", "")
                                    .replace("[i]", "").replace("[/i]", "")
                                    .replace("[list]", "").replace("[/list]", "")
                                    .replace("[*]", "• ")
                                    .replace("<br>", " ")
                                    .replace("<br/>", " ");
                                while let Some(start) = s.find('[') {
                                    if let Some(end) = s[start..].find(']') {
                                        s.replace_range(start..=start + end, "");
                                    } else {
                                        break;
                                    }
                                }
                                s.trim().to_string()
                            });

                            notes.push(PatchNote {
                                id: format!("steam-patch-{gid}"),
                                app_id: app_id.to_string(),
                                game_name: game_name.to_string(),
                                platform: "steam".to_string(),
                                title,
                                url: news_url,
                                date: date_str,
                                author,
                                summary: clean_summary,
                            });
                        }
                    }
                }
            }
        }
    }

    notes
}
