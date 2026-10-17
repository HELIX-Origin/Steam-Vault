//! Epic Games Store promotions and giveaways fetcher.

use reqwest::Client;
use serde_json::Value;

use super::{FreeGamePromotion, StoreDeal, CHROME_USER_AGENT};

const EPIC_FREE_GAMES_URL: &str =
    "https://store-site-backend-static-ipv4.ak.epicgames.com/freeGamesPromotions?locale=en-US&country=US&allowCountries=US";

/// Fetch Epic Games 100% free promotions and active store deals.
pub async fn fetch_epic_promotions(client: &Client) -> (Vec<FreeGamePromotion>, Vec<StoreDeal>) {
    let mut promotions = Vec::new();
    let mut deals = Vec::new();

    let resp = match client
        .get(EPIC_FREE_GAMES_URL)
        .header("User-Agent", CHROME_USER_AGENT)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return (promotions, deals),
    };

    let json: Value = match resp.json().await {
        Ok(j) => j,
        Err(_) => return (promotions, deals),
    };

    let elements = match json
        .pointer("/data/Catalog/searchStore/elements")
        .and_then(|v| v.as_array())
    {
        Some(arr) => arr,
        None => return (promotions, deals),
    };

    for el in elements {
        let title = el.get("title").and_then(|t| t.as_str()).unwrap_or("").trim().to_string();
        let id = el.get("id").and_then(|t| t.as_str()).unwrap_or("").to_string();
        if title.is_empty() || id.is_empty() {
            continue;
        }

        let description = el.get("description").and_then(|d| d.as_str()).map(str::to_string);
        let product_slug = el.get("productSlug").and_then(|s| s.as_str())
            .or_else(|| el.pointer("/catalogNs/mappings/0/pageSlug").and_then(|s| s.as_str()))
            .or_else(|| el.get("urlSlug").and_then(|s| s.as_str()))
            .unwrap_or("");

        let page_url = if !product_slug.is_empty() {
            format!("https://store.epicgames.com/p/{product_slug}")
        } else {
            "https://store.epicgames.com/free-games".to_string()
        };

        let mut image_url = None;
        if let Some(images) = el.get("keyImages").and_then(|k| k.as_array()) {
            for img in images {
                let img_type = img.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if img_type == "OfferImageWide" || img_type == "Thumbnail" {
                    image_url = img.get("url").and_then(|u| u.as_str()).map(str::to_string);
                    break;
                }
            }
        }

        let discount_price = el.pointer("/price/totalPrice/discountPrice").and_then(|p| p.as_i64()).unwrap_or(-1);
        let original_price = el.pointer("/price/totalPrice/originalPrice").and_then(|p| p.as_i64()).unwrap_or(-1);
        let fmt_original = el.pointer("/price/totalPrice/fmtPrice/originalPrice").and_then(|p| p.as_str()).map(str::to_string);
        let fmt_discount = el.pointer("/price/totalPrice/fmtPrice/discountPrice").and_then(|p| p.as_str()).map(str::to_string);

        let mut is_active = false;
        let mut start_date = None;
        let mut end_date = None;

        // 1. Check active promotional offers (game is CURRENTLY free)
        if let Some(promo_offers) = el
            .pointer("/promotions/promotionalOffers/0/promotionalOffers")
            .and_then(|v| v.as_array())
        {
            for offer in promo_offers {
                let discount_pct = offer.pointer("/discountSetting/discountPercentage").and_then(|p| p.as_i64()).unwrap_or(-1);
                // Active giveaway strictly requires discountPrice == 0 AND discountPercentage == 0
                if discount_price == 0 && discount_pct == 0 {
                    is_active = true;
                    start_date = offer.get("startDate").and_then(|s| s.as_str()).map(str::to_string);
                    end_date = offer.get("endDate").and_then(|s| s.as_str()).map(str::to_string);
                    break;
                }
            }
        }

        // 2. Check upcoming promotional offers (giveaways that will be free soon)
        if !is_active {
            if let Some(upcoming) = el
                .pointer("/promotions/upcomingPromotionalOffers/0/promotionalOffers")
                .and_then(|v| v.as_array())
            {
                for offer in upcoming {
                    let discount_pct = offer.pointer("/discountSetting/discountPercentage").and_then(|p| p.as_i64()).unwrap_or(-1);
                    if discount_pct == 0 {
                        start_date = offer.get("startDate").and_then(|s| s.as_str()).map(str::to_string);
                        end_date = offer.get("endDate").and_then(|s| s.as_str()).map(str::to_string);
                        break;
                    }
                }
            }
        }

        if is_active || (start_date.is_some() && end_date.is_some()) {
            promotions.push(FreeGamePromotion {
                id: format!("epic-{id}"),
                title: title.clone(),
                platform: "epic".to_string(),
                description: description.clone(),
                url: page_url.clone(),
                image_url: image_url.clone(),
                start_date: start_date.clone(),
                end_date: end_date.clone(),
                is_active,
                worth: fmt_original.clone().filter(|s| !s.is_empty()),
            });
        } else if discount_price > 0 && original_price > discount_price {
            // It's an active paid sale / deal on Epic Games Store!
            let discount_pct = if original_price > 0 {
                ((original_price - discount_price) as f64 / original_price as f64 * 100.0).round() as u32
            } else {
                0
            };
            if discount_pct >= 10 {
                deals.push(StoreDeal {
                    id: format!("epic-deal-{id}"),
                    title,
                    platform: "epic".to_string(),
                    url: page_url,
                    image_url,
                    discount_percent: discount_pct,
                    original_price: fmt_original,
                    final_price: fmt_discount.unwrap_or_else(|| format!("${:.2}", discount_price as f64 / 100.0)),
                    end_date: None,
                });
            }
        }
    }

    (promotions, deals)
}

