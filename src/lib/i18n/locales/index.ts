import type { TranslationDictionary } from "../types";
import { en } from "./en";
import { de } from "./de";
import { fr } from "./fr";
import { it } from "./it";
import { ko } from "./ko";
import { es } from "./es";
import { zhCN } from "./zh-CN";
import { zhTW } from "./zh-TW";
import { ru } from "./ru";
import { th } from "./th";
import { ja } from "./ja";
import { pt } from "./pt";
import { pl } from "./pl";
import { da } from "./da";
import { nl } from "./nl";
import { fi } from "./fi";
import { no } from "./no";
import { sv } from "./sv";
import { hu } from "./hu";
import { cs } from "./cs";
import { ro } from "./ro";
import { tr } from "./tr";
import { ptBR } from "./pt-BR";
import { bg } from "./bg";
import { el } from "./el";
import { ar } from "./ar";
import { uk } from "./uk";
import { es419 } from "./es-419";
import { vi } from "./vi";
import { id } from "./id";

/**
 * Complete registry of translation dictionaries for all 30 languages.
 * Keys include both ISO codes (e.g. "de", "zh-CN") and Valve Steam codes (e.g. "german", "schinese").
 */
export const LOCALES: Record<string, TranslationDictionary> = {
  // English
  en,
  english: en,

  // German
  de,
  german: de,

  // French
  fr,
  french: fr,

  // Italian
  it,
  italian: it,

  // Korean
  ko,
  korean: ko,
  koreana: ko,

  // Spanish (Spain)
  es,
  spanish: es,

  // Simplified Chinese
  "zh-CN": zhCN,
  zh: zhCN,
  schinese: zhCN,

  // Traditional Chinese
  "zh-TW": zhTW,
  tchinese: zhTW,

  // Russian
  ru,
  russian: ru,

  // Thai
  th,
  thai: th,

  // Japanese
  ja,
  japanese: ja,

  // Portuguese (Portugal)
  pt,
  portuguese: pt,

  // Polish
  pl,
  polish: pl,

  // Danish
  da,
  danish: da,

  // Dutch
  nl,
  dutch: nl,

  // Finnish
  fi,
  finnish: fi,

  // Norwegian
  no,
  norwegian: no,

  // Swedish
  sv,
  swedish: sv,

  // Hungarian
  hu,
  hungarian: hu,

  // Czech
  cs,
  czech: cs,

  // Romanian
  ro,
  romanian: ro,

  // Turkish
  tr,
  turkish: tr,

  // Portuguese (Brazil)
  "pt-BR": ptBR,
  brazilian: ptBR,

  // Bulgarian
  bg,
  bulgarian: bg,

  // Greek
  el,
  greek: el,

  // Arabic
  ar,
  arabic: ar,

  // Ukrainian
  uk,
  ukrainian: uk,

  // Spanish (Latin America)
  "es-419": es419,
  latam: es419,

  // Vietnamese
  vi,
  vietnamese: vi,

  // Indonesian
  id,
  indonesian: id
};

export {
  en,
  de,
  fr,
  it,
  ko,
  es,
  zhCN,
  zhTW,
  ru,
  th,
  ja,
  pt,
  pl,
  da,
  nl,
  fi,
  no,
  sv,
  hu,
  cs,
  ro,
  tr,
  ptBR,
  bg,
  el,
  ar,
  uk,
  es419,
  vi,
  id
};
