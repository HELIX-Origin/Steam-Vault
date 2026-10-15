// All 30 official languages supported by Steam (Valve Steamworks & Steam Web API).
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
    id: "de",
    steamCode: "german",
    webCode: "de",
    name: "German",
    nativeName: "Deutsch"
  },
  {
    id: "fr",
    steamCode: "french",
    webCode: "fr",
    name: "French",
    nativeName: "Français"
  },
  {
    id: "it",
    steamCode: "italian",
    webCode: "it",
    name: "Italian",
    nativeName: "Italiano"
  },
  {
    id: "ko",
    steamCode: "koreana",
    webCode: "ko",
    name: "Korean",
    nativeName: "한국어"
  },
  {
    id: "es",
    steamCode: "spanish",
    webCode: "es",
    name: "Spanish - Spain",
    nativeName: "Español (España)"
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
    id: "ru",
    steamCode: "russian",
    webCode: "ru",
    name: "Russian",
    nativeName: "Русский"
  },
  {
    id: "th",
    steamCode: "thai",
    webCode: "th",
    name: "Thai",
    nativeName: "ไทย"
  },
  {
    id: "ja",
    steamCode: "japanese",
    webCode: "ja",
    name: "Japanese",
    nativeName: "日本語"
  },
  {
    id: "pt",
    steamCode: "portuguese",
    webCode: "pt",
    name: "Portuguese - Portugal",
    nativeName: "Português"
  },
  {
    id: "pl",
    steamCode: "polish",
    webCode: "pl",
    name: "Polish",
    nativeName: "Polski"
  },
  {
    id: "da",
    steamCode: "danish",
    webCode: "da",
    name: "Danish",
    nativeName: "Dansk"
  },
  {
    id: "nl",
    steamCode: "dutch",
    webCode: "nl",
    name: "Dutch",
    nativeName: "Nederlands"
  },
  {
    id: "fi",
    steamCode: "finnish",
    webCode: "fi",
    name: "Finnish",
    nativeName: "Suomi"
  },
  {
    id: "no",
    steamCode: "norwegian",
    webCode: "no",
    name: "Norwegian",
    nativeName: "Norsk"
  },
  {
    id: "sv",
    steamCode: "swedish",
    webCode: "sv",
    name: "Swedish",
    nativeName: "Svenska"
  },
  {
    id: "hu",
    steamCode: "hungarian",
    webCode: "hu",
    name: "Hungarian",
    nativeName: "Magyar"
  },
  {
    id: "cs",
    steamCode: "czech",
    webCode: "cs",
    name: "Czech",
    nativeName: "Čeština"
  },
  {
    id: "ro",
    steamCode: "romanian",
    webCode: "ro",
    name: "Romanian",
    nativeName: "Română"
  },
  {
    id: "tr",
    steamCode: "turkish",
    webCode: "tr",
    name: "Turkish",
    nativeName: "Türkçe"
  },
  {
    id: "pt-BR",
    steamCode: "brazilian",
    webCode: "pt-BR",
    name: "Portuguese - Brazil",
    nativeName: "Português (Brasil)"
  },
  {
    id: "bg",
    steamCode: "bulgarian",
    webCode: "bg",
    name: "Bulgarian",
    nativeName: "Български"
  },
  {
    id: "el",
    steamCode: "greek",
    webCode: "el",
    name: "Greek",
    nativeName: "Ελληνικά"
  },
  {
    id: "ar",
    steamCode: "arabic",
    webCode: "ar",
    name: "Arabic",
    nativeName: "العربية",
    rtl: true
  },
  {
    id: "uk",
    steamCode: "ukrainian",
    webCode: "uk",
    name: "Ukrainian",
    nativeName: "Українська"
  },
  {
    id: "es-419",
    steamCode: "latam",
    webCode: "es-419",
    name: "Spanish - Latin America",
    nativeName: "Español (Latinoamérica)"
  },
  {
    id: "vi",
    steamCode: "vietnamese",
    webCode: "vi",
    name: "Vietnamese",
    nativeName: "Tiếng Việt"
  },
  {
    id: "id",
    steamCode: "indonesian",
    webCode: "id",
    name: "Indonesian",
    nativeName: "Bahasa Indonesia"
  }
];

/**
 * Finds a language by either id ("en"), steamCode ("english"), webCode, or normalized search.
 */
export function findLanguage(input: string | null | undefined): SteamLanguage {
  if (!input) return STEAM_LANGUAGES[0];
  const query = input.trim().toLowerCase();

  const match = STEAM_LANGUAGES.find(
    (lang) =>
      lang.id.toLowerCase() === query ||
      lang.steamCode.toLowerCase() === query ||
      lang.webCode.toLowerCase() === query ||
      lang.name.toLowerCase() === query ||
      lang.nativeName.toLowerCase() === query
  );

  if (match) return match;

  // Handle language prefix (e.g. "en-US" -> "en", "es-MX" -> "es-419" or "es", "pt-PT" -> "pt")
  const primaryCode = query.split(/[-_]/)[0];
  const prefixMatch = STEAM_LANGUAGES.find(
    (lang) => lang.id.toLowerCase() === primaryCode || lang.webCode.toLowerCase() === primaryCode
  );

  return prefixMatch ?? STEAM_LANGUAGES[0];
}