/// Fetch featured promotions and spotlight titles from Epic Games Store.
pub async fn fetch_epic_featured(client: &Client) -> Vec<super::FeaturedStoreItem> {
    let mut items = Vec::new();

    let resp = match client
        .get(EPIC_FREE_GAMES_URL)
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

    let elements = match json
        .pointer("/data/Catalog/searchStore/elements")
        .and_then(|v| v.as_array())
    {
        Some(arr) => arr,
        None => return items,
    };

    for el in elements.iter().take(6) {
        let title = el.get("title").and_then(|t| t.as_str()).unwrap_or("").trim().to_string();
        let id = el.get("id").and_then(|t| t.as_str()).unwrap_or("").to_string();
        if title.is_empty() || id.is_empty() {
            continue;
        }

        let description = el.get("description")
            .and_then(|d| d.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| "Featured title on the Epic Games Store.".to_string());

        let product_slug = el.get("productSlug").and_then(|s| s.as_str())
            .or_else(|| el.pointer("/catalogNs/mappings/0/pageSlug").and_then(|s| s.as_str()))
            .or_else(|| el.get("urlSlug").and_then(|s| s.as_str()))
            .unwrap_or("");

        let store_url = if !product_slug.is_empty() {
            format!("https://store.epicgames.com/p/{product_slug}")
        } else {
            "https://store.epicgames.com/free-games".to_string()
        };

        let mut main_image = String::new();
        let mut thumbnails = Vec::new();

        if let Some(images) = el.get("keyImages").and_then(|k| k.as_array()) {
            for img in images {
                let img_type = img.get("type").and_then(|t| t.as_str()).unwrap_or("");
                let url = img.get("url").and_then(|u| u.as_str()).unwrap_or("");
                if url.is_empty() {
                    continue;
                }

                if img_type == "OfferImageWide" && main_image.is_empty() {
                    main_image = url.to_string();
                } else if img_type == "featuredMedia" || img_type == "Thumbnail" {
                    if thumbnails.len() < 4 && !thumbnails.contains(&url.to_string()) {
                        thumbnails.push(url.to_string());
                    }
                }
            }
        }

        if main_image.is_empty() {
            if let Some(first) = thumbnails.first() {
                main_image = first.clone();
            } else {
                continue;
            }
        }

        if !thumbnails.contains(&main_image) {
            thumbnails.insert(0, main_image.clone());
        }

        let discount_price = el.pointer("/price/totalPrice/discountPrice").and_then(|p| p.as_i64()).unwrap_or(-1);
        let original_price_num = el.pointer("/price/totalPrice/originalPrice").and_then(|p| p.as_i64()).unwrap_or(-1);
        let fmt_original = el.pointer("/price/totalPrice/fmtPrice/originalPrice").and_then(|p| p.as_str()).map(str::to_string);
        let fmt_discount = el.pointer("/price/totalPrice/fmtPrice/discountPrice").and_then(|p| p.as_str()).map(str::to_string);

        let is_free = discount_price == 0;
        let final_price = if is_free {
            "FREE".to_string()
        } else if let Some(fd) = fmt_discount {
            fd
        } else if discount_price > 0 {
            format!("${:.2}", discount_price as f64 / 100.0)
        } else {
            "FREE".to_string()
        };

        let original_price = if is_free && fmt_original.is_some() {
            fmt_original
        } else if original_price_num > discount_price && discount_price >= 0 {
            Some(format!("${:.2}", original_price_num as f64 / 100.0))
        } else {
            None
        };

        let discount_pct = if is_free {
            Some(100)
        } else if original_price_num > 0 && discount_price >= 0 && original_price_num > discount_price {
            Some(((original_price_num - discount_price) * 100 / original_price_num) as u32)
        } else {
            None
        };

        let tag_label = if is_free {
            Some("100% Free on Epic".to_string())
        } else if let Some(pct) = discount_pct {
            Some(format!("Featured -{pct}% on Epic"))
        } else {
            Some("Featured on Epic".to_string())
        };

        items.push(super::FeaturedStoreItem {
            id: format!("epic-feat-{id}"),
            title,
            platform: "epic".to_string(),
            store_url,
            main_image,
            thumbnails,
            description,
            tags: vec!["Epic Games".to_string(), "Promotional Offer".to_string(), "Featured".to_string()],
            original_price,
            final_price,
            discount_percent: discount_pct,
            is_free,
            tag_label,
        });
    }

    items
}
