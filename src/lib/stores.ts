// Global app state (Svelte stores).

import { writable } from "svelte/store";
import type { Collection, Game, ProviderInfo, Settings, SteamInstall } from "./types";

export const settings = writable<Settings | null>(null);
export const providers = writable<ProviderInfo[]>([]);
export const install = writable<SteamInstall | null>(null);
export const games = writable<Game[]>([]);
export const collections = writable<Collection[]>([]);
export const steamRunning = writable<boolean>(false);
export const loading = writable<boolean>(false);
export const error = writable<string | null>(null);
