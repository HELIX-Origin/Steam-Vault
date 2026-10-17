import type { UniversalPlugin } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

/**
 * Known scene / repacker / dump release group tags stripped during title normalization.
 */
const SCENE_GROUPS = new Set([
  "codex",
  "plaza",
  "skidrow",
  "reloaded",
  "flt",
  "fairlight",
  "razor1911",
  "hoodlum",
  "tenoke",
  "rune",
  "elamigos",
  "fitgirl",
  "dodi",
  "gog",
  "kaos",
  "darksiders",
  "simplex",
  "tiiny",
  "tinyiso",
  " Goldberg",
  "goldberg",
  "ali213",
  "3dm",
  "cpy",
  "steampunks",
  "prophet",
  "vitality",
  "deviance",
  "hatred",
  "chronos",
  "siMPLEX",
  "repack",
  "portable",
  "setup",
  "install",
  "installer",
  "win64",
  "win32",
  "x64",
  "x86",
  "pc",
  " windows",
  "mac",
  "linux",
  "multilang",
  "multi",
]);

/**
 * Dictionary of concatenated lowercase game words for splitting run-together folder/exe names
 * such as "myfutafamily" -> "My Futa Family".
 */
const CONCAT_WORD_DICTIONARY = [
  "the", "legend", "of", "zelda", "wind", "waker", "twilight", "princess",
  "super", "mario", "sunshine", "kart", "smash", "bros", "melee", "brawl",
  "metroid", "prime", "echoes", "corruption", "paper", "thousand", "year", "door",
  "luigi", "luigis", "mansion", "animal", "crossing", "pikmin", "star", "fox",
  "adventures", "assault", "fire", "emblem", "path", "radiance", "fzero", "gx",
  "kirby", "air", "ride", "donkey", "kong", "jungle", "beat", "pokemon",
  "colosseum", "xd", "gale", "darkness", "resident", "evil", "final", "fantasy",
  "crystal", "chronicles", "tales", "symphonia", "skies", "arcadia", "legends",
  "eternal", "sonic", "adventure", "heroes", "shadow", "hedgehog", "mega", "man",
  "soulcalibur", "metal", "gear", "solid", "twin", "snakes", "time", "splitters",
  "relicts", "aeson", "my", "futa", "family", "cyberpunk", "witcher", "wild",
  "hunt", "elder", "scrolls", "skyrim", "fallout", "new", "vegas", "grand",
  "theft", "auto", "red", "dead", "redemption", "hollow", "knight", "celeste",
  "stardew", "valley", "terraria", "minecraft", "baldur", "baldurs", "gate",
  "divinity", "original", "sin", "dark", "souls", "elden", "ring", "sekiro",
  "bloodborne", "armored", "core", "monster", "hunter", "world", "rise",
  "dragon", "quest", "persona", "shin", "megami", "tensei", "kingdom", "hearts",
  "chrono", "trigger", "cross", "secret", "mana", "earthbound", "mother",
  "castlevania", "symphony", "night", "contra", "gradius", "silent", "hill",
  "devil", "may", "cry", "bayonetta", "nier", "automata", "replicant", "yakuza",
  "like", "a", "judgment", "sleeping", "dogs", "watch", "dogs", "assassins",
  "assassin", "creed", "far", "cry", "tom", "clancy", "splinter", "cell",
  "rainbow", "six", "ghost", "recon", "division", "prince", "persia", "rayman",
  "beyond", "good", "evil", "bioshock", "infinite", "borderlands", "dishonored",
  "prey", "doom", "eternal", "quake", "wolfenstein", "rage", "halo", "gears",
  "war", "forza", "horizon", "motorsport", "fable", "sea", "thieves", "ori",
  "blind", "forest", "will", "wisps", "cuphead", "dead", "cells", "hades",
  "bastion", "transistor", "pyre", "slay", "spire", "binding", "isaac",
  "enter", "gungeon", "nuclear", "throne", "risk", "rain", "faster", "than",
  "light", "into", "breach", "shovel", "knight", "katana", "zero", "hotline",
  "miami", "undertale", "deltarune", "omori", "oneshot", "to", "moon", "disco",
  "elysium", "outer", "wilds", "worlds", "subnautica", "below", "no", "mans",
  "sky", "factorio", "satisfactory", "rimworld", "dwarf", "fortress", "oxygen",
  "not", "included", "dont", "starve", "together", "klei", "mark", "ninja",
  "shank", "invisible", "inc", "griftlands", "cult", "lamb", "inscyption",
  "inscryption", "pony", "island", "hex", "return", "obra", "dinn", "papers",
  "please", "lucas", "pope", "stanley", "parable", "beginners", "guide",
  "portal", "half", "life", "alyx", "left", "dead", "team", "fortress", "counter",
  "strike", "global", "offensive", "dota", "artifact", "underlords", "ricochet",
  "day", "defeat", "garrys", "mod", "black", "mesa", "unreal", "tournament",
  "ut", "jazz", "jackrabbit", "epic", "pinball", "gears", "fortnite", "rocket",
  "league", "fall", "guys", "alan", "wake", "control", "max", "payne", "quantum",
  "break", "death", "stranding", "horizon", "zero", "dawn", "forbidden", "west",
  "god", "ragnarok", "spider", "man", "miles", "morales", "ghost", "tsushima",
  "last", "us", "part", "uncharted", "legacy", "thieves", "ratchet", "clank",
  "rift", "apart", "returnal", "demon", "sackboy", "big", "little", "planet",
  "infamous", "second", "son", "killzone", "resistance", "wipeout", "gran",
  "turismo", "blood", "truth", "astro", "bot", "rescue", "mission", "days",
  "gone", "detroit", "become", "human", "heavy", "rain", "beyond", "two",
  "souls", "until", "quarry", "dark", "pictures", "anthology", "little",
  "nightmares", "it", "takes", "way", "out", "brothers", "tale", "sons",
  "unravel", "fe", "lost", "in", "random", "alice", "madness", "returns",
  "mirror", "mirrors", "edge", "catalyst", "dead", "space", "mass", "effect",
  "andromeda", "dragon", "age", "origins", "inquisition", "veilguard", "jade",
  "empire", "star", "wars", "knights", "old", "republic", "jedi", "fallen",
  "order", "survivor", "battlefront", "squadrons", "titanfall", "apex",
  "need", "speed", "underground", "most", "wanted", "carbon", "prostreet",
  "shift", "hot", "pursuit", "rivals", "payback", "heat", "unbound", "burnout",
  "paradise", "revenge", "takedown", "skate", "ssx", "tricky", "simcity",
  "sims", "spore", "command", "conquer", "red", "alert", "tiberian", "sun",
  "generals", "dungeon", "keeper", "populous", "theme", "hospital", "park",
  "syndicate", "wing", "commander", "ultima", "system", "shock", "thief",
  "deus", "ex", "human", "revolution", "mankind", "divided", "tomb", "raider",
  "hitman", "blood", "money", "absolution", "just", "cause", "sleeping",
  "kane", "lynch", "legacy", "kain", "soul", "reaver", "defiance", "chrono",
  "xenogears", "xenosaga", "xenoblade", "vagrant", "story", "parasite", "eve",
  "brave", "fencer", "musashi", "front", "mission", "drakengard", "octopath",
  "traveler", "bravely", "default", "triangle", "strategy", "live", "star",
  "ocean", "valkyrie", "profile", "radiata", "stories", "grandia", "lunar",
  "silver", "story", "eternal", "blue", "suikoden", "wild", "arms", "breath",
  "shadow", "hearts", "atelier", "disgaea", "trails", "sky", "cold", "steel",
  "ys", "lacrimosa", "dana", "monstrum", "nox", "nordics"
];

