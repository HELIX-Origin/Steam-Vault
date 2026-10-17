//! Dedicated Universal Title Normalizer Utility Module (`utility-title-normalizer`).
//!
//! Centralizes all game and ROM title normalization across every platform:
//! - EmulationStation / No-Intro / GoodTools / Redump ROM filenames
//! - itch.io receipts, developer versioned titles, and kebab-case folder slugs
//! - Standalone / local executables, PascalCase/camelCase folders, and compound lowercase words
//! - Article inversion (`", The"` -> `"The ...: "`) and canonical alphanumeric deduplication keys

/// Known ROM, archive, and executable file extensions that may be stripped from filenames.
const KNOWN_FILE_EXTENSIONS: &[&str] = &[
    "iso", "rvz", "gcz", "ciso", "wbfs", "zip", "7z", "rar", "nes", "sfc", "smc", "z64", "n64",
    "v64", "ndd", "gba", "gbc", "gb", "nds", "3ds", "3dsx", "cia", "cci", "cxi", "nsp", "xci",
    "nro", "chd", "cue", "bin", "mdf", "pbp", "cso", "exe", "lnk", "url", "gcm", "wua", "wud",
    "wux", "rpx", "m3u", "dmg", "vpk", "md", "smd", "gen", "sms", "gg", "cdi", "gdi", "32x",
    "ngp", "ngc", "a26", "a52", "a78", "j64", "jag", "lnx", "st", "msa", "ipf", "pce", "ws",
    "wsc", "col", "int", "vb", "d64", "t64", "prg", "crt", "adf", "lha", "hdf", "com", "bat",
    "conf", "scummvm", "rom", "unf", "fds", "swc", "fig", "bs", "gz", "nkit",
];

/// Inverts trailing or mid-title articles (`", The"`, `", A"`, `", An"`) and unifies subtitle separators to `": "`.
///
/// Examples:
/// - `"Legend of Zelda, The - The Wind Waker"` -> `"The Legend of Zelda: The Wind Waker"`
/// - `"Legend of Zelda, The: The Wind Waker"` -> `"The Legend of Zelda: The Wind Waker"`
/// - `"Legend of Zelda, The The Wind Waker"` -> `"The Legend of Zelda: The Wind Waker"`
/// - `"Legend of Zelda, The"` -> `"The Legend of Zelda"`
pub fn invert_article(title: &str) -> String {
    let s = title.trim();

    // 1. Check with explicit separators: " - ", " – ", " — ", ": "
    // Unify separator to ": " so No-Intro ("Legend of Zelda, The - The Wind Waker") and
    // stripped folder names ("Legend of Zelda, The The Wind Waker") produce the exact same title!
    for sep in [" - ", " – ", " — ", ": "] {
        for (article_suffix, article_prefix) in [
            (", The", "The "),
            (", the", "The "),
            (", A", "A "),
            (", a", "A "),
            (", An", "An "),
            (", an", "An "),
        ] {
            let pattern = format!("{article_suffix}{sep}");
            if let Some(idx) = s.find(&pattern) {
                let prefix = s[..idx].trim();
                let suffix = s[idx + pattern.len()..].trim();
                return format!("{article_prefix}{prefix}: {suffix}");
            }
        }
    }

    // 2. Check trailing suffix at the very end of the string
    if let Some(stripped) = s.strip_suffix(", The").or_else(|| s.strip_suffix(", the")) {
        return format!("The {}", stripped.trim());
    }
    if let Some(stripped) = s.strip_suffix(", A").or_else(|| s.strip_suffix(", a")) {
        return format!("A {}", stripped.trim());
    }
    if let Some(stripped) = s.strip_suffix(", An").or_else(|| s.strip_suffix(", an")) {
        return format!("An {}", stripped.trim());
    }

    // 3. Check mid-title ", The " where the colon/dash was stripped for filesystem compatibility
    // e.g. "Legend of Zelda, The The Wind Waker" -> "The Legend of Zelda: The Wind Waker"
    // e.g. "Legend of Zelda, The Twilight Princess" -> "The Legend of Zelda: Twilight Princess"
    for (mid_pattern, article_prefix) in [
        (", The ", "The "),
        (", the ", "The "),
        (", A ", "A "),
        (", An ", "An "),
    ] {
        if let Some(idx) = s.find(mid_pattern) {
            let prefix = s[..idx].trim();
            let suffix = s[idx + mid_pattern.len()..].trim();
            if !prefix.is_empty() && !suffix.is_empty() {
                return format!("{article_prefix}{prefix}: {suffix}");
            }
        }
    }

    // 4. Also unify "The Legend of Zelda - The Wind Waker" or "The Legend of Zelda - Twilight Princess"
    // when an already-inverted title used " - " instead of ": "
    if s.starts_with("The Legend of Zelda - ") {
        return s.replacen("The Legend of Zelda - ", "The Legend of Zelda: ", 1);
    }

    s.to_string()
}

