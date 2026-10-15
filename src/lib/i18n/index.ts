// Hand-rolled reactive i18n localization engine for Steam-Vault.
// Full support for all 30 Valve/Steam officially supported platform languages.

import { derived, writable } from "svelte/store";
import { findLanguage, STEAM_LANGUAGES } from "./languages";
import { en, LOCALES } from "./locales";
import type { SteamLanguage, TranslationDictionary } from "./types";

export * from "./languages";
export * from "./types";

const STORAGE_KEY = "steam_vault_language";

/** Get initial language choice from storage, browser preference, or default to English. */
function getInitialLocale(): string {
  if (typeof window !== "undefined") {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) return stored;
    } catch {
      // Storage unavailable or blocked
    }

    if (navigator?.language) {
      const matched = findLanguage(navigator.language);
      return matched.id;
    }
  }
  return "en";
}

/** Reactive store for currently selected language code (e.g. "en", "de", "zh-CN", "ja"). */
export const locale = writable<string>(getInitialLocale());

/** Reactive store returning full metadata for the currently active language. */
export const activeLanguage = derived<typeof locale, SteamLanguage>(locale, ($loc) => findLanguage($loc));

/** Reactive store indicating whether current language requires RTL (Right-to-Left) direction. */
export const isRTL = derived<typeof activeLanguage, boolean>(activeLanguage, ($lang) => Boolean($lang.rtl));

/** Helper to retrieve nested object value by dot notation (e.g. "nav.library"). */
function getNestedValue(obj: unknown, path: string): string | undefined {
  if (!obj || typeof obj !== "object") return undefined;
  const parts = path.split(".");
  let current: any = obj;
  for (const part of parts) {
    if (current == null || typeof current !== "object") return undefined;
    current = current[part];
  }
  return typeof current === "string" ? current : undefined;
}

/**
 * Resolve translation for a given dot-notation key and optional parameter interpolation.
 * Automatically falls back to English if the key is missing from the active language.
 */
export function translate(
  key: string,
  params?: Record<string, string | number>,
  langCode?: string
): string {
  const targetCode = (langCode || "en").toLowerCase();
  const dict: TranslationDictionary = LOCALES[targetCode] || LOCALES[findLanguage(targetCode).id] || en;

  let value = getNestedValue(dict, key);
  if (!value && dict !== en) {
    value = getNestedValue(en, key);
  }

  if (!value) {
    return key;
  }

  if (params) {
    for (const [paramKey, paramVal] of Object.entries(params)) {
      value = value.replaceAll(`{${paramKey}}`, String(paramVal));
    }
  }

  return value;
}

/**
 * Reactive translation function store `$t`.
 * In Svelte components: `const { t } = ...; $t("nav.library")`.
 */
export const t = derived(locale, ($loc) => {
  return (key: string, params?: Record<string, string | number>): string => {
    return translate(key, params, $loc);
  };
});

/**
 * Switch active locale, persist to localStorage, and update DOM direction for RTL languages.
 */
export function setLocale(newLocale: string): void {
  const lang = findLanguage(newLocale);
  locale.set(lang.id);

  if (typeof window !== "undefined") {
    try {
      localStorage.setItem(STORAGE_KEY, lang.id);
    } catch {
      // Storage unavailable
    }

    if (typeof document !== "undefined") {
      document.documentElement.lang = lang.webCode || lang.id;
      document.documentElement.dir = lang.rtl ? "rtl" : "ltr";
    }
  }
}

/**
 * Synchronize with backend settings language (called when settings are loaded or saved).
 */
export function syncWithSettingsLanguage(settingsLang: string | null | undefined): void {
  if (!settingsLang) return;
  const target = findLanguage(settingsLang);
  setLocale(target.id);
}
