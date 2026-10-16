//! itch.io online metadata provider (keyless, search and game page scraper).

use crate::error::AppResult;
use crate::steam::models::AppMeta;

/// Look up a game on itch.io search and extract title, cover image, author, and description.
pub async fn itch_lookup(client: &reqwest::Client, name: &str) -> AppResult<Option<AppMeta>> {
    let clean = crate::metadata::vndb::clean_search_term(name);
    if clean.is_empty() {
        return Ok(None);
    }

    let url = format!("https://itch.io/search?q={}", urlencoding::encode(&clean));
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

    // Parse first game_cell in search results
    let cell_pos = html.find("class=\"game_cell\"");
    let Some(cell_pos) = cell_pos else {
        return Ok(None);
    };
    let chunk = &html[cell_pos..];
    let end_pos = chunk.find("</div>\n</div>").unwrap_or(chunk.len().min(3000));
    let cell = &chunk[..end_pos];

    // Extract title
    let mut title = None;
    if let Some(t_idx) = cell.find("class=\"game_title\"") {
        let t_chunk = &cell[t_idx..];
        if let Some(a_start) = t_chunk.find("<a ") {
            if let Some(tag_end) = t_chunk[a_start..].find('>') {
                let text_start = a_start + tag_end + 1;
                if let Some(a_end) = t_chunk[text_start..].find("</a>") {
                    let raw = t_chunk[text_start..text_start + a_end].trim().to_string();
                    if !raw.is_empty() {
                        title = Some(raw);
                    }
                }
            }
        }
    }

    // Extract author / developer
    let mut author = None;
    if let Some(auth_idx) = cell.find("class=\"game_author\"") {
        let a_chunk = &cell[auth_idx..];
        if let Some(a_start) = a_chunk.find("<a ") {
            if let Some(tag_end) = a_chunk[a_start..].find('>') {
                let text_start = a_start + tag_end + 1;
                if let Some(a_end) = a_chunk[text_start..].find("</a>") {
                    let raw = a_chunk[text_start..text_start + a_end].trim().to_string();
                    if !raw.is_empty() {
                        author = Some(raw);
                    }
                }
            }
        }
    }

    // Extract cover image src
    let mut cover_url = None;
    if let Some(img_idx) = cell.find("<img ") {
        let img_chunk = &cell[img_idx..];
        // Prefer data-screenshot_url or src
        let src_attr = if let Some(ss_idx) = img_chunk.find("data-screenshot_url=\"") {
            let start = ss_idx + 21;
            img_chunk[start..].find('"').map(|end| &img_chunk[start..start + end])
        } else if let Some(s_idx) = img_chunk.find("src=\"") {
            let start = s_idx + 5;
            img_chunk[start..].find('"').map(|end| &img_chunk[start..start + end])
        } else {
            None
        };

        if let Some(src) = src_attr {
            let clean_src = src.trim().to_string();
            if clean_src.starts_with("http") {
                cover_url = Some(clean_src);
            } else if clean_src.starts_with("//") {
                cover_url = Some(format!("https:{clean_src}"));
            }
        }
    }

    // Extract short description
    let mut description = None;
    if let Some(desc_idx) = cell.find("class=\"game_text\"") {
        let d_chunk = &cell[desc_idx..];
        if let Some(start) = d_chunk.find('>') {
            if let Some(end) = d_chunk[start + 1..].find("</div>") {
                let raw = d_chunk[start + 1..start + 1 + end].trim().to_string();
                if !raw.is_empty() {
                    description = Some(raw);
                }
            }
        }
    }

    if title.is_some() || cover_url.is_some() {
        let developers = author.map(|a| vec![a]).unwrap_or_default();
        let publishers = developers.clone();

        return Ok(Some(AppMeta {
            app_id: 0,
            name: title.or_else(|| Some(clean.clone())),
            developers,
            publishers,
            genres: vec!["Indie".to_string()],
            tags: vec!["itch.io".to_string()],
            short_description: description,
            header_image: cover_url.clone(),
            capsule_image: cover_url,
            fetched_at: Some(chrono::Utc::now().timestamp()),
            ..Default::default()
        }));
    }

    Ok(None)
}
