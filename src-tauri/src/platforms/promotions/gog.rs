//! GOG free giveaways and store deals fetcher.

use reqwest::Client;
use serde_json::Value;

use super::{FreeGamePromotion, StoreDeal, CHROME_USER_AGENT};

const GOG_CATALOG_DEALS: &str =
    "https://catalog.gog.com/v1/catalog?limit=24&order=desc:discount&discounted=eq:true";

const GOG_CATALOG_FREE: &str =
    "https://catalog.gog.com/v1/catalog?limit=12&discounted=eq:true&price=between:0,0";

/// Fetch GOG 100% free giveaways and active discounted store deals.
pub async fn fetch_gog_promotions(client: &Client) -> (Vec<FreeGamePromotion>, Vec<StoreDeal>) {
    let mut promotions = Vec::new();
    let mut deals = Vec::new();

    // 1. Check 100% free giveaways on GOG
    if let Ok(resp) = client
        .get(GOG_CATALOG_FREE)
        .header("User-Agent", CHROME_USER_AGENT)
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<Value>().await {
                if let Some(products) = json.get("products").and_then(|p| p.as_array()) {
                    for prod in products {
                        let title = prod.get("title").and_then(|t| t.as_str()).unwrap_or("").trim().to_string();
                        let slug = prod.get("slug").and_then(|s| s.as_str()).unwrap_or("");
                        let cover = prod.get("coverHorizontal").and_then(|c| c.as_str()).map(str::to_string);
                        let id = prod.get("id").and_then(|v| v.as_str()).unwrap_or(slug);

                        if !title.is_empty() && !slug.is_empty() {
                            promotions.push(FreeGamePromotion {
                                id: format!("gog-{id}"),
                                title,
                                platform: "gog".to_string(),
                                description: Some("100% Free DRM-free giveaway on GOG!".to_string()),
                                url: format!("https://www.gog.com/game/{slug}"),
                                image_url: cover,
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

    // 2. Check top discounted deals on GOG
    if let Ok(resp) = client
        .get(GOG_CATALOG_DEALS)
        .header("User-Agent", CHROME_USER_AGENT)
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<Value>().await {
                if let Some(products) = json.get("products").and_then(|p| p.as_array()) {
                    for prod in products {
                        let title = prod.get("title").and_then(|t| t.as_str()).unwrap_or("").trim().to_string();
                        let slug = prod.get("slug").and_then(|s| s.as_str()).unwrap_or("");
                        let cover = prod.get("coverHorizontal").and_then(|c| c.as_str()).map(str::to_string);
                        let discount_str = prod.pointer("/price/discount").and_then(|d| d.as_str()).unwrap_or("");
                        let final_price = prod.pointer("/price/final").and_then(|f| f.as_str()).unwrap_or("").to_string();
                        let base_price = prod.pointer("/price/base").and_then(|b| b.as_str()).map(str::to_string);
                        let id = prod.get("id").and_then(|v| v.as_str()).unwrap_or(slug);

                        let pct = discount_str
                            .trim_start_matches('-')
                            .trim_end_matches('%')
                            .parse::<u32>()
                            .unwrap_or(0);

                        if !title.is_empty() && !slug.is_empty() && pct >= 20 {
                            deals.push(StoreDeal {
                                id: format!("gog-deal-{id}"),
                                title,
                                platform: "gog".to_string(),
                                url: format!("https://www.gog.com/game/{slug}"),
                                image_url: cover,
                                discount_percent: pct,
                                original_price: base_price,
                                final_price,
                                end_date: None,
                            });
                        }
                    }
                }
            }
        }
    }

    (promotions, deals)
}

const GOG_CATALOG_TRENDING: &str =
    "https://catalog.gog.com/v1/catalog?limit=8&order=desc:trending";

/// Fetch trending spotlight featured items from the official GOG catalog.
pub async fn fetch_gog_featured(client: &Client) -> Vec<super::FeaturedStoreItem> {
    let mut items = Vec::new();

    let resp = match client
        .get(GOG_CATALOG_TRENDING)
        .header("User-Agent", CHROME_USER_AGENT)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return items,
    };

    let json: Value = match resp.json().await {
        Ok(j) => j,
        Err(_) => return items,
    };

    if let Some(products) = json.get("products").and_then(|p| p.as_array()) {
        for prod in products.iter().take(6) {
            let title = prod.get("title").and_then(|t| t.as_str()).unwrap_or("").trim().to_string();
            let slug = prod.get("slug").and_then(|s| s.as_str()).unwrap_or("");
            let cover_h = prod.get("coverHorizontal").and_then(|c| c.as_str()).map(str::to_string);
            let cover_v = prod.get("coverVertical").and_then(|c| c.as_str()).map(str::to_string);
            let discount_str = prod.pointer("/price/discount").and_then(|d| d.as_str()).unwrap_or("");
            let final_price = prod.pointer("/price/final").and_then(|f| f.as_str()).unwrap_or("").to_string();
            let base_price = prod.pointer("/price/base").and_then(|b| b.as_str()).map(str::to_string);
            let id = prod.get("id").and_then(|v| v.as_str()).unwrap_or(slug);

            if title.is_empty() || slug.is_empty() {
                continue;
            }

            let pct = discount_str
                .trim_start_matches('-')
                .trim_end_matches('%')
                .parse::<u32>()
                .unwrap_or(0);

            let main_image = cover_h.clone().or_else(|| cover_v.clone()).unwrap_or_default();
            if main_image.is_empty() {
                continue;
            }

            let mut thumbnails = Vec::new();
            if let Some(ref h) = cover_h {
                thumbnails.push(h.clone());
            }
            if let Some(ref v) = cover_v {
                if !thumbnails.contains(v) {
                    thumbnails.push(v.clone());
                }
            }
            if thumbnails.is_empty() {
                thumbnails.push(main_image.clone());
            }

            let mut tags = vec!["DRM-Free".to_string(), "GOG Trending".to_string()];
            if let Some(tag_arr) = prod.get("tags").and_then(|t| t.as_array()) {
                for t in tag_arr.iter().take(3) {
                    if let Some(name) = t.get("name").and_then(|n| n.as_str()) {
                        if !tags.contains(&name.to_string()) {
                            tags.push(name.to_string());
                        }
                    }
                }
            }

            let is_free = final_price == "$0.00" || final_price.to_uppercase() == "FREE";
            let tag_label = if is_free {
                Some("Free on GOG (DRM-Free)".to_string())
            } else if pct > 0 {
                Some(format!("Trending -{pct}% on GOG"))
            } else {
                Some("Trending on GOG (DRM-Free)".to_string())
            };

            items.push(super::FeaturedStoreItem {
                id: format!("gog-feat-{id}"),
                title,
                platform: "gog".to_string(),
                store_url: format!("https://www.gog.com/game/{slug}"),
                main_image,
                thumbnails,
                description: "Trending DRM-free title on GOG Galaxy with zero digital restrictions or mandatory client requirement.".to_string(),
                tags,
                original_price: base_price,
                final_price,
                discount_percent: if pct > 0 { Some(pct) } else { None },
                is_free,
                tag_label,
            });
        }
    }

    items
}
