//! Epic Games Store metadata provider (keyless, official Epic Games catalog endpoints).

use serde_json::Value;
use crate::error::AppResult;
use crate::steam::models::AppMeta;

/// Look up a game on Epic Games Store catalog search and return normalized [`AppMeta`].
pub async fn epic_lookup(client: &reqwest::Client, name: &str) -> AppResult<Option<AppMeta>> {
    let clean = crate::metadata::vndb::clean_search_term(name);
    if clean.is_empty() {
        return Ok(None);
    }

    // Epic Games Store free catalog search endpoint
    let url = "https://store-site-backend-static.ak.epicgames.com/freeGamesPromotions";
    let _resp = client
        .get(url)
        .query(&[("locale", "en-US"), ("country", "US"), ("allowCountries", "US")])
        .send()
        .await;

    // Fall back to product search if available or parse freeGamesPromotions elements
    let query_url = format!(
        "https://store-content-ipv4.ak.epicgames.com/api/en-US/content/products/{}",
        clean.to_lowercase().replace(' ', "-")
    );

    let resp_prod = client.get(&query_url).send().await;
    if let Ok(r) = resp_prod {
        if r.status().is_success() {
            if let Ok(val) = r.json::<Value>().await {
                if let Some(pages) = val.get("pages").and_then(Value::as_array) {
                    if let Some(first) = pages.first() {
                        let title = first.get("title").and_then(Value::as_str).map(String::from);
                        let description = first.get("description").and_then(Value::as_str).map(String::from);
                        let mut capsule = None;
                        let mut header = None;
                        let mut hero = None;
                        let mut logo = None;

                        if let Some(images) = first.get("data").and_then(|d| d.get("gallery")).and_then(Value::as_array) {
                            for img in images {
                                let t = img.get("type").and_then(Value::as_str).unwrap_or_default();
                                let src = img.get("src").and_then(Value::as_str).map(String::from);
                                if t == "DieselStoreFrontTall" || t == "OfferImageTall" {
                                    capsule = src;
                                } else if t == "DieselStoreFrontWide" || t == "OfferImageWide" {
                                    header = src;
                                } else if t == "hero" || t == "featuredMedia" {
                                    hero = src;
                                } else if t == "logo" || t == "ProductLogo" {
                                    logo = src;
                                }
                            }
                        }

                        if title.is_some() {
                            return Ok(Some(AppMeta {
                                app_id: 0,
                                name: title,
                                developers: vec![],
                                publishers: vec!["Epic Games".to_string()],
                                genres: vec![],
                                tags: vec!["Epic Games".to_string()],
                                short_description: description,
                                header_image: header.or_else(|| capsule.clone()),
                                capsule_image: capsule,
                                hero_image: hero,
                                logo_image: logo,
                                fetched_at: Some(chrono::Utc::now().timestamp()),
                                ..Default::default()
                            }));
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}
