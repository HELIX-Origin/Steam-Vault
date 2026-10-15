//! Custom artwork management for Steam games and non-Steam shortcuts.
//!
//! Steam reads a user's custom artwork from `userdata/<id3>/config/grid/`, one
//! file per *slot*, keyed by app id. Each slot has a fixed filename stem and an
//! extension Steam understands (`.png`, `.jpg`, or `.jpeg`):
//!
//! | Slot           | Stem           | Usual size |
//! |----------------|----------------|------------|
//! | Portrait grid  | `<appid>p`     | 600×900    |
//! | Landscape grid | `<appid>`      | 920×430    |
//! | Hero / banner  | `<appid>_hero` | 1920×620   |
//! | Logo           | `<appid>_logo` | any        |
//! | Icon           | `<appid>_icon` | 256×256    |
//!
//! This is the same mechanism Steam's own "Customize" screen writes, and it is
//! the only way to give art to a **non-Steam shortcut** (whose synthetic app id
//! has no store page). Writing these files does not require Steam to be closed:
//! Steam watches the folder and repaints with the new images.
//!
//! Every overwrite and removal first copies the previous image into the
//! timestamped [`crate::backup`] store, so nothing is destroyed irreversibly.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

/// Refuse to apply an image larger than this (64 MiB).
const MAX_ARTWORK_BYTES: usize = 64 * 1024 * 1024;

/// Refuse to base64 a preview larger than this (12 MiB) to keep IPC small.
const MAX_PREVIEW_BYTES: u64 = 12 * 1024 * 1024;

/// Image extensions Steam reads from `config/grid`, in preference order.
const KNOWN_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "ico"];

/// One of the five custom-artwork slots Steam supports per game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArtworkSlot {
    GridPortrait,
    GridLandscape,
    Hero,
    Logo,
    Icon,
}

impl ArtworkSlot {
    /// Every slot, in the order Steam's own customize screen presents them.
    pub const ALL: [ArtworkSlot; 5] = [
        ArtworkSlot::GridPortrait,
        ArtworkSlot::GridLandscape,
        ArtworkSlot::Hero,
        ArtworkSlot::Logo,
        ArtworkSlot::Icon,
    ];

    /// Stable identifier shared with the TypeScript layer.
    pub fn id(self) -> &'static str {
        match self {
            ArtworkSlot::GridPortrait => "gridPortrait",
            ArtworkSlot::GridLandscape => "gridLandscape",
            ArtworkSlot::Hero => "hero",
            ArtworkSlot::Logo => "logo",
            ArtworkSlot::Icon => "icon",
        }
    }

    /// Human label for the UI.
    pub fn label(self) -> &'static str {
        match self {
            ArtworkSlot::GridPortrait => "Portrait grid",
            ArtworkSlot::GridLandscape => "Landscape grid",
            ArtworkSlot::Hero => "Hero banner",
            ArtworkSlot::Logo => "Logo",
            ArtworkSlot::Icon => "Icon",
        }
    }

    /// Parse a slot identifier (camelCase, snake/kebab-case, or a friendly
    /// alias) into a slot.
    pub fn parse(value: &str) -> AppResult<Self> {
        let normalized = value
            .trim()
            .to_ascii_lowercase()
            .replace(['_', '-', ' '], "");
        match normalized.as_str() {
            "gridportrait" | "portrait" | "gridp" | "p" => Ok(ArtworkSlot::GridPortrait),
            "gridlandscape" | "landscape" | "header" | "grid" => Ok(ArtworkSlot::GridLandscape),
            "hero" | "banner" => Ok(ArtworkSlot::Hero),
            "logo" => Ok(ArtworkSlot::Logo),
            "icon" => Ok(ArtworkSlot::Icon),
            other => Err(AppError::Invalid(format!("unknown artwork slot '{other}'"))),
        }
    }

    /// The filename stem Steam expects for this slot and app id.
    pub fn stem(self, app_id: u32) -> String {
        match self {
            ArtworkSlot::GridPortrait => format!("{app_id}p"),
            ArtworkSlot::GridLandscape => app_id.to_string(),
            ArtworkSlot::Hero => format!("{app_id}_hero"),
            ArtworkSlot::Logo => format!("{app_id}_logo"),
            ArtworkSlot::Icon => format!("{app_id}_icon"),
        }
    }
}

