//! IndieDB metadata provider for indie games, indie engines, and standalone indie projects.

use crate::error::AppResult;
use crate::steam::models::AppMeta;

/// Look up an indie game on IndieDB and extract metadata & box art.
pub async fn indiedb_lookup(client: &reqwest::Client, name: &str) -> AppResult<Option<AppMeta>> {
    let clean = crate::metadata::vndb::clean_search_term(name);
    if clean.is_empty() {
        return Ok(None);
    }

    let url = format!(
        "https://www.indiedb.com/games?filter=t&kw={}",
        urlencoding::encode(&clean)
    );

    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await;

    let Ok(r) = resp else {
        return Ok(None);
    };

    if !r.status().is_success() {
        return Ok(None);
    }

    let html = r.text().await.unwrap_or_default();
    if html.is_empty() {
        return Ok(None);
    }

    // Parse first search result row from IndieDB games list
    // IndieDB structure has: <div class="row rowcontent"> ... <a href="/games/slug" title="Title"> ... <img src="thumb.jpg" ...
    let mut title = None;
    let mut image_url = None;
    let mut description = None;
    let mut tags = vec!["IndieDB".to_string(), "Indie".to_string()];

    if let Some(link_pos) = html.find("class=\"heading\"") {
        let chunk = &html[link_pos..];
        if let Some(title_start) = chunk.find(">") {
            if let Some(title_end) = chunk[title_start + 1..].find("<") {
                let found_title = chunk[title_start + 1..title_start + 1 + title_end].trim().to_string();
                if !found_title.is_empty() {
                    title = Some(found_title);
                }
            }
        }
    }

    // Try to find image src
    if let Some(img_pos) = html.find("class=\"thumb\"") {
        let chunk = &html[img_pos..];
        if let Some(src_start) = chunk.find("src=\"") {
            let src_chunk = &chunk[src_start + 5..];
            if let Some(src_end) = src_chunk.find("\"") {
                let img = src_chunk[..src_end].trim().to_string();
                if img.starts_with("http") || img.starts_with("//") {
                    let full = if img.starts_with("//") {
                        format!("https:{img}")
                    } else {
                        img
                    };
                    image_url = Some(full);
                }
            }
        }
    }

    // Try to extract summary
    if let Some(desc_pos) = html.find("class=\"summary\"") {
        let chunk = &html[desc_pos..];
        if let Some(p_start) = chunk.find("<p>") {
            if let Some(p_end) = chunk[p_start + 3..].find("</p>") {
                let desc = chunk[p_start + 3..p_start + 3 + p_end].trim().to_string();
                if !desc.is_empty() {
                    description = Some(desc);
                }
            }
        }
    }

    // Detect engine tags if mentioned in html
    let lower_html = html.to_lowercase();
    if lower_html.contains("engine: unity") || lower_html.contains("unity engine") {
        tags.push("Engine: Unity".to_string());
    } else if lower_html.contains("unreal engine") {
        tags.push("Engine: Unreal".to_string());
    } else if lower_html.contains("godot") {
        tags.push("Engine: Godot".to_string());
    } else if lower_html.contains("gamemaker") {
        tags.push("Engine: GameMaker".to_string());
    }

    if let Some(t) = title {
        return Ok(Some(AppMeta {
            app_id: 0,
            name: Some(t),
            developers: vec![],
            publishers: vec![],
            genres: vec!["Indie".to_string()],
            tags,
            short_description: description,
            header_image: image_url.clone(),
            capsule_image: image_url,
            fetched_at: Some(chrono::Utc::now().timestamp()),
            ..Default::default()
        }));
    }

    Ok(None)
}
