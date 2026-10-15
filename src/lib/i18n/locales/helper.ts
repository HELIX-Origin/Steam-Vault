import type { TranslationDictionary } from "../types";
import { en } from "./en";

export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

/**
 * Creates a fully populated TranslationDictionary by layering locale-specific overrides
 * over the English source of truth.
 */
export function createLocale(partial: DeepPartial<TranslationDictionary>): TranslationDictionary {
  return {
    nav: { ...en.nav, ...(partial.nav ?? {}) },
    status: { ...en.status, ...(partial.status ?? {}) },
    common: { ...en.common, ...(partial.common ?? {}) },
    dashboard: { ...en.dashboard, ...(partial.dashboard ?? {}) },
    library: { ...en.library, ...(partial.library ?? {}) },
    organize: { ...en.organize, ...(partial.organize ?? {}) },
    collections: { ...en.collections, ...(partial.collections ?? {}) },
    platforms: { ...en.platforms, ...(partial.platforms ?? {}) },
    storage: { ...en.storage, ...(partial.storage ?? {}) },
    artwork: { ...en.artwork, ...(partial.artwork ?? {}) },
    settings: { ...en.settings, ...(partial.settings ?? {}) }
  };
}