/// The on-disk state of one artwork slot for a game.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkSlotState {
    pub slot: ArtworkSlot,
    pub label: String,
    pub present: bool,
    pub path: Option<String>,
    pub bytes: u64,
    pub modified: Option<i64>,
}

/// The artwork state for one game across all five slots.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameArtwork {
    pub app_id: u32,
    pub name: String,
    pub slots: Vec<ArtworkSlotState>,
    /// How many of the five slots are currently filled.
    pub applied: usize,
}

/// The result of applying or removing a slot's artwork.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkOutcome {
    pub app_id: u32,
    pub slot: ArtworkSlot,
    /// `"applied"` or `"removed"`.
    pub action: String,
    pub path: Option<String>,
    pub bytes: u64,
    pub source: Option<String>,
    /// Where the replaced image was archived, when there was one.
    pub backup: Option<String>,
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/// The `config/grid` folder that holds a user's custom artwork.
pub fn grid_dir(root: &Path, id3: &str) -> PathBuf {
    root.join("userdata").join(id3).join("config").join("grid")
}

/// Every existing image for `stem`, in any Steam-readable extension.
fn slot_files(dir: &Path, stem: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for extension in KNOWN_EXTENSIONS {
        let candidate = dir.join(format!("{stem}.{extension}"));
        if candidate.is_file() {
            files.push(candidate);
        }
    }
    files
}

/// The first existing image for `stem`, if any.
fn find_file(dir: &Path, stem: &str) -> Option<PathBuf> {
    slot_files(dir, stem).into_iter().next()
}

fn modified_millis(time: std::time::SystemTime) -> Option<i64> {
    time.duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis() as i64)
}

// ---------------------------------------------------------------------------
// Inspection
// ---------------------------------------------------------------------------

/// Inspect a single slot's on-disk state.
pub fn inspect_slot(dir: &Path, app_id: u32, slot: ArtworkSlot) -> ArtworkSlotState {
    let stem = slot.stem(app_id);
    match find_file(dir, &stem) {
        Some(path) => {
            let metadata = std::fs::metadata(&path).ok();
            ArtworkSlotState {
                slot,
                label: slot.label().to_string(),
                present: true,
                bytes: metadata.as_ref().map(|m| m.len()).unwrap_or(0),
                modified: metadata
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(modified_millis),
                path: Some(path.to_string_lossy().to_string()),
            }
        }
        None => ArtworkSlotState {
            slot,
            label: slot.label().to_string(),
            present: false,
            path: None,
            bytes: 0,
            modified: None,
        },
    }
}