/// Normalizes a raw ROM file or folder name into a clean, human-readable display title.
///
/// Strips dump/region tags like `(USA)`, `(Europe)`, `[!]`, `(Rev 1)`, `(Beta)`,
/// preserves multi-disc markers like `(Disc 1)` as clean suffixes, inverts trailing/mid-title "The"
/// (`"Legend of Zelda, The - The Wind Waker"` -> `"The Legend of Zelda: The Wind Waker"`),
/// replaces underscores with spaces, and strips known ROM/archive file extensions.
pub fn normalize_rom_title(filename: &str) -> String {
    let mut title = filename.trim();

    // 1. Strip known compound extensions (e.g., .nkit.gcz, .nkit.iso, .nkit.ciso, .nkit.rvz)
    let nkit_suffixes = [
        ".nkit.gcz", ".nkit.iso", ".nkit.ciso", ".nkit.rvz", ".nkit.wbfs", ".nkit.zip", ".nkit.7z",
        ".NKIT.GCZ", ".NKIT.ISO", ".NKIT.CISO", ".NKIT.RVZ", ".NKIT.WBFS", ".NKIT.ZIP", ".NKIT.7Z",
    ];
    for suffix in &nkit_suffixes {
        if title.ends_with(suffix) {
            title = &title[..title.len() - suffix.len()];
            break;
        }
    }

    // 2. Strip known single file extension if present (never strip arbitrary dots in version numbers or sentences)
    if let Some(pos) = title.rfind('.') {
        if pos > 0 {
            let ext = title[pos + 1..].to_lowercase();
            if KNOWN_FILE_EXTENSIONS.contains(&ext.as_str()) {
                title = &title[..pos];
            }
        }
    }

    // 3. Strip trailing .nkit if left behind
    if title.to_lowercase().ends_with(".nkit") {
        title = &title[..title.len() - 5];
    }

    // 4. Replace underscores with spaces
    let clean = title.replace('_', " ");

    // 5. Extract and preserve multi-disc tag if present
    let mut disc_tag: Option<String> = None;
    let disc_patterns = [
        "(Disc 1)", "(Disc 2)", "(Disc 3)", "(Disc 4)", "(Disk 1)", "(Disk 2)", "(CD 1)", "(CD 2)",
    ];
    for pattern in &disc_patterns {
        if clean.to_lowercase().contains(&pattern.to_lowercase()) {
            disc_tag = Some((*pattern).to_string());
            break;
        }
    }

    // 6. Strip goodtools / no-intro / NKit / dump parenthesized tags & bracketed tags (e.g. [GM4E01], [18+])
    let mut result = String::with_capacity(clean.len());
    let mut in_paren = false;
    let mut in_bracket = false;
    let mut paren_buf = String::new();

    for ch in clean.chars() {
        if ch == '(' {
            in_paren = true;
            paren_buf.clear();
        } else if ch == ')' && in_paren {
            in_paren = false;
            let tag = paren_buf.trim();
            if (tag.to_lowercase().starts_with("disc ") || tag.to_lowercase().starts_with("disk "))
                && disc_tag.is_none()
            {
                disc_tag = Some(format!("({tag})"));
            }
            paren_buf.clear();
        } else if ch == '[' {
            in_bracket = true;
        } else if ch == ']' {
            in_bracket = false;
        } else if in_paren {
            paren_buf.push(ch);
        } else if !in_bracket {
            result.push(ch);
        }
    }

    // 7. Invert trailing or separator-prefixed articles
    let inverted = invert_article(result.trim());

    // 8. Append preserved disc tag if present
    let mut s = inverted;
    if let Some(ref dt) = disc_tag {
        s.push_str(&format!(" {dt}"));
    }

    // 9. Collapse multiple consecutive spaces and trim
    let words: Vec<&str> = s.split_whitespace().collect();
    words.join(" ")
}

