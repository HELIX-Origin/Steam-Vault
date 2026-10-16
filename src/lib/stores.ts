// Global app state (Svelte stores).

import { writable } from "svelte/store";
import type {
  Collection,
  Game,
  OwnedLibrary,
  PlatformScan,
  ProviderInfo,
  Settings,
  SteamInstall,
  StorageReport,
} from "./types";

export const settings = writable<Settings | null>(null);
export const providers = writable<ProviderInfo[]>([]);
export const install = writable<SteamInstall | null>(null);
export const games = writable<Game[]>([]);
export const selectedGameId = writable<number | null>(null);
export const collections = writable<Collection[]>([]);
export const steamRunning = writable<boolean>(false);
export const ownedLibrary = writable<OwnedLibrary | null>(null);
export const platformScan = writable<PlatformScan | null>(null);
export const storageReport = writable<StorageReport | null>(null);
export const loading = writable<boolean>(false);
function createCustomNamesStore() {
  let initial: Record<number, string> = {};
  if (typeof window !== "undefined" && window.localStorage) {
    try {
      const stored = localStorage.getItem("steam_vault_custom_names");
      if (stored) initial = JSON.parse(stored);
    } catch {}
  }
  const store = writable<Record<number, string>>(initial);
  if (typeof window !== "undefined" && window.localStorage) {
    store.subscribe((val) => {
      try {
        localStorage.setItem("steam_vault_custom_names", JSON.stringify(val));
      } catch {}
    });
  }
  return store;
}

export const customNames = createCustomNamesStore();
export const error = writable<string | null>(null);