/// Inspect every slot for each `(app_id, name)` entry.
pub fn inspect(dir: &Path, games: &[(u32, String)]) -> Vec<GameArtwork> {
    games
        .iter()
        .map(|(app_id, name)| {
            let slots: Vec<ArtworkSlotState> = ArtworkSlot::ALL
                .iter()
                .map(|slot| inspect_slot(dir, *app_id, *slot))
                .collect();
            let applied = slots.iter().filter(|state| state.present).count();
            GameArtwork {
                app_id: *app_id,
                name: name.clone(),
                slots,
                applied,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Mutation
// ---------------------------------------------------------------------------

/// Write `bytes` as the image for `app_id`/`slot`.
///
/// Any previous image for the same slot (in any Steam-readable extension) is
/// backed up and removed first, and the new file is written with an extension
/// matching the image's real format so Steam always loads it.
pub fn apply_bytes(
    dir: &Path,
    app_id: u32,
    slot: ArtworkSlot,
    bytes: &[u8],
    source: Option<String>,
) -> AppResult<ArtworkOutcome> {
    apply_bytes_with(dir, app_id, slot, bytes, source, true)
}

fn apply_bytes_with(
    dir: &Path,
    app_id: u32,
    slot: ArtworkSlot,
    bytes: &[u8],
    source: Option<String>,
    archive: bool,
) -> AppResult<ArtworkOutcome> {
    if bytes.is_empty() {
        return Err(AppError::Invalid("the image is empty".into()));
    }
    if bytes.len() > MAX_ARTWORK_BYTES {
        return Err(AppError::Invalid(format!(
            "the image is too large ({} bytes, limit {})",
            bytes.len(),
            MAX_ARTWORK_BYTES
        )));
    }
    let extension = sniff_extension(bytes).ok_or_else(|| {
        AppError::Invalid("unsupported image format (Steam needs PNG or JPEG)".into())
    })?;

    std::fs::create_dir_all(dir)?;
    let stem = slot.stem(app_id);

    // Archive and clear any existing image for this slot before writing.
    let mut backup = None;
    for existing in slot_files(dir, &stem) {
        if archive {
            let archived = crate::backup::backup_file(&existing)?;
            if backup.is_none() {
                backup = Some(archived.to_string_lossy().to_string());
            }
        }
        std::fs::remove_file(&existing)?;
    }

    let target = dir.join(format!("{stem}.{extension}"));
    std::fs::write(&target, bytes)?;

    Ok(ArtworkOutcome {
        app_id,
        slot,
        action: "applied".into(),
        path: Some(target.to_string_lossy().to_string()),
        bytes: bytes.len() as u64,
        source,
        backup,
    })
}

/// Remove the custom image for `app_id`/`slot` (after archiving it).
pub fn remove(dir: &Path, app_id: u32, slot: ArtworkSlot) -> AppResult<ArtworkOutcome> {
    remove_with(dir, app_id, slot, true)
}

fn remove_with(
    dir: &Path,
    app_id: u32,
    slot: ArtworkSlot,
    archive: bool,
) -> AppResult<ArtworkOutcome> {
    let stem = slot.stem(app_id);
    let files = slot_files(dir, &stem);
    if files.is_empty() {
        return Err(AppError::NotFound(format!(
            "no custom {} is set for app id {app_id}",
            slot.label().to_ascii_lowercase()
        )));
    }

    let mut backup = None;
    let mut bytes = 0u64;
    let mut last = None;
    for existing in files {
        if archive {
            let archived = crate::backup::backup_file(&existing)?;
            if backup.is_none() {
                backup = Some(archived.to_string_lossy().to_string());
            }
        }
        bytes += std::fs::metadata(&existing).map(|m| m.len()).unwrap_or(0);
        last = Some(existing.to_string_lossy().to_string());
        std::fs::remove_file(&existing)?;
    }

    Ok(ArtworkOutcome {
        app_id,
        slot,
        action: "removed".into(),
        path: last,
        bytes,
        source: None,
        backup,
    })
}

// ---------------------------------------------------------------------------
// Previews
// ---------------------------------------------------------------------------

/// Read a slot's image as a `data:` URL so the webview can preview it.
pub fn read_data_url(dir: &Path, app_id: u32, slot: ArtworkSlot) -> AppResult<Option<String>> {
    let stem = slot.stem(app_id);
    let Some(path) = find_file(dir, &stem) else {
        return Ok(None);
    };
    let metadata = std::fs::metadata(&path)?;
    if metadata.len() > MAX_PREVIEW_BYTES {
        return Err(AppError::Invalid(format!(
            "the image is too large to preview ({} bytes)",
            metadata.len()
        )));
    }
    let bytes = std::fs::read(&path)?;
    let mime = match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        Some("webp") => "image/webp",
        _ => "image/png",
    };
    Ok(Some(format!("data:{mime};base64,{}", base64(&bytes))))
}

/// Identify a PNG, JPEG, WebP, or ICO from its magic bytes.
fn sniff_extension(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some("png");
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("jpg");
    }
    // WebP: RIFFxxxxWEBP (Steam's Chromium CEF displays this as png)
    if bytes.starts_with(b"RIFF") && bytes.len() >= 12 && &bytes[8..12] == b"WEBP" {
        return Some("png");
    }
    // Windows Icon (.ico)
    if bytes.starts_with(&[0x00, 0x00, 0x01, 0x00]) {
        return Some("ico");
    }
    None
}

/// Minimal standard-alphabet base64 encoder (dependency-free).
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        out.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("steam-vault-art-{}-{tag}", std::process::id()))
    }

    #[test]
    fn slot_stems_match_steam() {
        assert_eq!(ArtworkSlot::GridPortrait.stem(620), "620p");
        assert_eq!(ArtworkSlot::GridLandscape.stem(620), "620");
        assert_eq!(ArtworkSlot::Hero.stem(620), "620_hero");
        assert_eq!(ArtworkSlot::Logo.stem(620), "620_logo");
        assert_eq!(ArtworkSlot::Icon.stem(620), "620_icon");
        // Non-Steam shortcuts (synthetic ids) use the same stems.
        assert_eq!(ArtworkSlot::GridPortrait.stem(0x8000_0001), "2147483649p");
    }

    #[test]
    fn slot_parse_accepts_aliases() {
        assert_eq!(
            ArtworkSlot::parse("gridPortrait").unwrap(),
            ArtworkSlot::GridPortrait
        );
        assert_eq!(
            ArtworkSlot::parse("grid-portrait").unwrap(),
            ArtworkSlot::GridPortrait
        );
        assert_eq!(
            ArtworkSlot::parse("grid_portrait").unwrap(),
            ArtworkSlot::GridPortrait
        );
        assert_eq!(
            ArtworkSlot::parse("HEADER").unwrap(),
            ArtworkSlot::GridLandscape
        );
        assert_eq!(ArtworkSlot::parse("banner").unwrap(), ArtworkSlot::Hero);
        assert!(ArtworkSlot::parse("nope").is_err());
    }

    #[test]
    fn sniff_detects_png_and_jpeg() {
        assert_eq!(
            sniff_extension(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]),
            Some("png")
        );
        assert_eq!(sniff_extension(&[0xFF, 0xD8, 0xFF, 0xE0]), Some("jpg"));
        assert_eq!(sniff_extension(b"GIF89a"), None);
        assert_eq!(sniff_extension(&[]), None);
    }

    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foob"), "Zm9vYg==");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn apply_writes_then_replaces_the_slot() {
        let dir = temp_dir("apply");
        let _ = std::fs::remove_dir_all(&dir);
        let png = [
            0x89u8, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x01, 0x02, 0x03,
        ];

        let outcome = apply_bytes_with(&dir, 620, ArtworkSlot::GridPortrait, &png, None, false)
            .expect("first apply");
        assert_eq!(outcome.action, "applied");
        assert_eq!(outcome.bytes, png.len() as u64);
        assert!(outcome.path.as_deref().unwrap().ends_with("620p.png"));

        // Replacing with a JPEG swaps the extension and leaves no PNG behind.
        let jpeg = [0xFFu8, 0xD8, 0xFF, 0xE0, 0x00];
        let outcome = apply_bytes_with(&dir, 620, ArtworkSlot::GridPortrait, &jpeg, None, false)
            .expect("replace");
        assert!(outcome.path.as_deref().unwrap().ends_with("620p.jpg"));
        assert!(!dir.join("620p.png").is_file());
        assert!(dir.join("620p.jpg").is_file());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_rejects_non_images() {
        let dir = temp_dir("reject");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            apply_bytes_with(&dir, 620, ArtworkSlot::Hero, b"not an image", None, false).is_err()
        );
        assert!(apply_bytes_with(&dir, 620, ArtworkSlot::Hero, &[], None, false).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_and_remove_round_trip() {
        let dir = temp_dir("inspect");
        let _ = std::fs::remove_dir_all(&dir);
        let png = [0x89u8, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x07];

        assert!(!inspect_slot(&dir, 620, ArtworkSlot::Hero).present);
        apply_bytes_with(&dir, 620, ArtworkSlot::Hero, &png, None, false).expect("apply");

        let state = inspect_slot(&dir, 620, ArtworkSlot::Hero);
        assert!(state.present);
        assert_eq!(state.bytes, png.len() as u64);

        let games = inspect(&dir, &[(620, "Portal".into())]);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].applied, 1);

        let outcome = remove_with(&dir, 620, ArtworkSlot::Hero, false).expect("remove");
        assert_eq!(outcome.action, "removed");
        assert!(!inspect_slot(&dir, 620, ArtworkSlot::Hero).present);
        assert!(remove_with(&dir, 620, ArtworkSlot::Hero, false).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