const LOWERCASE_WORDS = new Set([
  "a", "an", "the", "and", "but", "or", "for", "nor", "on", "at", "to", "from", "by", "of", "in", "with"
]);

const UPPERCASE_ROMAN_OR_ACRONYM = new Set([
  "i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix", "x", "xi", "xii", "xiii", "xiv", "xv", "xvi",
  "hd", "vr", "dx", "gx", "gt", "gta", "rpg", "fps", "mmo", "dlc", "goty", "ce", "se", "de", "ee", "3d", "2d", "64", "ds", "gba", "gb", "gbc", "nes", "snes", "n64", "gc", "wii", "ps1", "ps2", "ps3", "ps4", "ps5", "psp", "psv", "pc", "ai", "ui", "fx", "xd"
]);

/**
 * Inverts trailing articles (`", The"`, `", A"`, `", An"`) to the front of a title or segment.
 * Also handles subtitle boundaries like `"Legend of Zelda, The - The Wind Waker"` or `"Legend of Zelda, The: Twilight Princess"`.
 */
export function invertTrailingArticle(input: string): string {
  let s = input.trim();
  // Match `<prefix>, (The|A|An)` before ` - `, `: `, or end of string
  const subPattern = /^([^,:]+),\s*(The|A|An)(?:\s*[-:]\s*(.+))?$/i;
  const m = s.match(subPattern);
  if (m) {
    const mainBody = m[1].trim();
    const article = m[2].charAt(0).toUpperCase() + m[2].slice(1).toLowerCase();
    const rest = m[3]?.trim();
    if (rest) {
      // Check if rest also has trailing article
      const normRest = invertTrailingArticle(rest);
      return `${article} ${mainBody}: ${normRest}`;
    }
    return `${article} ${mainBody}`;
  }
  return s;
}