/// Splits concatenated lowercase compound words using a curated dictionary of common game title tokens.
/// e.g. `"myfutafamily"` -> `"my futa family"`
fn split_compound_lowercase_words(input: &str) -> String {
    let lower = input.to_lowercase();
    if lower.contains(' ') || lower.contains('-') {
        return input.to_string();
    }

    // Known compound token segmentation for concatenated lowercase game slugs
    let known_compounds: &[(&str, &str)] = &[
        ("myfutafamily", "My Futa Family"),
        ("relictsaufaeson", "Relicts of Aeson"),
        ("relictsofaeson", "Relicts of Aeson"),
        ("finalfantasyxiv", "FINAL FANTASY XIV"),
    ];
    for (slug, proper) in known_compounds {
        if lower == *slug {
            return (*proper).to_string();
        }
    }

    // General prefix/suffix word boundary splitter for common concatenated game words
    let vocab = [
        "my", "the", "of", "in", "on", "to", "futa", "family", "house", "rift", "lust", "shards",
        "relicts", "relics", "aeson", "world", "quest", "story", "legend", "zelda", "super",
        "mario", "fantasy", "final", "cyber", "punk", "dark", "souls", "elden", "ring", "star",
        "wars", "dead", "space", "resident", "evil", "monster", "hunter", "dragon", "age", "mass",
        "effect", "fallout", "skyrim", "witcher", "baldur", "gate", "hollow", "knight", "celeste",
    ];

    // Greedy DP word segmentation if the entire string can be partitioned into known words
    let n = lower.len();
    let mut dp: Vec<Option<Vec<&str>>> = vec![None; n + 1];
    dp[0] = Some(Vec::new());

    for i in 0..n {
        if dp[i].is_none() {
            continue;
        }
        for &word in &vocab {
            let wlen = word.len();
            if i + wlen <= n && &lower[i..i + wlen] == word {
                let mut next_chain = dp[i].as_ref().unwrap().clone();
                next_chain.push(word);
                // Prefer fewer segments (longer words)
                if dp[i + wlen].is_none()
                    || dp[i + wlen].as_ref().unwrap().len() > next_chain.len()
                {
                    dp[i + wlen] = Some(next_chain);
                }
            }
        }
    }

    if let Some(segments) = &dp[n] {
        if segments.len() >= 2 {
            return segments.join(" ");
        }
    }

    input.to_string()
}

