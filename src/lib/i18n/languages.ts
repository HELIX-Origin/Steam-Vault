import type { SteamLanguage } from "./types";

export const STEAM_LANGUAGES: SteamLanguage[] = [
  {
    id: "en",
    steamCode: "english",
    webCode: "en",
    name: "English",
    nativeName: "English"
  }
];

export function findLanguage(_code: string): SteamLanguage {
  return STEAM_LANGUAGES[0];
}
