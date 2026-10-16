import type { TranslationDictionary } from "../types";
import { en } from "./en";

/**
 * Standard English locale dictionary.
 * Custom community language support can be provided as drop-in plugins.
 */
export const LOCALES: Record<string, TranslationDictionary> = {
  en,
  english: en
};

export { en };