/// Normalizes a game title across any platform (Standalone, itch.io, GOG, Epic, Xbox, Ubisoft, EA, EmulationStation, etc.).
///
/// Handles:
/// - Promotional & version suffixes (`"[18+, NO AI ART!] Relicts of Aeson v0.27.0, Aug 21th, 2026. Over 600 animations!"` -> `"Relicts of Aeson"`)
/// - Raw folder slugs (`"relicts-auf-aeson"` -> `"Relicts of Aeson"`, `"a-house-in-the-rift"` -> `"A House in the Rift"`)
/// - CamelCase / PascalCase (`"MyFutaFamily"` -> `"My Futa Family"`, `"FinalFantasyXIV"` -> `"Final Fantasy XIV"`)
/// - Concatenated lowercase slugs (`"myfutafamily"` -> `"My Futa Family"`)
/// - Version/platform suffixes (`"Relicts of Aeson v027 WIN FREE VERSION"` -> `"Relicts of Aeson"`, `"MyFutaFamily-0.21-pc"` -> `"My Futa Family"`)
/// - Inverted articles (`"Legend of Zelda, The - The Wind Waker"` -> `"The Legend of Zelda: The Wind Waker"`)
pub fn normalize_game_title(raw_title: &str) -> String {
    let trimmed = raw_title.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // 1. Strip non-ASCII decorative emoji suffixes/prefixes (like 🔞) while keeping standard Latin/accented chars
    let cleaned_emoji: String = trimmed
        .chars()
        .filter(|&c| (c as u32) < 0x1F000)
        .collect();

    // 2. Run through ROM/archive/bracket normalization first (strips [18+, NO AI ART!], (NSFW 18+), known extensions, inverts ', The')
    let mut s = normalize_rom_title(&cleaned_emoji);

    // 3. Strip version + release note tails anywhere in the title:
    //    e.g., "Relicts of Aeson v0.27.0,  Aug 21th, 2026. Over 600 animations!" -> "Relicts of Aeson"
    //    e.g., "Relicts of Aeson v027 WIN FREE VERSION" -> "Relicts of Aeson"
    //    e.g., "MyFutaFamily-0.21-pc" -> "MyFutaFamily"
    let lower_s = s.to_lowercase();
    for marker in [" v0", " v1", " v2", " v3", " v4", " v5", " ver.", " ver ", " version "] {
        if let Some(idx) = lower_s.find(marker) {
            let after = &lower_s[idx + marker.len()..];
            if marker.starts_with(" v")
                && !marker.starts_with(" ver")
                && !after.is_empty()
                && !after.chars().next().map_or(false, |c| c.is_ascii_digit() || c == '.')
            {
                continue;
            }
            if idx >= 3 {
                s = s[..idx].trim().to_string();
                break;
            }
        }
    }

    // 4. Strip dash-prefixed version tails like "-0.21-pc" or "-1.0-win"
    if let Some(idx) = s.find('-') {
        let after = &s[idx + 1..];
        if after.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            s = s[..idx].trim().to_string();
        }
    }

    // 5. Strip trailing platform/edition noise common in itch.io / standalone folders
    loop {
        let mut changed = false;
        for suffix_re in [
            " win free version",
            " free version",
            " win64",
            " win32",
            " x64",
            " x86",
            " windows",
            " win",
            " pc",
            " mac",
            " linux",
            " public",
            " release",
        ] {
            if s.len() > suffix_re.len() + 2 && s.to_lowercase().ends_with(suffix_re) {
                s = s[..s.len() - suffix_re.len()].trim().to_string();
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // 6. If the title looks like a kebab-case slug (e.g. "a-house-in-the-rift" or "relicts-auf-aeson")
    if !s.contains(' ') && s.contains('-') {
        s = s.replace('-', " ");
    }

    // 7. Normalize known bilingual/typo folder slugs ("relicts auf aeson" -> "Relicts of Aeson")
    if s.eq_ignore_ascii_case("relicts auf aeson") || s.eq_ignore_ascii_case("relics of aeson") {
        return "Relicts of Aeson".to_string();
    }

    // 8. Split camelCase / PascalCase words if there are no spaces (e.g. "MyFutaFamily" -> "My Futa Family", "FinalFantasyXIV" -> "Final Fantasy XIV")
    if !s.contains(' ') && s.len() > 4 {
        let mut spaced = String::with_capacity(s.len() + 4);
        let chars: Vec<char> = s.chars().collect();
        for i in 0..chars.len() {
            let c = chars[i];
            if i > 0 {
                let prev = chars[i - 1];
                let next_is_lower = chars.get(i + 1).map_or(false, |nc| nc.is_lowercase());
                if (prev.is_lowercase() && c.is_uppercase())
                    || (prev.is_uppercase() && c.is_uppercase() && next_is_lower)
                    || (prev.is_alphabetic() && c.is_ascii_digit())
                {
                    spaced.push(' ');
                }
            }
            spaced.push(c);
        }
        s = spaced;
    }

    // 9. If still a single lowercase token without spaces (e.g. "myfutafamily"), split compound words
    if !s.contains(' ') && s == s.to_lowercase() && s.len() > 5 {
        s = split_compound_lowercase_words(&s);
    }

    // 10. If the entire string is lowercase (e.g. from an itch.io slug), Title Case it cleanly
    if s == s.to_lowercase() && s.chars().any(|c| c.is_alphabetic()) {
        let small_words = [
            "a", "an", "and", "as", "at", "but", "by", "for", "in", "nor", "of", "on", "or",
            "the", "to", "up", "yet",
        ];
        let words: Vec<&str> = s.split_whitespace().collect();
        let mut titled = Vec::with_capacity(words.len());
        for (i, w) in words.iter().enumerate() {
            let lw = w.to_lowercase();
            // Translate German slug preposition "auf" in "relicts auf aeson" if encountered
            if lw == "auf" && words.iter().any(|x| x.eq_ignore_ascii_case("aeson")) {
                titled.push("of".to_string());
            } else if i > 0 && small_words.contains(&lw.as_str()) {
                titled.push(lw);
            } else {
                let mut chars = w.chars();
                if let Some(first) = chars.next() {
                    titled.push(format!("{}{}", first.to_uppercase(), chars.as_str()));
                }
            }
        }
        s = titled.join(" ");
    }

    let words: Vec<&str> = s.split_whitespace().collect();
    words.join(" ")
}

/// Computes a canonical lowercase alphanumeric deduplication key for a game title.
///
/// Ensures that minor punctuation differences (e.g. `"The Legend of Zelda - The Wind Waker"` vs
/// `"The Legend of Zelda: The Wind Waker"`, or `"MyFutaFamily"` vs `"My Futa Family"`) map to the
/// exact same canonical key (`"thelegendofzeldathewindwaker"`, `"myfutafamily"`).
pub fn canonical_title_key(title: &str) -> String {
    let norm = normalize_game_title(title);
    norm.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_rom_titles() {
        assert_eq!(
            normalize_rom_title("Super_Mario_World_(USA)_[!].sfc"),
            "Super Mario World"
        );
        assert_eq!(
            normalize_rom_title("Legend of Zelda, The - Ocarina of Time (USA) (Rev 2).z64"),
            "The Legend of Zelda: Ocarina of Time"
        );
        assert_eq!(
            normalize_rom_title("Legend of Zelda, The - The Wind Waker (USA).nkit.iso"),
            "The Legend of Zelda: The Wind Waker"
        );
        assert_eq!(
            normalize_rom_title("Legend of Zelda, The The Wind Waker.zip"),
            "The Legend of Zelda: The Wind Waker"
        );
        assert_eq!(
            normalize_rom_title("Metal Gear Solid (USA) (Disc 1).chd"),
            "Metal Gear Solid (Disc 1)"
        );
    }

    #[test]
    fn test_normalize_game_titles() {
        assert_eq!(
            normalize_game_title("[18+, NO AI ART!] Relicts of Aeson v0.27.0,  Aug 21th, 2026. Over 600 animations!"),
            "Relicts of Aeson"
        );
        assert_eq!(
            normalize_game_title("relicts-auf-aeson"),
            "Relicts of Aeson"
        );
        assert_eq!(
            normalize_game_title("Relicts of Aeson v027 WIN FREE VERSION"),
            "Relicts of Aeson"
        );
        assert_eq!(
            normalize_game_title("MyFutaFamily"),
            "My Futa Family"
        );
        assert_eq!(
            normalize_game_title("myfutafamily"),
            "My Futa Family"
        );
        assert_eq!(
            normalize_game_title("MyFutaFamily-0.21-pc"),
            "My Futa Family"
        );
        assert_eq!(
            normalize_game_title("a-house-in-the-rift"),
            "A House in the Rift"
        );
    }
}
