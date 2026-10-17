import type { SteamLanguage } from "./types";

export const STEAM_LANGUAGES: SteamLanguage[] = [
  {
    id: "en",
    steamCode: "english",
    webCode: "en",
    name: "English",
    nativeName: "English"
  },
  {
    id: "ja",
    steamCode: "japanese",
    webCode: "ja",
    name: "Japanese",
    nativeName: "日本語"
  },
  {
    id: "zh-CN",
    steamCode: "schinese",
    webCode: "zh-CN",
    name: "Simplified Chinese",
    nativeName: "简体中文"
  },
  {
    id: "zh-TW",
    steamCode: "tchinese",
    webCode: "zh-TW",
    name: "Traditional Chinese",
    nativeName: "繁體中文"
  },
  {
    id: "es",
    steamCode: "spanish",
    webCode: "es",
    name: "Spanish",
    nativeName: "Español"
  },
  {
    id: "fr",
    steamCode: "french",
    webCode: "fr",
    name: "French",
    nativeName: "Français"
  },
  {
    id: "de",
    steamCode: "german",
    webCode: "de",
    name: "German",
    nativeName: "Deutsch"
  },
  {
    id: "it",
    steamCode: "italian",
    webCode: "it",
    name: "Italian",
    nativeName: "Italiano"
  },
  {
    id: "pt-BR",
    steamCode: "brazilian",
    webCode: "pt-BR",
    name: "Portuguese (Brazil)",
    nativeName: "Português (Brasil)"
  },
  {
    id: "ru",
    steamCode: "russian",
    webCode: "ru",
    name: "Russian",
    nativeName: "Русский"
  },
  {
    id: "ko",
    steamCode: "koreana",
    webCode: "ko",
    name: "Korean",
    nativeName: "한국어"
  },
  {
    id: "pl",
    steamCode: "polish",
    webCode: "pl",
    name: "Polish",
    nativeName: "Polski"
  }
];

export function findLanguage(code: string): SteamLanguage {
  if (!code) return STEAM_LANGUAGES[0];
  const query = code.toLowerCase().trim();

  // 1. Direct match on id, webCode, or steamCode
  const exact = STEAM_LANGUAGES.find(
    (l) =>
      l.id.toLowerCase() === query ||
      l.webCode.toLowerCase() === query ||
      l.steamCode.toLowerCase() === query
  );
  if (exact) return exact;

  // 2. Prefix / ISO 639-1 match (e.g. "ja-JP" -> "ja", "zh" -> "zh-CN", "pt" -> "pt-BR")
  const prefix = query.split(/[-_]/)[0];
  const prefixMatch = STEAM_LANGUAGES.find((l) => l.id.toLowerCase().startsWith(prefix));
  if (prefixMatch) return prefixMatch;

  return STEAM_LANGUAGES[0];
}