/**
 * Splits a single concatenated lowercase token into constituent words using greedy longest-match
 * against `CONCAT_WORD_DICTIONARY` (e.g. `"myfutafamily"` -> `"my futa family"`).
 */
function splitConcatenatedWords(token: string): string {
  const lower = token.toLowerCase();
  if (!/^[a-z]{8,}$/.test(lower)) {
    return token;
  }

  // Dynamic programming segmentation using dictionary words
  const n = lower.length;
  const dp: (string[] | null)[] = Array(n + 1).fill(null);
  dp[0] = [];

  const dictSet = new Set(CONCAT_WORD_DICTIONARY);

  for (let i = 0; i < n; i++) {
    const current = dp[i];
    if (!current) continue;
    for (let j = i + 2; j <= n; j++) {
      const sub = lower.slice(i, j);
      if (dictSet.has(sub)) {
        const candidate = [...current, sub];
        if (!dp[j] || candidate.length < dp[j]!.length) {
          dp[j] = candidate;
        }
      }
    }
  }

  if (dp[n] && dp[n]!.length >= 2) {
    return dp[n]!.join(" ");
  }
  return token;
}

/**
 * Applies smart Title Case to a cleaned title while preserving Roman numerals, acronyms, and articles.
 */
function toSmartTitleCase(input: string): string {
  // Normalise German "auf" typo in "Relicts auf Aeson" -> "Relicts of Aeson"
  let cleaned = input.replace(/\brelicts\s+auf\s+aeson\b/gi, "Relicts of Aeson");

  const tokens = cleaned.split(/\s+/);
  return tokens
    .map((word, idx) => {
      if (!word) return word;
      const rawLower = word.toLowerCase().replace(/^[^a-z0-9]+|[^a-z0-9]+$/g, "");
      if (UPPERCASE_ROMAN_OR_ACRONYM.has(rawLower)) {
        return word.toUpperCase();
      }
      if (idx > 0 && LOWERCASE_WORDS.has(rawLower) && !word.endsWith(":")) {
        return word.toLowerCase();
      }
      return word.charAt(0).toUpperCase() + word.slice(1);
    })
    .join(" ");
}

/**
 * Normalizes any PC or console game title:
 * - Strips file extensions (`.exe`, `.iso`, `.rvz`, `.wbfs`, `.chd`, `.nsp`, `.xci`, `.7z`, `.zip`, etc.)
 * - Strips bracketed/parenthesized ROM dump tags (`(USA)`, `(En,Fr,De)`, `[!]`, `(Rev 1)`, `(v1.02)`)
 * - Strips version numbers (`v1.2.3`, `Build 12345`) and scene/repacker tags (`-CODEX`, `FitGirl Repack`)
 * - Splits camelCase, PascalCase, snake_case, kebab-case, and concatenated lowercase words (`myfutafamily` -> `My Futa Family`)
 * - Inverts trailing articles (`Legend of Zelda, The` -> `The Legend of Zelda`)
 */
export function normalizeGameTitle(raw: string): string {
  if (!raw || !raw.trim()) return "";

  let s = raw.trim();

  // 1. Strip known file extensions
  s = s.replace(
    /\.(exe|lnk|url|bat|cmd|sh|appimage|iso|rvz|gcz|wbfs|wad|gcm|ciso|chd|cue|bin|gdi|cdi|mds|mdf|nrg|img|rom|nes|sfc|smc|z64|n64|v64|ndd|gb|gbc|gba|nds|dsi|3ds|cia|cci|cxi|nsp|xci|nsz|xcz|wux|wud|rpx|elf|dol|pbp|cso|pkg|vpk|xbe|xex|zar|zip|7z|rar|tar|gz|xz)$/i,
    ""
  );

  // 2. Strip bracketed tags `[...]` and parenthesized metadata tags `(USA)`, `(Rev 1)`, `(Disc 1)`, `(v1.0)`, etc.
  s = s.replace(/\[[^\]]*\]/g, " ");
  s = s.replace(
    /\((?:usa|europe|japan|world|en|fr|de|es|it|nl|pt|sv|no|da|fi|ru|ja|ko|zh|rev\s*\d*[a-z]?|v\d+[\d._a-z]*|disc\s*\d+|disk\s*\d+|proto|beta|demo|sample|unl|pirate|hack|alt|ntsc|pal|secam|arcade|hd|remaster(?:ed)?|switch|ps[1-5]|xbox|pc|windows|mac|linux|x64|x86|64-bit|32-bit|[a-z]{2}(?:,[a-z]{2})+)\)/gi,
    " "
  );
  // Also strip any trailing parenthesized tags that contain commas or version numbers
  s = s.replace(/\((?:[^)]*,[^)]*|v?\d+\.\d+[^)]*)\)/gi, " ");

  // 3. Replace underscores and dots (when not part of abbreviations) with spaces
  s = s.replace(/_/g, " ");
  s = s.replace(/\.(?=[a-zA-Z]{2,})/g, " ");

  // 4. Split camelCase / PascalCase boundaries (`MyFutaFamily` -> `My Futa Family`)
  s = s.replace(/([a-z])([A-Z])/g, "$1 $2");
  s = s.replace(/([A-Z]+)([A-Z][a-z])/g, "$1 $2");

  // 5. Replace kebab-case hyphens with spaces when the string has no spaces (`relicts-auf-aeson` -> `relicts auf aeson`)
  if (!s.includes(" ") && s.includes("-")) {
    s = s.replace(/-/g, " ");
  }

  // 6. Strip trailing version tokens (`v1.0.4`, `ver 1.2`, `build 1234`) and scene group suffixes
  s = s.replace(/\b(?:v|ver|version|build|patch|update)\s*\d+(?:[._-]\d+)*[a-z]?\b/gi, " ");
  const parts = s.split(/\s+-\s+|\s+/);
  const filteredParts: string[] = [];
  for (const p of parts) {
    const cleanToken = p.toLowerCase().replace(/[^a-z0-9]/g, "");
    if (SCENE_GROUPS.has(cleanToken) && filteredParts.length > 0) {
      continue;
    }
    filteredParts.push(p);
  }
  s = filteredParts.join(" ");

  // 7. Split concatenated single-token words (`myfutafamily` -> `my futa family`)
  s = s
    .split(/\s+/)
    .map((tok) => splitConcatenatedWords(tok))
    .join(" ");

  // 8. Invert trailing articles (`Legend of Zelda, The` -> `The Legend of Zelda`)
  s = invertTrailingArticle(s);

  // 9. Normalize ` - ` subtitle separator to `: ` for iconic series or standard titles
  s = s.replace(/\s+-\s+/g, ": ");

  // Re-run article inversion on segments separated by `: `
  if (s.includes(": ")) {
    const segs = s.split(": ").map((seg) => invertTrailingArticle(seg.trim()));
    s = segs.join(": ");
  }

  // 10. Collapse whitespace and apply smart Title Case
  s = s.replace(/\s+/g, " ").trim();
  s = toSmartTitleCase(s);

  return s;
}

/**
 * Normalizes an EmulationStation / console ROM filename or folder name into a canonical game title.
 */
export function normalizeRomTitle(rawFilename: string): string {
  return normalizeGameTitle(rawFilename);
}

/**
 * Produces a canonical comparison key for deduplicating games across platforms and ROM dumps.
 * E.g. `"Legend of Zelda, The - The Wind Waker (USA)"` and `"The Legend of Zelda: The Wind Waker"`
 * both produce `"legendofzeldathewindwaker"`.
 */
export function canonicalTitleKey(raw: string): string {
  const normalized = normalizeGameTitle(raw).toLowerCase();
  // Strip leading "the ", "a ", "an " and inner ": the " for canonical matching
  const withoutArticles = normalized
    .replace(/^(the|a|an)\s+/i, "")
    .replace(/:\s*(the|a|an)\s+/gi, " ");
  return withoutArticles.replace(/[^a-z0-9]/g, "");
}

/**
 * Dedicated Title Normalizer Utility Plugin (`utility-title-normalizer`).
 */
export const titleNormalizerPlugin: UniversalPlugin = {
  id: "utility-title-normalizer",
  name: "Universal Title Normalizer & Deduplicator",
  version: "1.0.0",
  pluginType: "utility",
  author: "HELIX Origin",
  description:
    "Automatically normalizes raw executable names, concatenated folder slugs, RPG Maker / Ren'Py manifests, and No-Intro/Redump ROM filenames into clean, canonical titles.",
  homepage: "https://github.com/HELIX-Origin/Universal-Game-Launcher",
  features: ["title_normalization", "rom_tag_stripping", "article_inversion", "deduplication"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "invertArticles",
          label: "Invert Trailing Articles ('Legend of Zelda, The' → 'The Legend of Zelda')",
          type: "boolean",
          default: true,
          description: "Automatically move trailing articles (, The / , A / , An) to the beginning of the title."
        },
        {
          key: "stripRomTags",
          label: "Strip Region, Revision & Dump Tags",
          type: "boolean",
          default: true,
          description: "Remove (USA), (Europe), (Rev 1), [!], and scene group tags from imported titles."
        },
        {
          key: "splitConcatenatedSlugs",
          label: "Split Concatenated & CamelCase Folder Names",
          type: "boolean",
          default: true,
          description: "Automatically split run-together folder and executable names into properly spaced Title Case words."
        }
      ]
    };
  }
};
